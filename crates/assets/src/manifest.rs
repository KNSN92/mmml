use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct VersionManifest {
    pub latest: LatestVersion,
    pub versions: Vec<Version>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct LatestVersion {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct Version {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: VersionType,
    pub url: String,
    pub time: String,
    pub release_time: String,
    pub sha1: String,
    pub compliance_level: u32,
}

#[derive(Debug, Deserialize)]
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
}
