//! Trailing spaces rule - detects whitespace at line endings

use super::new_lines::split_ending;
use crate::problem::{LintLevel, LintProblem};
use crate::rules::{LintContext, Rule, RuleLevel};

/// Rule that detects trailing spaces at the end of lines
#[derive(Debug)]
pub struct TrailingSpacesRule;

impl Rule for TrailingSpacesRule {
    fn name(&self) -> &'static str {
        "trailing-spaces"
    }

    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        let mut problems = Vec::new();

        for (line_idx, line) in context.lines.iter().enumerate() {
            // Check if line ends with whitespace (space or tab)
            if line.ends_with(' ') || line.ends_with('\t') {
                let trimmed_len = line.trim_end().len();
                let column = trimmed_len + 1; // 1-indexed

                problems.push(LintProblem::new(
                    line_idx + 1, // 1-indexed line number
                    column,
                    "trailing spaces",
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
        let suppressed = super::directives::lines_for_rule(content, self.name());
        let mut result = String::with_capacity(content.len());
        let mut changed = false;
        for (index, chunk) in content.split_inclusive('\n').enumerate() {
            let (line, ending) = split_ending(chunk);
            if !suppressed.contains(&(index + 1)) && line.ends_with([' ', '\t']) {
                result.push_str(line.trim_end());
                result.push_str(ending);
                changed = true;
            } else {
                result.push_str(chunk);
            }
        }
        changed.then_some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_trailing_spaces() {
        let yaml = "key: value\nkey2: value2\n";
        let context = LintContext::new(yaml.to_string());
        let rule = TrailingSpacesRule;
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_trailing_spaces_detected() {
        let yaml = "key: value   \nkey2: value2\n";
        let context = LintContext::new(yaml.to_string());
        let rule = TrailingSpacesRule;
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].line, 1);
        assert_eq!(problems[0].column, 11); // After "key: value"
        assert_eq!(problems[0].message, "trailing spaces");
        assert_eq!(problems[0].level, LintLevel::Error);
    }

    #[test]
    fn test_trailing_tabs() {
        let yaml = "key: value\t\nkey2: value2\n";
        let context = LintContext::new(yaml.to_string());
        let rule = TrailingSpacesRule;
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].line, 1);
    }

    #[test]
    fn test_multiple_lines_with_trailing_spaces() {
        let yaml = "key1: value1  \nkey2: value2\nkey3: value3   \n";
        let context = LintContext::new(yaml.to_string());
        let rule = TrailingSpacesRule;
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 2);
        assert_eq!(problems[0].line, 1);
        assert_eq!(problems[1].line, 3);
    }

    #[test]
    fn test_empty_lines() {
        let yaml = "key: value\n\nkey2: value2\n";
        let context = LintContext::new(yaml.to_string());
        let rule = TrailingSpacesRule;
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }
}
