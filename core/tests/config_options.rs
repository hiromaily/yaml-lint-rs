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
fn line_length_can_allow_long_words_and_mapping_values() {
    let source = "- key: https://example.com/long\n- https://example.com/long\n";
    let config = Config::load_from_str("rules: {line-length: {max: 20}}").unwrap();
    let problems = Linter::new(config).lint_string(source).unwrap();
    assert_eq!(problems.len(), 1);
    assert_eq!((problems[0].line, problems[0].column), (1, 21));

    let config = Config::load_from_str(
        "rules: {line-length: {max: 20, allow-non-breakable-inline-mappings: true}}",
    )
    .unwrap();
    assert!(Linter::new(config).lint_string(source).unwrap().is_empty());
}

#[test]
fn file_extends_inherits_options_and_decodes_ignore_file() {
    let directory =
        std::env::temp_dir().join(format!("yaml-lint-config-test-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let base = directory.join("base.yaml");
    let ignore = directory.join("ignored.txt");
    let child = directory.join("child.yaml");
    std::fs::write(
        &ignore,
        "*.generated.yaml\n"
            .chars()
            .flat_map(|ch| u32::from(ch).to_be_bytes())
            .collect::<Vec<_>>(),
    )
    .unwrap();
    std::fs::write(
        &base,
        format!(
            "ignore-from-file: '{}'\nrules: {{line-length: {{max: 5, level: warning}}}}\n",
            ignore.display()
        ),
    )
    .unwrap();
    let settings = format!(
        "extends: '{}'\nrules: {{line-length: {{allow-non-breakable-words: false}}}}\n",
        base.display()
    );
    std::fs::write(&child, &settings).unwrap();
    let loaded = Config::load_from_file(&child);
    std::fs::write(&base, format!("extends: '{}'\n", child.display())).unwrap();
    let cycle = Config::load_from_file(&child);
    std::fs::remove_file(base).unwrap();
    std::fs::remove_file(ignore).unwrap();
    std::fs::remove_file(child).unwrap();
    std::fs::remove_dir(directory).unwrap();

    let config = loaded.unwrap();
    assert_eq!(config.ignore, ["*.generated.yaml"]);
    let problems = Linter::new(config).lint_string("abcdef\n").unwrap();
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].column, 6);
    assert_eq!(problems[0].level, yaml_lint_core::LintLevel::Warning);
    assert!(
        cycle
            .unwrap_err()
            .to_string()
            .contains("Circular config inheritance")
    );
}

#[test]
fn indentation_sequence_policies_apply_to_mapping_values() {
    let source = "first:\n- a\nsecond:\n  - b\n";
    for (policy, expected) in [
        ("true", &[2][..]),
        ("false", &[4]),
        ("whatever", &[]),
        ("consistent", &[4]),
    ] {
        let config = Config::load_from_str(&format!(
            "rules: {{indentation: {{spaces: 2, indent-sequences: {policy}}}}}"
        ))
        .unwrap();
        let problems = Linter::new(config).lint_string(source).unwrap();
        let lines: Vec<_> = problems.iter().map(|problem| problem.line).collect();
        assert_eq!(lines, expected, "policy {policy}");
        assert!(problems.iter().all(|problem| problem.rule == "indentation"));
    }
    assert!(Config::load_from_str("rules: {indentation: {indent-sequences: invalid}}").is_err());
}

#[test]
fn flow_indentation_checks_values_and_closing_delimiters() {
    let config = Config::load_from_str("rules: {indentation: {spaces: 2}}").unwrap();
    let source = "items: [\n  first,\n   second,\n  ]\n";
    let problems = Linter::new(config).lint_string(source).unwrap();
    let diagnostics: Vec<_> = problems
        .iter()
        .map(|problem| {
            (
                problem.rule.as_str(),
                problem.line,
                problem.column,
                problem.message.as_str(),
            )
        })
        .collect();
    assert_eq!(
        diagnostics,
        [
            (
                "indentation",
                3,
                4,
                "wrong indentation: expected 2 but found 3"
            ),
            (
                "indentation",
                4,
                3,
                "wrong indentation: expected 0 but found 2"
            ),
        ]
    );
    let source = "items: [\n  first,\n  key: value,\n]\n";
    let config = Config::load_from_str("rules: {indentation: enable}").unwrap();
    assert!(Linter::new(config).lint_string(source).unwrap().is_empty());
}

