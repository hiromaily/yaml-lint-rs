//! Truthy rule - restricts boolean representations to avoid YAML 1.1 vs 1.2 ambiguities

use crate::problem::{LintLevel, LintProblem};
use crate::rules::{LintContext, Rule, RuleLevel};

/// YAML 1.1 truthy values (case-insensitive)
const YAML_11_TRUTHY_VALUES: &[&str] = &["y", "yes", "on", "true", "n", "no", "off", "false"];

/// Rule that restricts boolean representations
///
/// YAML 1.1 treats many values as booleans (yes, no, on, off, y, n)
/// while YAML 1.2 only treats true/false as booleans.
/// This rule helps avoid subtle bugs when switching parsers.
#[derive(Debug)]
pub struct TruthyRule {
    /// Allowed truthy values (default: ["true", "false"])
    allowed_values: Vec<String>,
    /// Also check mapping keys (default: false)
    check_keys: bool,
}

impl TruthyRule {
    /// Create a new rule with default settings
    pub fn new() -> Self {
        Self {
            allowed_values: vec!["true".to_string(), "false".to_string()],
            check_keys: false,
        }
    }

    /// Create a new rule with custom settings
    pub fn with_config(allowed_values: Vec<String>, check_keys: bool) -> Self {
        Self {
            allowed_values,
            check_keys,
        }
    }

    /// Check if a value is a YAML 1.1 truthy value
    fn is_truthy_value(value: &str) -> bool {
        YAML_11_TRUTHY_VALUES
            .iter()
            .any(|&v| v.eq_ignore_ascii_case(value))
    }

    /// Check if a value is in the allowed list
    fn is_allowed(&self, value: &str) -> bool {
        self.allowed_values
            .iter()
            .any(|allowed| allowed.eq_ignore_ascii_case(value))
    }

    /// Check if a value is quoted (starts and ends with quotes)
    fn is_quoted(value: &str) -> bool {
        (value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\''))
    }

    /// Extract key and value from a line
    /// Returns (key, value, value_column) if found
    fn parse_key_value(line: &str) -> Option<(&str, &str, usize)> {
        // Skip comments and empty lines
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return None;
        }

        // Skip list items for now (focus on key-value pairs)
        if trimmed.starts_with('-') {
            // Handle list item values like "- yes"
            let after_hyphen = trimmed.strip_prefix('-')?.trim();
            if after_hyphen.is_empty() || after_hyphen.starts_with('#') {
                return None;
            }
            // If there's no colon, it's a simple list value
            if !after_hyphen.contains(':') {
                let value_start = line.find(after_hyphen)?;
                return Some(("", after_hyphen, value_start + 1));
            }
        }

        // Find the colon that separates key from value
        // Need to handle colons inside quotes
        let colon_pos = Self::find_key_value_separator(line)?;

        let key = line[..colon_pos].trim();
        let after_colon = &line[colon_pos + 1..];
        let value = after_colon.trim();

        // Skip if value is empty or is a comment
        if value.is_empty() || value.starts_with('#') {
            return None;
        }

        // Handle inline comments - extract just the value part
        let value = Self::strip_inline_comment(value);
        if value.is_empty() {
            return None;
        }

        // Calculate value column (1-indexed)
        let value_start = line[colon_pos + 1..]
            .find(|c: char| !c.is_whitespace())
            .map(|pos| colon_pos + 1 + pos + 1) // +1 for 1-indexed
            .unwrap_or(colon_pos + 2);

        Some((key, value, value_start))
    }

    /// Find the position of a character outside of quoted strings
    /// Returns the index of the first occurrence of `target` that is not inside quotes
    fn find_char_outside_quotes(s: &str, target: char) -> Option<usize> {
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut chars = s.char_indices().peekable();

        while let Some((idx, ch)) = chars.next() {
            if !in_single_quote && !in_double_quote {
                match ch {
                    c if c == target => return Some(idx),
                    '\'' => in_single_quote = true,
                    '"' => in_double_quote = true,
                    _ => {}
                }
            } else if in_single_quote {
                if ch == '\'' {
                    // Handle escaped single quote ''
                    if chars.peek().is_some_and(|&(_, next_ch)| next_ch == '\'') {
                        chars.next();
                    } else {
                        in_single_quote = false;
                    }
                }
            } else if ch == '\\' {
                // Handle backslash escape in double-quoted strings
                chars.next();
            } else if ch == '"' {
                in_double_quote = false;
            }
        }
        None
    }

    /// Find the colon that separates key from value (not inside quotes)
    fn find_key_value_separator(line: &str) -> Option<usize> {
        Self::find_char_outside_quotes(line, ':')
    }

    /// Strip inline comment from value
    fn strip_inline_comment(value: &str) -> &str {
        match Self::find_char_outside_quotes(value, '#') {
            Some(idx) => value[..idx].trim(),
            None => value,
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
        let mut problems = Vec::new();

        for (line_idx, line) in context.lines.iter().enumerate() {
            let line_num = line_idx + 1;

            if let Some((key, value, value_col)) = Self::parse_key_value(line) {
                // Check value
                if !Self::is_quoted(value)
                    && Self::is_truthy_value(value)
                    && !self.is_allowed(value)
                {
                    problems.push(LintProblem::new(
                        line_num,
                        value_col,
                        format!(
                            "truthy value \"{}\" should be replaced with \"true\" or \"false\"",
                            value
                        ),
                        self.name(),
                        LintLevel::Error,
                    ));
                }

                // Check key if configured
                if self.check_keys
                    && !key.is_empty()
                    && !Self::is_quoted(key)
                    && Self::is_truthy_value(key)
                    && !self.is_allowed(key)
                {
                    let key_col = line.find(key).map(|p| p + 1).unwrap_or(1);
                    problems.push(LintProblem::new(
                        line_num,
                        key_col,
                        format!("truthy value \"{}\" used as key should be quoted", key),
                        self.name(),
                        LintLevel::Error,
                    ));
                }
            }
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
        assert!(problems[0].message.contains("yes"));
        assert!(problems[1].message.contains("no"));
    }

    #[test]
    fn test_yaml_11_on_off() {
        let yaml = "feature: on\nlegacy: off\n";
        let context = LintContext::new(yaml.to_string());
        let rule = TruthyRule::new();
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 2);
        assert!(problems[0].message.contains("on"));
        assert!(problems[1].message.contains("off"));
    }

    #[test]
    fn test_yaml_11_y_n() {
        let yaml = "flag: y\nother: n\n";
        let context = LintContext::new(yaml.to_string());
        let rule = TruthyRule::new();
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 2);
    }

    #[test]
    fn test_case_insensitive() {
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
        let rule = TruthyRule::new();
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
        assert!(problems[0].message.contains("key"));
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
