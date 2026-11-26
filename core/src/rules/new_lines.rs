//! Checks the line ending style in YAML source.

use crate::problem::{LintLevel, LintProblem};
use crate::rules::{LintContext, Rule};

/// Expected line ending style.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum NewLineType {
    /// LF.
    #[default]
    Unix,
    /// CRLF.
    Dos,
    /// CRLF on Windows and LF elsewhere.
    Platform,
}

/// Reports line endings that differ from the configured style.
#[derive(Debug, Default)]
pub struct NewLinesRule {
    /// Expected line ending style.
    pub newline_type: NewLineType,
}

impl Rule for NewLinesRule {
    fn name(&self) -> &'static str {
        "new-lines"
    }

    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        let expected = match self.newline_type {
            NewLineType::Dos => "\r\n",
            NewLineType::Platform if cfg!(windows) => "\r\n",
            _ => "\n",
        };
        let message = if expected == "\r\n" {
            r"wrong new line character: expected \r\n"
        } else {
            r"wrong new line character: expected \n"
        };

        let mut problems = Vec::new();
        let mut line_number = 1;
        let mut line_start = 0;
        for (offset, character) in context.content.match_indices(['\r', '\n']) {
            if offset < line_start {
                continue; // The LF of an already processed CRLF.
            }
            let ending = if character == "\r"
                && context.content.as_bytes().get(offset + 1) == Some(&b'\n')
            {
                "\r\n"
            } else {
                character
            };
            if ending != expected {
                problems.push(LintProblem::new(
                    line_number,
                    context.content[line_start..offset].chars().count() + 1,
                    message,
                    self.name(),
                    LintLevel::Error,
                ));
            }
            line_start = offset + ending.len();
            line_number += 1;
        }
        problems
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_mixed_line_endings_with_character_columns() {
        let context =
            LintContext::new("key: |\n  日本語\r\n\r\nother: value\rlast: value".to_string());
        let problems = NewLinesRule::default().check(&context);
        let positions: Vec<_> = problems.iter().map(|p| (p.line, p.column)).collect();
        assert_eq!(positions, [(2, 6), (3, 1), (4, 13)]);

        let problems = NewLinesRule {
            newline_type: NewLineType::Dos,
        }
        .check(&context);
        let positions: Vec<_> = problems.iter().map(|p| (p.line, p.column)).collect();
        assert_eq!(positions, [(1, 7), (4, 13)]);
    }

    #[test]
    fn accepts_platform_line_endings_and_input_without_an_ending() {
        let rule = NewLinesRule {
            newline_type: NewLineType::Platform,
        };
        let valid = if cfg!(windows) {
            "key: value\r\n"
        } else {
            "key: value\n"
        };
        for content in [valid, "key: value", ""] {
            assert!(
                rule.check(&LintContext::new(content.to_string()))
                    .is_empty()
            );
        }
    }
}
