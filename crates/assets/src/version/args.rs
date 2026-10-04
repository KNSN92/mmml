use serde::Deserialize;

use crate::version::rule::Rule;


#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct Arguments {
    #[serde(rename = "default-user-jvm")]
    pub default_user_jvm: Option<Vec<Argument>>,
    pub game: Vec<Argument>,
    pub jvm: Vec<Argument>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub enum Argument {
    Simple(String),
    Ruled {
        rules: Option<Vec<Rule>>,
        value: RuledArgumentValue,
    },
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub enum RuledArgumentValue {
    Single(String),
    Multiple(Vec<String>),
}
