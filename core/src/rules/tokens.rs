//! Shared YAML scanner helpers for token-based rules.

use crate::problem::{LintLevel, LintProblem};
use serde::{Deserialize, Deserializer};
use yaml_rust2::scanner::{Marker, Scanner, Token, TokenType};

pub(super) fn scan(content: &str) -> Vec<Token> {
    Scanner::new(content.chars()).collect()
}

pub(super) fn problem(mark: Marker, rule: &str, message: impl Into<String>) -> LintProblem {
    LintProblem::new(mark.line(), mark.col() + 1, message, rule, LintLevel::Error)
}

pub(super) fn is_property(token: &Token) -> bool {
    matches!(token.1, TokenType::Anchor(_) | TokenType::Tag(..))
}

pub(super) fn has_tag(tokens: &[Token], index: usize) -> bool {
    tokens[..index]
        .iter()
        .rev()
        .take_while(|token| is_property(token))
        .any(|token| matches!(token.1, TokenType::Tag(..)))
}

#[derive(Debug, Clone, Default)]
pub(super) struct Patterns(Vec<regex::Regex>);

impl Patterns {
    pub(super) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(super) fn matches(&self, value: &str) -> bool {
        self.0.iter().any(|pattern| pattern.is_match(value))
    }
}

impl PartialEq for Patterns {
    fn eq(&self, other: &Self) -> bool {
        self.0
            .iter()
            .map(regex::Regex::as_str)
            .eq(other.0.iter().map(regex::Regex::as_str))
    }
}

impl<'de> Deserialize<'de> for Patterns {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Vec::<String>::deserialize(deserializer)?
            .iter()
            .map(|pattern| regex::Regex::new(pattern).map_err(serde::de::Error::custom))
            .collect::<Result<Vec<_>, _>>()
            .map(Self)
    }
}
