//! Integration tests for configuration options

use yaml_lint_core::{Config, Linter};

#[test]
fn test_line_length_custom_max() {
    let config_yaml = r#"
rules:
  line-length:
    max: 120
"#;
    let config = Config::load_from_str(config_yaml).unwrap();
    let linter = Linter::new(config);

    // 100-character line (should pass with max: 120)
    let yaml = format!("key: {}\n", "x".repeat(95));
    let problems = linter.lint_string(&yaml).unwrap();
    assert!(
        problems.is_empty(),
        "Expected no problems with 100-char line and max: 120"
    );

    // 125-character line (should fail with max: 120)
    let yaml = format!("key: {}\n", "x".repeat(120));
    let problems = linter.lint_string(&yaml).unwrap();
    assert_eq!(
        problems.len(),
        1,
        "Expected 1 problem with 125-char line and max: 120"
    );
    assert_eq!(problems[0].rule, "line-length");
}

#[test]
fn test_indentation_fixed_spaces() {
    let config_yaml = r#"
rules:
  indentation:
    spaces: 2
"#;
    let config = Config::load_from_str(config_yaml).unwrap();
    let linter = Linter::new(config);

    // Valid 2-space indentation
    let yaml = "key:\n  subkey: value\n";
    let problems = linter.lint_string(yaml).unwrap();
    assert!(
        problems.is_empty(),
        "Expected no problems with 2-space indentation"
    );

    // Invalid 3-space indentation when expecting 2
    let yaml = "key:\n   subkey: value\n";
    let problems = linter.lint_string(yaml).unwrap();
    assert!(
        !problems.is_empty(),
        "Expected problems with 3-space indentation when expecting 2"
    );
}

#[test]
fn test_indentation_consistent() {
    let config_yaml = r#"
rules:
  indentation:
    spaces: consistent
"#;
    let config = Config::load_from_str(config_yaml).unwrap();
    let linter = Linter::new(config);

    // Consistent 2-space indentation
    let yaml = "key:\n  sub1: value\n  sub2: value\n";
    let problems = linter.lint_string(yaml).unwrap();
    assert!(
        problems.is_empty(),
        "Expected no problems with consistent 2-space indentation"
    );

    // Consistent 4-space indentation
    let yaml = "key:\n    sub1: value\n    sub2: value\n";
    let problems = linter.lint_string(yaml).unwrap();
    assert!(
        problems.is_empty(),
        "Expected no problems with consistent 4-space indentation"
    );
}

#[test]
fn test_colons_custom_spacing() {
    let config_yaml = r#"
rules:
  colons:
    max-spaces-before: 1
    max-spaces-after: 2
"#;
    let config = Config::load_from_str(config_yaml).unwrap();
    let linter = Linter::new(config);

    // Valid: 1 space before, 2 spaces after
    let yaml = "key :  value\n";
    let problems = linter.lint_string(yaml).unwrap();
    assert!(
        problems.is_empty(),
        "Expected no problems with custom colon spacing"
    );

    // Invalid: 2 spaces before (exceeds max-spaces-before: 1)
    let yaml = "key  : value\n";
    let problems = linter.lint_string(yaml).unwrap();
    assert!(
        !problems.is_empty(),
        "Expected problems with 2 spaces before colon"
    );
}

#[test]
fn test_empty_lines_custom_limits() {
    let config_yaml = r#"
rules:
  empty-lines:
    max: 1
    max-start: 1
    max-end: 1
"#;
    let config = Config::load_from_str(config_yaml).unwrap();
    let linter = Linter::new(config);

    // Valid: 1 empty line at start, 1 in middle, 1 at end
    let yaml = "\nkey1: value1\n\nkey2: value2\n\n";
    let problems = linter.lint_string(yaml).unwrap();
    assert!(
        problems.is_empty(),
        "Expected no problems with custom empty-lines limits"
    );

    // Invalid: 2 empty lines in middle (exceeds max: 1)
    let yaml = "key1: value1\n\n\nkey2: value2\n";
    let problems = linter.lint_string(yaml).unwrap();
    assert!(
        !problems.is_empty(),
        "Expected problems with 2 consecutive empty lines"
    );
}

