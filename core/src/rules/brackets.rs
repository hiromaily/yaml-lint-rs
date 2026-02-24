//! Brackets rule - controls spacing inside flow sequences `[]`

use crate::problem::{LintLevel, LintProblem};
use crate::rules::{LintContext, Rule, RuleLevel};

/// Rule that controls spacing inside flow sequences (brackets `[]`)
///
/// This rule enforces consistent spacing immediately inside `[` and `]` characters,
/// and can optionally forbid flow sequences entirely.
#[derive(Debug)]
pub struct BracketsRule {
    /// Forbid flow sequences entirely
    forbid: bool,
    /// Minimum spaces inside brackets
    min_spaces_inside: usize,
    /// Maximum spaces inside brackets
    max_spaces_inside: usize,
    /// Minimum spaces inside empty brackets (-1 means use min_spaces_inside)
    min_spaces_inside_empty: i32,
    /// Maximum spaces inside empty brackets (-1 means use max_spaces_inside)
    max_spaces_inside_empty: i32,
}

impl BracketsRule {
    /// Create a new rule with default settings
    pub fn new() -> Self {
        Self {
            forbid: false,
            min_spaces_inside: 0,
            max_spaces_inside: 0,
            min_spaces_inside_empty: -1,
            max_spaces_inside_empty: -1,
        }
    }

    /// Create a new rule with custom settings
    pub fn with_config(
        forbid: bool,
        min_spaces_inside: usize,
        max_spaces_inside: usize,
        min_spaces_inside_empty: i32,
        max_spaces_inside_empty: i32,
    ) -> Self {
        Self {
            forbid,
            min_spaces_inside,
            max_spaces_inside,
            min_spaces_inside_empty,
            max_spaces_inside_empty,
        }
    }

    /// Get effective min spaces for empty brackets
    fn effective_min_empty(&self) -> usize {
        if self.min_spaces_inside_empty < 0 {
            self.min_spaces_inside
        } else {
            self.min_spaces_inside_empty as usize
        }
    }

