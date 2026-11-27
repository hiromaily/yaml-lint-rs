//! Empty lines rule - limits consecutive blank lines

use super::new_lines::split_ending;
use crate::problem::{LintLevel, LintProblem};
use crate::rules::{LintContext, Rule, RuleLevel};
use std::collections::HashSet;

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
        let mut lines = context.content.split_inclusive('\n').peekable();
        let mut line_number = 0;
        while let Some(chunk) = lines.next() {
            line_number += 1;
            if !matches!(chunk, "\n" | "\r\n") {
                continue;
            }
            let first_line = line_number;
            let mut run = 1;
            while lines
                .peek()
                .is_some_and(|next| matches!(*next, "\n" | "\r\n"))
            {
                lines.next();
                line_number += 1;
                run += 1;
            }
            let limit = if lines.peek().is_none() {
                self.max_end
            } else if first_line == 1 {
                self.max_start
            } else {
                self.max
            };
            if run > limit && !matches!(context.content.as_str(), "\n" | "\r\n") {
                problems.push(LintProblem::new(
                    line_number,
                    1,
                    format!("too many blank lines ({run} > {limit})"),
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

    fn is_fixable(&self) -> bool {
        true
    }

    fn fix(&self, content: &str, _problem: &LintProblem) -> Option<String> {
        let lines: Vec<&str> = content.split_inclusive('\n').collect();
        if lines.is_empty() || matches!(content, "\n" | "\r\n") {
            return None;
        }

        let mut suppressed = HashSet::new();
        let mut standalone_targets = HashSet::new();
        for directive in super::directives::collect(content) {
            if directive.rule == self.name() {
                suppressed.insert(directive.target);
            }
            if directive.standalone {
                standalone_targets.insert(directive.target);
            }
        }
        let mut result = String::with_capacity(content.len());
        let skip_until = lines
            .iter()
            .position(|line| !matches!(*line, "\n" | "\r\n"))
            .unwrap_or(lines.len());
        let leading_limit = if skip_until == lines.len() {
            self.max_end
        } else {
            self.max_start
        };
        for line in &lines[..leading_limit.min(skip_until)] {
            result.push_str(split_ending(line).1);
        }

        // Keep a standalone directive attached to its original line when
        // shortening a blank run, even if the configured limit is zero.
        let keep_blank_lines = |result: &mut String, start: usize, count: usize, limit: usize| {
            let (original, kept) = if suppressed.contains(&(start + 1))
                || suppressed.contains(&(start + count))
            {
                (count, count)
            } else {
                let original = usize::from(count > 0 && standalone_targets.contains(&(start + 1)));
                (original, count.min(limit).max(original))
            };
            for line in &lines[start..start + original] {
                result.push_str(line);
            }
            for line in &lines[start + original..start + kept] {
                result.push_str(split_ending(line).1);
            }
        };

        // Process middle content
        let last_content = lines
            .iter()
            .rposition(|line| !matches!(*line, "\n" | "\r\n"));
        let mut consecutive_empty = 0;
        for (idx, line) in lines.iter().enumerate().skip(skip_until) {
            if matches!(*line, "\n" | "\r\n") {
                consecutive_empty += 1;
                // Don't add yet, wait until we know how many to add
            } else {
                // Add up to max empty lines before this content
                let start = idx - consecutive_empty;
                keep_blank_lines(&mut result, start, consecutive_empty, self.max);
                consecutive_empty = 0;

                result.push_str(line);

                // If this is the last content line, handle trailing empty lines
                if Some(idx) == last_content {
                    let start = idx + 1;
                    keep_blank_lines(&mut result, start, lines.len() - start, self.max_end);
                    break;
                }
            }
        }

        (result != content).then_some(result)
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
        assert!(problems[0].message.contains("too many blank lines (1 > 0)"));
    }

    #[test]
    fn test_empty_lines_at_end() {
        let yaml = "key: value\n\n";
        let context = LintContext::new(yaml.to_string());
        let rule = EmptyLinesRule::new();
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
        assert!(problems[0].message.contains("too many blank lines (1 > 0)"));
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
        assert!(problems[0].message.contains("too many blank lines (2 > 1)"));
    }

    #[test]
    fn test_custom_max_end() {
        let yaml = "key: value\n\n\n";
        let context = LintContext::new(yaml.to_string());
        let rule = EmptyLinesRule::with_config(2, 0, 1);
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
        assert!(problems[0].message.contains("too many blank lines (2 > 1)"));
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
    fn test_whitespace_only_lines_are_not_empty() {
        let yaml = "key1: value1\n   \n   \n   \nkey2: value2\n";
        let context = LintContext::new(yaml.to_string());
        let rule = EmptyLinesRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
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
        let context = LintContext::new("\n\n\n".into());
        let problems = EmptyLinesRule::new().check(&context);
        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].line, 3);
        assert_eq!(problems[0].message, "too many blank lines (3 > 0)");
    }
}
