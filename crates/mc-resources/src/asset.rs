use std::collections::HashMap;

use reqwest::IntoUrl;
use serde::Deserialize;

use crate::{
    download::{DownloadableFile, DownloadableResource},
    hash::ResourceHash,
};

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
