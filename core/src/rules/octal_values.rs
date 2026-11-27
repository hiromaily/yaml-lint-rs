//! Octal notation restrictions.

use super::{
    LintContext, Rule,
    tokens::{has_tag, problem, scan},
};
use crate::problem::LintProblem;
use serde::Deserialize;
use yaml_rust2::scanner::{TScalarStyle::Plain, Token, TokenType::Scalar};

/// Controls implicit and explicit octal notation.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct OctalValuesRule {
    forbid_implicit_octal: bool,
    forbid_explicit_octal: bool,
}

impl Default for OctalValuesRule {
    fn default() -> Self {
        Self {
            forbid_implicit_octal: true,
            forbid_explicit_octal: true,
        }
    }
}

fn octal_digits(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|digit| matches!(digit, b'0'..=b'7'))
}

impl Rule for OctalValuesRule {
    fn name(&self) -> &'static str {
        "octal-values"
    }

    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        let tokens = scan(&context.content);
        tokens
            .iter()
            .enumerate()
            .filter_map(|(index, Token(mark, token))| {
                let Scalar(Plain, value) = token else {
                    return None;
                };
                if has_tag(&tokens, index) {
                    return None;
                }
                let kind = if self.forbid_implicit_octal
                    && value.strip_prefix('0').is_some_and(octal_digits)
                {
                    "implicit"
                } else if self.forbid_explicit_octal
                    && value.strip_prefix("0o").is_some_and(octal_digits)
                {
                    "explicit"
                } else {
                    return None;
                };
                let mut diagnostic = problem(
                    *mark,
                    self.name(),
                    format!("forbidden {kind} octal value \"{value}\""),
                );
                diagnostic.column += value.len();
                Some(diagnostic)
            })
            .collect()
    }
}
