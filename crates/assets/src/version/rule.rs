use std::collections::HashMap;

use serde::Deserialize;

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
