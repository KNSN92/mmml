use std::{
    io,
    path::{Path, PathBuf},
    process::{ExitStatus, Stdio},
};

use thiserror::Error;
use tokio::process::{Child, Command};

#[derive(Debug, Error)]
pub enum JavaExecutionError {
    #[error("Java execution binary not found at path: {0}")]
    NotFound(PathBuf),
    #[error("IO error: {0}")]
    IOError(#[from] io::Error),
}

pub type JavaExecutionResult<T> = Result<T, JavaExecutionError>;

pub struct ExecutionParams {
    pub env: Vec<(String, String)>,
    pub stdin: Stdio,
    pub stdout: Stdio,
    pub stderr: Stdio,
}

impl Default for ExecutionParams {
    fn default() -> Self {
        Self {
            env: Vec::new(),
            stdin: Stdio::null(),
            stdout: Stdio::piped(),
            stderr: Stdio::piped(),
        }
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
        params: ExecutionParams,
    ) -> JavaExecutionResult<ExitStatus> {
        if !self.0.exists() {
            return Err(JavaExecutionError::NotFound(self.0.clone()));
        }
        let status = Command::new(&self.0)
            .current_dir(cwd)
            .args(args)
            .envs(params.env)
            .stdin(params.stdin)
            .stdout(params.stdout)
            .stderr(params.stderr)
            .status()
            .await?;
        Ok(status)
    }

    pub fn spawn(
        &self,
        cwd: impl AsRef<Path>,
        args: Vec<String>,
        params: ExecutionParams,
    ) -> JavaExecutionResult<Child> {
        let child = Command::new(&self.0)
            .current_dir(cwd)
            .args(args)
            .envs(params.env)
            .stdin(params.stdin)
            .stdout(params.stdout)
            .stderr(params.stderr)
            .spawn()?;
        Ok(child)
    }
}
