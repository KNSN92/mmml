use std::{error::Error, fs, path::PathBuf};

use assets::{
    download::ResourceDownloader,
    manifest::VersionManifest,
    version::{library::LibraryArtifact, rule::RuleContext},
};
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let downloader = ResourceDownloader::new(Client::new());
    let version_manifest = downloader.fetch(&VersionManifest::fetch_source()).await?;
    let version_info = downloader
        .fetch(version_manifest.latest_release().unwrap())
        .await?;
    let rule_ctx = RuleContext::new([]);
    let libraries = version_info.libraries.resolve(&rule_ctx)?;
    println!("Resolved artifacts:");
    for library in &libraries {
        if let Some(artifact) = library.artifact {
            print_artifact(artifact, library.name, 0);
        }
        if let Some(native) = library.native {
            let (name, indent) = match library.artifact {
                Some(_) => ("native", 1),
                None => (library.name, 0),
            };
            print_artifact(native, name, indent);
        }
    }
    let libraries_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./examples/libraries/");
    if libraries_dir.exists() {
        fs::remove_dir_all(&libraries_dir).unwrap();
    }
    fs::create_dir_all(&libraries_dir).unwrap();
    let libraries_dir = libraries_dir.canonicalize()?;
    for library in &libraries {
        if let Some(artifact) = library.artifact {
            downloader
                .download_to_file(&libraries_dir, artifact)
                .await?;
        }
        if let Some(artifact) = library.native {
            downloader
                .download_to_file(&libraries_dir, artifact)
                .await?;
        }
    }
    Ok(())
}

fn print_artifact(artifact: &LibraryArtifact, name: impl AsRef<str>, indent: usize) {
    let indent_str = "\t".repeat(indent);
    println!("{indent_str} - {}", name.as_ref());
    let indent_str = "\t".repeat(indent + 1);
    println!("{indent_str} - path = {}", artifact.path.display());
    println!("{indent_str} - url = {}", artifact.url);
    println!("{indent_str} - size = {}", artifact.size);
    println!("{indent_str} - sha1 = {}", artifact.sha1);
}
