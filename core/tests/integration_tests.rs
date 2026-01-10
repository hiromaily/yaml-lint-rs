//! Integration tests for yaml-lint-rs

use std::path::PathBuf;
use yaml_lint_core::{Config, Fixer, Linter};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
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
    assert_eq!(problems.len(), 3, "Expected 3 trailing space problems");

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

    assert!(
        !problems.is_empty(),
        "Expected to find colon spacing issues"
    );
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
    let yaml =
        "key: value   \nvery_long: this line is way too long and exceeds the limit\nkey2: test  \n";
    let problems = linter.lint_string(yaml).expect("Failed to lint string");

    // Problems should be sorted by line number
    for i in 0..problems.len() - 1 {
        assert!(
            problems[i].line <= problems[i + 1].line,
            "Problems not sorted by line"
        );
    }
}

// ==============================================================================
// Fix option tests
// ==============================================================================

#[test]
fn test_fix_trailing_spaces() {
    let config = Config::with_default_preset();
    let registry = config.create_registry();
    let fixer = Fixer::new(&registry);

    let content = "key: value   \nkey2: value2  \n";
    let result = fixer.fix("test.yaml", content);

    assert!(result.has_fixes());
    assert!(result.fixes_by_rule.contains_key("trailing-spaces"));
    assert_eq!(
        result.fixed_content,
        Some("key: value\nkey2: value2\n".to_string())
    );
}

#[test]
fn test_fix_newline_at_end() {
    let config = Config::with_default_preset();
    let registry = config.create_registry();
    let fixer = Fixer::new(&registry);

    let content = "key: value";
    let result = fixer.fix("test.yaml", content);

    assert!(result.has_fixes());
    assert!(result.fixes_by_rule.contains_key("new-line-at-end-of-file"));
    assert_eq!(result.fixed_content, Some("key: value\n".to_string()));
}

#[test]
fn test_fix_empty_lines() {
    let config = Config::with_default_preset();
    let registry = config.create_registry();
    let fixer = Fixer::new(&registry);

    let content = "key1: value1\n\n\n\nkey2: value2\n";
    let result = fixer.fix("test.yaml", content);

    assert!(result.has_fixes());
    assert!(result.fixes_by_rule.contains_key("empty-lines"));
    // Should reduce to max 2 empty lines
    assert_eq!(
        result.fixed_content,
        Some("key1: value1\n\n\nkey2: value2\n".to_string())
    );
}

#[test]
fn test_fix_multiple_issues() {
    let config = Config::with_default_preset();
    let registry = config.create_registry();
    let fixer = Fixer::new(&registry);

    let content = "key: value   "; // trailing space + no newline
    let result = fixer.fix("test.yaml", content);

    assert!(result.has_fixes());
    assert!(result.fixes_applied >= 2);
    assert_eq!(result.fixed_content, Some("key: value\n".to_string()));
}

#[test]
fn test_fix_no_changes_needed() {
    let config = Config::with_default_preset();
    let registry = config.create_registry();
    let fixer = Fixer::new(&registry);

    let content = "key: value\n";
    let result = fixer.fix("test.yaml", content);

    assert!(!result.has_fixes());
    assert!(result.fixed_content.is_none());
}

#[test]
fn test_fix_preserves_valid_content() {
    let config = Config::with_default_preset();
    let registry = config.create_registry();
    let fixer = Fixer::new(&registry);

    let content = "name: test\nitems:\n  - one\n  - two\n";
    let result = fixer.fix("test.yaml", content);

    // Should not change valid content
    assert!(!result.has_fixes());
}

#[test]
fn test_fix_unfixable_problems() {
    let config = Config::with_default_preset();
    let registry = config.create_registry();
    let fixer = Fixer::new(&registry);

    // Duplicate keys cannot be auto-fixed
    let content = "name: value1\nname: value2\n";
    let result = fixer.fix("test.yaml", content);

    assert!(result.has_unfixable());
    assert!(
        result
            .unfixable_problems
            .iter()
            .any(|p| p.rule == "key-duplicates")
    );
}

#[test]
fn test_fix_fixture_file() {
    let config = Config::with_default_preset();
    let registry = config.create_registry();
    let fixer = Fixer::new(&registry);

    let path = fixture_path("invalid/trailing-spaces.yaml");
    let content = std::fs::read_to_string(&path).expect("Failed to read fixture");
    let result = fixer.fix(&path.display().to_string(), &content);

    assert!(result.has_fixes());
    assert!(result.fixes_by_rule.contains_key("trailing-spaces"));

    // Verify fixed content has no trailing spaces
    if let Some(fixed) = &result.fixed_content {
        for line in fixed.lines() {
            assert!(
                !line.ends_with(' ') && !line.ends_with('\t'),
                "Fixed content still has trailing spaces"
            );
        }
    }
}
