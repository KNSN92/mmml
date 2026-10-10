use reqwest::IntoUrl;
use serde::Deserialize;
use url::Url;

use crate::{download::DownloadableManifest, hash::ResourceHash, version::VersionInfo};

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct VersionManifest {
    latest: LatestVersion,
    versions: Vec<Version>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct LatestVersion {
    release: String,
    snapshot: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct Version {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: VersionType,
    pub url: Url,
    pub time: String,
    pub release_time: String,
    pub sha1: ResourceHash,
    pub compliance_level: u32,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub enum VersionType {
    Release,
    Snapshot,
    OldAlpha,
    OldBeta,
}

impl VersionManifest {
    pub fn fetch_source() -> impl DownloadableManifest<Resource = Self> {
        VersionManifestSource
    }

    pub fn latest_version(&self) -> &LatestVersion {
        &self.latest
    }

    pub fn latest_release(&self) -> Option<&Version> {
        let latest_id = &self.latest.release;
        self.versions.iter().find(|v| &v.id == latest_id)
    }

    pub fn latest_snapshot(&self) -> Option<&Version> {
        let latest_id = &self.latest.snapshot;
        self.versions.iter().find(|v| &v.id == latest_id)
    }

    pub fn find_version(&self, version_id: &str) -> Option<&Version> {
        self.versions.iter().find(|v| v.id == version_id)
    }

    pub fn versions(&self) -> &[Version] {
        &self.versions
    }
}

impl DownloadableManifest for Version {
    type Resource = VersionInfo;

    fn url(&self) -> impl IntoUrl {
        self.url.clone()
    }

    fn size(&self) -> Option<u64> {
        None
    }

    fn hash(&self) -> Option<ResourceHash> {
        Some(self.sha1)
    }
}

struct VersionManifestSource;

impl DownloadableManifest for VersionManifestSource {
    type Resource = VersionManifest;

    fn url(&self) -> impl IntoUrl {
        "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json"
    }

    fn size(&self) -> Option<u64> {
        None
    }

    fn hash(&self) -> Option<ResourceHash> {
        None
    }
}
