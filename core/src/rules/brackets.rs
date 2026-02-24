//! Brackets rule - controls spacing inside flow sequences `[]`

use crate::problem::LintProblem;
use crate::rules::flow_collection::{FlowCollectionConfig, check_flow_collection};
use crate::rules::{LintContext, Rule, RuleLevel};

/// Rule that controls spacing inside flow sequences (brackets `[]`)
///
/// This rule enforces consistent spacing immediately inside `[` and `]` characters,
/// and can optionally forbid flow sequences entirely.
#[derive(Debug)]
pub struct BracketsRule {
    config: FlowCollectionConfig,
}

impl BracketsRule {
    /// Create a new rule with default settings
    pub fn new() -> Self {
        Self {
            config: FlowCollectionConfig {
                forbid: false,
                min_spaces_inside: 0,
                max_spaces_inside: 0,
                min_spaces_inside_empty: None,
                max_spaces_inside_empty: None,
            },
        }
    }

    /// Create a new rule with custom settings
    pub fn with_config(
        forbid: bool,
        min_spaces_inside: usize,
        max_spaces_inside: usize,
        min_spaces_inside_empty: Option<usize>,
        max_spaces_inside_empty: Option<usize>,
    ) -> Self {
        Self {
            config: FlowCollectionConfig {
                forbid,
                min_spaces_inside,
                max_spaces_inside,
                min_spaces_inside_empty,
                max_spaces_inside_empty,
            },
        }
    }
}

impl Default for BracketsRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for BracketsRule {
    fn name(&self) -> &'static str {
        "brackets"
    }

    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        check_flow_collection(
            context,
            &self.config,
            '[',
            ']',
            "brackets",
            "flow sequence (brackets)",
            self.name(),
        )
    }

    fn default_level(&self) -> RuleLevel {
        RuleLevel::Error
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correct_no_spaces() {
        let yaml = "sequence: [1, 2, 3]\n";
        let context = LintContext::new(yaml.to_string());
        let rule = BracketsRule::new();
        let problems = rule.check(&context);
        assert!(problems.is_empty());
    }

    #[test]
    fn test_spaces_inside_brackets() {
        let yaml = "sequence: [ 1, 2, 3 ]\n";
        let context = LintContext::new(yaml.to_string());
        let rule = BracketsRule::new();
        let problems = rule.check(&context);
        assert_eq!(problems.len(), 2);
        assert!(
            problems[0]
                .message
                .contains("too many spaces inside brackets")
        );
        assert!(
            problems[1]
                .message
                .contains("too many spaces inside brackets")
        );
    }

    #[test]
    fn test_spaces_required() {
        let yaml = "sequence: [1, 2, 3]\n";
        let context = LintContext::new(yaml.to_string());
        let rule = BracketsRule::with_config(false, 1, 1, None, None);
        let problems = rule.check(&context);
        assert_eq!(problems.len(), 2);
        assert!(
            problems[0]
                .message
                .contains("too few spaces inside brackets")
        );
        assert!(
            problems[1]
                .message
                .contains("too few spaces inside brackets")
        );
    }

    #[test]
    fn test_spaces_required_correct() {
        let yaml = "sequence: [ 1, 2, 3 ]\n";
        let context = LintContext::new(yaml.to_string());
        let rule = BracketsRule::with_config(false, 1, 1, None, None);
        let problems = rule.check(&context);
        assert!(problems.is_empty());
    }

    #[test]
    fn test_forbid() {
        let yaml = "sequence: [1, 2, 3]\n";
        let context = LintContext::new(yaml.to_string());
        let rule = BracketsRule::with_config(true, 0, 0, None, None);
        let problems = rule.check(&context);
        assert_eq!(problems.len(), 1);
        assert!(problems[0].message.contains("forbidden"));
    }

    #[test]
    fn test_empty_brackets_default() {
        let yaml = "sequence: []\n";
        let context = LintContext::new(yaml.to_string());
        let rule = BracketsRule::new();
        let problems = rule.check(&context);
        assert!(problems.is_empty());
    }

    #[test]
    fn test_empty_brackets_with_spaces() {
        let yaml = "sequence: [ ]\n";
        let context = LintContext::new(yaml.to_string());
        let rule = BracketsRule::new();
        let problems = rule.check(&context);
        assert_eq!(problems.len(), 1);
        assert!(
            problems[0]
                .message
                .contains("too many spaces inside empty brackets")
        );
    }

    #[test]
    fn test_empty_brackets_custom_empty_config() {
        let yaml = "sequence: [ ]\n";
        let context = LintContext::new(yaml.to_string());
        let rule = BracketsRule::with_config(false, 0, 0, Some(1), Some(1));
        let problems = rule.check(&context);
        assert!(problems.is_empty());
    }

    #[test]
    fn test_quoted_brackets_ignored() {
        let yaml = "key: \"[ value ]\"\n";
        let context = LintContext::new(yaml.to_string());
        let rule = BracketsRule::new();
        let problems = rule.check(&context);
        assert!(problems.is_empty());
    }

    #[test]
    fn test_single_quoted_brackets_ignored() {
        let yaml = "key: '[ value ]'\n";
        let context = LintContext::new(yaml.to_string());
        let rule = BracketsRule::new();
        let problems = rule.check(&context);
        assert!(problems.is_empty());
    }

    #[test]
    fn test_comment_brackets_ignored() {
        let yaml = "key: value  # [ not a bracket ]\n";
        let context = LintContext::new(yaml.to_string());
        let rule = BracketsRule::new();
        let problems = rule.check(&context);
        assert!(problems.is_empty());
    }

    #[test]
    fn test_nested_brackets() {
        let yaml = "sequence: [[1, 2], [3, 4]]\n";
        let context = LintContext::new(yaml.to_string());
        let rule = BracketsRule::new();
        let problems = rule.check(&context);
        assert!(problems.is_empty());
    }

    #[test]
    fn test_nested_brackets_with_spaces() {
        let yaml = "sequence: [ [1, 2], [3, 4] ]\n";
        let context = LintContext::new(yaml.to_string());
        let rule = BracketsRule::new();
        let problems = rule.check(&context);
        // Spaces after outer [ and before outer ]
        assert_eq!(problems.len(), 2);
    }

    #[test]
    fn test_multiple_brackets_on_line() {
        let yaml = "a: [1]\nb: [ 2 ]\n";
        let context = LintContext::new(yaml.to_string());
        let rule = BracketsRule::new();
        let problems = rule.check(&context);
        assert_eq!(problems.len(), 2); // spaces in second sequence
    }

    #[test]
    fn test_no_brackets() {
        let yaml = "key: value\nlist:\n  - item\n";
        let context = LintContext::new(yaml.to_string());
        let rule = BracketsRule::new();
        let problems = rule.check(&context);
        assert!(problems.is_empty());
    }

    #[test]
    fn test_forbid_no_brackets() {
        let yaml = "key: value\nlist:\n  - item\n";
        let context = LintContext::new(yaml.to_string());
        let rule = BracketsRule::with_config(true, 0, 0, None, None);
        let problems = rule.check(&context);
        assert!(problems.is_empty());
    }
}
