use std::path::{Path, PathBuf};

use platform::{Arch, Os};

use crate::distribution::{
    InstallableJavaDistributionBase, JavaDistribution, JavaDistributionError,
    JavaDistributionResult,
};

pub struct GraalVMDistribution(InstallableJavaDistributionBase);

impl GraalVMDistribution {
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

impl JavaDistribution for GraalVMDistribution {
    fn is_version_supported(version: u8) -> bool {
        version >= 17
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

        let (platform, file_ext) = match Os::current() {
            Os::Windows => ("windows", "zip"),
            Os::MacOS => ("macos", "tar.gz"),
            Os::Linux => ("linux", "tar.gz"),
            _ => return Err(JavaDistributionError::UnsupportedPlatform),
        };
        let arch = match Arch::current() {
            Arch::X86 => "x86",
            Arch::X64 => "x64",
            Arch::Arm => "arm",
            Arch::Aarch64 => "aarch64",
            _ => return Err(JavaDistributionError::UnsupportedArchitecture),
        };

        let url = if self.0.version > 17 {
            format!(
                "https://download.oracle.com/graalvm/{}/latest/graalvm-jdk-{}_{}-{}_bin.{}",
                self.0.version, self.0.version, platform, arch, file_ext
            )
        } else if self.0.version == 17 {
            format!(
                "https://download.oracle.com/graalvm/17/archive/graalvm-jdk-17.0.12_{}-{}_bin.{}",
                platform, arch, file_ext
            )
        } else {
            return Err(JavaDistributionError::UnsupportedVersion(self.0.version));
        };
        self.0.install(url).await
    }

    fn runtime(&self) -> JavaDistributionResult<crate::JavaRuntime> {
        self.0.runtime(self.bin_path()?)
    }
}
