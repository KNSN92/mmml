use std::{
    io,
    path::{Path, PathBuf},
    pin::Pin,
    task::{Context, Poll},
};

#[cfg(feature = "debug")]
use mc_resources::manifest::VersionManifest;
use mc_resources::{
    download::{
        DownloadableFile, DownloadableManifest, DownloadableResource, ResourceDownloadError,
        ResourceDownloader,
    },
    hash::ResourceHash,
};
use mockito::Server;
use reqwest::{Client, IntoUrl};
use serde::Deserialize;
use tokio::io::AsyncWrite;

#[derive(Clone)]
struct TestResource {
    url: String,
    size: Option<u64>,
    hash: Option<ResourceHash>,
    path: PathBuf,
}

impl DownloadableResource for TestResource {
    fn url(&self) -> impl IntoUrl {
        self.url.clone()
    }

    fn size(&self) -> Option<u64> {
        self.size
    }

    fn hash(&self) -> Option<ResourceHash> {
        self.hash
    }
}

impl DownloadableFile for TestResource {
    fn path(&self) -> impl AsRef<Path> {
        &self.path
    }
}

impl DownloadableManifest for TestResource {
    type Resource = serde_json::Value;

    fn url(&self) -> impl IntoUrl {
        self.url.clone()
    }

    fn size(&self) -> Option<u64> {
        self.size
    }

    fn hash(&self) -> Option<ResourceHash> {
        self.hash
    }
}

struct TestManifest(String);

impl DownloadableManifest for TestManifest {
    type Resource = serde_json::Value;

    fn url(&self) -> impl IntoUrl {
        self.0.clone()
    }

    fn size(&self) -> Option<u64> {
        None
    }

    fn hash(&self) -> Option<ResourceHash> {
        None
    }
}

#[cfg(feature = "debug")]
struct VersionManifestEndpoint(String);

#[cfg(feature = "debug")]
impl DownloadableManifest for VersionManifestEndpoint {
    type Resource = VersionManifest;

    fn url(&self) -> impl IntoUrl {
        self.0.clone()
    }

    fn size(&self) -> Option<u64> {
        None
    }

    fn hash(&self) -> Option<ResourceHash> {
        None
    }
}

fn resource(url: String) -> TestResource {
    TestResource {
        url,
        size: None,
        hash: None,
        path: "downloads/test.bin".into(),
    }
}

fn downloader() -> ResourceDownloader {
    ResourceDownloader::new(Client::new())
}

#[tokio::test]
async fn fetch_validates_metadata_and_deserializes_json() {
    let mut server = Server::new_async().await;
    let body = r#"{"ok":true}"#;
    let mock = server
        .mock("GET", "/manifest")
        .with_body(body)
        .create_async()
        .await;
    let mut manifest = TestManifest(format!("{}/manifest", server.url()));
    // A resource with both integrity fields exercises the complete validation path.
    let bytes = body.as_bytes();
    #[derive(Debug, Deserialize, PartialEq)]
    struct Payload {
        ok: bool,
    }
    #[derive(Clone)]
    struct CheckedManifest {
        url: String,
        size: Option<u64>,
        hash: Option<ResourceHash>,
    }
    impl DownloadableManifest for CheckedManifest {
        type Resource = Payload;
        fn url(&self) -> impl IntoUrl {
            self.url.clone()
        }
        fn size(&self) -> Option<u64> {
            self.size
        }
        fn hash(&self) -> Option<ResourceHash> {
            self.hash
        }
    }
    let checked = CheckedManifest {
        url: manifest.0.clone(),
        size: Some(bytes.len() as u64),
        hash: Some(ResourceHash::digest(bytes)),
    };
    let parsed: Payload = downloader().fetch(&checked).await.unwrap();
    assert_eq!(parsed, Payload { ok: true });
    mock.assert_async().await;

    // Optional size/hash are also accepted when omitted.
    let mock = server
        .mock("GET", "/without-integrity")
        .with_body(body)
        .create_async()
        .await;
    manifest.0 = format!("{}/without-integrity", server.url());
    assert_eq!(downloader().fetch(&manifest).await.unwrap()["ok"], true);
    mock.assert_async().await;
}

