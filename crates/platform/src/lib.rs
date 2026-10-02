use std::env::consts;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Os {
    Windows,
    MacOS,
    Linux,
    Other(&'static str),
}

impl Os {
    pub fn current() -> Self {
        match consts::OS {
            "windows" => Os::Windows,
            "macos" => Os::MacOS,
            "linux" => Os::Linux,
            _ => Os::Other(consts::OS),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Arch {
    X86,
    X64,
    Arm,
    Aarch64,
    Other(String),
}

impl Arch {
    pub fn current() -> Self {
        match consts::ARCH {
            "x86" => Arch::X86,
            "x86_64" => Arch::X64,
            "arm" => Arch::Arm,
            "aarch64" => Arch::Aarch64,
            other => Arch::Other(other.to_string()),
        }
    }
}
