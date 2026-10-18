use std::{collections::HashMap, sync::LazyLock};

use regex::Regex;
use thiserror::Error;

pub struct VariableResolver(HashMap<String, String>);

#[derive(Debug, Error)]
#[error("Variable not found: {0}")]
pub struct VariableNotFoundError(String);

pub type VariableResolutionResult<T> = Result<T, VariableNotFoundError>;

static VARIABLE_MATCHER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\$\{([a-zA-Z0-9_]+)\}").unwrap());

impl VariableResolver {
    pub fn new(variables: impl IntoIterator<Item = (String, String)>) -> Self {
        Self(variables.into_iter().collect())
    }

    pub fn resolve(&self, value: impl AsRef<str>) -> VariableResolutionResult<String> {
        let value = value.as_ref();
        let mut new = String::with_capacity(value.len());
        let mut last_match = 0;
        for caps in VARIABLE_MATCHER.captures_iter(value) {
            let m = caps.get(0).unwrap();
            new.push_str(&value[last_match..m.start()]);
            let var_name = &caps[1];
            let replacement = self
                .0
                .get(var_name)
                .ok_or_else(|| VariableNotFoundError(var_name.to_string()))?;
            new.push_str(replacement);
            last_match = m.end();
        }
        new.push_str(&value[last_match..]);
        Ok(new)
    }
}