    /// Get effective max spaces for empty brackets
    fn effective_max_empty(&self) -> usize {
        if self.max_spaces_inside_empty < 0 {
            self.max_spaces_inside
        } else {
            self.max_spaces_inside_empty as usize
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
        let mut problems = Vec::new();

        for (line_idx, line) in context.lines.iter().enumerate() {
            let line_num = line_idx + 1;
            let chars: Vec<char> = line.chars().collect();
            let mut in_single_quote = false;
            let mut in_double_quote = false;
            let mut i = 0;

            while i < chars.len() {
                let ch = chars[i];

                // Track quote state
                if ch == '\'' && !in_double_quote {
                    in_single_quote = !in_single_quote;
                    i += 1;
                    continue;
                }
                if ch == '"' && !in_single_quote {
                    if i > 0 && chars[i - 1] == '\\' {
                        i += 1;
                        continue;
                    }
                    in_double_quote = !in_double_quote;
                    i += 1;
                    continue;
                }

                // Skip if inside quotes
                if in_single_quote || in_double_quote {
                    i += 1;
                    continue;
                }

                // Skip comments
                if ch == '#' {
                    break;
                }

                if ch == '[' {
                    if self.forbid {
                        problems.push(LintProblem::new(
                            line_num,
                            i + 1,
                            "flow sequence (brackets) is forbidden",
                            self.name(),
                            LintLevel::Error,
                        ));
                        i += 1;
                        continue;
                    }

                    // Check if empty brackets
                    let after_open = i + 1;
                    if let Some(close_pos) = self.find_closing_bracket(&chars, after_open) {
                        let content_between = &chars[after_open..close_pos];
                        let is_empty = content_between.iter().all(|c| *c == ' ');

                        if is_empty {
                            let spaces = close_pos - after_open;
                            let min = self.effective_min_empty();
                            let max = self.effective_max_empty();

                            if spaces < min {
                                problems.push(LintProblem::new(
                                    line_num,
                                    after_open + 1,
                                    format!(
                                        "too few spaces inside empty brackets ({} < {})",
                                        spaces, min
                                    ),
                                    self.name(),
                                    LintLevel::Error,
                                ));
                            } else if spaces > max {
                                problems.push(LintProblem::new(
                                    line_num,
                                    after_open + 1,
                                    format!(
                                        "too many spaces inside empty brackets ({} > {})",
                                        spaces, max
                                    ),
                                    self.name(),
                                    LintLevel::Error,
                                ));
                            }
                            i = close_pos + 1;
                            continue;
                        }
                    }

                    // Non-empty: check spaces after opening bracket
                    let spaces_after = chars[after_open..]
                        .iter()
                        .take_while(|c| **c == ' ')
                        .count();

                    if spaces_after < self.min_spaces_inside {
                        problems.push(LintProblem::new(
                            line_num,
                            after_open + 1,
                            format!(
                                "too few spaces inside brackets ({} < {})",
                                spaces_after, self.min_spaces_inside
                            ),
                            self.name(),
                            LintLevel::Error,
                        ));
                    } else if spaces_after > self.max_spaces_inside {
                        problems.push(LintProblem::new(
                            line_num,
                            after_open + 1,
                            format!(
                                "too many spaces inside brackets ({} > {})",
                                spaces_after, self.max_spaces_inside
                            ),
                            self.name(),
                            LintLevel::Error,
                        ));
                    }
                } else if ch == ']' && !self.forbid {
                    // Check spaces before closing bracket
                    if i > 0 {
                        let spaces_before =
                            chars[..i].iter().rev().take_while(|c| **c == ' ').count();

                        // Only check if the previous non-space char is not '['
                        // (empty brackets are handled above)
                        let prev_non_space = chars[..i].iter().rev().find(|c| **c != ' ');

                        if prev_non_space != Some(&'[') {
                            if spaces_before < self.min_spaces_inside {
                                problems.push(LintProblem::new(
                                    line_num,
                                    i + 1,
                                    format!(
                                        "too few spaces inside brackets ({} < {})",
                                        spaces_before, self.min_spaces_inside
                                    ),
                                    self.name(),
                                    LintLevel::Error,
                                ));
                            } else if spaces_before > self.max_spaces_inside {
                                problems.push(LintProblem::new(
                                    line_num,
                                    i + 1,
                                    format!(
                                        "too many spaces inside brackets ({} > {})",
                                        spaces_before, self.max_spaces_inside
                                    ),
                                    self.name(),
                                    LintLevel::Error,
                                ));
                            }
                        }
                    }
                }

                i += 1;
            }
        }

        problems
    }

    fn default_level(&self) -> RuleLevel {
        RuleLevel::Error
    }
}

impl BracketsRule {
    /// Find closing bracket on the same line, respecting quotes
    /// Returns the position of `]` if found and content between is only spaces
    fn find_closing_bracket(&self, chars: &[char], start: usize) -> Option<usize> {
        let mut j = start;
        let mut in_sq = false;
        let mut in_dq = false;
        let mut depth = 0;

        while j < chars.len() {
            let c = chars[j];
            if c == '\'' && !in_dq {
                in_sq = !in_sq;
            } else if c == '"' && !in_sq {
                if j > 0 && chars[j - 1] == '\\' {
                    j += 1;
                    continue;
                }
                in_dq = !in_dq;
            } else if !in_sq && !in_dq {
                if c == '[' {
                    depth += 1;
                } else if c == ']' {
                    if depth == 0 {
                        return Some(j);
                    }
                    depth -= 1;
                } else if c != ' ' {
                    // Non-space content found, not empty
                    return None;
                }
            }
            j += 1;
        }
        None
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
        let rule = BracketsRule::with_config(false, 1, 1, -1, -1);
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
        let rule = BracketsRule::with_config(false, 1, 1, -1, -1);
        let problems = rule.check(&context);
        assert!(problems.is_empty());
    }

    #[test]
    fn test_forbid() {
        let yaml = "sequence: [1, 2, 3]\n";
        let context = LintContext::new(yaml.to_string());
        let rule = BracketsRule::with_config(true, 0, 0, -1, -1);
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
        let rule = BracketsRule::with_config(false, 0, 0, 1, 1);
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
        let rule = BracketsRule::with_config(true, 0, 0, -1, -1);
        let problems = rule.check(&context);
        assert!(problems.is_empty());
    }
}
