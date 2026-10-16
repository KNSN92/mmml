use std::collections::HashMap;

use reqwest::IntoUrl;
use serde::Deserialize;
use thiserror::Error;

use crate::{download::{DownloadableFile, DownloadableResource}, hash::ResourceHash};

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct AssetIndex {
    objects: HashMap<String, AssetObject>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct AssetObject {
    pub hash: ResourceHash,
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
    ContentLengthMismatch { expected: u64, actual: u64 },
    #[error("Hash mismatch: expected {expected} but got {actual}")]
    HashMismatch {
        expected: ResourceHash,
        actual: ResourceHash,
    },
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}

pub type AssetObjectResult<T> = Result<T, AssetObjectError>;

impl DownloadableResource for AssetObject {
    fn url(&self) -> impl IntoUrl {
        format!(
            "https://resources.download.minecraft.net/{}/{}",
            self.hash.prefix(),
            self.hash.to_string()
        )
    }

    fn size(&self) -> Option<u64> {
        Some(self.size)
    }

    fn hash(&self) -> Option<ResourceHash> {
        Some(self.hash)
    }
}

impl DownloadableFile for AssetObject {
    fn path(&self) -> impl AsRef<std::path::Path> {
        format!("{}/{}", self.hash.prefix(), self.hash.to_string())
    }
}