#[test]
fn multiline_string_indentation_preserves_scalar_content_boundaries() {
    let source = concat!(
        "plain: 日本語\n",
        "  continuation\n",
        "quoted: \"start\n",
        "  continuation\"\n",
        "literal: |\n",
        "  regular\n",
        "   extra space\n",
        "folded: >\n",
        "  regular\n",
        "   extra space\n",
        "next: value\n",
    );
    let config = Config::load_from_str("rules: {indentation: {spaces: 2}}").unwrap();
    assert!(Linter::new(config).lint_string(source).unwrap().is_empty());

    let config =
        Config::load_from_str("rules: {indentation: {spaces: 2, check-multi-line-strings: true}}")
            .unwrap();
    let problems = Linter::new(config).lint_string(source).unwrap();
    let positions: Vec<_> = problems.iter().map(|p| (p.line, p.column)).collect();
    assert_eq!(positions, [(2, 3), (4, 3), (7, 4), (10, 4)]);
    assert!(problems.iter().all(|p| p.rule == "indentation"));
    assert!(
        Config::load_from_str("rules: {indentation: {check-multi-line-strings: invalid}}").is_err()
    );
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
fn empty_truthy_allowlist_rejects_every_boolean_spelling() {
    let config = Config::load_from_str("rules: {truthy: {allowed-values: []}}").unwrap();
    let problems = Linter::new(config)
        .lint_string("[true, false, yes, n]")
        .unwrap();
    assert_eq!(problems.len(), 3);
    assert!(problems.iter().all(|problem| problem.rule == "truthy"));
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

#[test]
fn test_new_lines_configuration() {
    let config =
        Config::load_from_str("rules:\n  new-lines: {type: dos, level: warning}\n").unwrap();
    let linter = Linter::new(config);
    assert!(linter.lint_string("key: value\r\n").unwrap().is_empty());
    let problems = linter.lint_string("key: value\n").unwrap();
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].rule, "new-lines");
    assert_eq!(problems[0].level, yaml_lint_core::LintLevel::Warning);

    for settings in ["error", "{level: error}", "{type: unix}"] {
        let config = Config::load_from_str(&format!("rules:\n  new-lines: {settings}\n")).unwrap();
        let linter = Linter::new(config);
        assert!(linter.lint_string("key: value\n").unwrap().is_empty());
        assert_eq!(linter.lint_string("key: value\r\n").unwrap().len(), 1);
    }

    for invalid in ["lf", "true", "null"] {
        let config = format!("rules:\n  new-lines: {{type: {invalid}}}\n");
        assert!(Config::load_from_str(&config).is_err());
    }
}

#[test]
fn phase_3_preset_levels_and_registration() {
    use yaml_lint_core::{LintLevel, rules::RuleLevel};

    let default = Config::with_default_preset();
    let relaxed = Config::with_relaxed_preset();
    assert_eq!(default.create_registry().rule_names().len(), 23);
    assert_eq!(Config::new().create_registry().rule_names().len(), 23);
    assert_eq!(
        relaxed.get_rule_level("comments-indentation"),
        Some(RuleLevel::Disable)
    );
    assert_eq!(
        default.get_rule_level("quoted-strings"),
        Some(RuleLevel::Disable)
    );
    let problems = Linter::new(relaxed).lint_string("x: [a,b]\n").unwrap();
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].rule, "commas");
    assert_eq!(problems[0].level, LintLevel::Warning);
}

