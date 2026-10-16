use std::{collections::HashMap, path::PathBuf};

use platform::Os;
use reqwest::IntoUrl;
use serde::Deserialize;
use thiserror::Error;
use url::Url;

use crate::{download::DownloadableResource, hash::ResourceHash, version::rule::{RuleAction, RuleContext, RuleEvalError, RuleList}};

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

impl Libraries {
    pub fn resolve(&self, context: &RuleContext) -> LibraryResolutionResult<Vec<&LibraryArtifact>> {
        let mut artifacts = Vec::new();
        for library in &self.0 {
            artifacts.extend(library.resolve(context)?);
        }
        Ok(artifacts)
    }
}

impl Library {
    pub fn resolve(&self, context: &RuleContext) -> LibraryResolutionResult<Vec<&LibraryArtifact>> {
        let action = match &self.rules {
            Some(rules) => rules.apply(context)?,
            None => RuleAction::Allow,
        };
        if action == RuleAction::Disallow {
            return Ok(Vec::new());
        }

        let mut artifacts = Vec::new();
        if let Some(artifact) = &self.downloads.artifact {
            artifacts.push(artifact);
        }

        let os_name = match context.os {
            Os::Windows => "windows",
            Os::MacOS => "osx",
            Os::Linux => "linux",
            _ => return Err(LibraryResolutionError::UnsupportedOs(context.os)),
        };

        let needed_classifier = self.natives.as_ref().and_then(|classifier| classifier.get(os_name));
        match (needed_classifier, &self.downloads.classifiers) {
            (Some(needed_classifier), Some(classifiers)) => match classifiers.get(needed_classifier) {
                Some(artifact) => artifacts.push(artifact),
                None => return Err(LibraryResolutionError::MissingClassifier(Some(needed_classifier.clone()))),
            },
            (Some(_), None) => return Err(LibraryResolutionError::MissingClassifier(None)),
            _ => {}
        }

        Ok(artifacts)
    }
}
