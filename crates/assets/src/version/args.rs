use serde::Deserialize;
use thiserror::Error;

use crate::{
    variable::{VariableNotFoundError, VariableResolver},
    version::rule::{RuleAction, RuleContext, RuleEvalError, RuleList},
};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub enum CompatibleArguments {
    Text(String),
    Structured(Arguments),
}

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
        rules: Option<RuleList>,
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

#[derive(Debug, Error)]
pub enum ArgumentResolutionError {
    #[error("Rule evaluation failed: {0}")]
    RuleEval(#[from] RuleEvalError),
    #[error("Variable not found: {0}")]
    VariableNotFound(#[from] VariableNotFoundError),
}

pub type ArgumentResolutionResult<T> = Result<T, ArgumentResolutionError>;

impl Argument {
    pub fn resolve(
        &self,
        rule_ctx: &RuleContext,
        var_resolver: &VariableResolver,
    ) -> ArgumentResolutionResult<Vec<String>> {
        let (rules, value) = match self {
            Argument::Ruled { rules, value } => (rules, value),
            Argument::Simple(s) => return Ok(vec![var_resolver.resolve(s)?]),
        };
        let action = match rules {
            Some(rules) => rules.apply(rule_ctx)?,
            None => RuleAction::Allow,
        };
        if action == RuleAction::Allow {
            match value {
                RuledArgumentValue::Single(s) => Ok(vec![var_resolver.resolve(s)?]),
                RuledArgumentValue::Multiple(v) => v
                    .iter()
                    .map(|s| {
                        var_resolver
                            .resolve(s.as_str())
                            .map_err(ArgumentResolutionError::VariableNotFound)
                    })
                    .collect(),
            }
        } else {
            Ok(vec![])
        }
    }
}

#[derive(Debug)]
pub struct ResolvedArguments {
    pub default_user_jvm: Vec<String>,
    pub game: Vec<String>,
    pub jvm: Vec<String>,
}

impl Arguments {
    pub fn resolve_default_user_jvm(
        &self,
        rule_ctx: &RuleContext,
        var_resolver: &VariableResolver,
    ) -> ArgumentResolutionResult<Vec<String>> {
        match &self.default_user_jvm {
            Some(args) => resolve_args(args, rule_ctx, var_resolver),
            None => Ok(vec![]),
        }
    }

    pub fn resolve(
        &self,
        rule_ctx: &RuleContext,
        var_resolver: &VariableResolver,
    ) -> ArgumentResolutionResult<ResolvedArguments> {
        Ok(ResolvedArguments {
            default_user_jvm: self.resolve_default_user_jvm(rule_ctx, var_resolver)?,
            game: self.resolve_game(rule_ctx, var_resolver)?,
            jvm: self.resolve_jvm(rule_ctx, var_resolver)?,
        })
    }

    pub fn resolve_game(
        &self,
        rule_ctx: &RuleContext,
        var_resolver: &VariableResolver,
    ) -> ArgumentResolutionResult<Vec<String>> {
        resolve_args(&self.game, rule_ctx, var_resolver)
    }

    pub fn resolve_jvm(
        &self,
        rule_ctx: &RuleContext,
        var_resolver: &VariableResolver,
    ) -> ArgumentResolutionResult<Vec<String>> {
        resolve_args(&self.jvm, rule_ctx, var_resolver)
    }
}

fn resolve_args(
    args: &[Argument],
    rule_ctx: &RuleContext,
    var_resolver: &VariableResolver,
) -> ArgumentResolutionResult<Vec<String>> {
    args.iter().try_fold(Vec::new(), |mut acc, arg| {
        acc.extend(arg.resolve(rule_ctx, var_resolver)?);
        Ok(acc)
    })
}

impl CompatibleArguments {
    pub fn resolve(
        &self,
        rule_ctx: &RuleContext,
        var_resolver: &VariableResolver,
    ) -> ArgumentResolutionResult<ResolvedArguments> {
        let args = match self {
            CompatibleArguments::Structured(args) => return args.resolve(rule_ctx, var_resolver),
            CompatibleArguments::Text(text) => text.as_str(),
        };
        let args = args
            .split_whitespace()
            .map(|arg| {
                var_resolver
                    .resolve(arg)
                    .map_err(ArgumentResolutionError::VariableNotFound)
            })
            .collect::<ArgumentResolutionResult<Vec<String>>>()?;
        Ok(ResolvedArguments {
            default_user_jvm: vec![],
            game: args,
            jvm: vec![],
        })
    }
}
