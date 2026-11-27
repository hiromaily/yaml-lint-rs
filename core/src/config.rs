//! Configuration system for the linter

use crate::Result;
use crate::path_patterns::PathPatterns;
use crate::rules::indentation::{
    ConfiguredIndentation, IndentSequences, IndentSpaces, IndentationRule,
};
use crate::rules::{
    FlowPolicy, RuleLevel, anchors::AnchorsRule, commas::CommasRule,
    comments_indentation::CommentsIndentationRule, document_end::DocumentEndRule,
    empty_values::EmptyValuesRule, float_values::FloatValuesRule,
    key_duplicates::KeyDuplicatesOptions, key_ordering::KeyOrderingRule,
    new_line_at_end_of_file::NewLineAtEndOfFileRule, new_lines::NewLineType,
    octal_values::OctalValuesRule, quoted_strings::QuotedStringsRule,
};
use indexmap::IndexMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

const RELAXED_PRESET: &str = r#"
extends: default
rules:
  braces: {level: warning, max-spaces-inside: 1}
  brackets: {level: warning, max-spaces-inside: 1}
  colons: warning
  commas: warning
  comments: disable
  comments-indentation: disable
  document-start: disable
  empty-lines: warning
  hyphens: warning
  indentation: {level: warning, indent-sequences: consistent}
  line-length: {level: warning, allow-non-breakable-inline-mappings: true}
  truthy: disable
