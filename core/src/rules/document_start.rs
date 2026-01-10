//! Document start rule - requires or forbids `---` at document start

use crate::problem::{LintLevel, LintProblem};
use crate::rules::{LintContext, Rule, RuleLevel};

/// Configuration for document start requirement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentStartPresence {
    /// Require `---` at document start
    Required,
    /// Forbid `---` at document start
    Forbidden,
    /// No requirement (disabled)
    Disabled,
}

/// Rule that checks for `---` at document start
#[derive(Debug)]
pub struct DocumentStartRule {
    /// Whether `---` should be present
    pub presence: DocumentStartPresence,
}

impl DocumentStartRule {
    /// Create a new rule (disabled by default, matching yamllint)
    pub fn new() -> Self {
        Self {
            presence: DocumentStartPresence::Disabled,
        }
    }

    /// Create a rule that requires `---`
    pub fn required() -> Self {
        Self {
            presence: DocumentStartPresence::Required,
        }
    }

    /// Create a rule that forbids `---`
    pub fn forbidden() -> Self {
        Self {
            presence: DocumentStartPresence::Forbidden,
        }
    }
}

impl Default for DocumentStartRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for DocumentStartRule {
    fn name(&self) -> &'static str {
        "document-start"
    }

    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        if self.presence == DocumentStartPresence::Disabled {
            return Vec::new();
        }

        let mut problems = Vec::new();

        // Check if the document starts with `---`
        let has_document_start = context
            .lines
            .first()
            .map(|line| {
                let trimmed = line.trim();
                trimmed == "---" || trimmed.starts_with("--- ")
            })
            .unwrap_or(false);

        match self.presence {
            DocumentStartPresence::Required if !has_document_start => {
                problems.push(LintProblem::new(
                    1,
                    1,
                    "missing document start \"---\"",
                    self.name(),
                    LintLevel::Error,
                ));
            }
            DocumentStartPresence::Forbidden if has_document_start => {
                problems.push(LintProblem::new(
                    1,
                    1,
                    "found forbidden document start \"---\"",
                    self.name(),
                    LintLevel::Error,
                ));
            }
            _ => {}
        }

        problems
    }

    fn default_level(&self) -> RuleLevel {
        RuleLevel::Disable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disabled_rule() {
        let yaml = "key: value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = DocumentStartRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_required_present() {
        let yaml = "---\nkey: value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = DocumentStartRule::required();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_required_missing() {
        let yaml = "key: value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = DocumentStartRule::required();
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].line, 1);
        assert!(problems[0].message.contains("missing document start"));
    }

    #[test]
    fn test_forbidden_present() {
        let yaml = "---\nkey: value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = DocumentStartRule::forbidden();
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].line, 1);
        assert!(problems[0].message.contains("forbidden document start"));
    }

    #[test]
    fn test_forbidden_absent() {
        let yaml = "key: value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = DocumentStartRule::forbidden();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_document_start_with_space() {
        // "--- " with trailing space should be recognized
        let yaml = "--- \nkey: value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = DocumentStartRule::required();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_document_start_with_comment() {
        let yaml = "--- # comment\nkey: value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = DocumentStartRule::required();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_empty_document() {
        let yaml = "";
        let context = LintContext::new(yaml.to_string());
        let rule = DocumentStartRule::required();
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
    }
}
