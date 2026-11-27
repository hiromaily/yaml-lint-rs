//! Line length rule - enforces maximum line length

use crate::problem::{LintLevel, LintProblem};
use crate::rules::{LintContext, Rule, RuleLevel};
use yaml_rust2::scanner::{Scanner, Token, TokenType};

/// Rule that enforces maximum line length
#[derive(Debug)]
pub struct LineLengthRule {
    /// Maximum allowed line length
    pub max: usize,
}

#[derive(Debug)]
pub(crate) struct ConfiguredLineLength {
    rule: LineLengthRule,
    allow_non_breakable_words: bool,
    allow_non_breakable_inline_mappings: bool,
}

impl LineLengthRule {
    /// Create a new line length rule with the default max (80)
    pub fn new() -> Self {
        Self::with_max(80)
    }

    /// Create a line length rule with a custom max
    pub fn with_max(max: usize) -> Self {
        Self { max }
    }
}

impl ConfiguredLineLength {
    pub(crate) fn new(
        max: usize,
        allow_non_breakable_words: bool,
        allow_non_breakable_inline_mappings: bool,
    ) -> Self {
        Self {
            rule: LineLengthRule::with_max(max),
            allow_non_breakable_words,
            allow_non_breakable_inline_mappings,
        }
    }

    fn allows_long_line(&self, line: &str) -> bool {
        let content = line.trim_start_matches(' ');
        if content.is_empty()
            || !(self.allow_non_breakable_words || self.allow_non_breakable_inline_mappings)
        {
            return false;
        }

        let word = if content.starts_with('#') {
            let mut chars = content.trim_start_matches('#').chars();
            chars.next();
            chars.as_str()
        } else {
            content.strip_prefix("- ").unwrap_or(content)
        };
        !word.contains(' ')
            || (self.allow_non_breakable_inline_mappings && inline_mapping_has_one_word(content))
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
        ConfiguredLineLength::new(self.max, true, false).check(context)
    }
}

impl Rule for ConfiguredLineLength {
    fn name(&self) -> &'static str {
        self.rule.name()
    }

    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        let mut problems = Vec::new();

        for (line_idx, line) in context.lines.iter().enumerate() {
            // Skip trailing newline in length calculation
            let line_length = line.chars().count();

            if line_length > self.rule.max && !self.allows_long_line(line) {
                problems.push(LintProblem::new(
                    line_idx + 1,      // 1-indexed
                    self.rule.max + 1, // Column where it exceeds
                    format!(
                        "line too long ({} > {} characters)",
                        line_length, self.rule.max
                    ),
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

fn inline_mapping_has_one_word(line: &str) -> bool {
    let mut tokens = Scanner::new(line.chars());
    if !tokens.any(|Token(_, token)| matches!(token, TokenType::BlockMappingStart))
        || !tokens.any(|Token(_, token)| matches!(token, TokenType::Value))
    {
        return false;
    }

    let Some(Token(mark, TokenType::Scalar(..))) = tokens.next() else {
        return false;
    };
    !line.chars().skip(mark.col()).any(|ch| ch == ' ')
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
