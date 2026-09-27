use std::{io::Write, path::PathBuf};

use tempfile::{NamedTempFile, TempDir};
use tokio::fs::{self, File};

use crate::{
    JavaDistribution, JavaDistributionError, JavaDistributionResult, JavaRuntime,
    extract::{tar_gz_extract, zip_extract},
};

pub struct TemurinDistribution {
    path: PathBuf,
    version: u8,
}

macro_rules! platform {
    { $first_target_os:literal => $first_value:expr, $($target_os:literal => $value:expr,)* _ => $other_value:expr } => {
        if cfg!(target_os = $first_target_os) {
            $first_value
        } $( else if cfg!(target_os = $target_os) {
            $value
        } )* else {
            $other_value
        }
    }
}

macro_rules! arch {
    { $first_target_arch:literal => $first_value:expr, $($target_arch:literal => $value:expr,)* _ => $other_value:expr } => {
        if cfg!(target_arch = $first_target_arch) {
            $first_value
        } $( else if cfg!(target_arch = $target_arch) {
            $value
        } )* else {
            $other_value
        }
    }
}

impl TemurinDistribution {
    pub fn new(path: PathBuf, version: u8) -> Self {
        Self { path, version }
    }
}

impl JavaDistribution for TemurinDistribution {
    fn is_version_supported(version: u8) -> bool {
        !(version == 8 && cfg!(all(target_os = "macos", target_arch = "aarch64")))
    }

    async fn install(&self) -> JavaDistributionResult<()> {
        if !Self::is_version_supported(self.version) {
            return Err(JavaDistributionError::UnsupportedVersion(self.version));
        }

        let platform = platform! {
            "windows" => "windows",
            "macos" => "mac",
            "linux" => "linux",
            _ => return Err(JavaDistributionError::UnsupportedPlatform)
        };
        let arch = arch! {
            "x86" => "x86",
            "x86_64" => "x64",
            "arm" => "arm",
            "aarch64" => "aarch64",
            _ => return Err(JavaDistributionError::UnsupportedArchitecture)
        };
        let url = format!(
            "https://api.adoptium.net/v3/binary/latest/{}/ga/{}/{}/jre/hotspot/normal/eclipse?project=jdk",
            self.version, platform, arch,
        );

        let mut response = reqwest::get(url).await?.error_for_status()?;
        let mut archive_file = NamedTempFile::new()?;
        while let Some(chunk) = response.chunk().await? {
            archive_file.write_all(&chunk)?;
        }
        let extracted_dir = TempDir::new()?;

        platform! {
            "windows" => {
                zip_extract(File::from_std(archive_file.reopen()?), &extracted_dir, None).await?;
            },
            "macos" => {
                tar_gz_extract(File::from_std(archive_file.reopen()?), &extracted_dir, None).await?;
            },
            "linux" => {
                tar_gz_extract(File::from_std(archive_file.reopen()?), &extracted_dir, None).await?;
            },
            _ => return Err(JavaDistributionError::UnsupportedPlatform)
        }

        let extracted_archive_dir = fs::read_dir(&extracted_dir)
            .await?
            .next_entry()
            .await?
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Extracted archive directory not found",
            ))?
            .path();
        fs::rename(extracted_archive_dir, &self.path).await?;

        Ok(())
    }

    fn runtime(&self) -> JavaDistributionResult<JavaRuntime> {
        let binary_path = platform! {
            "windows" => self.path.join("./bin/java.exe"),
            "macos" => self.path.join("./Contents/Home/bin/java"),
            "linux" => self.path.join("./bin/java"),
            _ => return Err(JavaDistributionError::UnsupportedPlatform)
        };
        Ok(JavaRuntime::new(binary_path))
    }
}
