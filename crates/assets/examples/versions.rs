use std::{
    error::Error,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use assets::{
    manifest::{VersionManifest, VersionType},
    version::VersionInfo,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let manifest = VersionManifest::fetch().await?;
    let base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/versions");
    if base_path.exists() {
        fs::remove_dir_all(&base_path).unwrap();
    }
    fs::create_dir_all(&base_path).unwrap();
    let base_path = base_path.canonicalize().unwrap();
    for (version, version_url) in manifest
        .versions()
        .iter()
        .filter(|v| v.version_type != VersionType::Snapshot)
        .map(|v| (&v.id, &v.url))
    {
        let version_info = request_version_info(version, version_url).await?;
        let mut file = File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(base_path.join(format!("{}.txt", version)))
            .unwrap();
        file.write_all(format!("{:#?}", version_info).as_bytes())?;
    }
    Ok(())
}

#[cfg(not(feature = "debug"))]
async fn request_version_info(
    _version: &str,
    version_url: &str,
) -> Result<VersionInfo, Box<dyn Error>> {
    Ok(reqwest::get(version_url)
        .await?
        .error_for_status()?
        .json::<VersionInfo>()
        .await?)
}

#[cfg(feature = "debug")]
async fn request_version_info(
    version: &str,
    version_url: &str,
) -> Result<VersionInfo, Box<dyn Error>> {
    let version_info = reqwest::get(version_url)
        .await?
        .error_for_status()?
        .text()
        .await?;
    let mut deserializer = serde_json::Deserializer::from_str(&version_info);
    let version_info: VersionInfo =
        serde_path_to_error::deserialize(&mut deserializer).map_err(|e| {
            format!(
                "Failed to deserialize version info for version {version} url:{version_url} {e:?}"
            )
        })?;
    Ok(version_info)
}
