//! Quoting conventions for string scalars.

use super::{
    LintContext, Rule,
    tokens::{Patterns, is_property, problem, scan},
};
use crate::problem::LintProblem;
use regex::Regex;
use serde::{Deserialize, Deserializer};
use std::sync::LazyLock;
use yaml_rust2::scanner::{
    TScalarStyle::{self, *},
    Token,
    TokenType::*,
};

static TYPED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"^(?:~|null|Null|NULL|yes|Yes|YES|no|No|NO|true|True|TRUE|false|False|FALSE|on|On|ON|off|Off|OFF|",
        r"[+-]?(?:[0-9][0-9_]*|0[bB][01_]+|0o[0-7_]+|0[xX][0-9a-fA-F_]+|",
        r"(?:[0-9][0-9_]*\.[0-9_]*|\.[0-9][0-9_]*)(?:[eE][+-]?[0-9]+)?|[0-9]+[eE][+-]?[0-9]+|",
        r"[0-9][0-9_]*(?::[0-5]?[0-9])+(?:\.[0-9_]*)?|\.(?:inf|Inf|INF))|",
        r"\.(?:nan|NaN|NAN)|[0-9]{4}-[0-9]{1,2}-[0-9]{1,2}(?:[Tt ].+)?)$",
    )).expect("valid YAML scalar pattern")
});

#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum QuoteType {
    #[default]
    Any,
    Single,
    Double,
    Consistent,
}

impl QuoteType {
    fn name(self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::Single => "single",
            Self::Double => "double",
            Self::Consistent => "consistent",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
enum Required {
    #[default]
    Always,
    Optional,
    WhenNeeded,
}

impl<'de> Deserialize<'de> for Required {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match serde_yaml::Value::deserialize(deserializer)? {
            serde_yaml::Value::Bool(true) => Ok(Self::Always),
            serde_yaml::Value::Bool(false) => Ok(Self::Optional),
            serde_yaml::Value::String(value) if value == "only-when-needed" => Ok(Self::WhenNeeded),
            _ => Err(serde::de::Error::custom(
                "expected true, false, or only-when-needed",
            )),
        }
    }
}

/// Controls when strings must be quoted and which quotes they may use.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct QuotedStringsRule {
    quote_type: QuoteType,
    required: Required,
    extra_required: Patterns,
    extra_allowed: Patterns,
    allow_quoted_quotes: bool,
    check_keys: bool,
}

impl QuotedStringsRule {
    pub(crate) fn validate(&self) -> crate::Result<()> {
        for (invalid, option) in [
            (
                self.required == Required::Always && !self.extra_required.is_empty(),
                "extra-required",
            ),
            (
                self.required != Required::WhenNeeded && !self.extra_allowed.is_empty(),
                "extra-allowed",
            ),
        ] {
            if invalid {
                return Err(crate::LintError::ConfigError(format!(
                    "quoted-strings: {option} is incompatible with required"
                )));
            }
        }
        Ok(())
    }

    fn accepts_style(
        &self,
        style: TScalarStyle,
        value: &str,
        first: &mut Option<TScalarStyle>,
    ) -> bool {
        let matches = match self.quote_type {
            QuoteType::Any => true,
            QuoteType::Single => style == SingleQuoted,
            QuoteType::Double => style == DoubleQuoted,
            QuoteType::Consistent => *first.get_or_insert(style) == style,
        };
        matches
            || self.allow_quoted_quotes
                && match style {
                    SingleQuoted => value.contains('"'),
                    DoubleQuoted => value.contains('\''),
                    _ => false,
                }
    }
}

impl Rule for QuotedStringsRule {
    fn name(&self) -> &'static str {
        "quoted-strings"
    }

    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        let tokens = scan(&context.content);
        let mut flow_depth: usize = 0;
        let mut first_quote = None;
        let mut problems = Vec::new();
        for (index, Token(mark, token)) in tokens.iter().enumerate() {
            match token {
                FlowSequenceStart | FlowMappingStart => flow_depth += 1,
                FlowSequenceEnd | FlowMappingEnd => flow_depth = flow_depth.saturating_sub(1),
                DocumentStart | DocumentEnd => first_quote = None,
                _ => {}
            }
            let Scalar(style @ (Plain | SingleQuoted | DoubleQuoted), value) = token else {
                continue;
            };
            let mut preceding = tokens[..index].iter().rev();
            let explicit_type = preceding
                .by_ref()
                .take_while(|token| is_property(token))
                .any(|Token(_, token)| matches!(token, Tag(handle, _) if handle == "!!"));
            // Use a fresh look-behind since take_while consumes the first non-property token.
            let is_key = tokens[..index]
                .iter()
                .rev()
                .find(|token| !is_property(token))
                .is_some_and(|Token(_, token)| matches!(token, Key | FlowMappingStart))
                || tokens
                    .get(index + 1)
                    .is_some_and(|Token(_, token)| matches!(token, Value));
            if explicit_type || is_key && !self.check_keys {
                continue;
            }
            let typed = TYPED.is_match(value);
            let extra_required = self.extra_required.matches(value);
            let node = if is_key { "key" } else { "value" };
            let quote = self.quote_type.name();
            let message = if *style == Plain {
                if typed {
                    continue;
                }
                if self.required == Required::Always {
                    format!("string {node} is not quoted with {quote} quotes")
                } else if extra_required {
                    format!("string {node} is not quoted")
                } else {
                    continue;
                }
            } else if self.required == Required::WhenNeeded
                && !typed
                && !extra_required
                && !self.extra_allowed.matches(value)
                && can_be_plain(value, flow_depth > 0)
            {
                format!("string {node} is redundantly quoted with {quote} quotes")
            } else if !self.accepts_style(*style, value, &mut first_quote) {
                format!("string {node} is not quoted with {quote} quotes")
            } else {
                continue;
            };
            problems.push(problem(*mark, self.name(), message));
        }
        problems
    }
}

fn can_be_plain(value: &str, in_flow: bool) -> bool {
    if value.is_empty()
        || value.chars().any(char::is_control)
        || in_flow && value.contains([',', '[', ']', '{', '}'])
    {
        return false;
    }
    let tokens = scan(&format!("key: {value}"));
    matches!(tokens.as_slice(), [
        Token(_, StreamStart(_)), Token(_, BlockMappingStart), Token(_, Key),
        Token(_, Scalar(Plain, _)), Token(_, Value), Token(_, Scalar(Plain, plain)),
        Token(_, BlockEnd), Token(_, StreamEnd),
    ] if plain == value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redundant_quotes_depend_on_yaml_context() {
        let rule = QuotedStringsRule {
            required: Required::WhenNeeded,
            ..QuotedStringsRule::default()
        };
        let context = LintContext::new(
            "block: 'a,b'\nflow: ['a,b']\nstring: 'off'\nempty: ''\nspace: ' padded '\n".into(),
        );
        let problems = rule.check(&context);
        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].line, 1);
    }
}