#[tokio::test]
async fn fetch_reports_http_integrity_and_json_errors() {
    let mut server = Server::new_async().await;
    let error_response = server
        .mock("GET", "/status")
        .with_status(503)
        .create_async()
        .await;
    let error = downloader()
        .fetch(&TestManifest(format!("{}/status", server.url())))
        .await
        .unwrap_err();
    assert!(matches!(error, ResourceDownloadError::NetworkError(_)));
    error_response.assert_async().await;

    let body = b"hello";
    let length_response = server
        .mock("GET", "/length")
        .with_body(body)
        .create_async()
        .await;
    let mut item = resource(format!("{}/length", server.url()));
    item.size = Some(3);
    let error = downloader().fetch(&item).await.unwrap_err();
    assert!(matches!(
        error,
        ResourceDownloadError::ContentLengthMismatch {
            expected: 3,
            actual: 5
        }
    ));
    length_response.assert_async().await;

    let chunked_response = server
        .mock("GET", "/chunked-length")
        .with_chunked_body(|writer| writer.write_all(b"hello"))
        .create_async()
        .await;
    let mut item = resource(format!("{}/chunked-length", server.url()));
    item.size = Some(4);
    let error = downloader().fetch(&item).await.unwrap_err();
    assert!(matches!(
        error,
        ResourceDownloadError::ContentLengthMismatch {
            expected: 4,
            actual: 5
        }
    ));
    chunked_response.assert_async().await;

    let hash_response = server
        .mock("GET", "/hash")
        .with_body(body)
        .create_async()
        .await;
    let mut item = resource(format!("{}/hash", server.url()));
    item.hash = Some(ResourceHash::digest(b"different"));
    let error = downloader().fetch(&item).await.unwrap_err();
    assert!(matches!(error, ResourceDownloadError::HashMismatch { .. }));
    hash_response.assert_async().await;

    let json_response = server
        .mock("GET", "/json")
        .with_body("not-json")
        .create_async()
        .await;
    let error = downloader()
        .fetch(&TestManifest(format!("{}/json", server.url())))
        .await
        .unwrap_err();
    assert!(matches!(
        error,
        ResourceDownloadError::DeserializationError(_)
    ));
    json_response.assert_async().await;
}

#[cfg(feature = "debug")]
#[tokio::test]
async fn debug_fetch_error_contains_the_nested_json_path() {
    let mut server = Server::new_async().await;
    let body = r#"{"latest":{"release":"1","snapshot":"1"},"versions":[{"id":"1","type":"release","url":"https://example.invalid/1.json","time":"now","releaseTime":"now","sha1":"bad","complianceLevel":1}]}"#;
    let mock = server
        .mock("GET", "/version-manifest")
        .with_body(body)
        .create_async()
        .await;
    let error = downloader()
        .fetch(&VersionManifestEndpoint(format!(
            "{}/version-manifest",
            server.url()
        )))
        .await
        .unwrap_err();
    match error {
        ResourceDownloadError::DeserializationError(error) => {
            let path = error.path().to_string();
            assert!(path.contains("versions"), "{path}");
            assert!(path.contains("sha1"), "{path}");
        }
        other => panic!("expected deserialization error, got {other:?}"),
    }
    mock.assert_async().await;
}

#[tokio::test]
async fn fetch_reports_connection_failure() {
    let server = Server::new_async().await;
    let url = format!("{}/gone", server.url());
    drop(server);
    let error = downloader().fetch(&TestManifest(url)).await.unwrap_err();
    assert!(matches!(error, ResourceDownloadError::NetworkError(_)));
}

#[tokio::test]
async fn download_accepts_chunked_body_without_content_length() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/chunked")
        .with_chunked_body(|writer| {
            writer.write_all(b"ab")?;
            writer.write_all(b"cd")
        })
        .create_async()
        .await;
    let mut item = resource(format!("{}/chunked", server.url()));
    item.size = Some(4);
    item.hash = Some(ResourceHash::digest(b"abcd"));
    let mut output = Vec::new();
    downloader().download(&item, &mut output).await.unwrap();
    assert_eq!(output, b"abcd");
    mock.assert_async().await;
}

#[tokio::test]
async fn download_rejects_short_long_and_wrong_hash_without_writing() {
    let mut server = Server::new_async().await;

    for (path, body, expected_size) in [
        ("/short", b"abc".as_slice(), 4),
        ("/long", b"abc".as_slice(), 2),
    ] {
        let mock = server
            .mock("GET", path)
            .with_chunked_body(move |writer| writer.write_all(body))
            .create_async()
            .await;
        let mut item = resource(format!("{}{}", server.url(), path));
        item.size = Some(expected_size);
        let mut output = Vec::new();
        let error = downloader().download(&item, &mut output).await.unwrap_err();
        assert!(matches!(
            error,
            ResourceDownloadError::ContentLengthMismatch { .. }
        ));
        assert!(output.is_empty());
        mock.assert_async().await;
    }

    let mock = server
        .mock("GET", "/bad-hash")
        .with_chunked_body(|writer| writer.write_all(b"abc"))
        .create_async()
        .await;
    let mut item = resource(format!("{}/bad-hash", server.url()));
    item.size = Some(3);
    item.hash = Some(ResourceHash::digest(b"different"));
    let mut output = Vec::new();
    assert!(matches!(
        downloader().download(&item, &mut output).await,
        Err(ResourceDownloadError::HashMismatch { .. })
    ));
    assert!(output.is_empty());
    mock.assert_async().await;
}

