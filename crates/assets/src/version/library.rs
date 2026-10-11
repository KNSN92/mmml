use std::collections::HashMap;

use reqwest::IntoUrl;
use serde::Deserialize;
use url::Url;

use crate::{download::DownloadableResource, hash::ResourceHash, version::rule::Rule};

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct Libraries(pub Vec<Library>);

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct Library {
    pub name: String,
    pub url: Option<Url>,
    pub downloads: LibraryDownloads,
    pub natives: Option<HashMap<String, String>>,
    pub extract: Option<LibraryExtract>,
    pub rules: Option<Vec<Rule>>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct LibraryDownloads {
    pub artifact: Option<LibraryArtifact>,
    pub classifiers: Option<HashMap<String, LibraryArtifact>>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct LibraryArtifact {
    pub path: String,
    pub sha1: ResourceHash,
    pub size: u64,
    pub url: Url,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct LibraryExtract {
    pub exclude: Vec<String>,
}

impl DownloadableResource for LibraryArtifact {
    fn url(&self) -> impl IntoUrl {
        self.url.clone()
    }

    fn size(&self) -> Option<u64> {
        Some(self.size)
    }

    fn hash(&self) -> Option<ResourceHash> {
        Some(self.sha1)
    }
}
