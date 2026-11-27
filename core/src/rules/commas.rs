//! Flow collection comma spacing.

use super::{LintContext, Rule};
use crate::problem::{LintLevel, LintProblem};
use serde::Deserialize;
use yaml_rust2::scanner::{Scanner, Token, TokenType::*};

/// Controls spacing around commas in flow collections.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct CommasRule {
    max_spaces_before: i64,
    min_spaces_after: i64,
    max_spaces_after: i64,
}

impl Default for CommasRule {
    fn default() -> Self {
        Self {
            max_spaces_before: 0,
            min_spaces_after: 1,
            max_spaces_after: 1,
        }
    }
}

impl Rule for CommasRule {
    fn name(&self) -> &'static str {
        "commas"
    }

    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        if !context.content.contains(',') {
            return Vec::new();
        }
        let mut tokens = Scanner::new(context.content.chars()).peekable();
        let mut problems = Vec::new();
        let mut line_number = 0;
        let mut chars = Vec::new();
        while let Some(Token(mark, token)) = tokens.next() {
            if !matches!(token, FlowEntry) {
                continue;
            }
            if mark.line() != line_number {
                let Some(line) = context.lines.get(mark.line() - 1) else {
                    continue;
                };
                line_number = mark.line();
                chars.clear();
                chars.extend(line.chars());
            }
            let Some((_, following)) = chars.get(mark.col()..).and_then(|tail| tail.split_first())
            else {
                continue;
            };
            let before = chars[..mark.col()]
                .iter()
                .rev()
                .take_while(|&&ch| matches!(ch, ' ' | '\t'))
                .count();
            let after = following
                .iter()
                .take_while(|&&ch| matches!(ch, ' ' | '\t'))
                .count();
            let mut report = |column, message| {
                problems.push(LintProblem::new(
                    mark.line(),
                    column,
                    message,
                    self.name(),
                    LintLevel::Error,
                ));
            };
            if let Ok(max) = usize::try_from(self.max_spaces_before)
                && (before > max || mark.col() == before)
            {
                report(mark.col().max(1), "too many spaces before comma");
            }
            if tokens
                .peek()
                .is_some_and(|Token(next, _)| next.line() == mark.line())
                && following.get(after).is_some_and(|&ch| ch != '#')
            {
                let column = mark.col() + after + 1;
                if usize::try_from(self.max_spaces_after).is_ok_and(|max| after > max) {
                    report(column, "too many spaces after comma");
                } else if usize::try_from(self.min_spaces_after).is_ok_and(|min| after < min) {
                    report(column + 1, "too few spaces after comma");
                }
            }
        }
        problems
    }
}
