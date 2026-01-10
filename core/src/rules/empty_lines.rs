//! Empty lines rule - limits consecutive blank lines

use crate::problem::{LintLevel, LintProblem};
use crate::rules::{LintContext, Rule, RuleLevel};

/// Rule that limits consecutive empty lines in YAML files
///
/// This rule helps maintain clean, readable files by preventing
/// excessive blank lines that can make documents harder to navigate.
#[derive(Debug)]
pub struct EmptyLinesRule {
    /// Maximum allowed consecutive empty lines (default: 2)
    max: usize,
    /// Maximum empty lines at start of file (default: 0)
    max_start: usize,
    /// Maximum empty lines at end of file (default: 0)
    max_end: usize,
}

impl EmptyLinesRule {
    /// Create a new rule with default settings
    pub fn new() -> Self {
        Self {
            max: 2,
            max_start: 0,
            max_end: 0,
        }
    }

    /// Create a new rule with custom settings
    pub fn with_config(max: usize, max_start: usize, max_end: usize) -> Self {
        Self {
            max,
            max_start,
            max_end,
        }
    }

    /// Check if a line is empty (only whitespace)
    fn is_empty_line(line: &str) -> bool {
        line.trim().is_empty()
    }
}

impl Default for EmptyLinesRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for EmptyLinesRule {
    fn name(&self) -> &'static str {
        "empty-lines"
    }

    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        let mut problems = Vec::new();

        if context.lines.is_empty() {
            return problems;
        }

        // Check empty lines at start of file
        let mut start_empty_count = 0;
        for line in &context.lines {
            if Self::is_empty_line(line) {
                start_empty_count += 1;
            } else {
                break;
            }
        }

        if start_empty_count > self.max_start {
            problems.push(LintProblem::new(
                1,
                1,
                format!(
                    "too many blank lines at start of file ({} > {})",
                    start_empty_count, self.max_start
                ),
                self.name(),
                LintLevel::Error,
            ));
        }

        // If entire file is empty lines, don't report as end-of-file error too
        if start_empty_count == context.lines.len() {
            return problems;
        }

        // Check empty lines at end of file
        let mut end_empty_count = 0;
        for line in context.lines.iter().rev() {
            if Self::is_empty_line(line) {
                end_empty_count += 1;
            } else {
                break;
            }
        }

        if end_empty_count > self.max_end {
            let line_num = context.lines.len() - end_empty_count + 1;
            problems.push(LintProblem::new(
                line_num,
                1,
                format!(
                    "too many blank lines at end of file ({} > {})",
                    end_empty_count, self.max_end
                ),
                self.name(),
                LintLevel::Error,
            ));
        }

        // Check consecutive empty lines in the middle
        let mut consecutive_empty = 0;
        let mut empty_block_start = 0;

        for (idx, line) in context.lines.iter().enumerate() {
            if Self::is_empty_line(line) {
                if consecutive_empty == 0 {
                    empty_block_start = idx + 1; // 1-indexed
                }
                consecutive_empty += 1;
            } else {
                if consecutive_empty > self.max {
                    // Don't report if this is the start of file (already reported)
                    let is_at_start = empty_block_start == 1;

                    if !is_at_start {
                        problems.push(LintProblem::new(
                            empty_block_start,
                            1,
                            format!(
                                "too many blank lines ({} > {})",
                                consecutive_empty, self.max
                            ),
                            self.name(),
                            LintLevel::Error,
                        ));
                    }
                }
                consecutive_empty = 0;
            }
        }

        problems
    }

    fn default_level(&self) -> RuleLevel {
        RuleLevel::Error
    }

    fn is_fixable(&self) -> bool {
        true
    }

    fn fix(&self, content: &str, _problem: &LintProblem) -> Option<String> {
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return None;
        }

        let mut result_lines: Vec<&str> = Vec::new();

        // Skip empty lines at start (keep max_start)
        let mut start_empty_count = 0;
        let mut skip_until = 0;
        for (idx, line) in lines.iter().enumerate() {
            if line.trim().is_empty() {
                start_empty_count += 1;
            } else {
                skip_until = idx;
                break;
            }
        }
        // Add allowed empty lines at start
        let start_empty_to_add = self.max_start.min(start_empty_count);
        result_lines.extend(std::iter::repeat_n("", start_empty_to_add));

        // Process middle content
        let mut consecutive_empty = 0;
        for (idx, line) in lines.iter().enumerate().skip(skip_until) {
            let is_last_content_line =
                idx == lines.len() - 1 || lines[idx + 1..].iter().all(|l| l.trim().is_empty());

            if line.trim().is_empty() {
                consecutive_empty += 1;
                // Don't add yet, wait until we know how many to add
            } else {
                // Add up to max empty lines before this content
                let empty_to_add = consecutive_empty.min(self.max);
                result_lines.extend(std::iter::repeat_n("", empty_to_add));
                consecutive_empty = 0;

                result_lines.push(line);

                // If this is the last content line, handle trailing empty lines
                if is_last_content_line {
                    let end_empty_count = lines.len() - idx - 1;
                    let end_empty_to_add = self.max_end.min(end_empty_count);
                    result_lines.extend(std::iter::repeat_n("", end_empty_to_add));
                    break;
                }
            }
        }

        let mut result = result_lines.join("\n");
        if content.ends_with('\n') {
            result.push('\n');
        }

        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_empty_lines() {
        let yaml = "key1: value1\nkey2: value2\n";
        let context = LintContext::new(yaml.to_string());
        let rule = EmptyLinesRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_single_empty_line() {
        let yaml = "key1: value1\n\nkey2: value2\n";
        let context = LintContext::new(yaml.to_string());
        let rule = EmptyLinesRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_two_empty_lines_allowed() {
        let yaml = "key1: value1\n\n\nkey2: value2\n";
        let context = LintContext::new(yaml.to_string());
        let rule = EmptyLinesRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_three_empty_lines_error() {
        let yaml = "key1: value1\n\n\n\nkey2: value2\n";
        let context = LintContext::new(yaml.to_string());
        let rule = EmptyLinesRule::new();
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
        assert!(problems[0].message.contains("too many blank lines (3 > 2)"));
    }

    #[test]
    fn test_empty_line_at_start() {
        let yaml = "\nkey: value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = EmptyLinesRule::new();
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
        assert!(problems[0].message.contains("at start of file"));
    }

    #[test]
    fn test_empty_lines_at_end() {
        let yaml = "key: value\n\n";
        let context = LintContext::new(yaml.to_string());
        let rule = EmptyLinesRule::new();
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
        assert!(problems[0].message.contains("at end of file"));
    }

    #[test]
    fn test_custom_max() {
        let yaml = "key1: value1\n\nkey2: value2\n";
        let context = LintContext::new(yaml.to_string());
        let rule = EmptyLinesRule::with_config(0, 0, 0);
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
        assert!(problems[0].message.contains("too many blank lines (1 > 0)"));
    }

    #[test]
    fn test_custom_max_start() {
        let yaml = "\n\nkey: value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = EmptyLinesRule::with_config(2, 1, 0);
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
        assert!(problems[0].message.contains("at start of file (2 > 1)"));
    }

    #[test]
    fn test_custom_max_end() {
        let yaml = "key: value\n\n\n";
        let context = LintContext::new(yaml.to_string());
        let rule = EmptyLinesRule::with_config(2, 0, 1);
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
        assert!(problems[0].message.contains("at end of file (2 > 1)"));
    }

    #[test]
    fn test_empty_file() {
        let yaml = "";
        let context = LintContext::new(yaml.to_string());
        let rule = EmptyLinesRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_whitespace_only_lines_count_as_empty() {
        let yaml = "key1: value1\n   \n   \n   \nkey2: value2\n";
        let context = LintContext::new(yaml.to_string());
        let rule = EmptyLinesRule::new();
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
        assert!(problems[0].message.contains("too many blank lines (3 > 2)"));
    }

    #[test]
    fn test_multiple_violations() {
        let yaml = "\nkey1: value1\n\n\n\nkey2: value2\n\n";
        let context = LintContext::new(yaml.to_string());
        let rule = EmptyLinesRule::new();
        let problems = rule.check(&context);

        // Should report: start (1 > 0), middle (3 > 2), end (1 > 0)
        assert_eq!(problems.len(), 3);
    }

    #[test]
    fn test_all_empty_lines_file() {
        // File with only empty lines should only report start error, not both start and end
        let yaml = "\n\n\n";
        let context = LintContext::new(yaml.to_string());
        let rule = EmptyLinesRule::new();
        let problems = rule.check(&context);

        // Should only report start error (3 > 0), not end error too
        assert_eq!(problems.len(), 1);
        assert!(problems[0].message.contains("at start of file"));
    }
}
