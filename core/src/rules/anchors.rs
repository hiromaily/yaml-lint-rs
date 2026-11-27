//! Anchor and alias validation.

use super::{LintContext, Rule, tokens::problem};
use crate::problem::LintProblem;
use crate::yaml_rust2::scanner::{Scanner, Token, TokenType::*};
use serde::Deserialize;
use std::collections::HashMap;

/// Checks declarations and uses of anchors within each document.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct AnchorsRule {
    forbid_undeclared_aliases: bool,
    forbid_duplicated_anchors: bool,
    forbid_unused_anchors: bool,
}

impl Default for AnchorsRule {
    fn default() -> Self {
        Self {
            forbid_undeclared_aliases: true,
            forbid_duplicated_anchors: false,
            forbid_unused_anchors: false,
        }
    }
}

impl Rule for AnchorsRule {
    fn name(&self) -> &'static str {
        "anchors"
    }

    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        if !context.content.contains(['&', '*']) {
            return Vec::new();
        }
        let mut problems = Vec::new();
        let mut anchors = HashMap::new();
        for Token(mark, token) in Scanner::new(context.content.chars()) {
            match token {
                Anchor(name) => {
                    let duplicate = anchors.insert(name.clone(), (mark, false)).is_some();
                    if self.forbid_duplicated_anchors && duplicate {
                        problems.push(problem(
                            mark,
                            self.name(),
                            format!("found duplicated anchor \"{name}\""),
                        ));
                    }
                }
                Alias(name) => {
                    if let Some((_, used)) = anchors.get_mut(&name) {
                        *used = true;
                    } else if self.forbid_undeclared_aliases {
                        problems.push(problem(
                            mark,
                            self.name(),
                            format!("found undeclared alias \"{name}\""),
                        ));
                    }
                }
                DocumentStart | DocumentEnd | StreamEnd => {
                    for (name, (mark, used)) in anchors.drain() {
                        if self.forbid_unused_anchors && !used {
                            problems.push(problem(
                                mark,
                                self.name(),
                                format!("found unused anchor \"{name}\""),
                            ));
                        }
                    }
                }
                _ => {}
            }
        }
        problems.sort();
        problems
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redefined_anchors_resolve_to_the_latest_declaration() {
        let rule = AnchorsRule {
            forbid_unused_anchors: true,
            ..AnchorsRule::default()
        };
        let context = LintContext::new("- &item first\n- &item second\n- *item\n".into());
        assert!(rule.check(&context).is_empty());

        let context = LintContext::new("- &item first\n- &item second\n".into());
        let problems = rule.check(&context);
        assert_eq!(problems.len(), 1);
        assert_eq!((problems[0].line, problems[0].column), (2, 3));
    }
}
