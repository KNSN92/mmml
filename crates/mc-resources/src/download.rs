use std::path::Path;

use reqwest::{Client, IntoUrl};
use safe_path::SafeJoin;
use serde::de::DeserializeOwned;
use tempfile::NamedTempFile;
use thiserror::Error;
use tokio::{
    fs,
    io::{AsyncWrite, AsyncWriteExt},
};

use crate::hash::ResourceHash;

#[derive(Debug, Error)]
pub enum ResourceDownloadError {
    #[error("Content length mismatch: expected {expected} but got {actual:?}")]
    ContentLengthMismatch { expected: u64, actual: u64 },
    #[error("Hash mismatch: expected {expected} but got {actual}")]
    HashMismatch {
        expected: ResourceHash,
        actual: ResourceHash,
    },
    #[cfg(not(feature = "debug"))]
    #[error("Deserialization error: {0}")]
    DeserializationError(#[from] serde_json::Error),
    #[cfg(feature = "debug")]
    #[error("Deserialization error: {0}")]
    DeserializationError(#[from] serde_path_to_error::Error<serde_json::Error>),
    #[error("Safe join error: {0}")]
    SafeJoinError(#[from] safe_path::SafeJoinError),
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}

pub type ResourceDownloadResult<T> = Result<T, ResourceDownloadError>;

pub trait DownloadableResource {
    fn url(&self) -> impl IntoUrl;
    fn size(&self) -> Option<u64>;
    fn hash(&self) -> Option<ResourceHash>;
}
pub trait DownloadableFile: DownloadableResource {
    fn path(&self) -> impl AsRef<Path>;
}
pub trait DownloadableManifest {
    type Resource: DeserializeOwned;
    fn url(&self) -> impl IntoUrl;
    fn size(&self) -> Option<u64>;
    fn hash(&self) -> Option<ResourceHash>;
}

pub struct ResourceDownloader(Client);

impl ResourceDownloader {
    pub fn new(client: Client) -> Self {
        ResourceDownloader(client)
    }

    #[cfg(not(feature = "debug"))]
    pub async fn fetch<T: DownloadableManifest>(
        &self,
        resource: &T,
    ) -> ResourceDownloadResult<T::Resource> {
        let response = self
            .0
            .get(resource.url())
            .send()
            .await?
            .error_for_status()?;
        let content_length = response.content_length();
        if let Some(expected_size) = resource.size()
            && let Some(content_length) = content_length
            && content_length != expected_size
        {
            return Err(ResourceDownloadError::ContentLengthMismatch {
                expected: expected_size,
                actual: content_length,
            });
        }
        let bytes = response.bytes().await?;
        let content_length = bytes.len() as u64;
        if let Some(expected_size) = resource.size()
            && content_length != expected_size
        {
            return Err(ResourceDownloadError::ContentLengthMismatch {
                expected: expected_size,
                actual: bytes.len() as u64,
            });
        }
        let actual_hash = ResourceHash::digest(&bytes);
        if let Some(expected_hash) = resource.hash()
            && expected_hash != actual_hash
        {
            return Err(ResourceDownloadError::HashMismatch {
                expected: expected_hash,
                actual: actual_hash,
            });
        }
        Ok(serde_json::from_slice(&bytes)?)
    }

    #[cfg(feature = "debug")]
    pub async fn fetch<T: DownloadableManifest>(
        &self,
        resource: &T,
    ) -> ResourceDownloadResult<T::Resource> {
        let response = self
            .0
            .get(resource.url())
            .send()
            .await?
            .error_for_status()?;
        let content_length = response.content_length();
        if let Some(expected_size) = resource.size()
            && let Some(content_length) = content_length
            && content_length != expected_size
        {
            return Err(ResourceDownloadError::ContentLengthMismatch {
                expected: expected_size,
                actual: content_length,
            });
        }
        let bytes = response.bytes().await?;
        let content_length = bytes.len() as u64;
        if let Some(expected_size) = resource.size()
            && content_length != expected_size
        {
            return Err(ResourceDownloadError::ContentLengthMismatch {
                expected: expected_size,
                actual: bytes.len() as u64,
            });
        }
        let actual_hash = ResourceHash::digest(&bytes);
        if let Some(expected_hash) = resource.hash()
            && expected_hash != actual_hash
        {
            return Err(ResourceDownloadError::HashMismatch {
                expected: expected_hash,
                actual: actual_hash,
            });
        }
        let mut deserializer = serde_json::Deserializer::from_slice(&bytes);
        let manifest = serde_path_to_error::deserialize(&mut deserializer)?;
        Ok(manifest)
    }

    pub async fn download(
        &self,
        resource: &impl DownloadableResource,
        mut writer: impl AsyncWrite + Unpin,
    ) -> ResourceDownloadResult<()> {
        let mut response = self
            .0
            .get(resource.url())
            .send()
            .await?
            .error_for_status()?;
        let content_length = response.content_length();
        if let Some(expected_size) = resource.size()
            && let Some(content_length) = content_length
            && content_length != expected_size
        {
            return Err(ResourceDownloadError::ContentLengthMismatch {
                expected: expected_size,
                actual: content_length,
            });
        }
        let mut content_length = 0;
        let mut actual_hash = ResourceHash::digest_chunks();
        let mut chunks = Vec::new();
        while let Some(chunk) = response.chunk().await? {
            content_length += chunk.len() as u64;
            if let Some(expected_size) = resource.size()
                && content_length > expected_size
            {
                break;
            }
            actual_hash.update(&chunk);
            chunks.push(chunk);
        }
        if let Some(expected_size) = resource.size()
            && content_length != expected_size
        {
            return Err(ResourceDownloadError::ContentLengthMismatch {
                expected: expected_size,
                actual: content_length,
            });
        }
        // sha1 is always 20 bytes
        let actual_hash = actual_hash.finalize();
        if let Some(expected_hash) = resource.hash()
            && expected_hash != actual_hash
        {
            return Err(ResourceDownloadError::HashMismatch {
                expected: expected_hash,
                actual: actual_hash,
            });
        }
        for chunk in chunks {
            writer.write_all(&chunk).await?;
        }
        writer.flush().await?;
        Ok(())
    }

    pub async fn download_to_file(
        &self,
        base_path: impl AsRef<Path>,
        resource: &impl DownloadableFile,
    ) -> ResourceDownloadResult<()> {
        let path = base_path.as_ref().safe_join(resource.path().as_ref())?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let temporary = NamedTempFile::new_in(parent)?;
        let mut file = fs::File::from_std(temporary.reopen()?);
        self.download(resource, &mut file).await?;
        file.sync_all().await?;
        drop(file);
        temporary.persist(&path).map_err(|e| e.error)?;
        Ok(())
    }
}
