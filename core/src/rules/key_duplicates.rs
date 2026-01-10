//! Key duplicates rule - detects duplicate keys in YAML mappings

use crate::problem::{LintLevel, LintProblem};
use crate::rules::{LintContext, Rule, RuleLevel};
use std::collections::HashSet;
use yaml_rust2::YamlLoader;

/// Rule that detects duplicate keys in mappings
#[derive(Debug)]
pub struct KeyDuplicatesRule;

impl Rule for KeyDuplicatesRule {
    fn name(&self) -> &'static str {
        "key-duplicates"
    }

    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        let mut problems = Vec::new();

        // Try to parse the YAML to check for duplicates
        // We need to use a custom approach since yaml-rust2 might silently merge duplicates
        // For now, we'll do a simple line-based check for obvious duplicates
        problems.extend(check_duplicate_keys_in_lines(context));

        // Also try to parse with yaml-rust2 to catch syntax issues
        // Note: We just verify the YAML is parseable; duplicate detection is line-based
        let _ = YamlLoader::load_from_str(&context.content);

        problems
    }

    fn default_level(&self) -> RuleLevel {
        RuleLevel::Error
    }
}

/// Check for duplicate keys using a line-based approach
fn check_duplicate_keys_in_lines(context: &LintContext) -> Vec<LintProblem> {
    let mut problems = Vec::new();
    let mut key_tracker: Vec<(usize, HashSet<String>)> = vec![(0, HashSet::new())];

    for (line_idx, line) in context.lines.iter().enumerate() {
        // Skip empty lines and comments
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Calculate indentation level
        let indent = line.len() - line.trim_start().len();

        // Pop trackers for deeper indentation levels
        while key_tracker.len() > 1 && key_tracker.last().unwrap().0 >= indent {
            key_tracker.pop();
        }

        // Check if this is a list item first
        if let Some(after_hyphen) = trimmed.strip_prefix('-') {
            // List item - create new scope for this item
            key_tracker.push((indent, HashSet::new()));

            // Check if the list item has inline key-value (e.g., "- name: value")
            let after_dash = after_hyphen.trim_start();
            if let Some(colon_pos) = find_key_colon(after_dash) {
                let key_part = &after_dash[..colon_pos].trim();
                let key = extract_key(key_part);

                if !key.is_empty() {
                    // This is the first key in this list item's scope, just add it
                    let current_level = key_tracker.last_mut().unwrap();
                    current_level.1.insert(key.clone());
                }

                // Check if this key has nested content
                let after_colon = &after_dash[colon_pos + 1..].trim();
                if after_colon.is_empty() || after_colon.starts_with('#') {
                    // Create new level for nested content
                    key_tracker.push((indent + 2, HashSet::new()));
                }
            }
            continue;
        }

        // Check if this line looks like a key-value pair
        if let Some(colon_pos) = find_key_colon(line) {
            let key_part = &line[..colon_pos].trim();

            // Skip if it's a tag or directive
            if key_part.starts_with('%') || key_part.starts_with('!') {
                continue;
            }

            // Extract the key (remove quotes if present)
            let key = extract_key(key_part);

            if !key.is_empty() {
                // Check if key exists at current indentation level
                let current_level = key_tracker.last_mut().unwrap();

                if current_level.1.contains(&key) {
                    problems.push(LintProblem::new(
                        line_idx + 1,
                        1,
                        format!("found duplicate key \"{}\"", key),
                        "key-duplicates",
                        LintLevel::Error,
                    ));
                } else {
                    current_level.1.insert(key.clone());
                }
            }

            // Check if this key has a nested value (no value on same line)
            let after_colon = &line[colon_pos + 1..].trim();
            if after_colon.is_empty() || after_colon.starts_with('#') {
                // This is a parent key, create new level
                key_tracker.push((indent, HashSet::new()));
            }
        }
    }

    problems
}

/// Find the position of the key-value separator colon (outside of quotes)
fn find_key_colon(line: &str) -> Option<usize> {
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escaped = false;

    for (idx, ch) in line.chars().enumerate() {
        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' if in_double_quote => escaped = true,
            '\'' if !in_double_quote => in_single_quote = !in_single_quote,
            '"' if !in_single_quote => in_double_quote = !in_double_quote,
            ':' if !in_single_quote && !in_double_quote => {
                // Check if this is a key-value colon (followed by space or end)
                let rest = &line[idx + 1..];
                if rest.is_empty() || rest.starts_with(' ') || rest.starts_with('#') {
                    return Some(idx);
                }
            }
            _ => {}
        }
    }

    None
}

/// Extract key from key part (remove quotes)
fn extract_key(key_part: &str) -> String {
    let key = key_part.trim();

    // Remove surrounding quotes
    if ((key.starts_with('"') && key.ends_with('"'))
        || (key.starts_with('\'') && key.ends_with('\'')))
        && key.len() >= 2
    {
        return key[1..key.len() - 1].to_string();
    }

    key.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_duplicates() {
        let yaml = "key1: value1\nkey2: value2\nkey3: value3\n";
        let context = LintContext::new(yaml.to_string());
        let rule = KeyDuplicatesRule;
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_simple_duplicate() {
        let yaml = "key: value1\nkey: value2\n";
        let context = LintContext::new(yaml.to_string());
        let rule = KeyDuplicatesRule;
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].line, 2);
        assert!(problems[0].message.contains("duplicate key"));
    }

    #[test]
    fn test_duplicate_with_quotes() {
        let yaml = "\"key\": value1\n\"key\": value2\n";
        let context = LintContext::new(yaml.to_string());
        let rule = KeyDuplicatesRule;
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
    }

    #[test]
    fn test_nested_no_duplicate() {
        let yaml = "parent:\n  key: value1\nanother:\n  key: value2\n";
        let context = LintContext::new(yaml.to_string());
        let rule = KeyDuplicatesRule;
        let problems = rule.check(&context);

        // Same key name in different scopes is OK
        assert!(problems.is_empty());
    }

    #[test]
    fn test_nested_duplicate() {
        let yaml = "parent:\n  key: value1\n  key: value2\n";
        let context = LintContext::new(yaml.to_string());
        let rule = KeyDuplicatesRule;
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].line, 3);
    }

    #[test]
    fn test_multiple_duplicates() {
        let yaml = "key1: value1\nkey1: value2\nkey2: value3\nkey2: value4\n";
        let context = LintContext::new(yaml.to_string());
        let rule = KeyDuplicatesRule;
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 2);
    }

    #[test]
    fn test_comment_ignored() {
        let yaml = "key: value1\n# key: comment\nkey2: value2\n";
        let context = LintContext::new(yaml.to_string());
        let rule = KeyDuplicatesRule;
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_colon_in_string_ignored() {
        let yaml = "key: \"value:with:colon\"\nkey2: value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = KeyDuplicatesRule;
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_list_items_different_scope() {
        let yaml = "list:\n  - key: value1\n  - key: value2\n";
        let context = LintContext::new(yaml.to_string());
        let rule = KeyDuplicatesRule;
        let problems = rule.check(&context);

        // Same key in different list items is OK
        assert!(problems.is_empty());
    }
}
