//! Truthy rule - restricts boolean representations to avoid YAML 1.1 vs 1.2 ambiguities

use super::{LintContext, Rule, RuleLevel, tokens};
use crate::problem::LintProblem;
use yaml_rust2::scanner::{TScalarStyle, TokenType};

pub(crate) const YAML_11_VALUES: &[&str] = &[
    "YES", "Yes", "yes", "NO", "No", "no", "TRUE", "True", "true", "FALSE", "False", "false", "ON",
    "On", "on", "OFF", "Off", "off",
];

const YAML_12_VALUES: &[&str] = &["TRUE", "True", "true", "FALSE", "False", "false"];

/// Reports unquoted boolean spellings outside the configured allowlist.
#[derive(Debug)]
pub struct TruthyRule {
    allowed_values: Vec<String>,
    check_keys: bool,
}

impl TruthyRule {
    /// Create a rule with lowercase booleans allowed and key checking enabled.
    pub fn new() -> Self {
        Self::with_config(vec!["true".into(), "false".into()], true)
    }

    /// Create a rule with a case-sensitive allowlist.
    pub fn with_config(mut allowed_values: Vec<String>, check_keys: bool) -> Self {
        allowed_values.sort();
        Self {
            allowed_values,
            check_keys,
        }
    }
}

impl Default for TruthyRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for TruthyRule {
    fn name(&self) -> &'static str {
        "truthy"
    }

    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        let mut values = YAML_11_VALUES;
        let mut previous = None;
        let mut problems = Vec::new();
        for yaml_rust2::scanner::Token(mark, token) in
            yaml_rust2::scanner::Scanner::new(context.content.chars())
        {
            match &token {
                TokenType::VersionDirective(1, 2) => values = YAML_12_VALUES,
                TokenType::VersionDirective(..) | TokenType::DocumentEnd => values = YAML_11_VALUES,
                TokenType::Scalar(TScalarStyle::Plain, value)
                    if !matches!(previous, Some(TokenType::Tag(..)))
                        && (self.check_keys || !matches!(previous, Some(TokenType::Key)))
                        && values.contains(&value.as_str())
                        && !self.allowed_values.contains(value) =>
                {
                    problems.push(tokens::problem(
                        mark,
                        self.name(),
                        format!(
                            "truthy value should be one of [{}]",
                            self.allowed_values.join(", ")
                        ),
                    ));
                }
                _ => {}
            }
            previous = Some(token);
        }
        problems
    }

    fn default_level(&self) -> RuleLevel {
        RuleLevel::Warning
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_booleans() {
        let yaml = "enabled: true\ndisabled: false\n";
        let context = LintContext::new(yaml.to_string());
        let rule = TruthyRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_yaml_11_yes_no() {
        let yaml = "enabled: yes\ndisabled: no\n";
        let context = LintContext::new(yaml.to_string());
        let rule = TruthyRule::new();
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 2);
        assert_eq!((problems[0].line, problems[0].column), (1, 10));
        assert_eq!((problems[1].line, problems[1].column), (2, 11));
    }

    #[test]
    fn test_yaml_11_on_off() {
        let yaml = "feature: on\nlegacy: off\n";
        let context = LintContext::new(yaml.to_string());
        let rule = TruthyRule::new();
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 2);
        assert_eq!((problems[0].line, problems[0].column), (1, 10));
        assert_eq!((problems[1].line, problems[1].column), (2, 9));
    }

    #[test]
    fn test_y_and_n_are_not_checked() {
        let yaml = "flag: y\nother: n\n";
        let context = LintContext::new(yaml.to_string());
        assert!(TruthyRule::new().check(&context).is_empty());
    }

    #[test]
    fn test_standard_case_variants() {
        let yaml = "a: YES\nb: No\nc: ON\nd: Off\n";
        let context = LintContext::new(yaml.to_string());
        let rule = TruthyRule::new();
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 4);
    }

    #[test]
    fn test_quoted_values_allowed() {
        let yaml = "country: \"NO\"\nanswer: 'yes'\n";
        let context = LintContext::new(yaml.to_string());
        let rule = TruthyRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_custom_allowed_values() {
        let yaml = "enabled: yes\ndisabled: no\n";
        let context = LintContext::new(yaml.to_string());
        let rule = TruthyRule::with_config(
            vec![
                "true".to_string(),
                "false".to_string(),
                "yes".to_string(),
                "no".to_string(),
            ],
            false,
        );
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_check_keys_disabled() {
        let yaml = "yes: value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = TruthyRule::with_config(vec![], false);
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_check_keys_enabled() {
        let yaml = "yes: value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = TruthyRule::with_config(vec!["true".to_string(), "false".to_string()], true);
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
        assert_eq!((problems[0].line, problems[0].column), (1, 1));
    }

    #[test]
    fn test_list_items() {
        let yaml = "items:\n  - yes\n  - no\n  - true\n";
        let context = LintContext::new(yaml.to_string());
        let rule = TruthyRule::new();
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 2);
    }

    #[test]
    fn test_inline_comment() {
        let yaml = "enabled: yes  # comment\n";
        let context = LintContext::new(yaml.to_string());
        let rule = TruthyRule::new();
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
    }

    #[test]
    fn test_nested_values() {
        let yaml = "config:\n  debug: yes\n  verbose: on\n";
        let context = LintContext::new(yaml.to_string());
        let rule = TruthyRule::new();
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 2);
    }

    #[test]
    fn test_non_truthy_values() {
        let yaml = "name: John\ncount: 42\npath: /etc/config\n";
        let context = LintContext::new(yaml.to_string());
        let rule = TruthyRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_empty_and_comment_lines() {
        let yaml = "# comment\n\nkey: true\n";
        let context = LintContext::new(yaml.to_string());
        let rule = TruthyRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }
}
