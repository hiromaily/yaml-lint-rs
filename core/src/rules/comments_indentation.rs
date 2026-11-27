//! Standalone comment indentation.

use super::{LintContext, Rule, directives::comments};
use crate::problem::{LintLevel, LintProblem};

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
                let (_, text) = code.next().expect("peeked line");
                previous_indent = indentation(text);
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

fn indentation(line: &str) -> usize {
    line.len() - line.trim_start().len()
}
