//! Hyphens rule - controls spacing after list item hyphens

use super::tokens;
use crate::problem::LintProblem;
use crate::rules::{LintContext, Rule, RuleLevel};
use crate::yaml_rust2::scanner::{Scanner, Token, TokenType};

/// Rule that controls spacing after list item hyphens in YAML sequences
///
/// This rule enforces consistent formatting of list items by checking
/// the number of spaces after the `-` character.
#[derive(Debug)]
pub struct HyphensRule {
    /// Maximum spaces allowed after hyphen (default: 1)
    max_spaces_after: usize,
}

impl HyphensRule {
    /// Create a new rule with default settings
    pub fn new() -> Self {
        Self {
            max_spaces_after: 1,
        }
    }

    /// Create a new rule with custom settings
    pub fn with_config(max_spaces_after: usize) -> Self {
        Self { max_spaces_after }
    }
}

impl Default for HyphensRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for HyphensRule {
    fn name(&self) -> &'static str {
        "hyphens"
    }

    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        if !context.content.contains('-') {
            return Vec::new();
        }
        let mut problems = Vec::new();
        for Token(mark, token) in Scanner::new(context.content.chars()) {
            // BlockEntry's marker is after the separator and its whitespace.
            if token == TokenType::BlockEntry
                && let Some(line) = context.lines.get(mark.line() - 1)
                && line.chars().nth(mark.col()).is_some()
                && tokens::spaces_before(line, mark.col()) > self.max_spaces_after
            {
                problems.push(LintProblem::new(
                    mark.line(),
                    mark.col(),
                    "too many spaces after hyphen",
                    self.name(),
                    crate::LintLevel::Error,
                ));
            }
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
        let yaml = "list:\n  - item1\n  - item2\n";
        let context = LintContext::new(yaml.to_string());
        let rule = HyphensRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_too_many_spaces() {
        let yaml = "list:\n  -  item1\n  - item2\n";
        let context = LintContext::new(yaml.to_string());
        let rule = HyphensRule::new();
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].line, 2);
        assert!(problems[0].message.contains("too many spaces after hyphen"));
    }

    #[test]
    fn test_three_spaces() {
        let yaml = "list:\n  -   item\n";
        let context = LintContext::new(yaml.to_string());
        let rule = HyphensRule::new();
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].column, 6);
    }

    #[test]
    fn test_custom_max_spaces() {
        let yaml = "list:\n  -  item1\n  -   item2\n";
        let context = LintContext::new(yaml.to_string());
        let rule = HyphensRule::with_config(2);
        let problems = rule.check(&context);

        // First item has 2 spaces (ok), second has 3 (error)
        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].line, 3);
    }

    #[test]
    fn test_nested_lists() {
        let yaml = "list:\n  - item1\n  - nested:\n      - subitem\n";
        let context = LintContext::new(yaml.to_string());
        let rule = HyphensRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_empty_list_item() {
        let yaml = "list:\n  -\n  - item\n";
        let context = LintContext::new(yaml.to_string());
        let rule = HyphensRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_list_item_with_trailing_space() {
        let yaml = "list:\n  - \n  - item\n";
        let context = LintContext::new(yaml.to_string());
        let rule = HyphensRule::new();
        let problems = rule.check(&context);

        // Empty item with trailing space is valid
        assert!(problems.is_empty());
    }

    #[test]
    fn test_block_scalar() {
        let yaml = "list:\n  - |\n    multiline\n";
        let context = LintContext::new(yaml.to_string());
        let rule = HyphensRule::new();
        let problems = rule.check(&context);

        // Block scalar indicator without space is valid
        assert!(problems.is_empty());
    }

    #[test]
    fn test_flow_sequence_not_affected() {
        let yaml = "list: [item1, item2]\n";
        let context = LintContext::new(yaml.to_string());
        let rule = HyphensRule::new();
        let problems = rule.check(&context);

        // Flow sequences don't use hyphens
        assert!(problems.is_empty());
    }

    #[test]
    fn test_comment_ignored() {
        let yaml = "# - this is a comment\nlist:\n  - item\n";
        let context = LintContext::new(yaml.to_string());
        let rule = HyphensRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_multiple_violations() {
        let yaml = "list:\n  -  item1\n  -   item2\n  - item3\n";
        let context = LintContext::new(yaml.to_string());
        let rule = HyphensRule::new();
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 2);
        assert_eq!(problems[0].line, 2);
        assert_eq!(problems[1].line, 3);
    }

    #[test]
    fn test_plain_scalar_starting_with_hyphen() {
        let yaml = "list:\n  -item\n";
        let context = LintContext::new(yaml.to_string());
        let rule = HyphensRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_anchor() {
        let yaml = "list:\n  - &anchor value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = HyphensRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_alias() {
        let yaml = "list:\n  - *alias\n";
        let context = LintContext::new(yaml.to_string());
        let rule = HyphensRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_document_start_marker() {
        let yaml = "---\nkey: value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = HyphensRule::new();
        let problems = rule.check(&context);

        // Document start marker should not be flagged
        assert!(problems.is_empty());
    }

    #[test]
    fn test_document_end_marker() {
        let yaml = "key: value\n...\n";
        let context = LintContext::new(yaml.to_string());
        let rule = HyphensRule::new();
        let problems = rule.check(&context);

        // Document end marker should not be flagged
        assert!(problems.is_empty());
    }

    #[test]
    fn test_document_with_list() {
        let yaml = "---\nlist:\n  - item1\n  - item2\n";
        let context = LintContext::new(yaml.to_string());
        let rule = HyphensRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }
}
