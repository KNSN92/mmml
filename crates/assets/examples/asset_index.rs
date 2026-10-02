use std::{error::Error, fs, path::PathBuf};

use assets::asset::AssetIndex;
use assets::{manifest::VersionManifest, version::VersionInfo};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let manifest = VersionManifest::fetch().await?;
    let latest_release = manifest.latest_release().unwrap();
    let base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples");
    fs::create_dir_all(&base_path).unwrap();
    let version_info = reqwest::get(&latest_release.url)
        .await?
        .error_for_status()?
        .json::<VersionInfo>()
        .await?;
    let asset_index = request_asset_index(&version_info.asset_index.url).await?;
    let mut file = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(base_path.join(format!("{}.txt", version_info.asset_index.id)))
        .await?;
    file.write_all(format!("{:#?}", asset_index).as_bytes())
        .await?;
    let base_path = base_path.join("objects");
    if base_path.exists() {
        fs::remove_dir_all(&base_path)?;
    }
    let client = reqwest::Client::new();
    for object in asset_index.objects() {
        let base_path = base_path.join(object.hash.prefix());
        fs::create_dir_all(&base_path)?;
        let mut file = File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(base_path.join(object.hash.to_string()))
            .await?;
        object.fetch(client.clone(), &mut file).await?;
    }
    Ok(())
}

#[cfg(not(feature = "debug"))]
async fn request_asset_index(version_url: &str) -> Result<AssetIndex, Box<dyn Error>> {
    Ok(reqwest::get(version_url)
        .await?
        .error_for_status()?
        .json::<AssetIndex>()
        .await?)
}

#[cfg(feature = "debug")]
async fn request_asset_index(version_url: &str) -> Result<AssetIndex, Box<dyn Error>> {
    let version_info = reqwest::get(version_url)
        .await?
        .error_for_status()?
        .text()
        .await?;
    let mut deserializer = serde_json::Deserializer::from_str(&version_info);
    let version_info: AssetIndex = serde_path_to_error::deserialize(&mut deserializer)
        .map_err(|e| format!("Failed to deserialize asset index for url:{version_url} {e:?}"))?;
    Ok(version_info)
}