#[test]
fn test_hyphens_custom_spacing() {
    let config_yaml = r#"
rules:
  hyphens:
    max-spaces-after: 2
"#;
    let config = Config::load_from_str(config_yaml).unwrap();
    let linter = Linter::new(config);

    // Valid: 2 spaces after hyphen
    let yaml = "list:\n  -  item\n";
    let problems = linter.lint_string(yaml).unwrap();
    assert!(
        problems.is_empty(),
        "Expected no problems with 2 spaces after hyphen"
    );

    // Invalid: 3 spaces after hyphen (exceeds max: 2)
    let yaml = "list:\n  -   item\n";
    let problems = linter.lint_string(yaml).unwrap();
    assert!(
        !problems.is_empty(),
        "Expected problems with 3 spaces after hyphen"
    );
}

#[test]
fn test_comments_custom_options() {
    let config_yaml = r#"
rules:
  comments:
    require-starting-space: false
    min-spaces-from-content: 3
"#;
    let config = Config::load_from_str(config_yaml).unwrap();
    let linter = Linter::new(config);

    // Valid: no space after #, 3 spaces before comment
    let yaml = "#Comment\nkey: value   #Comment\n";
    let problems = linter.lint_string(yaml).unwrap();
    assert!(
        problems.is_empty(),
        "Expected no problems with custom comment options"
    );

    // Invalid: only 2 spaces before inline comment (needs 3)
    let yaml = "key: value  #Comment\n";
    let problems = linter.lint_string(yaml).unwrap();
    assert!(
        !problems.is_empty(),
        "Expected problems with 2 spaces before comment (needs 3)"
    );
}

#[test]
fn test_truthy_custom_allowed_values() {
    let config_yaml = r#"
rules:
  truthy:
    allowed-values: ["true", "false", "yes", "no"]
    check-keys: false
"#;
    let config = Config::load_from_str(config_yaml).unwrap();
    let linter = Linter::new(config);

    // Valid: yes/no are allowed
    let yaml = "enabled: yes\ndisabled: no\n";
    let problems = linter.lint_string(yaml).unwrap();
    assert!(
        problems.is_empty(),
        "Expected no problems when yes/no are allowed"
    );

    // Invalid: on/off are not allowed
    let yaml = "feature: on\n";
    let problems = linter.lint_string(yaml).unwrap();
    assert!(
        !problems.is_empty(),
        "Expected problems with 'on' when not in allowed-values"
    );
}

#[test]
fn test_truthy_check_keys() {
    let config_yaml = r#"
rules:
  truthy:
    allowed-values: ["true", "false"]
    check-keys: true
"#;
    let config = Config::load_from_str(config_yaml).unwrap();
    let linter = Linter::new(config);

    // Invalid: truthy key (yes) when check-keys is true
    let yaml = "yes: value\n";
    let problems = linter.lint_string(yaml).unwrap();
    assert!(
        !problems.is_empty(),
        "Expected problems with truthy key when check-keys is true"
    );
    assert_eq!(problems[0].rule, "truthy");
}

#[test]
fn test_document_start_required() {
    let config_yaml = r#"
rules:
  document-start:
    present: true
"#;
    let config = Config::load_from_str(config_yaml).unwrap();
    let linter = Linter::new(config);

    // Valid: document starts with ---
    let yaml = "---\nkey: value\n";
    let problems = linter.lint_string(yaml).unwrap();
    assert!(problems.is_empty(), "Expected no problems with --- present");

    // Invalid: document doesn't start with ---
    let yaml = "key: value\n";
    let problems = linter.lint_string(yaml).unwrap();
    assert!(!problems.is_empty(), "Expected problems without ---");
    assert_eq!(problems[0].rule, "document-start");
}

