use std::io;

use thiserror::Error;

use crate::JavaRuntime;

#[derive(Debug, Error)]
pub enum JavaDistributionError {
    #[error("Unsupported Java version: {0}")]
    UnsupportedVersion(u8),
    #[error("IO error: {0}")]
    IOError(#[from] io::Error),
}

pub type JavaDistributionResult<T> = Result<T, JavaDistributionError>;

pub trait JavaDistribution {
    fn is_version_supported(&self, version: u8) -> bool;
    fn install(&self, version: u8) -> JavaDistributionResult<()>;
    fn runtime(&self, version: u8) -> JavaDistributionResult<JavaRuntime>;
}