#[test]
fn phase_3_configured_options_and_yaml_boundaries() {
    let cases: &[(&str, &str, &[usize])] = &[
        (
            "anchors: {forbid-unused-anchors: true, forbid-duplicated-anchors: true}",
            "---\na: &same x\nb: *same\nc: &same z\n---\nd: *same\n",
            &[4, 4, 6],
        ),
        (
            "commas: {max-spaces-before: -1, min-spaces-after: 0, max-spaces-after: 1}",
            "a: [\"1,2\" ,3,  4]\nb: |\n  [1,2]\n",
            &[1],
        ),
        (
            "comments-indentation: {}",
            "a: |\n  literal\n    # scalar content\n  # scalar content\n# outside\nb: 1\n",
            &[],
        ),
        (
            "document-end: {present: false}",
            "---\na: |\n  ...\n...\n---\nb: \"...\"\n",
            &[4],
        ),
        (
            "empty-values: {forbid-in-block-mappings: false}",
            "a:\nb: {c: }\nd:\n  - # comment\n  - null\n  - ''\n",
            &[2, 4],
        ),
        (
            "float-values: {forbid-nan: true, forbid-scientific-notation: true, require-numeral-before-decimal: true}",
            "a: -.5\nb: 1e3\nc: .NaN\nd: '.NaN'\ne: !!str 1e3\nf: 1.2\n",
            &[1, 2, 3],
        ),
        (
            "key-ordering: {ignored-keys: ['^ignore']}",
            "z: {a: 1, b: 2}\nignore-me: 0\na: 1\nb: 2\n",
            &[3, 4],
        ),
        (
            "octal-values: {forbid-implicit-octal: false}",
            "a: [012, 0o12, '0o13', !!str 0o14, 019, 0o18]\n",
            &[1],
        ),
        (
            "quoted-strings: {required: only-when-needed, quote-type: single, extra-required: ['^force'], extra-allowed: ['^ok']}",
            "a: 'unused'\nb: 'ok'\nc: force-me\nd: 'false'\ne: plain\nf: \"123\"\ng: [',']\n",
            &[1, 3, 6],
        ),
        (
            "quoted-strings: {quote-type: consistent, check-keys: true}",
            "---\n'a': 'value'\n\"b\": 'value'\n---\n\"c\": \"value\"\n",
            &[3],
        ),
    ];
    for &(rule, yaml, expected) in cases {
        let config = Config::load_from_str(&format!("rules:\n  {rule}\n")).unwrap();
        let problems = Linter::new(config).lint_string(yaml).unwrap();
        let lines: Vec<_> = problems.iter().map(|problem| problem.line).collect();
        assert_eq!(lines, expected, "{rule}: {problems:?}");
    }
}

#[test]
fn phase_3_invalid_configuration_is_reported() {
    for rule in [
        "anchors: {forbid-unused-anchors: 1}",
        "commas: {min-spaces-aftr: 2}",
        "key-ordering: {ignored-keys: ['[']}",
        "quoted-strings: {required: never}",
        "quoted-strings: {required: true, extra-allowed: [foo]}",
    ] {
        assert!(
            Config::load_from_str(&format!("rules:\n  {rule}\n")).is_err(),
            "{rule}"
        );
    }
}

#[test]
fn phase_3_pcre_patterns_reach_key_ordering_and_quoted_strings() {
    let config = Config::load_from_str(
        r"rules:
  key-ordering: {ignored-keys: ['^(\w+)\1$']}
  quoted-strings:
    required: only-when-needed
    extra-required: ['^force(?=me$)']
    extra-allowed: ['(?<=let)me$']
",
    )
    .unwrap();
    let yaml = "b: plain\nああ: forceme\nc: 'letme'\na: forceyou\n";
    let problems = Linter::new(config).lint_string(yaml).unwrap();
    let diagnostics: Vec<_> = problems
        .iter()
        .map(|p| (p.line, p.column, p.rule.as_str()))
        .collect();
    assert_eq!(
        diagnostics,
        [(2, 5, "quoted-strings"), (4, 1, "key-ordering")]
    );
}

