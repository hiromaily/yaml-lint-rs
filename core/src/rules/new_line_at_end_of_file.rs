//! New line at end of file rule - ensures files end with a newline

use crate::problem::{LintLevel, LintProblem};
use crate::rules::{LintContext, Rule, RuleLevel};

/// Rule that checks if a file ends with a newline character
///
/// This is a POSIX standard requirement for text files and prevents
/// unnecessary diffs in version control when adding content to file end.
#[derive(Debug)]
pub struct NewLineAtEndOfFileRule;

impl Rule for NewLineAtEndOfFileRule {
    fn name(&self) -> &'static str {
        "new-line-at-end-of-file"
    }

    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        let mut problems = Vec::new();

        // Empty files are considered valid
        if context.content.is_empty() {
            return problems;
        }

        // Check if content ends with a newline
        if !context.content.ends_with('\n') {
            let line_count = context.lines.len();
            let last_line_len = context.lines.last().map(|l| l.len()).unwrap_or(0);

            problems.push(LintProblem::new(
                line_count,
                last_line_len + 1,
                "no new line character at the end of file",
                self.name(),
                LintLevel::Error,
            ));
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
        // Simply add a newline at the end if missing
        if content.is_empty() || content.ends_with('\n') {
            return None; // Already valid
        }

        Some(format!("{}\n", content))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_with_newline() {
        let yaml = "key: value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = NewLineAtEndOfFileRule;
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_file_without_newline() {
        let yaml = "key: value";
        let context = LintContext::new(yaml.to_string());
        let rule = NewLineAtEndOfFileRule;
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].line, 1);
        assert_eq!(problems[0].column, 11); // After "key: value"
        assert_eq!(
            problems[0].message,
            "no new line character at the end of file"
        );
        assert_eq!(problems[0].level, LintLevel::Error);
    }

    #[test]
    fn test_multiline_file_without_newline() {
        let yaml = "key1: value1\nkey2: value2";
        let context = LintContext::new(yaml.to_string());
        let rule = NewLineAtEndOfFileRule;
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].line, 2);
        assert_eq!(problems[0].column, 13); // After "key2: value2"
    }

    #[test]
    fn test_multiline_file_with_newline() {
        let yaml = "key1: value1\nkey2: value2\n";
        let context = LintContext::new(yaml.to_string());
        let rule = NewLineAtEndOfFileRule;
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_empty_file() {
        let yaml = "";
        let context = LintContext::new(yaml.to_string());
        let rule = NewLineAtEndOfFileRule;
        let problems = rule.check(&context);

        // Empty files are considered valid
        assert!(problems.is_empty());
    }

    #[test]
    fn test_only_newline() {
        let yaml = "\n";
        let context = LintContext::new(yaml.to_string());
        let rule = NewLineAtEndOfFileRule;
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_multiple_trailing_newlines() {
        let yaml = "key: value\n\n\n";
        let context = LintContext::new(yaml.to_string());
        let rule = NewLineAtEndOfFileRule;
        let problems = rule.check(&context);

        // Multiple trailing newlines are still valid (ends with \n)
        assert!(problems.is_empty());
    }
}
