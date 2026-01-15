//! Configuration system for the linter

use crate::Result;
use crate::rules::RuleLevel;
use indexmap::IndexMap;
use std::path::Path;

/// Configuration for a single rule
#[derive(Debug, Clone, PartialEq)]
pub enum RuleConfig {
    /// Simple level configuration (e.g., "error", "warning", "disable")
    Level(RuleLevel),
    /// Detailed configuration with options
    Detailed {
        level: RuleLevel,
        options: RuleOptions,
    },
}

impl RuleConfig {
    /// Get the rule level
    pub fn level(&self) -> RuleLevel {
        match self {
            RuleConfig::Level(level) => *level,
            RuleConfig::Detailed { level, .. } => *level,
        }
    }

    /// Get the rule options if available
    pub fn options(&self) -> Option<&RuleOptions> {
        match self {
            RuleConfig::Level(_) => None,
            RuleConfig::Detailed { options, .. } => Some(options),
        }
    }
}

/// Rule-specific options
#[derive(Debug, Clone, PartialEq)]
pub enum RuleOptions {
    LineLength {
        max: usize,
    },
    Indentation {
        spaces: IndentConfig,
    },
    Colons {
        max_spaces_before: usize,
        max_spaces_after: usize,
    },
    EmptyLines {
        max: usize,
        max_start: usize,
        max_end: usize,
    },
    Hyphens {
        max_spaces_after: usize,
    },
    Comments {
        require_starting_space: bool,
        ignore_shebangs: bool,
        min_spaces_from_content: usize,
    },
    Truthy {
        allowed_values: Vec<String>,
        check_keys: bool,
    },
    DocumentStart {
        present: DocumentStartConfig,
    },
}

/// Indentation configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndentConfig {
    /// Fixed number of spaces
    Fixed(usize),
    /// Consistent indentation (auto-detect)
    Consistent,
}

/// Document start configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentStartConfig {
    /// Require `---` at document start
    Required,
    /// Forbid `---` at document start
    Forbidden,
    /// No requirement (disabled)
    Disabled,
}

/// Main configuration structure
#[derive(Debug, Clone)]
pub struct Config {
    /// Rule configurations
    pub rules: IndexMap<String, RuleConfig>,
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
        let default_rules = [
            ("trailing-spaces", RuleLevel::Error),
            ("line-length", RuleLevel::Error),
            ("document-start", RuleLevel::Disable),
            ("colons", RuleLevel::Error),
            ("key-duplicates", RuleLevel::Error),
            ("indentation", RuleLevel::Error),
            ("new-line-at-end-of-file", RuleLevel::Error),
            ("empty-lines", RuleLevel::Error),
            ("hyphens", RuleLevel::Error),
            ("comments", RuleLevel::Error),
            ("truthy", RuleLevel::Warning),
        ];

        for (rule_name, level) in default_rules {
            config
                .rules
                .insert(rule_name.to_string(), RuleConfig::Level(level));
        }

