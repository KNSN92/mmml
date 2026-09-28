use std::{fs, path::PathBuf};

use java::{ExecutionParams, distribution::{JavaDistribution, TemurinDistribution}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cwd = PathBuf::from(env!("CARGO_MANIFEST_PATH")).join("../examples/").canonicalize()?;
    let path = cwd.join("./temurin_jvm");
    let distribution = TemurinDistribution::new(&path, 25);
    if !distribution.is_installed().await? {
        distribution.install().await?;
    }
    let java = distribution.runtime()?;
    java.execute(cwd, vec!["HelloWorld".into()], ExecutionParams::default())
        .await?;
    Ok(())
}
