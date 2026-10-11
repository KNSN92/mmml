use reqwest::IntoUrl;
use serde::Deserialize;
use url::Url;

use crate::{
    asset::AssetIndex,
    download::{DownloadableManifest, DownloadableResource},
    hash::ResourceHash,
    version::{args::CompatibleArguments, library::Libraries},
};

pub mod args;
pub mod library;
pub mod rule;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct VersionInfo {
    #[serde(alias = "minecraftArguments")]
    pub arguments: Option<CompatibleArguments>,
    pub asset_index: AssetInfo,
    pub assets: String,
    pub compliance_level: Option<u32>,
    pub downloads: Downloads,
    pub id: String,
    pub java_version: Option<JavaVersion>,
    pub libraries: Libraries,
    pub logging: Option<Loggings>,
    pub main_class: String,
    pub minimum_launcher_version: u32,
    pub release_time: String,
    pub time: String,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct AssetInfo {
    pub id: String,
    pub sha1: ResourceHash,
    pub size: u64,
    pub total_size: u64,
    pub url: Url,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct Downloads {
    pub client: DownloadInfo,
    pub client_mappings: Option<DownloadInfo>,
    pub server: Option<DownloadInfo>,
    pub server_mappings: Option<DownloadInfo>,
    pub windows_server: Option<DownloadInfo>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct DownloadInfo {
    pub sha1: ResourceHash,
    pub size: u64,
    pub url: Url,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct JavaVersion {
    pub component: String,
    pub major_version: u32,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct Loggings {
    pub client: LoggingInfo,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct LoggingInfo {
    pub argument: String,
    pub file: LoggingFile,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct LoggingFile {
    pub id: String,
    pub sha1: ResourceHash,
    pub size: u64,
    pub url: Url,
}

impl DownloadableManifest for AssetInfo {
    type Resource = AssetIndex;

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

impl DownloadableResource for DownloadInfo {
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

impl DownloadableResource for LoggingFile {
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
