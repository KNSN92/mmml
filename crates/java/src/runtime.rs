use std::{
    io,
    path::{Path, PathBuf},
};

use thiserror::Error;
use tokio::process::Command;

#[derive(Debug, Error)]
pub enum JavaExecutionError {
    #[error("Java execution binary not found at path: {0}")]
    NotFound(PathBuf),
    #[error("Java execution failed with exit code: {0:?}")]
    Failed(Option<i32>),
    #[error("IO error: {0}")]
    IOError(#[from] io::Error),
}

pub type JavaExecutionResult<T> = Result<T, JavaExecutionError>;

pub struct ExecuteParams {
    env: Vec<(String, String)>,
    //TODO: stdin, stdout, stderrを指定出来るようにする
}

impl Default for ExecuteParams {
    fn default() -> Self {
        Self { env: Vec::new() }
    }
}

pub struct JavaRuntime(PathBuf);

impl JavaRuntime {
    pub fn new(path: impl AsRef<Path>) -> Self {
        JavaRuntime(path.as_ref().into())
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    pub async fn execute(
        &self,
        cwd: impl AsRef<Path>,
        args: Vec<String>,
        params: ExecuteParams,
    ) -> JavaExecutionResult<()> {
        if !self.0.exists() {
            return Err(JavaExecutionError::NotFound(self.0.clone()));
        }
        let status = Command::new(&self.0)
            .current_dir(cwd)
            .args(args)
            .envs(params.env)
            .status()
            .await?;
        if !status.success() {
            return Err(JavaExecutionError::Failed(status.code()));
        }
        Ok(())
    }

    pub fn spawn(
        &self,
        cwd: impl AsRef<Path>,
        args: Vec<String>,
        params: ExecuteParams,
    ) -> JavaExecutionResult<tokio::process::Child> {
        let child = Command::new(&self.0)
            .current_dir(cwd)
            .args(args)
            .envs(params.env)
            .spawn()?;
        Ok(child)
    }
}
