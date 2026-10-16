//TODO: 要テスト追加絶対！

use std::path::{Component, Path, PathBuf};

use thiserror::Error;

pub trait SanitizeNew {
    fn sanitize_new(path: impl AsRef<str>) -> Self;
}

impl SanitizeNew for PathBuf {
    fn sanitize_new(path: impl AsRef<str>) -> Self {
        path.as_ref().replace('\\', "/")
            .split("/")
            .map(sanitize_filename::sanitize)
            .collect()
    }
}

pub trait NormalizePath {
    fn norm_path(&self) -> PathBuf;
}

impl<T: AsRef<Path>> NormalizePath for T {
    fn norm_path(&self) -> PathBuf {
        self.as_ref().components()
            .fold(PathBuf::new(), |mut path, component| {
                match component {
                    Component::Normal(seg) => path.push(seg),
                    Component::ParentDir => {
                        path.pop();
                    }
                    Component::CurDir => {}
                    _ => path.push(component),
                };
                path
            })
    }
}

#[derive(Debug, Error)]
pub enum SafeJoinError {
    #[error("Absolute path found")]
    AbsolutePath,
    #[error("File path traversal detected")]
    PathTraversal,
}

pub trait SafeJoin {
    fn safe_join(&self, path: impl AsRef<Path>) -> Result<PathBuf, SafeJoinError>;
}

impl<T: AsRef<Path>> SafeJoin for T {
    fn safe_join(&self, path: impl AsRef<Path>) -> Result<PathBuf, SafeJoinError> {
        let base = self.as_ref();
        let path = path.as_ref();
        if path.is_absolute() {
            return Err(SafeJoinError::AbsolutePath);
        }
        let path = base.join(path);
        if !path.starts_with(&base) {
            return Err(SafeJoinError::PathTraversal);
        }
        Ok(path)
    }
}