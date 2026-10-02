use std::collections::HashMap;

use serde::Deserialize;
use thiserror::Error;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::hash::FileHash;

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct AssetIndex {
    objects: HashMap<String, AssetObject>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct AssetObject {
    pub hash: FileHash,
    pub size: u64,
}

impl AssetIndex {
    pub fn assets(&self) -> impl Iterator<Item = (&str, &AssetObject)> {
        self.objects
            .iter()
            .map(|(hash, object)| (hash.as_str(), object))
    }

    pub fn objects(&self) -> impl Iterator<Item = &AssetObject> {
        self.objects.values()
    }

    pub fn get_object(&self, name: &str) -> Option<&AssetObject> {
        self.objects.get(name)
    }
}

#[derive(Debug, Error)]
pub enum AssetObjectError {
    #[error("Content length mismatch: expected {expected} but got {actual:?}")]
    ContentLengthMismatch { expected: u64, actual: Option<u64> },
    #[error("Invalid hash format: {0}")]
    InvalidHashFormat(#[from] hex::FromHexError),
    #[error("Hash mismatch: expected {expected} but got {actual}")]
    HashMismatch {
        expected: FileHash,
        actual: FileHash,
    },
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}

pub type AssetObjectResult<T> = Result<T, AssetObjectError>;

impl AssetObject {
    pub async fn fetch(
        &self,
        client: reqwest::Client,
        writer: impl AsyncWrite + Unpin,
    ) -> AssetObjectResult<()> {
        let url = format!(
            "https://resources.download.minecraft.net/{}/{}",
            self.hash.prefix(),
            self.hash.to_string()
        );
        self.fetch_with_url(client, &url, writer).await
    }

    pub async fn fetch_with_url(
        &self,
        client: reqwest::Client,
        url: &str,
        mut writer: impl AsyncWrite + Unpin,
    ) -> AssetObjectResult<()> {
        let mut response = client.get(url).send().await?.error_for_status()?;
        let content_length = response.content_length();
        if content_length != Some(self.size) {
            return Err(AssetObjectError::ContentLengthMismatch {
                expected: self.size,
                actual: content_length,
            });
        }
        let mut actual_hash = FileHash::digest_chunks();
        while let Some(chunk) = response.chunk().await? {
            writer.write_all(&chunk).await?;
            actual_hash.update(&chunk);
        }
        // sha1 is always 20 bytes
        let actual_hash = actual_hash.finalize();
        if self.hash != actual_hash {
            return Err(AssetObjectError::HashMismatch {
                expected: self.hash,
                actual: actual_hash,
            });
        }
        Ok(())
    }
}
