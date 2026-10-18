use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use platform::Os;
use reqwest::IntoUrl;
use serde::Deserialize;
use thiserror::Error;
use url::Url;

use crate::{
    download::{DownloadableFile, DownloadableResource},
    hash::ResourceHash,
    version::rule::{RuleAction, RuleContext, RuleEvalError, RuleList},
};

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct Libraries(pub Vec<Library>);

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct Library {
    pub name: String,
    pub url: Option<Url>,
    pub downloads: LibraryDownloads,
    pub natives: Option<HashMap<String, String>>,
    pub extract: Option<LibraryExtract>,
    pub rules: Option<RuleList>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct LibraryDownloads {
    pub artifact: Option<LibraryArtifact>,
    pub classifiers: Option<HashMap<String, LibraryArtifact>>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct LibraryArtifact {
    pub path: PathBuf,
    pub sha1: ResourceHash,
    pub size: u64,
    pub url: Url,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct LibraryExtract {
    pub exclude: Vec<String>,
}

impl DownloadableResource for LibraryArtifact {
    fn url(&self) -> impl IntoUrl {
        self.url.clone()
    }

    fn size(&self) -> Option<u64> {
        Some(self.size)
    }

    fn hash(&self) -> Option<ResourceHash> {
        Some(self.sha1)
    }
}

impl DownloadableFile for LibraryArtifact {
    fn path(&self) -> impl AsRef<Path> {
        &self.path
    }
}

#[derive(Debug, Error)]
pub enum LibraryResolutionError {
    #[error("Unsupported OS: {0:?}")]
    UnsupportedOs(Os),
    #[error("Missing classifier for OS: {0:?}")]
    MissingClassifier(Option<String>),
    #[error("Rule evaluation error: {0}")]
    RuleEvalError(#[from] RuleEvalError),
}

pub type LibraryResolutionResult<T> = Result<T, LibraryResolutionError>;

pub struct ResolvedLibrary<'a> {
    pub name: &'a str,
    pub url: Option<&'a Url>,
    pub extract: Option<&'a LibraryExtract>,
    pub artifact: Option<&'a LibraryArtifact>,
    pub native: Option<&'a LibraryArtifact>,
}

impl Libraries {
    pub fn resolve(
        &self,
        context: &RuleContext,
    ) -> LibraryResolutionResult<Vec<ResolvedLibrary<'_>>> {
        let mut libraries = Vec::new();
        for library in &self.0 {
            let Some(resolved) = library.resolve(context)? else {
                continue;
            };
            libraries.push(resolved);
        }
        Ok(libraries)
    }
}

impl Library {
    pub fn resolve(
        &self,
        context: &RuleContext,
    ) -> LibraryResolutionResult<Option<ResolvedLibrary<'_>>> {
        let action = match &self.rules {
            Some(rules) => rules.apply(context)?,
            None => RuleAction::Allow,
        };
        if action == RuleAction::Disallow {
            return Ok(None);
        }

        let mut artifacts = ResolvedLibrary {
            name: &self.name,
            url: self.url.as_ref(),
            extract: self.extract.as_ref(),
            artifact: self.downloads.artifact.as_ref(),
            native: None,
        };
        let Some(natives) = &self.natives else {
            return Ok(Some(artifacts));
        };

        let os_name = match context.os {
            Os::Windows => "windows",
            Os::MacOS => "osx",
            Os::Linux => "linux",
            _ => return Err(LibraryResolutionError::UnsupportedOs(context.os)),
        };
        let Some(needed_classifier) = natives.get(os_name).map(String::as_str) else {
            return Ok(Some(artifacts));
        };
        let Some(classifiers) = &self.downloads.classifiers else {
            return Err(LibraryResolutionError::MissingClassifier(None));
        };
        match classifiers.get(needed_classifier) {
            Some(artifact) => artifacts.native = Some(artifact),
            None => {
                return Err(LibraryResolutionError::MissingClassifier(Some(
                    needed_classifier.to_string(),
                )));
            }
        }

        Ok(Some(artifacts))
    }
}
