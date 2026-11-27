//! Standalone comment indentation.

use super::{LintContext, Rule, directives::comments, tokens::scan};
use crate::problem::{LintLevel, LintProblem};
use crate::yaml_rust2::scanner::{TScalarStyle, Token, TokenType};
use std::collections::HashMap;

/// Checks that standalone comments align with surrounding YAML.
#[derive(Debug, Default)]
pub struct CommentsIndentationRule;

impl Rule for CommentsIndentationRule {
    fn name(&self) -> &'static str {
        "comments-indentation"
    }

    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        let standalone: Vec<_> = comments(&context.content)
            .filter(|&(_, line, hash)| line[..hash].trim().is_empty())
            .map(|(line, _, hash)| (line, hash))
            .collect();
        if standalone.is_empty() {
            return Vec::new();
        }
        let scalar_indents = scalar_body_indents(context);
        let mut code = context
            .lines
            .iter()
            .enumerate()
            .filter(|&(_, line)| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
            .peekable();
        let mut previous_indent = 0;
        let mut previous_comment = None;
        let mut problems = Vec::new();
        for (line, indent) in standalone {
            while code.peek().is_some_and(|(index, _)| *index + 1 < line) {
                let (index, text) = code.next().expect("peeked line");
                previous_indent = scalar_indents
                    .get(&index)
                    .copied()
                    .unwrap_or_else(|| indentation(text));
                previous_comment = None;
            }
            let next = code.peek().map_or(0, |(_, text)| indentation(text));
            let mut other = previous_indent.max(next);
            if let Some(last) = previous_comment {
                other = last;
            }
            if indent != next && indent != other {
                problems.push(LintProblem::new(
                    line,
                    indent + 1,
                    "comment not indented like content",
                    self.name(),
                    LintLevel::Error,
                ));
            }
            previous_comment = Some(indent);
        }
        problems
    }
}

fn scalar_body_indents(context: &LintContext) -> HashMap<usize, usize> {
    let mut indents = HashMap::new();
    let mut key = None;
    let mut value_indent = None;
    for Token(mark, kind) in scan(&context.content) {
        match kind {
            TokenType::Key => key = Some(mark),
            TokenType::Value => {
                value_indent = key
                    .filter(|key| key.line() == mark.line())
                    .map(|key| key.col());
            }
            TokenType::Anchor(_) | TokenType::Tag(..) => {}
            TokenType::Scalar(TScalarStyle::Literal | TScalarStyle::Folded, value)
                if !value.trim().is_empty() =>
            {
                if let Some(indent) = value_indent.take() {
                    // The scanner marks the body. Comments after it align
                    // with the mapping key, including compact sequence mappings.
                    for (index, line) in context.lines.iter().enumerate().skip(mark.line() - 1) {
                        if line.trim().is_empty() {
                            continue;
                        }
                        if indentation(line) < mark.col() {
                            break;
                        }
                        indents.insert(index, indent);
                    }
                }
            }
            _ => value_indent = None,
        }
    }
    indents
}

fn indentation(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standalone_comment_uses_mapping_indent_after_folded_scalar() {
        let yaml = "---\n- string: >\n    content\n  \n  # comment\n";
        assert!(
            CommentsIndentationRule
                .check(&LintContext::new(yaml.into()))
                .is_empty()
        );
    }

    #[test]
    fn standalone_comment_rejects_misalignment_after_folded_scalar() {
        let yaml = "---\n- string: >\n    content\n  \n # comment\n";
        let problems = CommentsIndentationRule.check(&LintContext::new(yaml.into()));
        assert_eq!(problems.len(), 1);
        assert_eq!((problems[0].line, problems[0].column), (5, 2));
    }
}
