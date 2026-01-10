//! Line length rule - enforces maximum line length

use crate::problem::{LintLevel, LintProblem};
use crate::rules::{LintContext, Rule, RuleLevel};

/// Rule that enforces maximum line length
#[derive(Debug)]
pub struct LineLengthRule {
    /// Maximum allowed line length
    pub max: usize,
}

impl LineLengthRule {
    /// Create a new line length rule with the default max (80)
    pub fn new() -> Self {
        Self { max: 80 }
    }

    /// Create a line length rule with a custom max
    pub fn with_max(max: usize) -> Self {
        Self { max }
    }
}

impl Default for LineLengthRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for LineLengthRule {
    fn name(&self) -> &'static str {
        "line-length"
    }

    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        let mut problems = Vec::new();

        for (line_idx, line) in context.lines.iter().enumerate() {
            // Skip trailing newline in length calculation
            let line_length = line.len();

            if line_length > self.max {
                problems.push(LintProblem::new(
                    line_idx + 1, // 1-indexed
                    self.max + 1, // Column where it exceeds
                    format!("line too long ({} > {} characters)", line_length, self.max),
                    self.name(),
                    LintLevel::Error,
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
    fn test_line_within_limit() {
        let yaml = "key: value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = LineLengthRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_line_exceeds_limit() {
        // Create a line with 81 characters (exceeds default 80)
        // "key: " = 5 chars, so we need 76 more to get 81
        let long_line = "key: ".to_string() + &"x".repeat(76); // 5 + 76 = 81
        let yaml = format!("{}\n", long_line);
        let context = LintContext::new(yaml);
        let rule = LineLengthRule::new();
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].line, 1);
        assert_eq!(problems[0].column, 81); // Column where it exceeds
        assert!(problems[0].message.contains("81 > 80"));
    }

    #[test]
    fn test_custom_max_length() {
        let yaml = "key: ".to_string() + &"x".repeat(100); // 104 characters
        let context = LintContext::new(yaml);
        let rule = LineLengthRule::with_max(120);
        let problems = rule.check(&context);

        assert!(problems.is_empty()); // Should pass with max=120
    }

    #[test]
    fn test_multiple_long_lines() {
        let line1 = "key1: ".to_string() + &"x".repeat(81); // 86 chars
        let line2 = "key2: value"; // Short
        let line3 = "key3: ".to_string() + &"x".repeat(90); // 96 chars
        let yaml = format!("{}\n{}\n{}\n", line1, line2, line3);
        let context = LintContext::new(yaml);
        let rule = LineLengthRule::new();
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 2);
        assert_eq!(problems[0].line, 1);
        assert_eq!(problems[1].line, 3);
    }

    #[test]
    fn test_exactly_at_limit() {
        // Exactly 80 characters should pass
        let yaml = "x".repeat(80) + "\n";
        let context = LintContext::new(yaml);
        let rule = LineLengthRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_empty_line() {
        let yaml = "\n";
        let context = LintContext::new(yaml.to_string());
        let rule = LineLengthRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }
}
