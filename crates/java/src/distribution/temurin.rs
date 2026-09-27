use std::path::Path;

use crate::{
    InstallableJavaDistributionBase,
    JavaDistribution,
    JavaDistributionError,
    JavaDistributionResult,
    JavaRuntime,
    arch,
    platform,
};

pub struct TemurinDistribution(InstallableJavaDistributionBase);

impl TemurinDistribution {
    pub fn new(path: impl AsRef<Path>, version: u8) -> Self {
        Self(InstallableJavaDistributionBase { path: path.as_ref().into(), version })
    }
}

impl JavaDistribution for TemurinDistribution {
    fn is_version_supported(version: u8) -> bool {
        !(version == 8 && cfg!(all(target_os = "macos", target_arch = "aarch64")))
    }

    async fn is_installed(&self) -> JavaDistributionResult<bool> {
        let binary_path = platform! {
            "windows" => self.0.path.join("./bin/java.exe"),
            "macos" => self.0.path.join("./Contents/Home/bin/java"),
            "linux" => self.0.path.join("./bin/java"),
            _ => return Err(JavaDistributionError::UnsupportedPlatform)
        };
        Ok(binary_path.exists())
    }

    async fn install(&self) -> JavaDistributionResult<()> {
        if !Self::is_version_supported(self.0.version) {
            return Err(JavaDistributionError::UnsupportedVersion(self.0.version));
        }
        if self.is_installed().await? {
            return Err(JavaDistributionError::AlreadyInstalled);
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
            self.0.version, platform, arch,
        );

        self.0.install(url).await
    }

    fn runtime(&self) -> JavaDistributionResult<JavaRuntime> {
        let extra_path = platform! {
            "windows" => "./bin/java.exe",
            "macos" => "./Contents/Home/bin/java",
            "linux" => "./bin/java",
            _ => return Err(JavaDistributionError::UnsupportedPlatform)
        };
        self.0.runtime(extra_path)
    }
}
