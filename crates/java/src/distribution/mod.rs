use std::io;

use thiserror::Error;

use crate::{JavaRuntime, extract::DecompressError};


#[derive(Debug, Error)]
pub enum JavaDistributionError {
    #[error("Unsupported Java version: {0}")]
    UnsupportedVersion(u8),
    #[error("Unsupported Platform")]
    UnsupportedPlatform,
    #[error("Unsupported Architecture")]
    UnsupportedArchitecture,
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
    fn install(&self) -> impl Future<Output = JavaDistributionResult<()>> + Send;
    fn runtime(&self) -> JavaDistributionResult<JavaRuntime>;
}
