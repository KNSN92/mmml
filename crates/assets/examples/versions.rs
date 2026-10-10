use std::{
    error::Error,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use assets::{
    download::ResourceDownloader,
    manifest::{VersionManifest, VersionType},
};
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let downloader = ResourceDownloader::new(Client::new());
    let manifest = downloader.fetch(&VersionManifest::fetch_source()).await?;
    let base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/versions");
    if base_path.exists() {
        fs::remove_dir_all(&base_path).unwrap();
    }
    fs::create_dir_all(&base_path).unwrap();
    let base_path = base_path.canonicalize().unwrap();
    for version in manifest
        .versions()
        .iter()
        .filter(|v| v.version_type != VersionType::Snapshot)
    {
        let version_info = downloader.fetch(version).await?;
        let mut file = File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(base_path.join(format!("{}.txt", version_info.id)))
            .unwrap();
        file.write_all(format!("{:#?}", version_info).as_bytes())?;
    }
    Ok(())
}
