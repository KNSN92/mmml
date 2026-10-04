use std::collections::HashMap;

use platform::{Arch, Os};
use regex::Regex;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct Rule {
    action: RuleAction,
    features: Option<FeatureRule>,
    os: Option<OsRule>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub enum RuleAction {
    Allow,
    Disallow,
}

pub type FeatureRule = HashMap<String, bool>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct OsRule {
    name: Option<String>,
    version: Option<String>,
    #[allow(unused, reason = "todo")]
    version_range: Option<OsVersionRange>,
    arch: Option<String>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
#[allow(unused, reason = "todo")]
pub struct OsVersionRange {
    min: Option<String>,
    max: Option<String>,
}

//TODO: Add tests for RuleContext and Rule evaluation~
pub struct RuleContext {
    features: HashMap<String, bool>,
    os: Os,
    arch: Arch,
}

impl RuleContext {
    pub fn new(features: impl IntoIterator<Item = (String, bool)>) -> Self {
        Self {
            features: features.into_iter().collect(),
            os: Os::current(),
            arch: Arch::current(),
        }
    }

    pub fn with_platform(
        os: Os,
        arch: Arch,
        features: impl IntoIterator<Item = (String, bool)>,
    ) -> Self {
        Self {
            features: features.into_iter().collect(),
            os,
            arch,
        }
    }
}

#[derive(Debug, Error)]
pub enum RuleEvalError {
    #[error("Feature not found in context: {0}")]
    FeatureNotFound(String),
    #[error("Unknown OS in rule: {0}")]
    UnknownOsName(String),
    #[error("Cannot get OS version for the current platform")]
    CannotGetOsVersion,
    #[error("Unknown architecture in rule: {0}")]
    UnknownArch(String),
    #[error("Invalid regex in rule: {0}")]
    InvalidRegex(#[from] regex::Error),
}

pub type RuleEvalResult<T> = std::result::Result<T, RuleEvalError>;

impl Rule {
    pub fn apply(&self, context: &RuleContext) -> RuleEvalResult<Option<RuleAction>> {
        if self.matches(context)? {
            Ok(Some(self.action))
        } else {
            Ok(None)
        }
    }

    pub fn matches(&self, context: &RuleContext) -> RuleEvalResult<bool> {
        if let Some(expected_features) = &self.features {
            for (feature, &expected) in expected_features.iter() {
                let actual = *context
                    .features
                    .get(feature)
                    .ok_or_else(|| RuleEvalError::FeatureNotFound(feature.clone()))?;
                if actual != expected {
                    return Ok(false);
                }
            }
        }
        if let Some(os_rule) = &self.os {
            if !os_rule.matches(context)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

impl OsRule {
    fn matches(&self, context: &RuleContext) -> RuleEvalResult<bool> {
        if let Some(name) = self.name.as_deref() {
            let expected_os = match name {
                "windows" => Os::Windows,
                "osx" => Os::MacOS,
                "linux" => Os::Linux,
                _ => return Err(RuleEvalError::UnknownOsName(name.to_string())),
            };
            if context.os != expected_os {
                return Ok(false);
            }
        }
        if let Some(version) = self.version.as_deref() {
            let actual_version = context
                .os
                .version()
                .ok_or_else(|| RuleEvalError::CannotGetOsVersion)?;
            if !Regex::new(version)?.is_match(actual_version) {
                return Ok(false);
            }
        }
        //TODO: Implement version_range matching
        if let Some(arch) = self.arch.as_deref() {
            let expected_arch = match arch {
                "x86" => Arch::X86,
                "x64" => Arch::X64,
                "arm" => Arch::Arm,
                "aarch64" => Arch::Aarch64,
                _ => return Err(RuleEvalError::UnknownArch(arch.to_string())),
            };
            if context.arch != expected_arch {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
