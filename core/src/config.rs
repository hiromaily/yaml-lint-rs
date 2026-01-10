//! Configuration system for the linter

use crate::Result;
use crate::rules::RuleLevel;
use indexmap::IndexMap;
use std::path::Path;

/// Main configuration structure
#[derive(Debug, Clone)]
pub struct Config {
    /// Rule configurations
    pub rules: IndexMap<String, RuleLevel>,
    /// File patterns to ignore
    pub ignore: Vec<String>,
}

impl Config {
    /// Create a new empty config
    pub fn new() -> Self {
        Self {
            rules: IndexMap::new(),
            ignore: Vec::new(),
        }
    }

    /// Create config with default preset
    pub fn with_default_preset() -> Self {
        let mut config = Self::new();

        // Default configuration - all rules enabled as errors
        config
            .rules
            .insert("trailing-spaces".to_string(), RuleLevel::Error);
        config
            .rules
            .insert("line-length".to_string(), RuleLevel::Error);
        config
            .rules
            .insert("document-start".to_string(), RuleLevel::Disable);
        config.rules.insert("colons".to_string(), RuleLevel::Error);
        config
            .rules
            .insert("key-duplicates".to_string(), RuleLevel::Error);
        config
            .rules
            .insert("indentation".to_string(), RuleLevel::Error);
        config
            .rules
            .insert("new-line-at-end-of-file".to_string(), RuleLevel::Error);
        config
            .rules
            .insert("empty-lines".to_string(), RuleLevel::Error);
        config.rules.insert("hyphens".to_string(), RuleLevel::Error);

        config
    }

    /// Create config with relaxed preset
    pub fn with_relaxed_preset() -> Self {
        let mut config = Self::new();

        // Relaxed configuration - most rules as warnings
        config
            .rules
            .insert("trailing-spaces".to_string(), RuleLevel::Warning);
        config
            .rules
            .insert("line-length".to_string(), RuleLevel::Warning);
        config
            .rules
            .insert("document-start".to_string(), RuleLevel::Disable);
        config
            .rules
            .insert("colons".to_string(), RuleLevel::Warning);
        config
            .rules
            .insert("key-duplicates".to_string(), RuleLevel::Error);
        config
            .rules
            .insert("indentation".to_string(), RuleLevel::Warning);
        config
            .rules
            .insert("new-line-at-end-of-file".to_string(), RuleLevel::Warning);
        config
            .rules
            .insert("empty-lines".to_string(), RuleLevel::Warning);
        config
            .rules
            .insert("hyphens".to_string(), RuleLevel::Warning);

        config
    }

    /// Load config from a YAML file
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::load_from_str(&content)
    }

    /// Load config from YAML string
    pub fn load_from_str(content: &str) -> Result<Self> {
        let yaml: serde_yaml::Value = serde_yaml::from_str(content)
            .map_err(|e| crate::LintError::ConfigError(format!("Invalid YAML: {}", e)))?;

        let mut config = Self::new();

        // Check for extends
        if let Some(extends) = yaml.get("extends").and_then(|v| v.as_str()) {
            config = match extends {
                "default" => Self::with_default_preset(),
                "relaxed" => Self::with_relaxed_preset(),
                _ => {
                    return Err(crate::LintError::ConfigError(format!(
                        "Unknown preset: {}",
                        extends
                    )));
                }
            };
        }

        // Parse rules
        if let Some(rules) = yaml.get("rules").and_then(|v| v.as_mapping()) {
            for (key, value) in rules {
                let rule_name = key.as_str().ok_or_else(|| {
                    crate::LintError::ConfigError("Rule name must be a string".to_string())
                })?;

                let level = match value {
                    serde_yaml::Value::String(s) => match s.as_str() {
                        "error" => RuleLevel::Error,
                        "warning" => RuleLevel::Warning,
                        "disable" => RuleLevel::Disable,
                        _ => {
                            return Err(crate::LintError::ConfigError(format!(
                                "Invalid rule level: {}",
                                s
                            )));
                        }
                    },
                    serde_yaml::Value::Mapping(_) => {
                        // For now, treat any mapping as enabled with error level
                        // In the future, this will parse rule-specific options
                        RuleLevel::Error
                    }
                    _ => {
                        return Err(crate::LintError::ConfigError(
                            "Rule value must be a string or mapping".to_string(),
                        ));
                    }
                };

                config.rules.insert(rule_name.to_string(), level);
            }
        }

        // Parse ignore patterns
        if let Some(ignore) = yaml.get("ignore").and_then(|v| v.as_str()) {
            config.ignore = ignore.lines().map(|s| s.to_string()).collect();
        }

        Ok(config)
    }

    /// Find a config file starting from the given directory
    pub fn find_config_file(start_dir: &Path) -> Option<std::path::PathBuf> {
        let config_names = [".yamllint", ".yamllint.yml", ".yamllint.yaml"];

        let mut current = start_dir.to_path_buf();

        loop {
            for name in &config_names {
                let config_path = current.join(name);
                if config_path.exists() {
                    return Some(config_path);
                }
            }

            // Move up one directory
            if !current.pop() {
                break;
            }
        }

        None
    }

    /// Get the level for a specific rule
    pub fn get_rule_level(&self, rule_name: &str) -> Option<RuleLevel> {
        self.rules.get(rule_name).copied()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::with_default_preset()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_preset() {
        let config = Config::with_default_preset();
        assert_eq!(
            config.get_rule_level("trailing-spaces"),
            Some(RuleLevel::Error)
        );
    }

    #[test]
    fn test_relaxed_preset() {
        let config = Config::with_relaxed_preset();
        assert_eq!(
            config.get_rule_level("trailing-spaces"),
            Some(RuleLevel::Warning)
        );
    }

    #[test]
    fn test_load_from_str_with_extends() {
        let yaml = r#"
extends: default
rules:
  trailing-spaces: warning
"#;
        let config = Config::load_from_str(yaml).unwrap();
        assert_eq!(
            config.get_rule_level("trailing-spaces"),
            Some(RuleLevel::Warning)
        );
        // Should still have other rules from default preset
        assert_eq!(
            config.get_rule_level("key-duplicates"),
            Some(RuleLevel::Error)
        );
    }

    #[test]
    fn test_load_from_str_without_extends() {
        let yaml = r#"
rules:
  trailing-spaces: error
  line-length: disable
"#;
        let config = Config::load_from_str(yaml).unwrap();
        assert_eq!(
            config.get_rule_level("trailing-spaces"),
            Some(RuleLevel::Error)
        );
        assert_eq!(
            config.get_rule_level("line-length"),
            Some(RuleLevel::Disable)
        );
    }

    #[test]
    fn test_invalid_preset() {
        let yaml = "extends: nonexistent";
        let result = Config::load_from_str(yaml);
        assert!(result.is_err());
    }
}
