use serde::Deserialize;
use thiserror::Error;
use url::Url;

use crate::hash::FileHash;

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
    pub sha1: FileHash,
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

#[derive(Debug, Error)]
pub enum VersionManifestError {
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
}

pub type VersionManifestResult<T> = Result<T, VersionManifestError>;

const VERSION_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

impl VersionManifest {
    pub async fn fetch() -> VersionManifestResult<Self> {
        Self::fetch_with_url(VERSION_MANIFEST_URL).await
    }

    pub async fn fetch_with_url(url: &str) -> VersionManifestResult<Self> {
        let response = reqwest::get(url).await?;
        let response = response.error_for_status()?;
        let manifest = response.json::<VersionManifest>().await?;
        Ok(manifest)
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
