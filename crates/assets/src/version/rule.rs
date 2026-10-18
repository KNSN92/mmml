use std::collections::HashMap;

use platform::{Arch, Os};
use regex::Regex;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct RuleList(Vec<Rule>);

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
struct Rule {
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

type FeatureRule = HashMap<String, bool>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
struct OsRule {
    name: Option<String>,
    version: Option<String>,
    #[allow(unused, reason = "todo")]
    version_range: Option<OsVersionRange>,
    arch: Option<String>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
#[allow(unused, reason = "todo")]
struct OsVersionRange {
    min: Option<String>,
    max: Option<String>,
}

//TODO: Add tests for RuleContext and Rule evaluation~
pub struct RuleContext {
    features: HashMap<String, bool>,
    pub(crate) os: Os,
    pub(crate) arch: Arch,
    #[cfg(test)]
    os_version_override: Option<String>,
}

impl RuleContext {
    pub fn new(features: impl IntoIterator<Item = (String, bool)>) -> Self {
        Self {
            features: features.into_iter().collect(),
            os: Os::current(),
            arch: Arch::current(),
            #[cfg(test)]
            os_version_override: None,
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
            #[cfg(test)]
            os_version_override: None,
        }
    }

    fn os_version(&self) -> Option<&str> {
        #[cfg(test)]
        if let Some(version) = self.os_version_override.as_deref() {
            return Some(version);
        }
        self.os.version()
    }

    #[cfg(test)]
    fn with_os_version(mut self, version: impl Into<String>) -> Self {
        self.os_version_override = Some(version.into());
        self
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

impl RuleList {
    pub fn apply(&self, context: &RuleContext) -> RuleEvalResult<RuleAction> {
        let mut action = RuleAction::Disallow;
        for rule in &self.0 {
            if let Some(rule_action) = rule.apply(context)? {
                action = rule_action;
            }
        }
        Ok(action)
    }
}

impl Rule {
    fn apply(&self, context: &RuleContext) -> RuleEvalResult<Option<RuleAction>> {
        if self.matches(context)? {
            Ok(Some(self.action))
        } else {
            Ok(None)
        }
    }

    fn matches(&self, context: &RuleContext) -> RuleEvalResult<bool> {
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
                .os_version()
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

#[cfg(test)]
mod tests {
    use platform::{Arch, Os};

    use super::{RuleAction, RuleContext, RuleEvalError, RuleList};

    fn rules(json: &str) -> RuleList {
        serde_json::from_str(json).unwrap()
    }

    fn context() -> RuleContext {
        RuleContext::with_platform(Os::Windows, Arch::X64, [("demo".to_string(), true)])
            .with_os_version("10.0.19045")
    }

    #[test]
    fn rule_list_defaults_to_disallow_and_empty_list_stays_disallowed() {
        assert_eq!(rules("[]").apply(&context()).unwrap(), RuleAction::Disallow);
        assert_eq!(
            rules(r#"[{"action":"disallow"}]"#)
                .apply(&context())
                .unwrap(),
            RuleAction::Disallow
        );
    }

    #[test]
    fn last_matching_rule_wins_and_nonmatching_rules_are_ignored() {
        let ctx = context();
        assert_eq!(
            rules(r#"[{"action":"allow"},{"action":"disallow"}]"#)
                .apply(&ctx)
                .unwrap(),
            RuleAction::Disallow
        );
        assert_eq!(
            rules(r#"[{"action":"allow"},{"action":"disallow","os":{"name":"linux"}}]"#)
                .apply(&ctx)
                .unwrap(),
            RuleAction::Allow
        );
        assert_eq!(
            rules(r#"[{"action":"allow","os":{"name":"linux"}}]"#)
                .apply(&ctx)
                .unwrap(),
            RuleAction::Disallow
        );
    }

    #[test]
    fn matches_os_arch_features_and_os_version_regex() {
        let ctx = context();
        assert_eq!(
            rules(r#"[{"action":"allow","features":{"demo":true},"os":{"name":"windows","arch":"x64","version":"10\\.0\\.19.*"}}]"#)
                .apply(&ctx)
                .unwrap(),
            RuleAction::Allow
        );
        assert_eq!(
            rules(r#"[{"action":"allow","os":{"name":"windows","arch":"x86","version":"10\\.0\\.19.*"}}]"#)
                .apply(&ctx)
                .unwrap(),
            RuleAction::Disallow
        );
        assert_eq!(
            rules(r#"[{"action":"allow","os":{"name":"windows","version":"^11\\."}}]"#)
                .apply(&ctx)
                .unwrap(),
            RuleAction::Disallow
        );
        assert_eq!(
            rules(r#"[{"action":"allow","features":{"demo":false}}]"#)
                .apply(&ctx)
                .unwrap(),
            RuleAction::Disallow
        );
    }

    #[test]
    fn reports_unknown_features_os_arch_and_invalid_regex() {
        assert!(matches!(
            rules(r#"[{"action":"allow","features":{"missing":true}}]"#).apply(&context()),
            Err(RuleEvalError::FeatureNotFound(name)) if name == "missing"
        ));
        assert!(matches!(
            rules(r#"[{"action":"allow","os":{"name":"plan9"}}]"#).apply(&context()),
            Err(RuleEvalError::UnknownOsName(name)) if name == "plan9"
        ));
        assert!(matches!(
            rules(r#"[{"action":"allow","os":{"arch":"mips"}}]"#).apply(&context()),
            Err(RuleEvalError::UnknownArch(name)) if name == "mips"
        ));
        assert!(matches!(
            rules(r#"[{"action":"allow","os":{"version":"["}}]"#).apply(&context()),
            Err(RuleEvalError::InvalidRegex(_))
        ));
    }

    #[test]
    fn reports_missing_os_version_when_context_cannot_supply_it() {
        let ctx = RuleContext::with_platform(Os::Other("other"), Arch::X64, []);
        let rule = rules(r#"[{"action":"allow","os":{"version":".*"}}]"#);
        assert!(matches!(
            rule.apply(&ctx),
            Err(RuleEvalError::CannotGetOsVersion)
        ));
    }
}
