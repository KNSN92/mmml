// Special thanks to the Lighty Launcher Lib Developers
// https://github.com/Lighty-Launcher/LightyLauncherLib

use std::path::{Component, Path, PathBuf};

use async_compression::tokio::bufread::GzipDecoder;
use async_zip::tokio::read::seek::ZipFileReader;
use futures::StreamExt;
use tokio::{
    fs::{self},
    io::{self, AsyncRead, AsyncSeek, BufReader},
};
use tokio_tar::Archive as TarArchive;
use tokio_util::compat::{FuturesAsyncReadCompatExt, TokioAsyncReadCompatExt};

#[derive(Debug, thiserror::Error)]
pub enum DecompressError {
    #[error("Zip entry not found: Zip entry index: {0}")]
    EntryNotFound(usize),
    #[error("Absolute path found in zip entry: {0}")]
    AbsolutePath(String),
    #[error("File path traversal detected in zip entry: {0}")]
    PathTraversal(String),
    #[error("Zip entry size too large: {0}")]
    SizeTooLarge(u64),
    #[error("Zip decompression error: {0}")]
    ZipError(#[from] async_zip::error::ZipError),
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}

pub async fn zip_extract(
    file: impl AsyncRead + AsyncSeek + Unpin,
    dest: impl AsRef<Path>,
    max_size: Option<u64>,
) -> Result<(), DecompressError> {
    let dest = fs::canonicalize(dest).await?;
    let file = BufReader::new(file);
    let mut archive = ZipFileReader::new(file.compat()).await?;
    let entries_count = archive.file().entries().len();
    for entry_idx in 0..entries_count {
        let entry = archive
            .file()
            .entries()
            .get(entry_idx)
            .ok_or(DecompressError::EntryNotFound(entry_idx))?;
        let is_dir = entry.dir()?;
        let filesize = entry.uncompressed_size();
        if let Some(max_size) = max_size {
            if filesize > max_size {
                return Err(DecompressError::SizeTooLarge(filesize));
            }
        }
        let filename = entry.filename().as_str()?;
        let path = sanitize_path(filename);
        if path.is_absolute() {
            return Err(DecompressError::AbsolutePath(filename.to_string()));
        }
        let path = norm_path(&dest.join(path));
        if !path.starts_with(&dest) {
            return Err(DecompressError::PathTraversal(
                path.to_string_lossy().to_string(),
            ));
        }
        if is_dir {
            fs::create_dir_all(&path).await?;
        } else {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).await?;
            }
            let mut reader = archive.reader_with_entry(entry_idx).await?.compat();
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&path)
                .await?;
            io::copy(&mut reader, &mut file).await?;
        }
    }
    Ok(())
}

pub async fn tar_gz_extract(
    file: impl AsyncRead + AsyncSeek + Unpin,
    dest: impl AsRef<Path>,
    max_size: Option<u64>,
) -> Result<(), DecompressError> {
    let dest = fs::canonicalize(dest).await?;
    let decoder = GzipDecoder::new(BufReader::new(file));
    let mut ar = TarArchive::new(decoder);
    let mut entries = ar.entries()?;
    while let Some(entry) = entries.next().await {
        let mut entry = entry?;
        let path = entry.path()?.to_path_buf();
        if path.is_absolute() {
            return Err(DecompressError::AbsolutePath(
                path.to_string_lossy().to_string(),
            ));
        }
        let path = dest.join(&path);
        let entry_type = entry.header().entry_type();
        if entry_type.is_symlink() || entry_type.is_hard_link() {
            continue;
        }
        let entry_size = entry.header().size()?;
        if let Some(max_size) = max_size {
            if entry_size > max_size {
                return Err(DecompressError::SizeTooLarge(entry_size));
            }
        }
        if !entry.unpack_in(&dest).await? {
            return Err(DecompressError::PathTraversal(
                path.to_string_lossy().to_string(),
            ));
        }
    }
    Ok(())
}

fn sanitize_path(path: &str) -> PathBuf {
    path.replace('\\', "/")
        .split("/")
        .map(sanitize_filename::sanitize)
        .collect()
}

fn norm_path(path: &Path) -> PathBuf {
    path.components()
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
