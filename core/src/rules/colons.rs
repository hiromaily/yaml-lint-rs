//! Colons rule - validates spacing around colons in mappings

use super::tokens;
use crate::problem::{LintLevel, LintProblem};
use crate::rules::{LintContext, Rule, RuleLevel};
use yaml_rust2::scanner::{Scanner, Token, TokenType};

/// Rule that checks spacing around colons in key-value mappings
#[derive(Debug)]
pub struct ColonsRule {
    /// Maximum spaces allowed before colon
    pub max_spaces_before: usize,
    /// Maximum spaces allowed after colon
    pub max_spaces_after: usize,
}

impl ColonsRule {
    /// Create a new colons rule with defaults (0 before, 1 after)
    pub fn new() -> Self {
        Self {
            max_spaces_before: 0,
            max_spaces_after: 1,
        }
    }

    /// Create a colons rule with custom spacing
    pub fn with_spacing(max_before: usize, max_after: usize) -> Self {
        Self {
            max_spaces_before: max_before,
            max_spaces_after: max_after,
        }
    }
}

impl Default for ColonsRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for ColonsRule {
    fn name(&self) -> &'static str {
        "colons"
    }

    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        use TokenType::*;
        if !context.content.contains([':', '?']) {
            return Vec::new();
        }
        let mut scanner = Scanner::new(context.content.chars()).peekable();
        let mut previous = None;
        let mut problems = Vec::new();
        while let Some(Token(mark, token)) = scanner.next() {
            let line = context.lines.get(mark.line() - 1);
            let mut after = None;
            match token {
                Value => {
                    let spaces = line.map_or(0, |line| tokens::spaces_before(line, mark.col()));
                    if !matches!(previous, Some(Alias(_))) || spaces != 1 {
                        if spaces < mark.col() && spaces > self.max_spaces_before {
                            problems.push(LintProblem::new(
                                mark.line(),
                                mark.col(),
                                "too many spaces before colon",
                                self.name(),
                                LintLevel::Error,
                            ));
                        }
                        after = Some("too many spaces after colon");
                    }
                }
                Key if line.and_then(|line| line.chars().nth(mark.col())) == Some('?') => {
                    after = Some("too many spaces after question mark");
                }
                _ => {}
            }
            if let (Some(message), Some(line), Some(Token(next, _))) = (after, line, scanner.peek())
                && next.line() == mark.line()
            {
                let mut tail = line.chars().skip(mark.col() + 1).peekable();
                let mut spaces = 0;
                while tail.next_if(|ch| matches!(ch, ' ' | '\t')).is_some() {
                    spaces += 1;
                }
                if spaces > self.max_spaces_after && tail.peek().is_some_and(|ch| *ch != '#') {
                    problems.push(LintProblem::new(
                        mark.line(),
                        mark.col() + spaces + 1,
                        message,
                        self.name(),
                        LintLevel::Error,
                    ));
                }
            }
            previous = Some(token);
        }
        problems
    }

    fn default_level(&self) -> RuleLevel {
        RuleLevel::Error
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correct_spacing() {
        let yaml = "key: value\nkey2: value2\n";
        let context = LintContext::new(yaml.to_string());
        let rule = ColonsRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_spaces_before_colon() {
        let yaml = "key : value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = ColonsRule::new();
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
        assert!(problems[0].message.contains("too many spaces before colon"));
    }

    #[test]
    fn test_too_many_spaces_after_colon() {
        let yaml = "key:  value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = ColonsRule::new();
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
        assert!(problems[0].message.contains("too many spaces after colon"));
    }

    #[test]
    fn test_custom_spacing() {
        let yaml = "key :  value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = ColonsRule::with_spacing(1, 2);
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_colon_at_end_of_line() {
        let yaml = "key:\n  value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = ColonsRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_colon_in_string_ignored() {
        let yaml = "key: \"value:with:colons\"\n";
        let context = LintContext::new(yaml.to_string());
        let rule = ColonsRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_comment_line_ignored() {
        let yaml = "# comment: with colon\nkey: value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = ColonsRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_document_marker_ignored() {
        let yaml = "---\nkey: value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = ColonsRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_no_space_after_colon() {
        let yaml = "key:value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = ColonsRule::new();
        let problems = rule.check(&context);

        // This should pass - no space after is technically valid in YAML
        // but yamllint might flag it. For now, we only check max spaces.
        assert!(problems.is_empty());
    }
}
