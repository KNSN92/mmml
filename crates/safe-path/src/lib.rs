//TODO: 要テスト追加絶対！

use std::path::{Component, Path, PathBuf};

use thiserror::Error;

pub trait SanitizeNew {
    fn sanitize_new(path: impl AsRef<str>) -> Self;
}

impl SanitizeNew for PathBuf {
    fn sanitize_new(path: impl AsRef<str>) -> Self {
        path.as_ref()
            .replace('\\', "/")
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
        self.as_ref()
            .components()
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

        let mut normalized = PathBuf::new();
        let mut depth = 0usize;
        for component in path.components() {
            match component {
                Component::Normal(segment) => {
                    normalized.push(segment);
                    depth += 1;
                }
                Component::CurDir => {}
                Component::ParentDir => {
                    if depth == 0 {
                        return Err(SafeJoinError::PathTraversal);
                    }
                    normalized.pop();
                    depth -= 1;
                }
                Component::RootDir | Component::Prefix(_) => {
                    return Err(SafeJoinError::AbsolutePath);
                }
            }
        }
        Ok(base.join(normalized))
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{NormalizePath, SafeJoin, SafeJoinError};

    #[test]
    fn normalizes_dot_segments_without_leaving_the_base() {
        assert_eq!(
            Path::new("base").safe_join("./a/../b").unwrap(),
            Path::new("base/b")
        );
    }

    #[test]
    fn rejects_parent_traversal_at_any_depth() {
        assert!(matches!(
            Path::new("base").safe_join("../escape"),
            Err(SafeJoinError::PathTraversal)
        ));
        assert!(matches!(
            Path::new("base").safe_join("a/../../escape"),
            Err(SafeJoinError::PathTraversal)
        ));
    }

    #[test]
    fn rejects_absolute_paths_and_preserves_existing_normalizer() {
        assert!(matches!(
            Path::new("base").safe_join("/absolute"),
            Err(SafeJoinError::AbsolutePath)
        ));
        assert_eq!(Path::new("a/../b").norm_path(), Path::new("b"));
    }
}
