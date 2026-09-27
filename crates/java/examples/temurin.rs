use std::{fs, path::PathBuf};

use java::{ExecuteParams, JavaDistribution, temurin::TemurinDistribution};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cwd = PathBuf::from(env!("CARGO_MANIFEST_PATH")).join("../examples/").canonicalize()?;
    let path = cwd.join("./temurin_jvm");
    let distribution = TemurinDistribution::new(&path, 25);
    if distribution.is_installed().await? {
        distribution.install().await?;
    }else {
        if path.exists() {
            fs::remove_dir_all(&path)?;
        }
        fs::create_dir_all(&path)?;
    }
    let java = distribution.runtime()?;
    java.execute(cwd, vec!["HelloWorld".into()], ExecuteParams::default())
        .await?;
    Ok(())
}