        config
    }

    /// Create config with relaxed preset
    pub fn with_relaxed_preset() -> Self {
        let mut config = Self::new();

        // Relaxed configuration - most rules as warnings
        let relaxed_rules = [
            ("trailing-spaces", RuleLevel::Warning),
            ("line-length", RuleLevel::Warning),
            ("document-start", RuleLevel::Disable),
            ("colons", RuleLevel::Warning),
            ("key-duplicates", RuleLevel::Error),
            ("indentation", RuleLevel::Warning),
            ("new-line-at-end-of-file", RuleLevel::Warning),
            ("empty-lines", RuleLevel::Warning),
            ("hyphens", RuleLevel::Warning),
            ("comments", RuleLevel::Warning),
            ("truthy", RuleLevel::Warning),
        ];

        for (rule_name, level) in relaxed_rules {
            config
                .rules
                .insert(rule_name.to_string(), RuleConfig::Level(level));
        }

        config
    }

    /// Load config from a YAML file
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::load_from_str(&content)
    }

    /// Parse rule-specific options based on rule name
    fn parse_rule_options(rule_name: &str, map: &serde_yaml::Mapping) -> Result<RuleOptions> {
        match rule_name {
            "line-length" => Self::parse_line_length_options(map),
            "indentation" => Self::parse_indentation_options(map),
            "colons" => Self::parse_colons_options(map),
            "empty-lines" => Self::parse_empty_lines_options(map),
            "hyphens" => Self::parse_hyphens_options(map),
            "comments" => Self::parse_comments_options(map),
            "truthy" => Self::parse_truthy_options(map),
            "document-start" => Self::parse_document_start_options(map),
            _ => Err(crate::LintError::ConfigError(format!(
                "Rule '{}' does not support options",
                rule_name
            ))),
        }
    }

    /// Parse line-length options
    fn parse_line_length_options(map: &serde_yaml::Mapping) -> Result<RuleOptions> {
        let max = map
            .get(serde_yaml::Value::String("max".to_string()))
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(80);

        if max == 0 {
            return Err(crate::LintError::ConfigError(
                "line-length max must be greater than 0".to_string(),
            ));
        }

        Ok(RuleOptions::LineLength { max })
    }

    /// Parse indentation options
    fn parse_indentation_options(map: &serde_yaml::Mapping) -> Result<RuleOptions> {
        let spaces_value = map.get(serde_yaml::Value::String("spaces".to_string()));

        let spaces = match spaces_value {
            Some(serde_yaml::Value::Number(n)) => {
                let num = n.as_u64().ok_or_else(|| {
                    crate::LintError::ConfigError(
                        "indentation spaces must be a positive integer".to_string(),
                    )
                })?;

                if num == 0 || num > 16 {
                    return Err(crate::LintError::ConfigError(
                        "indentation spaces must be between 1 and 16".to_string(),
                    ));
                }

                IndentConfig::Fixed(num as usize)
            }
            Some(serde_yaml::Value::String(s)) if s == "consistent" => IndentConfig::Consistent,
            None => IndentConfig::Consistent, // Default
            _ => {
                return Err(crate::LintError::ConfigError(
                    "indentation spaces must be a number or 'consistent'".to_string(),
                ));
            }
        };

        Ok(RuleOptions::Indentation { spaces })
    }

    /// Parse colons options
    fn parse_colons_options(map: &serde_yaml::Mapping) -> Result<RuleOptions> {
        let max_spaces_before = map
            .get(serde_yaml::Value::String("max-spaces-before".to_string()))
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(0);

        let max_spaces_after = map
            .get(serde_yaml::Value::String("max-spaces-after".to_string()))
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(1);

        Ok(RuleOptions::Colons {
            max_spaces_before,
            max_spaces_after,
        })
    }

    /// Parse empty-lines options
    fn parse_empty_lines_options(map: &serde_yaml::Mapping) -> Result<RuleOptions> {
        let max = map
            .get(serde_yaml::Value::String("max".to_string()))
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(2);

        let max_start = map
            .get(serde_yaml::Value::String("max-start".to_string()))
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(0);

        let max_end = map
            .get(serde_yaml::Value::String("max-end".to_string()))
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(0);

        Ok(RuleOptions::EmptyLines {
            max,
            max_start,
            max_end,
        })
    }

    /// Parse hyphens options
    fn parse_hyphens_options(map: &serde_yaml::Mapping) -> Result<RuleOptions> {
        let max_spaces_after = map
            .get(serde_yaml::Value::String("max-spaces-after".to_string()))
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(1);

        Ok(RuleOptions::Hyphens { max_spaces_after })
    }

    /// Parse comments options
    fn parse_comments_options(map: &serde_yaml::Mapping) -> Result<RuleOptions> {
        let require_starting_space = map
            .get(serde_yaml::Value::String(
                "require-starting-space".to_string(),
            ))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let ignore_shebangs = map
            .get(serde_yaml::Value::String("ignore-shebangs".to_string()))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let min_spaces_from_content = map
            .get(serde_yaml::Value::String(
                "min-spaces-from-content".to_string(),
            ))
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(2);

        Ok(RuleOptions::Comments {
            require_starting_space,
            ignore_shebangs,
            min_spaces_from_content,
        })
    }

    /// Parse truthy options
    fn parse_truthy_options(map: &serde_yaml::Mapping) -> Result<RuleOptions> {
        let allowed_values = map
            .get(serde_yaml::Value::String("allowed-values".to_string()))
            .and_then(|v| v.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec!["true".to_string(), "false".to_string()]);

        if allowed_values.is_empty() {
            return Err(crate::LintError::ConfigError(
                "truthy allowed-values cannot be empty".to_string(),
            ));
        }

        let check_keys = map
            .get(serde_yaml::Value::String("check-keys".to_string()))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(RuleOptions::Truthy {
            allowed_values,
            check_keys,
        })
    }

    /// Parse document-start options
    fn parse_document_start_options(map: &serde_yaml::Mapping) -> Result<RuleOptions> {
        let present_value = map.get(serde_yaml::Value::String("present".to_string()));

        let present_config = match present_value {
            Some(serde_yaml::Value::Bool(true)) => DocumentStartConfig::Required,
            Some(serde_yaml::Value::Bool(false)) => DocumentStartConfig::Forbidden,
            None => DocumentStartConfig::Disabled,
            Some(_) => {
                return Err(crate::LintError::ConfigError(
                    "document-start 'present' must be a boolean (true or false)".to_string(),
                ));
            }
        };

        Ok(RuleOptions::DocumentStart {
            present: present_config,
        })
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

                let rule_config = match value {
                    // Simple string level: "error", "warning", "disable"
                    serde_yaml::Value::String(s) => {
                        let level = match s.as_str() {
                            "error" => RuleLevel::Error,
                            "warning" => RuleLevel::Warning,
                            "disable" => RuleLevel::Disable,
                            _ => {
                                return Err(crate::LintError::ConfigError(format!(
                                    "Invalid rule level: {}",
                                    s
                                )));
                            }
                        };
                        RuleConfig::Level(level)
                    }
                    // Mapping with options
                    serde_yaml::Value::Mapping(map) => {
                        // Extract explicit level if specified, otherwise default to Error
                        let level = map
                            .get(serde_yaml::Value::String("level".to_string()))
                            .and_then(|v| v.as_str())
                            .map(|s| match s {
                                "error" => Ok(RuleLevel::Error),
                                "warning" => Ok(RuleLevel::Warning),
                                "disable" => Ok(RuleLevel::Disable),
                                _ => Err(crate::LintError::ConfigError(format!(
                                    "Invalid rule level: {}",
                                    s
                                ))),
                            })
                            .transpose()?
                            .unwrap_or(RuleLevel::Error);

                        // Parse rule-specific options
                        let options = Self::parse_rule_options(rule_name, map)?;

                        RuleConfig::Detailed { level, options }
                    }
                    _ => {
                        return Err(crate::LintError::ConfigError(
                            "Rule value must be a string or mapping".to_string(),
                        ));
                    }
                };

                config.rules.insert(rule_name.to_string(), rule_config);
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
        self.rules.get(rule_name).map(|config| config.level())
    }

    /// Create a RuleRegistry configured according to this Config
    pub fn create_registry(&self) -> crate::rules::RuleRegistry {
        // Macro to reduce boilerplate when constructing rules with options
        macro_rules! construct_rule {
            // Rule without options
            ($rule_type:path) => {
                Box::new($rule_type)
            };
            // Rule with simple options pattern
            ($rule_config:expr, $pattern:pat => $with_config:expr, $default:expr) => {
                if let Some($pattern) = $rule_config.options() {
                    Box::new($with_config)
                } else {
                    Box::new($default)
                }
            };
        }

        let mut registry = crate::rules::RuleRegistry::new();

        // If no rules configured, use defaults
        if self.rules.is_empty() {
            return crate::rules::RuleRegistry::with_defaults();
        }

        // Register each rule with its configuration
        for (rule_name, rule_config) in &self.rules {
            let level = rule_config.level();

            // Construct rule with options
            let rule: Box<dyn crate::rules::Rule> = match rule_name.as_str() {
                "trailing-spaces" => {
                    construct_rule!(crate::rules::trailing_spaces::TrailingSpacesRule)
                }
                "line-length" => construct_rule!(
                    rule_config,
                    RuleOptions::LineLength { max } =>
                        crate::rules::line_length::LineLengthRule::with_max(*max),
                    crate::rules::line_length::LineLengthRule::new()
                ),
                "document-start" => {
                    if let Some(RuleOptions::DocumentStart { present }) = rule_config.options() {
                        match present {
                            DocumentStartConfig::Required => {
                                Box::new(crate::rules::document_start::DocumentStartRule::required())
                            }
                            DocumentStartConfig::Forbidden => Box::new(
                                crate::rules::document_start::DocumentStartRule::forbidden(),
                            ),
                            DocumentStartConfig::Disabled => {
                                Box::new(crate::rules::document_start::DocumentStartRule::new())
                            }
                        }
                    } else {
                        Box::new(crate::rules::document_start::DocumentStartRule::new())
                    }
                }
                "colons" => construct_rule!(
                    rule_config,
                    RuleOptions::Colons {
                        max_spaces_before,
                        max_spaces_after,
                    } => crate::rules::colons::ColonsRule::with_spacing(
                        *max_spaces_before,
                        *max_spaces_after
                    ),
                    crate::rules::colons::ColonsRule::new()
                ),
                "key-duplicates" => {
                    construct_rule!(crate::rules::key_duplicates::KeyDuplicatesRule)
                }
                "indentation" => {
                    if let Some(RuleOptions::Indentation { spaces }) = rule_config.options() {
                        match spaces {
                            IndentConfig::Fixed(n) => Box::new(
                                crate::rules::indentation::IndentationRule::with_spaces(*n),
                            ),
                            IndentConfig::Consistent => {
                                Box::new(crate::rules::indentation::IndentationRule::consistent())
                            }
                        }
                    } else {
                        Box::new(crate::rules::indentation::IndentationRule::new())
                    }
                }
                "new-line-at-end-of-file" => {
                    construct_rule!(crate::rules::new_line_at_end_of_file::NewLineAtEndOfFileRule)
                }
                "empty-lines" => construct_rule!(
                    rule_config,
                    RuleOptions::EmptyLines {
                        max,
                        max_start,
                        max_end,
                    } => crate::rules::empty_lines::EmptyLinesRule::with_config(
                        *max,
                        *max_start,
                        *max_end
                    ),
                    crate::rules::empty_lines::EmptyLinesRule::new()
                ),
                "hyphens" => construct_rule!(
                    rule_config,
                    RuleOptions::Hyphens { max_spaces_after } =>
                        crate::rules::hyphens::HyphensRule::with_config(*max_spaces_after),
                    crate::rules::hyphens::HyphensRule::new()
                ),
                "comments" => construct_rule!(
                    rule_config,
                    RuleOptions::Comments {
                        require_starting_space,
                        ignore_shebangs,
                        min_spaces_from_content,
                    } => crate::rules::comments::CommentsRule::with_config(
                        *require_starting_space,
                        *ignore_shebangs,
                        *min_spaces_from_content
                    ),
                    crate::rules::comments::CommentsRule::new()
                ),
                "truthy" => construct_rule!(
                    rule_config,
                    RuleOptions::Truthy {
                        allowed_values,
                        check_keys,
                    } => crate::rules::truthy::TruthyRule::with_config(
                        allowed_values.clone(),
                        *check_keys
                    ),
                    crate::rules::truthy::TruthyRule::new()
                ),
                _ => continue, // Skip unknown rules
            };

            registry.register(rule);
            registry.set_level(rule_name, level);
        }

        registry
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
    fn test_load_from_str_with_line_length_options() {
        let yaml = r#"
rules:
  line-length:
    max: 120
"#;
        let config = Config::load_from_str(yaml).unwrap();
        let rule_config = config.rules.get("line-length").unwrap();

        match rule_config {
            RuleConfig::Detailed { level, options } => {
                assert_eq!(*level, RuleLevel::Error);
                match options {
                    RuleOptions::LineLength { max } => assert_eq!(*max, 120),
                    _ => panic!("Expected LineLength options"),
                }
            }
            _ => panic!("Expected Detailed configuration"),
        }
    }

    #[test]
    fn test_load_from_str_with_indentation_options() {
        let yaml = r#"
rules:
  indentation:
    spaces: 2
"#;
        let config = Config::load_from_str(yaml).unwrap();
        let rule_config = config.rules.get("indentation").unwrap();

        match rule_config {
            RuleConfig::Detailed { options, .. } => match options {
                RuleOptions::Indentation { spaces } => {
                    assert_eq!(*spaces, IndentConfig::Fixed(2))
                }
                _ => panic!("Expected Indentation options"),
            },
            _ => panic!("Expected Detailed configuration"),
        }
    }

    #[test]
    fn test_backwards_compatibility_string_values() {
        let yaml = r#"
rules:
  trailing-spaces: error
  line-length: warning
  document-start: disable
"#;
        let config = Config::load_from_str(yaml).unwrap();

        assert_eq!(
            config.get_rule_level("trailing-spaces"),
            Some(RuleLevel::Error)
        );
        assert_eq!(
            config.get_rule_level("line-length"),
            Some(RuleLevel::Warning)
        );
        assert_eq!(
            config.get_rule_level("document-start"),
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
