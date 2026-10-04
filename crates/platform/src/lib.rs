use std::{env::consts, sync::LazyLock};

use os_version::OsVersion;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Windows,
    MacOS,
    Linux,
    Other(&'static str),
}

static CURRENT_OS: LazyLock<Os> = LazyLock::new(|| match consts::OS {
    "windows" => Os::Windows,
    "macos" => Os::MacOS,
    "linux" => Os::Linux,
    _ => Os::Other(consts::OS),
});

static OS_VERSION: LazyLock<Option<String>> = LazyLock::new(|| {
    let os_version = os_version::detect().ok()?;
    match os_version {
        OsVersion::Windows(os) => Some(os.version),
        OsVersion::MacOS(os) => Some(os.version),
        OsVersion::Linux(os) => os.version,
        _ => None,
    }
});

static CURRENT_ARCH: LazyLock<Arch> = LazyLock::new(|| match consts::ARCH {
    "x86" => Arch::X86,
    "x86_64" => Arch::X64,
    "arm" => Arch::Arm,
    "aarch64" => Arch::Aarch64,
    _ => Arch::Other(consts::ARCH),
});

impl Os {
    pub fn current() -> Self {
        *CURRENT_OS
    }

    pub fn version(&self) -> Option<&'static str> {
        // selfと実際のOSが一致しない場合は、バージョン情報を返さない
        if *self != Os::current() {
            return None;
        }
        OS_VERSION.as_deref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arch {
    X86,
    X64,
    Arm,
    Aarch64,
    Other(&'static str),
}

impl Arch {
    pub fn current() -> Self {
        *CURRENT_ARCH
    }
}
