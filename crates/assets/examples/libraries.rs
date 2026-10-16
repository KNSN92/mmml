use std::error::Error;

use assets::{download::ResourceDownloader, manifest::VersionManifest, version::rule::RuleContext};
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let downloader = ResourceDownloader::new(Client::new());
    let version_manifest = downloader.fetch(&VersionManifest::fetch_source()).await?;
    let version_info = downloader.fetch(version_manifest.latest_release().unwrap()).await?;
    let rule_ctx = RuleContext::new([]);
    let artifacts = version_info.libraries.resolve(&rule_ctx)?;
    println!("Resolved artifacts:");
    for artifact in artifacts {
        println!(" - {}", artifact.path.file_name().unwrap().to_string_lossy());
        println!("\t - path = {}", artifact.path.display());
        println!("\t - url = {}", artifact.url);
        println!("\t - size = {}", artifact.size);
        println!("\t - sha1 = {}", artifact.sha1);
    }
    Ok(())
}