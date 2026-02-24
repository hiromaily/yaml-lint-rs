//! Braces rule - controls spacing inside flow mappings `{}`

use crate::problem::LintProblem;
use crate::rules::flow_collection::{FlowCollectionConfig, check_flow_collection};
use crate::rules::{LintContext, Rule, RuleLevel};

/// Rule that controls spacing inside flow mappings (braces `{}`)
///
/// This rule enforces consistent spacing immediately inside `{` and `}` characters,
/// and can optionally forbid flow mappings entirely.
#[derive(Debug)]
pub struct BracesRule {
    config: FlowCollectionConfig,
}

impl BracesRule {
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

impl Default for BracesRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for BracesRule {
    fn name(&self) -> &'static str {
        "braces"
    }

    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        check_flow_collection(
            context,
            &self.config,
            '{',
            '}',
            "braces",
            "flow mapping (braces)",
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
        let yaml = "mapping: {key: value}\n";
        let context = LintContext::new(yaml.to_string());
        let rule = BracesRule::new();
        let problems = rule.check(&context);
        assert!(problems.is_empty());
    }

    #[test]
    fn test_spaces_inside_braces() {
        let yaml = "mapping: { key: value }\n";
        let context = LintContext::new(yaml.to_string());
        let rule = BracesRule::new();
        let problems = rule.check(&context);
        assert_eq!(problems.len(), 2);
        assert!(
            problems[0]
                .message
                .contains("too many spaces inside braces")
        );
        assert!(
            problems[1]
                .message
                .contains("too many spaces inside braces")
        );
    }

    #[test]
    fn test_spaces_required() {
        let yaml = "mapping: {key: value}\n";
        let context = LintContext::new(yaml.to_string());
        let rule = BracesRule::with_config(false, 1, 1, None, None);
        let problems = rule.check(&context);
        assert_eq!(problems.len(), 2);
        assert!(problems[0].message.contains("too few spaces inside braces"));
        assert!(problems[1].message.contains("too few spaces inside braces"));
    }

    #[test]
    fn test_spaces_required_correct() {
        let yaml = "mapping: { key: value }\n";
        let context = LintContext::new(yaml.to_string());
        let rule = BracesRule::with_config(false, 1, 1, None, None);
        let problems = rule.check(&context);
        assert!(problems.is_empty());
    }

    #[test]
    fn test_forbid() {
        let yaml = "mapping: {key: value}\n";
        let context = LintContext::new(yaml.to_string());
        let rule = BracesRule::with_config(true, 0, 0, None, None);
        let problems = rule.check(&context);
        assert_eq!(problems.len(), 1);
        assert!(problems[0].message.contains("forbidden"));
    }

    #[test]
    fn test_empty_braces_default() {
        let yaml = "mapping: {}\n";
        let context = LintContext::new(yaml.to_string());
        let rule = BracesRule::new();
        let problems = rule.check(&context);
        assert!(problems.is_empty());
    }

    #[test]
    fn test_empty_braces_with_spaces() {
        let yaml = "mapping: { }\n";
        let context = LintContext::new(yaml.to_string());
        let rule = BracesRule::new();
        let problems = rule.check(&context);
        assert_eq!(problems.len(), 1);
        assert!(
            problems[0]
                .message
                .contains("too many spaces inside empty braces")
        );
    }

    #[test]
    fn test_empty_braces_custom_empty_config() {
        let yaml = "mapping: { }\n";
        let context = LintContext::new(yaml.to_string());
        // Allow 1 space inside empty braces
        let rule = BracesRule::with_config(false, 0, 0, Some(1), Some(1));
        let problems = rule.check(&context);
        assert!(problems.is_empty());
    }

    #[test]
    fn test_quoted_braces_ignored() {
        let yaml = "key: \"{ value }\"\n";
        let context = LintContext::new(yaml.to_string());
        let rule = BracesRule::new();
        let problems = rule.check(&context);
        assert!(problems.is_empty());
    }

    #[test]
    fn test_single_quoted_braces_ignored() {
        let yaml = "key: '{ value }'\n";
        let context = LintContext::new(yaml.to_string());
        let rule = BracesRule::new();
        let problems = rule.check(&context);
        assert!(problems.is_empty());
    }

    #[test]
    fn test_comment_braces_ignored() {
        let yaml = "key: value  # { not a brace }\n";
        let context = LintContext::new(yaml.to_string());
        let rule = BracesRule::new();
        let problems = rule.check(&context);
        assert!(problems.is_empty());
    }

    #[test]
    fn test_nested_braces() {
        let yaml = "mapping: {outer: {inner: value}}\n";
        let context = LintContext::new(yaml.to_string());
        let rule = BracesRule::new();
        let problems = rule.check(&context);
        assert!(problems.is_empty());
    }

    #[test]
    fn test_nested_braces_with_spaces() {
        let yaml = "mapping: { outer: { inner: value } }\n";
        let context = LintContext::new(yaml.to_string());
        let rule = BracesRule::new();
        let problems = rule.check(&context);
        // Spaces after outer {, before outer }, after inner {, before inner }
        assert_eq!(problems.len(), 4);
    }

    #[test]
    fn test_multiple_braces_on_line() {
        let yaml = "a: {x: 1}\nb: { y: 2 }\n";
        let context = LintContext::new(yaml.to_string());
        let rule = BracesRule::new();
        let problems = rule.check(&context);
        assert_eq!(problems.len(), 2); // spaces in second mapping
    }

    #[test]
    fn test_no_braces() {
        let yaml = "key: value\nlist:\n  - item\n";
        let context = LintContext::new(yaml.to_string());
        let rule = BracesRule::new();
        let problems = rule.check(&context);
        assert!(problems.is_empty());
    }

    #[test]
    fn test_forbid_no_braces() {
        let yaml = "key: value\nlist:\n  - item\n";
        let context = LintContext::new(yaml.to_string());
        let rule = BracesRule::with_config(true, 0, 0, None, None);
        let problems = rule.check(&context);
        assert!(problems.is_empty());
    }
}
