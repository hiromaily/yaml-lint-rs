//! Colons rule - validates spacing around colons in mappings

use crate::problem::{LintLevel, LintProblem};
use crate::rules::{LintContext, Rule, RuleLevel};

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
        let mut problems = Vec::new();

        for (line_idx, line) in context.lines.iter().enumerate() {
            // Skip comment lines and empty lines
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Skip document markers and directives
            if trimmed.starts_with("---") || trimmed.starts_with("...") || trimmed.starts_with('%')
            {
                continue;
            }

            // Find colons that are part of key-value pairs (not in strings)
            // Simple approach: look for colons followed by space or end of line
            let mut in_single_quote = false;
            let mut in_double_quote = false;
            let mut escaped = false;

            for (col_idx, ch) in line.char_indices() {
                if escaped {
                    escaped = false;
                    continue;
                }

                match ch {
                    '\\' if in_double_quote => escaped = true,
                    '\'' if !in_double_quote => in_single_quote = !in_single_quote,
                    '"' if !in_single_quote => in_double_quote = !in_double_quote,
                    ':' if !in_single_quote && !in_double_quote => {
                        // Found a colon outside of quotes
                        // Check if it's part of a key-value pair (followed by space, newline, or comment)
                        let rest = &line[col_idx + 1..];
                        let next_char = rest.chars().next();

                        // This looks like a mapping colon if followed by space, nothing, or comment
                        if matches!(next_char, None | Some(' ') | Some('#') | Some('\n')) {
                            // Check spaces before colon
                            let spaces_before = line[..col_idx]
                                .chars()
                                .rev()
                                .take_while(|&c| c == ' ')
                                .count();

                            if spaces_before > self.max_spaces_before {
                                problems.push(LintProblem::new(
                                    line_idx + 1,
                                    col_idx + 1,
                                    format!(
                                        "too many spaces before colon ({} > {})",
                                        spaces_before, self.max_spaces_before
                                    ),
                                    self.name(),
                                    LintLevel::Error,
                                ));
                            }

                            // Check spaces after colon
                            if let Some(' ') = next_char {
                                let spaces_after = rest.chars().take_while(|&c| c == ' ').count();

                                if spaces_after > self.max_spaces_after {
                                    problems.push(LintProblem::new(
                                        line_idx + 1,
                                        col_idx + 2,
                                        format!(
                                            "too many spaces after colon ({} > {})",
                                            spaces_after, self.max_spaces_after
                                        ),
                                        self.name(),
                                        LintLevel::Error,
                                    ));
                                }
                            } else if next_char.is_none() {
                                // Colon at end of line is okay (value on next line)
                            } else if let Some('#') = next_char {
                                // Colon followed by comment - should have space
                                if self.max_spaces_after > 0 {
                                    problems.push(LintProblem::new(
                                        line_idx + 1,
                                        col_idx + 2,
                                        "missing space after colon",
                                        self.name(),
                                        LintLevel::Error,
                                    ));
                                }
                            }

                            // Only check first colon in line for simplicity
                            break;
                        }
                    }
                    _ => {}
                }
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
