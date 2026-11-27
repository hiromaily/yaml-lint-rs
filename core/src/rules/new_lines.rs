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

impl NewLineType {
    pub(crate) fn ending(self) -> &'static str {
        match self {
            Self::Dos => "\r\n",
            Self::Platform if cfg!(windows) => "\r\n",
            _ => "\n",
        }
    }
}

// Split a chunk from split_inclusive('\n') into its text and original terminator.
pub(super) fn split_ending(line: &str) -> (&str, &str) {
    if let Some(text) = line.strip_suffix("\r\n") {
        (text, "\r\n")
    } else if let Some(text) = line.strip_suffix('\n') {
        (text, "\n")
    } else {
        (line, "")
    }
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
        let expected = self.newline_type.ending();
        let message = if expected == "\r\n" {
            r"wrong new line character: expected \r\n"
        } else {
            r"wrong new line character: expected \n"
        };

        let Some((offset, _)) = context.content.match_indices('\n').next() else {
            return Vec::new();
        };
        let (line, ending) = split_ending(&context.content[..=offset]);
        if ending == expected {
            return Vec::new();
        }
        vec![LintProblem::new(
            1,
            line.chars().count() + 1,
            message,
            self.name(),
            LintLevel::Error,
        )]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checks_only_the_first_ending_with_character_columns() {
        let context = LintContext::new("key: 日本語\r\nother: value\n".into());
        let problems = NewLinesRule::default().check(&context);
        assert_eq!(problems.len(), 1);
        assert_eq!((problems[0].line, problems[0].column), (1, 9));

        assert!(
            NewLinesRule {
                newline_type: NewLineType::Dos
            }
            .check(&context)
            .is_empty()
        );
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
