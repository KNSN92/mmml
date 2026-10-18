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
    variable::{VariableNotFoundError, VariableResolver},
    version::rule::{RuleAction, RuleContext, RuleEvalError, RuleList},
};

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct Libraries(Vec<Library>);

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
    #[error("Missing classifier for OS: {0:?}")]
    MissingClassifier(Option<String>),
    #[error("Rule evaluation error: {0}")]
    RuleEvalError(#[from] RuleEvalError),
    #[error("Variable not found: {0}")]
    VariableNotFound(#[from] VariableNotFoundError),
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
        rule_ctx: &RuleContext,
        var_resolver: &VariableResolver,
    ) -> LibraryResolutionResult<Vec<ResolvedLibrary<'_>>> {
        let mut libraries = Vec::new();
        for library in &self.0 {
            let Some(resolved) = library.resolve(rule_ctx, var_resolver)? else {
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
        rule_ctx: &RuleContext,
        var_resolver: &VariableResolver,
    ) -> LibraryResolutionResult<Option<ResolvedLibrary<'_>>> {
        let action = match &self.rules {
            Some(rules) => rules.apply(rule_ctx)?,
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

        let os_name = match rule_ctx.os {
            Os::Windows => "windows",
            Os::MacOS => "osx",
            Os::Linux => "linux",
            _ => return Ok(Some(artifacts)),
        };
        let Some(needed_classifier) = natives.get(os_name).map(String::as_str) else {
            return Ok(Some(artifacts));
        };
        let needed_classifier = var_resolver.resolve(needed_classifier)?;
        let Some(classifiers) = &self.downloads.classifiers else {
            return Err(LibraryResolutionError::MissingClassifier(None));
        };
        match classifiers.get(&needed_classifier) {
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use platform::{Arch, Os};

    use crate::{variable::VariableResolver, version::rule::RuleContext};

    use super::{Libraries, LibraryResolutionError};

    fn ctx(os: Os) -> RuleContext {
        RuleContext::with_platform(os, Arch::X64, [])
    }

    fn vars() -> VariableResolver {
        VariableResolver::new([("arch".to_string(), "64".to_string())])
    }

    fn parse(json: &str) -> Libraries {
        serde_json::from_str(json).unwrap()
    }

    const ARTIFACT: &str = r#"{"path":"org/example/demo/1/demo-1.jar","sha1":"a9993e364706816aba3e25717850c26c9cd0d89d","size":3,"url":"https://example.invalid/demo.jar"}"#;

    #[test]
    fn resolves_plain_and_os_allowed_libraries_and_retains_extract_metadata() {
        let libraries = parse(&format!(
            r#"[
              {{"name":"org.example:plain:1","downloads":{{"artifact":{ARTIFACT}}}}},
              {{"name":"org.example:linux:1","downloads":{{"artifact":{ARTIFACT}}},"rules":[{{"action":"allow","os":{{"name":"linux"}}}}],"extract":{{"exclude":["META-INF/"]}}}}
            ]"#
        ));
        let resolved = libraries.resolve(&ctx(Os::Linux), &vars()).unwrap();
        assert_eq!(resolved.len(), 2);
        assert_eq!(resolved[1].name, "org.example:linux:1");
        assert_eq!(resolved[1].extract.unwrap().exclude, ["META-INF/"]);
        assert_eq!(
            resolved[1].artifact.unwrap().path,
            Path::new("org/example/demo/1/demo-1.jar")
        );
    }

    #[test]
    fn resolves_native_classifier_and_skips_library_for_nonmatching_rule() {
        let libraries = parse(&format!(
            r#"[
              {{"name":"org.example:native:1","downloads":{{"artifact":{ARTIFACT},"classifiers":{{"natives-linux-64":{ARTIFACT}}}}},"natives":{{"linux":"natives-linux-${{arch}}"}}}},
              {{"name":"org.example:windows:1","downloads":{{"artifact":{ARTIFACT}}},"rules":[{{"action":"allow","os":{{"name":"windows"}}}}]}}
            ]"#
        ));
        let resolved = libraries.resolve(&ctx(Os::Linux), &vars()).unwrap();
        assert_eq!(resolved.len(), 1);
        assert_eq!(
            resolved[0].native.unwrap().path,
            Path::new("org/example/demo/1/demo-1.jar")
        );
    }

    #[test]
    fn unsupported_os_returns_library_without_native_artifact() {
        let libraries = parse(&format!(
            r#"[{{"name":"org.example:native:1","downloads":{{"artifact":{ARTIFACT}}},"natives":{{"linux":"natives-linux-${{arch}}"}}}}]"#
        ));
        let resolved = libraries
            .resolve(&ctx(Os::Other("haiku")), &vars())
            .unwrap();
        assert_eq!(resolved.len(), 1);
        assert!(resolved[0].native.is_none());
    }

    #[test]
    fn reports_missing_classifier_and_unresolved_classifier_variable() {
        let no_classifiers = parse(&format!(
            r#"[{{"name":"org.example:native:1","downloads":{{"artifact":{ARTIFACT}}},"natives":{{"linux":"natives-linux-${{arch}}"}}}}]"#
        ));
        assert!(matches!(
            no_classifiers.resolve(&ctx(Os::Linux), &vars()),
            Err(LibraryResolutionError::MissingClassifier(None))
        ));

        let classifiers = parse(&format!(
            r#"[{{"name":"org.example:native:1","downloads":{{"artifact":{ARTIFACT},"classifiers":{{}}}},"natives":{{"linux":"natives-linux-${{arch}}"}}}}]"#
        ));
        assert!(matches!(
            classifiers.resolve(&ctx(Os::Linux), &vars()),
            Err(LibraryResolutionError::MissingClassifier(Some(_)))
        ));
        let missing_var = VariableResolver::new([]);
        assert!(matches!(
            classifiers.resolve(&ctx(Os::Linux), &missing_var),
            Err(LibraryResolutionError::VariableNotFound(_))
        ));
    }

    #[test]
    fn propagates_invalid_rule_values() {
        let libraries = parse(&format!(
            r#"[{{"name":"org.example:bad:1","downloads":{{"artifact":{ARTIFACT}}},"rules":[{{"action":"allow","os":{{"name":"unknown-os"}}}}]}}]"#
        ));
        assert!(matches!(
            libraries.resolve(&ctx(Os::Linux), &vars()),
            Err(LibraryResolutionError::RuleEvalError(_))
        ));
    }
}