"#;

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
    Anchors(AnchorsRule),
    Commas(CommasRule),
    CommentsIndentation,
    DocumentEnd(DocumentEndRule),
    EmptyValues(EmptyValuesRule),
    FloatValues(FloatValuesRule),
    KeyDuplicates(KeyDuplicatesOptions),
    KeyOrdering(KeyOrderingRule),
    OctalValues(OctalValuesRule),
    QuotedStrings(QuotedStringsRule),
    LineLength {
        max: usize,
    },
    LineLengthWithExemptions {
        max: usize,
        allow_non_breakable_words: bool,
        allow_non_breakable_inline_mappings: bool,
    },
    Indentation {
        spaces: IndentConfig,
    },
    IndentationWithSequences {
        spaces: IndentConfig,
        sequences: IndentSequences,
        check_multi_line_strings: bool,
    },
    Colons {
        max_spaces_before: usize,
        max_spaces_after: usize,
    },
    NewLines {
        newline_type: NewLineType,
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
    Braces {
        forbid: FlowPolicy,
        min_spaces_inside: usize,
        max_spaces_inside: usize,
        min_spaces_inside_empty: Option<usize>,
        max_spaces_inside_empty: Option<usize>,
    },
    Brackets {
        forbid: FlowPolicy,
        min_spaces_inside: usize,
        max_spaces_inside: usize,
        min_spaces_inside_empty: Option<usize>,
        max_spaces_inside_empty: Option<usize>,
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
    /// Filename patterns used when discovering YAML files in directories.
    pub yaml_files: Vec<String>,
    rule_ignores: IndexMap<String, PathPatterns>,
}

impl Config {
    /// Create a new empty config
    pub fn new() -> Self {
        Self {
            rules: IndexMap::new(),
            ignore: Vec::new(),
            yaml_files: ["*.yaml", "*.yml", ".yamllint"].map(str::to_owned).into(),
            rule_ignores: IndexMap::new(),
        }
    }

    /// Create config with default preset
    pub fn with_default_preset() -> Self {
        let mut config = Self::new();

        let default_rules = [
            ("trailing-spaces", RuleLevel::Error),
            ("line-length", RuleLevel::Error),
            ("document-start", RuleLevel::Warning),
            ("colons", RuleLevel::Error),
            ("key-duplicates", RuleLevel::Error),
            ("indentation", RuleLevel::Error),
            ("new-line-at-end-of-file", RuleLevel::Error),
            ("new-lines", RuleLevel::Error),
            ("empty-lines", RuleLevel::Error),
            ("hyphens", RuleLevel::Error),
            ("comments", RuleLevel::Warning),
            ("truthy", RuleLevel::Warning),
            ("braces", RuleLevel::Error),
            ("brackets", RuleLevel::Error),
            ("anchors", RuleLevel::Error),
            ("commas", RuleLevel::Error),
            ("comments-indentation", RuleLevel::Warning),
            ("document-end", RuleLevel::Disable),
            ("empty-values", RuleLevel::Disable),
            ("float-values", RuleLevel::Disable),
            ("key-ordering", RuleLevel::Disable),
            ("octal-values", RuleLevel::Disable),
            ("quoted-strings", RuleLevel::Disable),
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
        Self::load_from_str(RELAXED_PRESET).expect("built-in relaxed preset is valid")
    }

    /// Load config from a YAML file
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = read_config_text(path)?;
        Self::load_from_str(&content)
    }

    /// Parse rule-specific options based on rule name
    fn parse_rule_options(rule_name: &str, map: &serde_yaml::Mapping) -> Result<RuleOptions> {
        match rule_name {
            "line-length" => Self::parse_line_length_options(map),
            "indentation" => Self::parse_indentation_options(map),
            "colons" => Self::parse_colons_options(map),
            "new-lines" => Self::parse_new_lines_options(map),
            "empty-lines" => Self::parse_empty_lines_options(map),
            "hyphens" => Self::parse_hyphens_options(map),
            "comments" => Self::parse_comments_options(map),
            "truthy" => Self::parse_truthy_options(map),
            "document-start" => Self::parse_document_start_options(map),
            "braces" => Self::parse_braces_options(map),
            "brackets" => Self::parse_brackets_options(map),
            "anchors" => Self::parse_options(rule_name, map).map(RuleOptions::Anchors),
            "commas" => Self::parse_options(rule_name, map).map(RuleOptions::Commas),
            "comments-indentation" => {
                if map.keys().all(|key| key.as_str() == Some("level")) {
                    Ok(RuleOptions::CommentsIndentation)
                } else {
                    Err(crate::LintError::ConfigError(
                        "comments-indentation does not support options".into(),
                    ))
                }
            }
            "document-end" => Self::parse_options(rule_name, map).map(RuleOptions::DocumentEnd),
            "empty-values" => Self::parse_options(rule_name, map).map(RuleOptions::EmptyValues),
            "float-values" => Self::parse_options(rule_name, map).map(RuleOptions::FloatValues),
            "key-duplicates" => Self::parse_options(rule_name, map).map(RuleOptions::KeyDuplicates),
            "key-ordering" => Self::parse_options(rule_name, map).map(RuleOptions::KeyOrdering),
            "octal-values" => Self::parse_options(rule_name, map).map(RuleOptions::OctalValues),
            "quoted-strings" => {
                let rule: QuotedStringsRule = Self::parse_options(rule_name, map)?;
                rule.validate()?;
                Ok(RuleOptions::QuotedStrings(rule))
            }
            _ => Err(crate::LintError::ConfigError(format!(
                "Rule '{}' does not support options",
                rule_name
            ))),
        }
    }

    fn parse_options<T: serde::de::DeserializeOwned>(
        rule: &str,
        map: &serde_yaml::Mapping,
    ) -> Result<T> {
        let mut options = map.clone();
        options.remove("level");
        serde_yaml::from_value(serde_yaml::Value::Mapping(options))
            .map_err(|error| crate::LintError::ConfigError(format!("{rule}: {error}")))
    }

    /// Parse new-lines options
    fn parse_new_lines_options(map: &serde_yaml::Mapping) -> Result<RuleOptions> {
        let newline_type = match map.get("type") {
            None => NewLineType::default(),
            Some(value) => match value.as_str() {
                Some("unix") => NewLineType::Unix,
                Some("dos") => NewLineType::Dos,
                Some("platform") => NewLineType::Platform,
                _ => {
                    return Err(crate::LintError::ConfigError(
                        "new-lines 'type' must be unix, dos, or platform".to_string(),
                    ));
                }
            },
        };

        Ok(RuleOptions::NewLines { newline_type })
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

        let option = |name, default| match map.get(name) {
            Some(value) => value.as_bool().ok_or_else(|| {
                crate::LintError::ConfigError(format!("line-length '{name}' must be a boolean"))
            }),
            None => Ok(default),
        };
        if !map.contains_key("allow-non-breakable-words")
            && !map.contains_key("allow-non-breakable-inline-mappings")
        {
            return Ok(RuleOptions::LineLength { max });
        }
        Ok(RuleOptions::LineLengthWithExemptions {
            max,
            allow_non_breakable_words: option("allow-non-breakable-words", true)?,
            allow_non_breakable_inline_mappings: option(
                "allow-non-breakable-inline-mappings",
                false,
            )?,
        })
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

        let check_multi_line_strings = match map.get("check-multi-line-strings") {
            Some(value) => value.as_bool().ok_or_else(|| {
                crate::LintError::ConfigError(
                    "indentation 'check-multi-line-strings' must be a boolean".into(),
                )
            })?,
            None => false,
        };
        let sequences = match map.get("indent-sequences") {
            None if !check_multi_line_strings => return Ok(RuleOptions::Indentation { spaces }),
            None => IndentSequences::Indented,
            Some(serde_yaml::Value::Bool(true)) => IndentSequences::Indented,
            Some(serde_yaml::Value::Bool(false)) => IndentSequences::Unindented,
            Some(value) if value.as_str() == Some("whatever") => IndentSequences::Whatever,
            Some(value) if value.as_str() == Some("consistent") => IndentSequences::Consistent,
            _ => return Err(crate::LintError::ConfigError(
                "indentation 'indent-sequences' must be true, false, 'whatever', or 'consistent'"
                    .into(),
            )),
        };
        Ok(RuleOptions::IndentationWithSequences {
            spaces,
            sequences,
            check_multi_line_strings,
        })
    }

    /// Parse colons options
    fn parse_colons_options(map: &serde_yaml::Mapping) -> Result<RuleOptions> {
        Ok(RuleOptions::Colons {
            max_spaces_before: Self::parse_spacing_limit(map, "colons", "max-spaces-before", 0)?,
            max_spaces_after: Self::parse_spacing_limit(map, "colons", "max-spaces-after", 1)?,
        })
    }

    fn parse_spacing_limit(
        map: &serde_yaml::Mapping,
        rule: &str,
        name: &str,
        default: usize,
    ) -> Result<usize> {
        match map.get(name) {
            Some(value) if value.as_i64() == Some(-1) => Ok(usize::MAX),
            Some(value) => value
                .as_u64()
                .and_then(|n| usize::try_from(n).ok())
                .ok_or_else(|| {
                    crate::LintError::ConfigError(format!(
                        "{rule} '{name}' must be -1 or a non-negative integer"
                    ))
                }),
            None => Ok(default),
        }
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
        Ok(RuleOptions::Hyphens {
            max_spaces_after: Self::parse_spacing_limit(map, "hyphens", "max-spaces-after", 1)?,
        })
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

        let min_spaces_from_content = match map.get("min-spaces-from-content") {
            Some(value) if value.as_i64() == Some(-1) => 0,
            Some(value) => value
                .as_u64()
                .and_then(|n| usize::try_from(n).ok())
                .ok_or_else(|| {
                    crate::LintError::ConfigError(
                        "comments 'min-spaces-from-content' must be -1 or a non-negative integer"
                            .into(),
                    )
                })?,
            None => 2,
        };

        Ok(RuleOptions::Comments {
            require_starting_space,
            ignore_shebangs,
            min_spaces_from_content,
        })
    }

    /// Parse truthy options
    fn parse_truthy_options(map: &serde_yaml::Mapping) -> Result<RuleOptions> {
        let allowed_values = match map.get("allowed-values") {
            Some(value) => Self::pattern_list("truthy allowed-values", value)?,
            None => vec!["true".into(), "false".into()],
        };

        if let Some(value) = allowed_values
            .iter()
            .find(|value| !crate::rules::truthy::YAML_11_VALUES.contains(&value.as_str()))
        {
            return Err(crate::LintError::ConfigError(format!(
                "truthy allowed-values contains an invalid value: {value:?}"
            )));
        }

        let check_keys = match map.get("check-keys") {
            Some(value) => value.as_bool().ok_or_else(|| {
                crate::LintError::ConfigError("truthy check-keys must be a boolean".into())
            })?,
            None => true,
        };

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

    /// Parse common flow collection options (shared by braces and brackets)
    fn parse_flow_collection_options<F>(
        map: &serde_yaml::Mapping,
        constructor: F,
    ) -> Result<RuleOptions>
    where
        F: FnOnce(FlowPolicy, usize, usize, Option<usize>, Option<usize>) -> RuleOptions,
    {
        let forbid = match map.get("forbid") {
            None | Some(serde_yaml::Value::Bool(false)) => FlowPolicy::Allow,
            Some(serde_yaml::Value::Bool(true)) => FlowPolicy::Forbid,
            Some(value) if value.as_str() == Some("non-empty") => FlowPolicy::EmptyOnly,
            _ => {
                return Err(crate::LintError::ConfigError(
                    "forbid must be a boolean or 'non-empty'".into(),
                ));
            }
        };

        let min_spaces_inside = map
            .get("min-spaces-inside")
            .and_then(|v| v.as_u64())
            .map_or(0, |n| n as usize);
        let max_spaces_inside = match map.get("max-spaces-inside").and_then(|v| v.as_i64()) {
            Some(-1) => usize::MAX,
            Some(n) if n >= 0 => n as usize,
            _ => 0,
        };

        let min_spaces_inside_empty = map
            .get(serde_yaml::Value::String(
                "min-spaces-inside-empty".to_string(),
            ))
            .and_then(|v| v.as_i64())
            .map(|v| if v < 0 { None } else { Some(v as usize) })
            .unwrap_or(None);

        let max_spaces_inside_empty = map
            .get(serde_yaml::Value::String(
                "max-spaces-inside-empty".to_string(),
            ))
            .and_then(|v| v.as_i64())
            .map(|v| if v < 0 { None } else { Some(v as usize) })
            .unwrap_or(None);

        Ok(constructor(
            forbid,
            min_spaces_inside,
            max_spaces_inside,
            min_spaces_inside_empty,
            max_spaces_inside_empty,
        ))
    }

    /// Parse braces options
    fn parse_braces_options(map: &serde_yaml::Mapping) -> Result<RuleOptions> {
        Self::parse_flow_collection_options(map, |forbid, min, max, min_empty, max_empty| {
            RuleOptions::Braces {
                forbid,
                min_spaces_inside: min,
                max_spaces_inside: max,
                min_spaces_inside_empty: min_empty,
                max_spaces_inside_empty: max_empty,
            }
        })
    }

    /// Parse brackets options
    fn parse_brackets_options(map: &serde_yaml::Mapping) -> Result<RuleOptions> {
        Self::parse_flow_collection_options(map, |forbid, min, max, min_empty, max_empty| {
            RuleOptions::Brackets {
                forbid,
                min_spaces_inside: min,
                max_spaces_inside: max,
                min_spaces_inside_empty: min_empty,
                max_spaces_inside_empty: max_empty,
            }
        })
    }

    /// Load config from YAML string
    pub fn load_from_str(content: &str) -> Result<Self> {
        let yaml = Self::read_config_layers(content, &mut HashSet::new())?;

        let mut config = Self::new();

        // Check for extends
        if let Some(extends) = yaml.get("extends").and_then(|v| v.as_str()) {
            config = match extends {
                "default" => Self::with_default_preset(),
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
                            "enable" => config
                                .rules
                                .get(rule_name)
                                .map(RuleConfig::level)
                                .filter(|level| *level != RuleLevel::Disable)
                                .unwrap_or(RuleLevel::Error),
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
                        let inherited = config.rules.get(rule_name);
                        let level = match map.get("level") {
                            Some(value) => match value.as_str() {
                                Some("error") => RuleLevel::Error,
                                Some("warning") => RuleLevel::Warning,
                                Some("disable") => RuleLevel::Disable,
                                _ => {
                                    return Err(crate::LintError::ConfigError(format!(
                                        "Invalid level for rule {rule_name}"
                                    )));
                                }
                            },
                            None => inherited
                                .map(RuleConfig::level)
                                .filter(|level| *level != RuleLevel::Disable)
                                .unwrap_or(RuleLevel::Error),
                        };

                        if let Some(patterns) = Self::ignore_patterns(value)? {
                            config
                                .rule_ignores
                                .insert(rule_name.to_owned(), PathPatterns::new(&patterns)?);
                        }
                        let mut options = map.clone();
                        options.remove("ignore");
                        options.remove("ignore-from-file");
                        options.remove("level");
                        if options.is_empty() {
                            match inherited.and_then(RuleConfig::options) {
                                Some(options) => RuleConfig::Detailed {
                                    level,
                                    options: options.clone(),
                                },
                                None if matches!(
                                    rule_name,
                                    "trailing-spaces" | "new-line-at-end-of-file"
                                ) =>
                                {
                                    RuleConfig::Level(level)
                                }
                                None => RuleConfig::Detailed {
                                    level,
                                    options: Self::parse_rule_options(rule_name, &options)?,
                                },
                            }
                        } else {
                            let options = Self::parse_rule_options(rule_name, &options)?;
                            RuleConfig::Detailed { level, options }
                        }
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

        if let Some(locale) = yaml.get("locale") {
            let locale = locale
                .as_str()
                .ok_or_else(|| crate::LintError::ConfigError("locale must be a string".into()))?;
            let rule_config = config.rules.get("key-ordering");
            let rule = match rule_config.and_then(RuleConfig::options) {
                Some(RuleOptions::KeyOrdering(rule)) => rule.clone(),
                _ => KeyOrderingRule::default(),
            }
            .with_locale(locale)?;

            if let Some(rule_config) = rule_config {
                let level = rule_config.level();
                config.rules.insert(
                    "key-ordering".into(),
                    RuleConfig::Detailed {
                        level,
                        options: RuleOptions::KeyOrdering(rule),
                    },
                );
            }
        }

        if let Some(patterns) = Self::ignore_patterns(&yaml)? {
            PathPatterns::new(&patterns)?;
            config.ignore = patterns;
        }
        if let Some(patterns) = yaml.get("yaml-files") {
            config.yaml_files = Self::pattern_list("yaml-files", patterns)?;
            PathPatterns::new(&config.yaml_files)?;
        }

        Ok(config)
    }

    fn read_config_layers(content: &str, seen: &mut HashSet<PathBuf>) -> Result<serde_yaml::Value> {
        use serde_yaml::Value;

        let mut yaml: Value = serde_yaml::from_str(content)
            .map_err(|error| crate::LintError::ConfigError(format!("Invalid YAML: {error}")))?;
        let Some(layer) = yaml.as_mapping_mut() else {
            return Ok(yaml);
        };
        let Some(Value::String(path)) = layer.get("extends") else {
            return Ok(yaml);
        };
        if path == "default" {
            return Ok(yaml);
        }

        let base = if path == "relaxed" {
            Self::read_config_layers(RELAXED_PRESET, seen)?
        } else {
            let path = Path::new(path);
            let canonical = path.canonicalize().map_err(|error| {
                crate::LintError::ConfigError(format!("extends {}: {error}", path.display()))
            })?;
            if !seen.insert(canonical) {
                return Err(crate::LintError::ConfigError(format!(
                    "Circular config inheritance at {}",
                    path.display()
                )));
            }
            let parent = read_config_text(path).map_err(|error| {
                crate::LintError::ConfigError(format!("extends {}: {error}", path.display()))
            })?;
            Self::read_config_layers(&parent, seen)?
        };
        let Value::Mapping(mut base) = base else {
            return Err(crate::LintError::ConfigError(format!(
                "Extended config {path} must be a mapping"
            )));
        };
        layer.remove("extends");
        if layer.contains_key("ignore") || layer.contains_key("ignore-from-file") {
            base.remove("ignore");
            base.remove("ignore-from-file");
        }
        for (key, mut value) in std::mem::take(layer) {
            if key.as_str() == Some("rules")
                && let Some(Value::Mapping(inherited)) = base.get_mut(&key)
                && let Value::Mapping(overrides) = &mut value
            {
                for (rule, settings) in std::mem::take(overrides) {
                    match (inherited.get_mut(&rule), settings) {
                        (Some(Value::String(base)), Value::String(level))
                            if base == "disable" && level == "enable" =>
                        {
                            *base = "error".into();
                        }
                        (Some(base), Value::String(level))
                            if level == "enable"
                                && (base.is_mapping()
                                    || matches!(base.as_str(), Some("warning" | "error"))) => {}
                        (Some(Value::Mapping(base)), Value::Mapping(settings)) => {
                            if settings.contains_key("ignore")
                                || settings.contains_key("ignore-from-file")
                            {
                                base.remove("ignore");
                                base.remove("ignore-from-file");
                            }
                            base.extend(settings);
                        }
                        (Some(Value::String(level)), Value::Mapping(mut settings)) => {
                            if matches!(level.as_str(), "error" | "warning") {
                                settings
                                    .entry(Value::String("level".into()))
                                    .or_insert_with(|| Value::String(level.clone()));
                            }
                            inherited.insert(rule, Value::Mapping(settings));
                        }
                        (_, settings) => {
                            inherited.insert(rule, settings);
                        }
                    }
                }
            } else {
                base.insert(key, value);
            }
        }
        Ok(Value::Mapping(base))
    }

    fn pattern_list(key: &str, value: &serde_yaml::Value) -> Result<Vec<String>> {
        value
            .as_sequence()
            .and_then(|items| {
                items
                    .iter()
                    .map(|item| item.as_str().map(str::to_owned))
                    .collect()
            })
            .ok_or_else(|| {
                crate::LintError::ConfigError(format!("{key} must be a sequence of strings"))
            })
    }

    fn ignore_patterns(value: &serde_yaml::Value) -> Result<Option<Vec<String>>> {
        match (value.get("ignore"), value.get("ignore-from-file")) {
            (Some(_), Some(_)) => Err(crate::LintError::ConfigError(
                "ignore and ignore-from-file cannot be used together".into(),
            )),
            (Some(serde_yaml::Value::String(patterns)), None) => {
                Ok(Some(patterns.lines().map(str::to_owned).collect()))
            }
            (Some(patterns), None) => Self::pattern_list("ignore", patterns).map(Some),
            (None, Some(files)) => {
                let files = match files {
                    serde_yaml::Value::String(file) => vec![file.clone()],
                    _ => Self::pattern_list("ignore-from-file", files)?,
                };
                let mut patterns = Vec::new();
                for file in files {
                    let content = read_config_text(Path::new(&file)).map_err(|error| {
                        crate::LintError::ConfigError(format!("ignore-from-file {file:?}: {error}"))
                    })?;
                    patterns.extend(content.lines().map(str::to_owned));
                }
                Ok(Some(patterns))
            }
            (None, None) => Ok(None),
        }
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

        let newline_type = self
            .rules
            .get("new-lines")
            .filter(|rule| rule.level() != RuleLevel::Disable)
            .and_then(RuleConfig::options)
            .and_then(|options| match options {
                RuleOptions::NewLines { newline_type } => Some(*newline_type),
                _ => None,
            })
            .unwrap_or_default();

        // Register each rule with its configuration
        for (rule_name, rule_config) in &self.rules {
            let level = rule_config.level();

            // Construct rule with options
            let rule: Box<dyn crate::rules::Rule> = match rule_name.as_str() {
                "trailing-spaces" => {
                    construct_rule!(crate::rules::trailing_spaces::TrailingSpacesRule)
                }
                "line-length" => match rule_config.options() {
                    Some(RuleOptions::LineLength { max }) => {
                        Box::new(crate::rules::line_length::LineLengthRule::with_max(*max))
                    }
                    Some(RuleOptions::LineLengthWithExemptions {
                        max,
                        allow_non_breakable_words,
                        allow_non_breakable_inline_mappings,
                    }) => Box::new(crate::rules::line_length::ConfiguredLineLength::new(
                        *max,
                        *allow_non_breakable_words,
                        *allow_non_breakable_inline_mappings,
                    )),
                    _ => Box::new(crate::rules::line_length::LineLengthRule::new()),
                },
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
                        Box::new(crate::rules::document_start::DocumentStartRule::required())
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
                "key-duplicates" => construct_rule!(
                    rule_config,
                    RuleOptions::KeyDuplicates(rule) => rule.clone(),
                    crate::rules::key_duplicates::KeyDuplicatesRule
                ),
                "indentation" => match rule_config.options() {
                    Some(RuleOptions::Indentation {
                        spaces: IndentConfig::Fixed(n),
                    }) => Box::new(IndentationRule::with_spaces(*n)),
                    Some(RuleOptions::IndentationWithSequences {
                        spaces,
                        sequences,
                        check_multi_line_strings,
                    }) => Box::new(ConfiguredIndentation {
                        rule: IndentationRule {
                            spaces: match spaces {
                                IndentConfig::Fixed(n) => IndentSpaces::Fixed(*n),
                                IndentConfig::Consistent => IndentSpaces::Consistent,
                            },
                        },
                        sequences: *sequences,
                        check_multi_line_strings: *check_multi_line_strings,
                    }),
                    _ => Box::new(IndentationRule::new()),
                },
                "new-lines" => Box::new(crate::rules::new_lines::NewLinesRule { newline_type }),
                "new-line-at-end-of-file" => {
                    Box::new(NewLineAtEndOfFileRule::with_ending(newline_type.ending()))
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
                "braces" => construct_rule!(
                    rule_config,
                    RuleOptions::Braces {
                        forbid,
                        min_spaces_inside,
                        max_spaces_inside,
                        min_spaces_inside_empty,
                        max_spaces_inside_empty,
                    } => crate::rules::braces::BracesRule::with_policy(
                        *forbid,
                        *min_spaces_inside,
                        *max_spaces_inside,
                        *min_spaces_inside_empty,
                        *max_spaces_inside_empty
                    ),
                    crate::rules::braces::BracesRule::new()
                ),
                "brackets" => construct_rule!(
                    rule_config,
                    RuleOptions::Brackets {
                        forbid,
                        min_spaces_inside,
                        max_spaces_inside,
                        min_spaces_inside_empty,
                        max_spaces_inside_empty,
                    } => crate::rules::brackets::BracketsRule::with_policy(
                        *forbid,
                        *min_spaces_inside,
                        *max_spaces_inside,
                        *min_spaces_inside_empty,
                        *max_spaces_inside_empty
                    ),
                    crate::rules::brackets::BracketsRule::new()
                ),
                "anchors" => {
                    construct_rule!(
                        rule_config,
                        RuleOptions::Anchors(rule) => rule.clone(),
                        AnchorsRule::default()
                    )
                }
                "commas" => {
                    construct_rule!(
                        rule_config,
                        RuleOptions::Commas(rule) => rule.clone(),
                        CommasRule::default()
                    )
                }
                "comments-indentation" => Box::new(CommentsIndentationRule),
                "document-end" => {
                    construct_rule!(
                        rule_config,
                        RuleOptions::DocumentEnd(rule) => rule.clone(),
                        DocumentEndRule::default()
                    )
                }
                "empty-values" => {
                    construct_rule!(
                        rule_config,
                        RuleOptions::EmptyValues(rule) => rule.clone(),
                        EmptyValuesRule::default()
                    )
                }
                "float-values" => {
                    construct_rule!(
                        rule_config,
                        RuleOptions::FloatValues(rule) => rule.clone(),
                        FloatValuesRule::default()
                    )
                }
                "key-ordering" => {
                    construct_rule!(
                        rule_config,
                        RuleOptions::KeyOrdering(rule) => rule.clone(),
                        KeyOrderingRule::default()
                    )
                }
                "octal-values" => {
                    construct_rule!(
                        rule_config,
                        RuleOptions::OctalValues(rule) => rule.clone(),
                        OctalValuesRule::default()
                    )
                }
                "quoted-strings" => {
                    construct_rule!(
                        rule_config,
                        RuleOptions::QuotedStrings(rule) => rule.clone(),
                        QuotedStringsRule::default()
                    )
                }
                _ => continue, // Skip unknown rules
            };

            registry.register(rule);
            registry.set_level(rule_name, level);
            if let Some(patterns) = self.rule_ignores.get(rule_name) {
                registry.set_ignore(rule_name, patterns.clone());
            }
        }

        registry
    }
}

fn read_config_text(path: &Path) -> std::io::Result<String> {
    let bytes = std::fs::read(path)?;
    let invalid =
        || std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid Unicode encoding");
    let (bytes, width, little_endian) = match bytes.as_slice() {
        [0, 0, 0xfe, 0xff, rest @ ..] => (rest, 4, false),
        [0xff, 0xfe, 0, 0, rest @ ..] => (rest, 4, true),
        [0xfe, 0xff, rest @ ..] => (rest, 2, false),
        [0xff, 0xfe, rest @ ..] => (rest, 2, true),
        data @ [0, 0, 0, _, ..] => (data, 4, false),
        data @ [_, 0, 0, 0, ..] => (data, 4, true),
        data @ [0, _, ..] => (data, 2, false),
        data @ [_, 0, ..] => (data, 2, true),
        data => {
            return std::str::from_utf8(data.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(data))
                .map(str::to_owned)
                .map_err(|_| invalid());
        }
    };

    if width == 2 {
        let (units, remainder) = bytes.as_chunks::<2>();
        if !remainder.is_empty() {
            return Err(invalid());
        }
        let decode = if little_endian {
            u16::from_le_bytes
        } else {
            u16::from_be_bytes
        };
        char::decode_utf16(units.iter().copied().map(decode))
            .map(|value| value.map_err(|_| invalid()))
            .collect()
    } else {
        let (units, remainder) = bytes.as_chunks::<4>();
        if !remainder.is_empty() {
            return Err(invalid());
        }
        let decode = if little_endian {
            u32::from_le_bytes
        } else {
            u32::from_be_bytes
        };
        units
            .iter()
            .copied()
            .map(|unit| char::from_u32(decode(unit)).ok_or_else(invalid))
            .collect()
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
            Some(RuleLevel::Error)
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
