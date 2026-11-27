//! Ordering of scalar keys in mappings.

use super::{
    LintContext, Rule,
    tokens::{Patterns, is_property, problem, scan},
};
use crate::problem::LintProblem;
use serde::Deserialize;
use yaml_rust2::scanner::{Token, TokenType::*};

/// Enforces Unicode code point ordering of scalar mapping keys.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct KeyOrderingRule {
    ignored_keys: Patterns,
}

impl Rule for KeyOrderingRule {
    fn name(&self) -> &'static str {
        "key-ordering"
    }

    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        let tokens = scan(&context.content);
        let mut collections = Vec::new();
        let mut problems = Vec::new();
        for (index, Token(_, token)) in tokens.iter().enumerate() {
            match token {
                BlockMappingStart | FlowMappingStart => collections.push(Some(None)),
                BlockSequenceStart | FlowSequenceStart => collections.push(None),
                BlockEnd | FlowMappingEnd | FlowSequenceEnd => {
                    collections.pop();
                }
                Key => {
                    let Some(Some(previous)) = collections.last_mut() else {
                        continue;
                    };
                    let Some(Token(mark, Scalar(_, key))) =
                        tokens[index + 1..].iter().find(|token| !is_property(token))
                    else {
                        continue;
                    };
                    if self.ignored_keys.matches(key) {
                        continue;
                    }
                    if previous.is_some_and(|previous: &str| previous > key.as_str()) {
                        problems.push(problem(
                            *mark,
                            self.name(),
                            format!("wrong ordering of key \"{key}\" in mapping"),
                        ));
                    } else {
                        *previous = Some(key.as_str());
                    }
                }
                _ => {}
            }
        }
        problems
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_scalar_keys_with_node_properties() {
        let content = "z: 1\n&anchor a: 2\n!!str b: 3\n";
        let problems = KeyOrderingRule::default().check(&LintContext::new(content.into()));
        let locations: Vec<_> = problems.iter().map(|p| (p.line, p.column)).collect();
        assert_eq!(locations, [(2, 9), (3, 7)]);
    }
}