#[test]
fn phase_3_invalid_locale_is_reported() {
    for value in ["42", "yaml_lint_nonexistent_locale", r#""bad\0locale""#] {
        let error = Config::load_from_str(&format!("locale: {value}\n")).unwrap_err();
        assert!(error.to_string().contains("locale"), "{error}");
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn phase_3_locale_controls_key_ordering() {
    let lint = |settings: &str, rule: &str| {
        let config = Config::load_from_str(&format!("{settings}rules:\n  key-ordering: {rule}\n"))?;
        Linter::new(config).lint_string("a: 1\nB: 2\n")
    };

    let problems = lint("", "enable").unwrap();
    assert_eq!((problems[0].line, problems[0].column), (2, 1));
    assert_eq!(
        problems[0].message,
        "wrong ordering of key \"B\" in mapping"
    );
    assert_eq!(lint("locale: C\n", "enable").unwrap(), problems);

    // Minimal Linux environments may only have the C locale installed.
    let settings = "locale: en_US.UTF-8\n";
    if cfg!(target_os = "linux") && Config::load_from_str(settings).is_err() {
        return;
    }
    assert!(lint(settings, "enable").unwrap().is_empty());
    assert!(lint(settings, "{ignored-keys: ['^z']}").unwrap().is_empty());
    assert_eq!(lint("", "enable").unwrap(), problems);
}

#[test]
fn duplicate_keys_follow_yaml_scopes_and_merge_key_configuration() {
    let yaml = concat!(
        "block: |\n  artefact: 1\n  artefact: 2\n",
        "flow: {é: 1, \"é\": 2, <<: {}, <<: {}}\n",
        "list:\n  - key: 1\n    key: 2\n  - key: 3\n",
        "---\nblock: 2\n"
    );
    for (option, expected) in [
        ("{}", vec![(4, 14), (7, 5)]),
        (
            "{forbid-duplicated-merge-keys: true}",
            vec![(4, 14), (4, 30), (7, 5)],
        ),
    ] {
        let config =
            Config::load_from_str(&format!("rules:\n  key-duplicates: {option}\n")).unwrap();
        let problems = Linter::new(config).lint_string(yaml).unwrap();
        let positions: Vec<_> = problems.iter().map(|p| (p.line, p.column)).collect();
        assert_eq!(positions, expected);
    }
}

#[test]
fn top_level_ignore_accepts_lines_and_lists() {
    for settings in [
        "ignore: |\n  generated/\n  !keep.yaml\n",
        "ignore: ['generated/', '!keep.yaml']",
    ] {
        let config = Config::load_from_str(settings).unwrap();
        assert_eq!(config.ignore, ["generated/", "!keep.yaml"]);
    }
    assert!(Config::load_from_str("ignore: true").is_err());
}

#[test]
fn path_configuration_rejects_non_string_patterns() {
    for yaml in [
        "yaml-files: '*.yaml'",
        "yaml-files: [true]",
        "ignore: [42]",
        "rules: {trailing-spaces: {ignore: true}}",
        "rules: {trailing-spaces: {ignore: '*.yaml', ignore-from-file: .gitignore}}",
    ] {
        assert!(Config::load_from_str(yaml).is_err(), "{yaml}");
    }
}

#[test]
fn rule_ignores_apply_to_linting_and_fixing() {
    use std::path::Path;
    use yaml_lint_core::Fixer;

    let config = Config::load_from_str(
        "rules:\n  trailing-spaces:\n    ignore: |\n      /generated/\n      !keep.yaml\n  truthy: enable\n",
    )
    .unwrap();
    let registry = config.create_registry();
    let linter = Linter::new(config);
    let source = "enabled: yes  \n";
    let ignored = "generated/config.yaml";

    let problems = linter
        .lint_string_with_path(source, Some(Path::new(ignored)))
        .unwrap();
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].rule, "truthy");
    assert_eq!(linter.lint_string(source).unwrap().len(), 2);
    assert_eq!(
        linter
            .lint_string_with_path(source, Some(Path::new("generated/keep.yaml")))
            .unwrap()
            .len(),
        2
    );

    let fixer = Fixer::new(&registry);
    assert!(!fixer.dry_run(ignored, source).has_fixes());
    assert!(fixer.dry_run("generated/keep.yaml", source).has_fixes());
}

#[test]
fn truthy_respects_yaml_versions_and_scalar_styles() {
    let config = Config::load_from_str("rules: {truthy: {allowed-values: [\"false\"]}}").unwrap();
    let linter = Linter::new(config);
    let yaml = "%YAML 1.2\n---\n[yes, TRUE, !!str True, 'True']\n...\n---\n[yes, false]\n";
    let positions: Vec<_> = linter
        .lint_string(yaml)
        .unwrap()
        .into_iter()
        .map(|problem| (problem.line, problem.column, problem.rule))
        .collect();
    assert_eq!(
        positions,
        [(3, 7, "truthy".into()), (6, 2, "truthy".into())]
    );
}

#[test]
fn flow_rules_can_forbid_non_empty_collections() {
    let config = Config::load_from_str(
        "rules:\n  braces: {forbid: non-empty}\n  brackets: {forbid: non-empty}\n",
    )
    .unwrap();
    let linter = Linter::new(config);
    let input = "map: {\n  # kept empty\n  }\nlist: []\na: {x: 1}\nb: [x]\n";
    let positions: Vec<_> = linter
        .lint_string(input)
        .unwrap()
        .into_iter()
        .map(|problem| (problem.line, problem.column, problem.rule))
        .collect();
    assert_eq!(
        positions,
        [(5, 5, "braces".into()), (6, 5, "brackets".into())]
    );
}

#[test]
fn colons_handle_explicit_keys_containing_mappings() {
    let config = Config::load_from_str("rules: {colons: enable}").unwrap();
    let linter = Linter::new(config);
    let problems = linter
        .lint_string("? a: 1\n: value\nflow: {key:  value}\n")
        .unwrap();
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].rule, "colons");
    assert_eq!((problems[0].line, problems[0].column), (3, 13));
}
