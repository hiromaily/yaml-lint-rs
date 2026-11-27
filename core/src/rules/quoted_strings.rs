//! Quoting conventions for string scalars.

use super::{
    LintContext, Rule,
    tokens::{Patterns, is_property, problem, quoted_len, scan},
};
use crate::problem::LintProblem;
use crate::yaml_rust2::scanner::{
    TScalarStyle::{self, *},
    Token,
    TokenType::*,
};
use regex::Regex;
use serde::{Deserialize, Deserializer};
use std::sync::LazyLock;

static TYPED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"^(?:~|null|Null|NULL|yes|Yes|YES|no|No|NO|true|True|TRUE|false|False|FALSE|on|On|ON|off|Off|OFF|",
        r"[+-]?(?:0|[1-9][0-9_]*|0[0-7_]+|0[bB][01_]+|0o[0-7_]+|0[xX][0-9a-fA-F_]+|",
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
        let source = &context.content;
        let tokens = scan(source);
        let check_continuations = self.required == Required::WhenNeeded
            && (source.contains("\\\n") || source.contains("\\\r"));
        // Scanner indices count characters. Map them to byte offsets only
        // when we need to inspect the original spelling of a quoted scalar.
        let byte_offsets: Option<Vec<_>> = (check_continuations && !source.is_ascii())
            .then(|| source.char_indices().map(|(index, _)| index).collect());
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
                && !(*style == DoubleQuoted && check_continuations && {
                    let start = byte_offsets
                        .as_ref()
                        .map_or(mark.index(), |offsets| offsets[mark.index()]);
                    joins_lines(&source[start..])
                })
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

fn joins_lines(source: &str) -> bool {
    let mut bytes = source.as_bytes()[..quoted_len(source)].iter();
    while let Some(byte) = bytes.next() {
        if *byte == b'\\' && matches!(bytes.next(), Some(b'\r' | b'\n')) {
            return true;
        }
    }
    false
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
    fn distinguishes_octal_values_from_strings_with_invalid_octal_digits() {
        let context = LintContext::new("- 0777\n- 0o777\n- 0800\n- 0o800\n".into());
        let problems = QuotedStringsRule::default().check(&context);
        let lines: Vec<_> = problems.iter().map(|problem| problem.line).collect();
        assert_eq!(lines, [3, 4]);
    }

    #[test]
    fn keeps_quotes_that_join_physical_lines() {
        let rule = QuotedStringsRule {
            required: Required::WhenNeeded,
            ..QuotedStringsRule::default()
        };
        for ending in ["\n", "\r\n"] {
            let content = format!(
                "prefix: 日本語{ending}joined: \"word\\{ending}  part\"{ending}simple: \"plain\"{ending}"
            );
            let problems = rule.check(&LintContext::new(content));
            assert_eq!(problems.iter().map(|p| p.line).collect::<Vec<_>>(), [4]);
        }
    }

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