struct FailingWriter;

impl AsyncWrite for FailingWriter {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        _buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Poll::Ready(Err(io::Error::other("write failed")))
    }
    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Err(io::Error::other("flush failed")))
    }
    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

struct FlushFailWriter;

impl AsyncWrite for FlushFailWriter {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buffer: &[u8],
    ) -> Poll<io::Result<usize>> {
        Poll::Ready(Ok(buffer.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Err(io::Error::other("flush failed")))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

#[tokio::test]
async fn download_propagates_writer_errors() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/write-fail")
        .with_body("payload")
        .create_async()
        .await;
    let item = resource(format!("{}/write-fail", server.url()));
    assert!(matches!(
        downloader().download(&item, FailingWriter).await,
        Err(ResourceDownloadError::IOError(_))
    ));
    mock.assert_async().await;

    let mock = server
        .mock("GET", "/flush-fail")
        .with_body("payload")
        .create_async()
        .await;
    let item = resource(format!("{}/flush-fail", server.url()));
    assert!(matches!(
        downloader().download(&item, FlushFailWriter).await,
        Err(ResourceDownloadError::IOError(_))
    ));
    mock.assert_async().await;
}

#[tokio::test]
async fn download_to_file_creates_parent_and_replaces_only_after_validation() {
    let temp = tempfile::tempdir().unwrap();
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/valid")
        .with_body("verified")
        .create_async()
        .await;
    let mut item = resource(format!("{}/valid", server.url()));
    item.size = Some(8);
    item.hash = Some(ResourceHash::digest(b"verified"));
    item.path = "nested/new/object.bin".into();
    let target = temp.path().join("nested/new/object.bin");
    downloader()
        .download_to_file(temp.path(), &item)
        .await
        .unwrap();
    assert_eq!(tokio::fs::read(&target).await.unwrap(), b"verified");
    mock.assert_async().await;

    tokio::fs::write(&target, b"old-content").await.unwrap();
    let replace_mock = server
        .mock("GET", "/replace")
        .with_body("replaced")
        .create_async()
        .await;
    let mut replacement = item.clone();
    replacement.url = format!("{}/replace", server.url());
    replacement.hash = Some(ResourceHash::digest(b"replaced"));
    downloader()
        .download_to_file(temp.path(), &replacement)
        .await
        .unwrap();
    assert_eq!(tokio::fs::read(&target).await.unwrap(), b"replaced");
    replace_mock.assert_async().await;

    let mock = server
        .mock("GET", "/invalid")
        .with_body("corrupt!")
        .create_async()
        .await;
    let mut invalid = item.clone();
    invalid.url = format!("{}/invalid", server.url());
    invalid.hash = Some(ResourceHash::digest(b"verified"));
    let error = downloader()
        .download_to_file(temp.path(), &invalid)
        .await
        .unwrap_err();
    assert!(matches!(error, ResourceDownloadError::HashMismatch { .. }));
    assert_eq!(tokio::fs::read(&target).await.unwrap(), b"replaced");
    mock.assert_async().await;
}

#[tokio::test]
async fn download_to_file_rejects_absolute_and_traversal_paths() {
    let temp = tempfile::tempdir().unwrap();
    let mut item = resource("http://127.0.0.1:0/unused".to_string());
    item.path = PathBuf::from("/absolute.bin");
    let error = downloader()
        .download_to_file(temp.path(), &item)
        .await
        .unwrap_err();
    assert!(matches!(
        error,
        ResourceDownloadError::SafeJoinError(safe_path::SafeJoinError::AbsolutePath)
    ));

    item.path = PathBuf::from("../escape.bin");
    let error = downloader()
        .download_to_file(temp.path(), &item)
        .await
        .unwrap_err();
    assert!(matches!(
        error,
        ResourceDownloadError::SafeJoinError(safe_path::SafeJoinError::PathTraversal)
    ));
    assert!(!temp.path().parent().unwrap().join("escape.bin").exists());
}
