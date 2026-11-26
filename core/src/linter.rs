//! Main linting orchestration

use crate::Result;
use crate::config::Config;
use crate::problem::LintProblem;
use crate::rules::{LintContext, RuleRegistry};
use std::path::Path;

/// Main linter that orchestrates the linting process
#[derive(Debug)]
pub struct Linter {
    config: Config,
    registry: RuleRegistry,
}

impl Linter {
    /// Create a new linter with the given configuration
    pub fn new(config: Config) -> Self {
        let registry = config.create_registry();

        Self { config, registry }
    }

    /// Create a linter with default configuration
    pub fn with_defaults() -> Self {
        Self::new(Config::default())
    }

    /// Lint a file at the given path
    pub fn lint_file(&self, path: &Path) -> Result<Vec<LintProblem>> {
        let content = std::fs::read_to_string(path)?;
        self.lint_string(&content)
    }

    /// Lint a YAML string
    pub fn lint_string(&self, content: &str) -> Result<Vec<LintProblem>> {
        // First, try to parse the YAML to catch syntax errors
        // For now, we'll skip YAML parsing errors and just run line-based rules
        // In future, we'll add proper YAML parsing with yaml-rust2

        let context = LintContext::new(content.to_string());
        let problems = self.registry.check_all(&context);

        Ok(problems)
    }

    /// Get a reference to the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lint_valid_yaml() {
        let linter = Linter::with_defaults();
        let yaml = "key: value\nkey2: value2\n";
        let problems = linter.lint_string(yaml).unwrap();
        assert!(problems.is_empty());
    }

    #[test]
    fn test_lint_with_trailing_spaces() {
        let linter = Linter::with_defaults();
        let yaml = "key: value   \nkey2: value2\n";
        let problems = linter.lint_string(yaml).unwrap();
        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].rule, "trailing-spaces");
    }

    #[test]
    fn test_lint_respects_config() {
        let mut config = Config::new();
        config.rules.insert(
            "trailing-spaces".to_string(),
            crate::config::RuleConfig::Level(crate::rules::RuleLevel::Disable),
        );

        let linter = Linter::new(config);
        let yaml = "key: value   \n";
        let problems = linter.lint_string(yaml).unwrap();
        assert!(problems.is_empty()); // Rule is disabled
    }
}

#[cfg(test)]
mod directive_tests {
    use super::*;

    #[test]
    fn disable_line_targets_only_the_named_rule() {
        let yaml =
            "a: value   \nb: value # yamllint disable-line rule:trailing-spaces   \nc: value   \n";
        let problems = Linter::with_defaults().lint_string(yaml).unwrap();
        let trailing_lines: Vec<_> = problems
            .iter()
            .filter(|p| p.rule == "trailing-spaces")
            .map(|p| p.line)
            .collect();
        assert_eq!(trailing_lines, [1, 3]);
        assert!(problems.iter().any(|p| p.line == 2 && p.rule == "comments"));
    }

    #[test]
    fn standalone_disable_line_targets_the_following_line() {
        let yaml = "# yamllint disable-line rule:trailing-spaces\na: value   \nb: value   \n";
        let problems = Linter::with_defaults().lint_string(yaml).unwrap();
        let trailing_lines: Vec<_> = problems
            .iter()
            .filter(|p| p.rule == "trailing-spaces")
            .map(|p| p.line)
            .collect();
        assert_eq!(trailing_lines, [3]);
    }

    #[test]
    fn disable_line_text_in_scalar_values_is_ignored() {
        let yaml = concat!(
            "block: |-\n",
            "  # yamllint disable-line rule:trailing-spaces\n",
            "  block value   \n",
            "quoted: 'first line\n",
            "  # yamllint disable-line rule:trailing-spaces\n",
            "  last line'   \n",
            "plain: value# yamllint disable-line rule:trailing-spaces   \n",
            "ordinary: value  # text # yamllint disable-line rule:trailing-spaces   \n",
            "real: value  # yamllint disable-line rule:trailing-spaces   \n",
        );
        let problems = Linter::with_defaults().lint_string(yaml).unwrap();
        let trailing_lines: Vec<_> = problems
            .iter()
            .filter(|problem| problem.rule == "trailing-spaces")
            .map(|problem| problem.line)
            .collect();
        assert_eq!(trailing_lines, [3, 6, 7, 8]);
    }
}
