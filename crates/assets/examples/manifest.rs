use std::{error::Error, fs::File, io::Write, path::PathBuf};

use assets::manifest::VersionManifest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let manifest = VersionManifest::fetch().await?;
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
