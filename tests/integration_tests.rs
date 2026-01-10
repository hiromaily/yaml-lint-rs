//! Integration tests for yaml-lint-rs

use std::path::PathBuf;
use yaml_lint_core::{Config, Linter};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn test_valid_simple_yaml() {
    let linter = Linter::with_defaults();
    let path = fixture_path("valid/simple.yaml");
    let problems = linter.lint_file(&path).expect("Failed to lint file");

    assert!(
        problems.is_empty(),
        "Expected no problems but found: {:?}",
        problems
    );
}

#[test]
fn test_valid_no_document_start() {
    let linter = Linter::with_defaults();
    let path = fixture_path("valid/no-document-start.yaml");
    let problems = linter.lint_file(&path).expect("Failed to lint file");

    // Should pass because document-start is disabled by default
    assert!(
        problems.is_empty(),
        "Expected no problems but found: {:?}",
        problems
    );
}

#[test]
fn test_trailing_spaces_detected() {
    let linter = Linter::with_defaults();
    let path = fixture_path("invalid/trailing-spaces.yaml");
    let problems = linter.lint_file(&path).expect("Failed to lint file");

    assert!(!problems.is_empty(), "Expected to find trailing spaces");
    assert_eq!(problems.len(), 2, "Expected 2 problems");

    for problem in &problems {
        assert_eq!(problem.rule, "trailing-spaces");
    }
}

#[test]
fn test_long_lines_detected() {
    let linter = Linter::with_defaults();
    let path = fixture_path("invalid/long-lines.yaml");
    let problems = linter.lint_file(&path).expect("Failed to lint file");

    assert!(!problems.is_empty(), "Expected to find long lines");
    assert!(problems.iter().any(|p| p.rule == "line-length"));
}

#[test]
fn test_duplicate_keys_detected() {
    let linter = Linter::with_defaults();
    let path = fixture_path("invalid/duplicate-keys.yaml");
    let problems = linter.lint_file(&path).expect("Failed to lint file");

    assert!(!problems.is_empty(), "Expected to find duplicate keys");
    assert!(problems.iter().any(|p| p.rule == "key-duplicates"));
}

#[test]
fn test_bad_indentation_detected() {
    let linter = Linter::with_defaults();
    let path = fixture_path("invalid/bad-indentation.yaml");
    let problems = linter.lint_file(&path).expect("Failed to lint file");

    assert!(!problems.is_empty(), "Expected to find indentation issues");
    assert!(problems.iter().any(|p| p.rule == "indentation"));
}

#[test]
fn test_bad_colons_detected() {
    let linter = Linter::with_defaults();
    let path = fixture_path("invalid/bad-colons.yaml");
    let problems = linter.lint_file(&path).expect("Failed to lint file");

    assert!(!problems.is_empty(), "Expected to find colon spacing issues");
    assert!(problems.iter().any(|p| p.rule == "colons"));
}

#[test]
fn test_lint_string() {
    let linter = Linter::with_defaults();
    let yaml = "key: value   \n";
    let problems = linter.lint_string(yaml).expect("Failed to lint string");

    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].rule, "trailing-spaces");
}

#[test]
fn test_config_from_string() {
    let config_yaml = r#"
extends: default
rules:
  trailing-spaces: warning
  line-length: disable
"#;

    let config = Config::load_from_str(config_yaml).expect("Failed to load config");
    let linter = Linter::new(config);

    let yaml = "key: value   \n";
    let problems = linter.lint_string(yaml).expect("Failed to lint string");

    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].level, yaml_lint_core::LintLevel::Warning);
}

#[test]
fn test_relaxed_preset() {
    let config = Config::with_relaxed_preset();
    let linter = Linter::new(config);

    let yaml = "key: value   \n";
    let problems = linter.lint_string(yaml).expect("Failed to lint string");

    // Relaxed preset should still catch trailing spaces but as warning
    assert!(!problems.is_empty());
    assert_eq!(problems[0].level, yaml_lint_core::LintLevel::Warning);
}

#[test]
fn test_disable_rule() {
    let mut config = Config::with_default_preset();
    config.rules.insert(
        "trailing-spaces".to_string(),
        yaml_lint_core::rules::RuleLevel::Disable,
    );
    let linter = Linter::new(config);

    let yaml = "key: value   \n";
    let problems = linter.lint_string(yaml).expect("Failed to lint string");

    // Trailing spaces should be ignored
    assert!(problems.is_empty());
}

#[test]
fn test_problems_sorted() {
    let linter = Linter::with_defaults();
    let yaml = "key: value   \nvery_long: this line is way too long and exceeds the limit\nkey2: test  \n";
    let problems = linter.lint_string(yaml).expect("Failed to lint string");

    // Problems should be sorted by line number
    for i in 0..problems.len() - 1 {
        assert!(
            problems[i].line <= problems[i + 1].line,
            "Problems not sorted by line"
        );
    }
}
