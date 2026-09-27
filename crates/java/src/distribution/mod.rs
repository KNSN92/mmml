use std::{io::{self, Write}, path::{Path, PathBuf}};

use tempfile::{NamedTempFile, TempDir};
use tokio::fs::{self, File};
use thiserror::Error;

use crate::{JavaRuntime, extract::{DecompressError, tar_gz_extract, zip_extract}, platform};

pub mod temurin;

#[derive(Debug, Error)]
pub enum JavaDistributionError {
    #[error("Unsupported Java version: {0}")]
    UnsupportedVersion(u8),
    #[error("Unsupported Platform")]
    UnsupportedPlatform,
    #[error("Unsupported Architecture")]
    UnsupportedArchitecture,
    #[error("Java runtime already installed")]
    AlreadyInstalled,
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("Zip decompression error: {0}")]
    ZipDecompressionError(#[from] DecompressError),
    #[error("IO error: {0}")]
    IOError(#[from] io::Error),
}

pub type JavaDistributionResult<T> = Result<T, JavaDistributionError>;

pub trait JavaDistribution {
    fn is_version_supported(version: u8) -> bool;
    fn is_installed(&self) -> impl Future<Output = JavaDistributionResult<bool>> + Send;
    fn install(&self) -> impl Future<Output = JavaDistributionResult<()>> + Send;
    fn runtime(&self) -> JavaDistributionResult<JavaRuntime>;
}

pub struct InstallableJavaDistributionBase {
    pub path: PathBuf,
    pub version: u8,
}

impl InstallableJavaDistributionBase {
    
    pub async fn install(&self, url: String) -> JavaDistributionResult<()> {
        let mut response = reqwest::get(url).await?.error_for_status()?;
        let mut archive_file = NamedTempFile::new()?;
        while let Some(chunk) = response.chunk().await? {
            archive_file.write_all(&chunk)?;
        }
        let extracted_dir = TempDir::new()?;

        platform! {
            "windows" => zip_extract(File::from_std(archive_file.reopen()?), &extracted_dir, None).await?,
            "macos" => tar_gz_extract(File::from_std(archive_file.reopen()?), &extracted_dir, None).await?,
            "linux" => tar_gz_extract(File::from_std(archive_file.reopen()?), &extracted_dir, None).await?,
            _ => return Err(JavaDistributionError::UnsupportedPlatform)
        }

        let extracted_archive_dir = fs::read_dir(&extracted_dir)
            .await?
            .next_entry()
            .await?
            .ok_or(io::Error::new(
                io::ErrorKind::NotFound,
                "Extracted archive directory not found",
            ))?
            .path();
        fs::rename(extracted_archive_dir, &self.path).await?;

        Ok(())
    }

    pub fn runtime(&self, extra_path: impl AsRef<Path>) -> JavaDistributionResult<JavaRuntime> {
        let path = self.path.join(extra_path);
        Ok(JavaRuntime::new(path))
    }

}