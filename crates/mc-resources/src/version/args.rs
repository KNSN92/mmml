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

#[cfg(test)]
mod tests {
    use platform::{Arch, Os};

    use crate::{variable::VariableResolver, version::rule::RuleContext};

    use super::{ArgumentResolutionError, CompatibleArguments};

    fn ctx() -> RuleContext {
        RuleContext::with_platform(Os::Linux, Arch::X64, [("demo".to_string(), false)])
    }

    fn vars() -> VariableResolver {
        VariableResolver::new([
            ("name".to_string(), "Player One".to_string()),
            ("version".to_string(), "26.1".to_string()),
        ])
    }

    #[test]
    fn resolves_structured_arguments_and_default_user_jvm() {
        let arguments: CompatibleArguments = serde_json::from_str(
            r#"{
              "default-user-jvm":["-Xmx4G"],
              "game":["--username", "${name}", {"rules":[{"action":"allow","os":{"name":"linux"}}],"value":["--linux", "yes"]}, {"rules":[{"action":"allow","features":{"demo":false}}],"value":"--not-demo"}],
              "jvm":["-Dversion=${version}"]
            }"#,
        )
        .unwrap();
        let resolved = arguments.resolve(&ctx(), &vars()).unwrap();
        assert_eq!(resolved.default_user_jvm, ["-Xmx4G"]);
        assert_eq!(
            resolved.game,
            ["--username", "Player One", "--linux", "yes", "--not-demo"]
        );
        assert_eq!(resolved.jvm, ["-Dversion=26.1"]);
    }

    #[test]
    fn resolves_legacy_text_by_whitespace_before_variable_replacement() {
        let arguments: CompatibleArguments =
            serde_json::from_str(r#""--username ${name} --version ${version}""#).unwrap();
        let resolved = arguments.resolve(&ctx(), &vars()).unwrap();
        assert!(resolved.default_user_jvm.is_empty());
        assert!(resolved.jvm.is_empty());
        assert_eq!(
            resolved.game,
            ["--username", "Player One", "--version", "26.1"]
        );
    }

    #[test]
    fn omits_optional_default_user_jvm_when_absent() {
        let arguments: CompatibleArguments =
            serde_json::from_str(r#"{"game":[],"jvm":[]}"#).unwrap();
        assert!(
            arguments
                .resolve(&ctx(), &vars())
                .unwrap()
                .default_user_jvm
                .is_empty()
        );
    }

    #[test]
    fn propagates_missing_variable_and_rule_context_errors() {
        let arguments: CompatibleArguments =
            serde_json::from_str(r#"{"game":["${missing}"],"jvm":[]}"#).unwrap();
        assert!(matches!(
            arguments.resolve(&ctx(), &vars()),
            Err(ArgumentResolutionError::VariableNotFound(_))
        ));

        let arguments: CompatibleArguments = serde_json::from_str(
            r#"{"game":[{"rules":[{"action":"allow","features":{"unknown":true}}],"value":"--flag"}],"jvm":[]}"#,
        )
        .unwrap();
        assert!(matches!(
            arguments.resolve(&ctx(), &vars()),
            Err(ArgumentResolutionError::RuleEval(_))
        ));
    }
}
