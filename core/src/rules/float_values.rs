//! Style constraints for plain floating-point scalars.

use super::{
    LintContext, Rule,
    tokens::{has_tag, problem, scan},
};
use crate::problem::LintProblem;
use regex::Regex;
use serde::Deserialize;
use std::sync::LazyLock;
use yaml_rust2::scanner::{TScalarStyle::Plain, Token, TokenType::Scalar};

static DECIMAL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[+-]?\.[0-9]+(?:[eE][+-]?[0-9]+)?$").expect("valid decimal pattern")
});
static SCIENTIFIC: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[+-]?(?:\.[0-9]+|[0-9]+(?:\.[0-9]*)?)[eE][+-]?[0-9]+$")
        .expect("valid scientific pattern")
});

/// Checks configured restrictions on floating-point notation.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct FloatValuesRule {
    require_numeral_before_decimal: bool,
    forbid_scientific_notation: bool,
    forbid_nan: bool,
    forbid_inf: bool,
}

impl Rule for FloatValuesRule {
    fn name(&self) -> &'static str {
        "float-values"
    }

    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        let tokens = scan(&context.content);
        let mut problems = Vec::new();
        for (index, Token(mark, token)) in tokens.iter().enumerate() {
            let Scalar(Plain, value) = token else {
                continue;
            };
            if has_tag(&tokens, index) {
                continue;
            }
            let checks = [
                (
                    self.forbid_nan && matches!(value.as_str(), ".nan" | ".NaN" | ".NAN"),
                    "forbidden not a number value",
                ),
                (
                    self.forbid_inf
                        && matches!(
                            value.trim_start_matches(['+', '-']),
                            ".inf" | ".Inf" | ".INF"
                        ),
                    "forbidden infinite value",
                ),
                (
                    self.forbid_scientific_notation && SCIENTIFIC.is_match(value),
                    "forbidden scientific notation",
                ),
                (
                    self.require_numeral_before_decimal && DECIMAL.is_match(value),
                    "forbidden decimal missing 0 prefix",
                ),
            ];
            for (violates, message) in checks {
                if violates {
                    problems.push(problem(
                        *mark,
                        self.name(),
                        format!("{message} \"{value}\""),
                    ));
                }
            }
        }
        problems
    }
}
