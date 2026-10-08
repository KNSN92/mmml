use std::collections::HashMap;

use serde::Deserialize;
use url::Url;

use crate::{
    hash::ResourceHash,
    version::{args::CompatibleArguments, rule::Rule},
};

pub mod args;
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
    pub libraries: Vec<LibraryInfo>,
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
pub struct LibraryInfo {
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