#[test]
fn test_document_start_forbidden() {
    let config_yaml = r#"
rules:
  document-start:
    present: false
"#;
    let config = Config::load_from_str(config_yaml).unwrap();
    let linter = Linter::new(config);

    // Valid: document doesn't start with ---
    let yaml = "key: value\n";
    let problems = linter.lint_string(yaml).unwrap();
    assert!(problems.is_empty(), "Expected no problems without ---");

    // Invalid: document starts with --- when forbidden
    let yaml = "---\nkey: value\n";
    let problems = linter.lint_string(yaml).unwrap();
    assert!(
        !problems.is_empty(),
        "Expected problems with --- when forbidden"
    );
}

#[test]
fn test_backwards_compatibility_string_levels() {
    let config_yaml = r#"
rules:
  trailing-spaces: error
  line-length: warning
  document-start: disable
"#;
    let config = Config::load_from_str(config_yaml).unwrap();
    let linter = Linter::new(config);

    // Should work exactly as before
    let yaml = "key: value   \n";
    let problems = linter.lint_string(yaml).unwrap();
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].rule, "trailing-spaces");
    assert_eq!(problems[0].level, yaml_lint_core::LintLevel::Error);
}

#[test]
fn test_mixed_configuration() {
    let config_yaml = r#"
extends: default
rules:
  line-length:
    max: 100
  trailing-spaces: warning
  document-start: disable
"#;
    let config = Config::load_from_str(config_yaml).unwrap();
    let linter = Linter::new(config);

    // line-length should use custom max: 100
    let yaml = format!("key: {}\n", "x".repeat(90));
    let problems = linter.lint_string(&yaml).unwrap();
    assert!(
        problems.is_empty(),
        "Expected no problems with 95-char line and max: 100"
    );

    // trailing-spaces should be warning level
    let yaml = "key: value   \n";
    let problems = linter.lint_string(yaml).unwrap();
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].level, yaml_lint_core::LintLevel::Warning);
}

#[test]
fn test_config_with_explicit_level_in_options() {
    let config_yaml = r#"
rules:
  line-length:
    level: warning
    max: 120
"#;
    let config = Config::load_from_str(config_yaml).unwrap();
    let linter = Linter::new(config);

    let yaml = format!("key: {}\n", "x".repeat(120));
    let problems = linter.lint_string(&yaml).unwrap();
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].rule, "line-length");
    assert_eq!(problems[0].level, yaml_lint_core::LintLevel::Warning);
}

#[test]
fn test_invalid_option_value_error() {
    let config_yaml = r#"
rules:
  line-length:
    max: 0
"#;
    let result = Config::load_from_str(config_yaml);
    assert!(result.is_err(), "Expected error for max: 0");
    assert!(result.unwrap_err().to_string().contains("greater than 0"));
}

#[test]
fn test_invalid_indentation_spaces_error() {
    let config_yaml = r#"
rules:
  indentation:
    spaces: 0
"#;
    let result = Config::load_from_str(config_yaml);
    assert!(result.is_err(), "Expected error for spaces: 0");
    assert!(result.unwrap_err().to_string().contains("between 1 and 16"));
}

#[test]
fn test_empty_truthy_allowed_values_error() {
    let config_yaml = r#"
rules:
  truthy:
    allowed-values: []
"#;
    let result = Config::load_from_str(config_yaml);
    assert!(result.is_err(), "Expected error for empty allowed-values");
    assert!(result.unwrap_err().to_string().contains("cannot be empty"));
}

#[test]
fn test_rule_without_options_support_error() {
    let config_yaml = r#"
rules:
  trailing-spaces:
    some-option: value
"#;
    let result = Config::load_from_str(config_yaml);
    assert!(
        result.is_err(),
        "Expected error for rule without options support"
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("does not support options")
    );
}

#[test]
fn test_document_start_invalid_type_error() {
    let config_yaml = r#"
rules:
  document-start:
    present: "true"
"#;
    let result = Config::load_from_str(config_yaml);
    assert!(result.is_err(), "Expected error for non-boolean value");
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("must be a boolean"),
        "Error message should indicate boolean type required"
    );
}
