//! Explicit YAML document end markers.

use super::{LintContext, Rule, tokens::problem};
use crate::problem::{LintLevel, LintProblem};
use serde::Deserialize;
use yaml_rust2::scanner::{Scanner, Token, TokenType::*};

/// Requires or forbids an explicit document end marker.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DocumentEndRule {
    present: bool,
}

impl Default for DocumentEndRule {
    fn default() -> Self {
        Self { present: true }
    }
}

impl Rule for DocumentEndRule {
    fn name(&self) -> &'static str {
        "document-end"
    }

    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        let mut problems = Vec::new();
        let mut in_document = false;
        for Token(mark, token) in Scanner::new(context.content.chars()) {
            match token {
                DocumentStart | StreamEnd => {
                    if in_document && self.present {
                        let line = if matches!(token, StreamEnd) {
                            mark.line().saturating_sub(1).max(1)
                        } else {
                            mark.line()
                        };
                        problems.push(LintProblem::new(
                            line,
                            1,
                            "missing document end \"...\"",
                            self.name(),
                            LintLevel::Error,
                        ));
                    }
                    in_document = matches!(token, DocumentStart);
                }
                DocumentEnd => {
                    if !self.present {
                        problems.push(problem(
                            mark,
                            self.name(),
                            "found forbidden document end \"...\"",
                        ));
                    }
                    in_document = false;
                }
                StreamStart(_) | BlockEnd | VersionDirective(..) | TagDirective(..) => {}
                _ => in_document = true,
            }
        }
        problems
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_the_next_document_start_or_last_line() {
        let rule = DocumentEndRule::default();
        let context = LintContext::new("---\na: 1\n---\nb: 2\n".into());
        let locations: Vec<_> = rule
            .check(&context)
            .iter()
            .map(|problem| (problem.line, problem.column))
            .collect();
        assert_eq!(locations, [(3, 1), (4, 1)]);
        assert!(
            rule.check(&LintContext::new("# comment\n".into()))
                .is_empty()
        );
    }
}
