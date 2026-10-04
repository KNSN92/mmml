use std::collections::HashMap;

use platform::{Arch, Os};
use regex::Regex;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct Rule {
    pub action: RuleAction,
    pub features: Option<FeatureRule>,
    pub os: Option<OsRule>,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
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
    pub name: Option<String>,
    pub version: Option<String>,
    pub version_range: Option<OsVersionRange>,
    pub arch: Option<String>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct OsVersionRange {
    pub min: Option<String>,
    pub max: Option<String>,
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
    pub fn matches(&self, context: &RuleContext) -> RuleEvalResult<bool> {
        let is_allowed = self.matches_inner(context)?;
        match self.action {
            RuleAction::Allow => Ok(is_allowed),
            RuleAction::Disallow => Ok(!is_allowed),
        }
    }

    fn matches_inner(&self, context: &RuleContext) -> RuleEvalResult<bool> {
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
