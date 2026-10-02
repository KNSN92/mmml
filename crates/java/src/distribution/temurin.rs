use std::path::{Path, PathBuf};

use platform::{Arch, Os};

use crate::{
    JavaRuntime,
    distribution::{
        InstallableJavaDistributionBase, JavaDistribution, JavaDistributionError,
        JavaDistributionResult,
    },
};

pub struct TemurinDistribution(InstallableJavaDistributionBase);

impl TemurinDistribution {
    pub fn new(path: impl AsRef<Path>, version: u8) -> Self {
        Self(InstallableJavaDistributionBase {
            path: path.as_ref().into(),
            version,
        })
    }

    fn bin_path(&self) -> JavaDistributionResult<PathBuf> {
        let path = match Os::current() {
            Os::Windows => self.0.path.join("./bin/java.exe"),
            Os::MacOS => self.0.path.join("./Contents/Home/bin/java"),
            Os::Linux => self.0.path.join("./bin/java"),
            _ => return Err(JavaDistributionError::UnsupportedPlatform),
        };
        Ok(path)
    }
}

impl JavaDistribution for TemurinDistribution {
    fn is_version_supported(version: u8) -> bool {
        !(version == 8 && cfg!(all(target_os = "macos", target_arch = "aarch64")))
    }

    async fn is_installed(&self) -> JavaDistributionResult<bool> {
        Ok(self.bin_path()?.exists())
    }

    async fn install(&self) -> JavaDistributionResult<()> {
        if !Self::is_version_supported(self.0.version) {
            return Err(JavaDistributionError::UnsupportedVersion(self.0.version));
        }
        if self.is_installed().await? {
            return Err(JavaDistributionError::AlreadyInstalled);
        }

        let platform = match Os::current() {
            Os::Windows => "windows",
            Os::MacOS => "mac",
            Os::Linux => "linux",
            _ => return Err(JavaDistributionError::UnsupportedPlatform),
        };
        let arch = match Arch::current() {
            Arch::X86 => "x86",
            Arch::X64 => "x64",
            Arch::Arm => "arm",
            Arch::Aarch64 => "aarch64",
            _ => return Err(JavaDistributionError::UnsupportedArchitecture),
        };
        let url = format!(
            "https://api.adoptium.net/v3/binary/latest/{}/ga/{}/{}/jre/hotspot/normal/eclipse?project=jdk",
            self.0.version, platform, arch,
        );

        self.0.install(url).await
    }

    fn runtime(&self) -> JavaDistributionResult<JavaRuntime> {
        self.0.runtime(self.bin_path()?)
    }
}
