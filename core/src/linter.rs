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
