use std::{error::Error, fs, path::PathBuf};

use mc_resources::download::ResourceDownloader;
use mc_resources::manifest::VersionManifest;
use mc_resources::version::VersionInfo;
use reqwest::Client;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let downloader = ResourceDownloader::new(Client::new());
    let manifest = downloader.fetch(&VersionManifest::fetch_source()).await?;
    let latest_release = manifest.latest_release().unwrap();
    let base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples");
    fs::create_dir_all(&base_path).unwrap();
    let version_info = reqwest::get(latest_release.url.clone())
        .await?
        .error_for_status()?
        .json::<VersionInfo>()
        .await?;
    let asset_index = downloader.fetch(&version_info.asset_index).await?;
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
    for object in asset_index.objects() {
        let base_path = base_path.join(object.hash.prefix());
        downloader.download_to_file(base_path, object).await?;
    }
    Ok(())
}
