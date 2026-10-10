use std::{error::Error, fs::File, io::Write, path::PathBuf};

use assets::{download::ResourceDownloader, manifest::VersionManifest};
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let downloader = ResourceDownloader::new(Client::new());
    let manifest = downloader.fetch(&VersionManifest::fetch_source()).await?;
    let examples = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("./examples/")
        .canonicalize()?;
    let mut file = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(examples.join("version_manifest.txt"))?;
    file.write_all(format!("{:#?}", manifest).as_bytes())?;
    Ok(())
}
