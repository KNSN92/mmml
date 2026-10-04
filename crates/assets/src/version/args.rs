use std::{collections::HashMap, sync::LazyLock};

use regex::Regex;
use serde::Deserialize;
use thiserror::Error;

use crate::version::rule::{Rule, RuleAction, RuleContext};

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

pub struct ArgumentsContext {
    variables: HashMap<String, String>,
    rule_context: RuleContext,
}

impl ArgumentsContext {
    pub fn new(
        variables: impl IntoIterator<Item = (String, String)>,
        rule_context: RuleContext,
    ) -> Self {
        Self {
            variables: variables.into_iter().collect(),
            rule_context,
        }
    }
}

#[derive(Debug, Error)]
pub enum ArgumentResolutionError {
    #[error("Rule evaluation failed: {0}")]
    RuleEval(#[from] crate::version::rule::RuleEvalError),
    #[error("Variable not found: {0}")]
    VariableNotFound(String),
}

pub type ArgumentResolutionResult<T> = Result<T, ArgumentResolutionError>;

impl Argument {
    pub fn resolve(&self, context: &ArgumentsContext) -> ArgumentResolutionResult<Vec<String>> {
        let (rules, value) = match self {
            Argument::Ruled { rules, value } => (rules, value),
            Argument::Simple(s) => return Ok(vec![resolve_value(s.as_str(), context)?]),
        };
        let action = if let Some(rules) = rules {
            let mut action = RuleAction::Disallow;
            for rule in rules {
                if let Some(rule_action) = rule.apply(&context.rule_context)? {
                    action = rule_action;
                }
            }
            action
        } else {
            RuleAction::Allow
        };
        if action == RuleAction::Allow {
            match value {
                RuledArgumentValue::Single(s) => Ok(vec![resolve_value(s.as_str(), context)?]),
                RuledArgumentValue::Multiple(v) => v
                    .iter()
                    .map(|s| resolve_value(s.as_str(), context))
                    .collect(),
            }
        } else {
            Ok(vec![])
        }
    }
}

static VARIABLE_MATCHER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\$\{([a-zA-Z0-9_]+)\}").unwrap());

fn resolve_value(value: &str, context: &ArgumentsContext) -> ArgumentResolutionResult<String> {
    let mut new = String::with_capacity(value.len());
    let mut last_match = 0;
    for caps in VARIABLE_MATCHER.captures_iter(value) {
        let m = caps.get(0).unwrap();
        new.push_str(&value[last_match..m.start()]);
        let var_name = &caps[1];
        let replacement = context
            .variables
            .get(var_name)
            .ok_or_else(|| ArgumentResolutionError::VariableNotFound(var_name.to_string()))?;
        new.push_str(replacement);
        last_match = m.end();
    }
    new.push_str(&value[last_match..]);
    Ok(new)
}

impl Arguments {
    pub fn resolve_default_user_jvm(
        &self,
        context: &ArgumentsContext,
    ) -> ArgumentResolutionResult<Vec<String>> {
        match &self.default_user_jvm {
            Some(args) => resolve_args(args, context),
            None => Ok(vec![]),
        }
    }

    pub fn resolve_game(
        &self,
        context: &ArgumentsContext,
    ) -> ArgumentResolutionResult<Vec<String>> {
        resolve_args(&self.game, context)
    }

    pub fn resolve_jvm(&self, context: &ArgumentsContext) -> ArgumentResolutionResult<Vec<String>> {
        resolve_args(&self.jvm, context)
    }
}

fn resolve_args(
    args: &[Argument],
    context: &ArgumentsContext,
) -> ArgumentResolutionResult<Vec<String>> {
    args.iter().try_fold(Vec::new(), |mut acc, arg| {
        acc.extend(arg.resolve(context)?);
        Ok(acc)
    })
}
