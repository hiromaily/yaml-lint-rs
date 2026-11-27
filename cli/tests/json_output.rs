use serde_json::Value;
use std::path::Path;
use std::process::{Command, Output};

fn lint(preset: &str, args: &[&str], files: &[&str]) -> Output {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures");
    Command::new(env!("CARGO_BIN_EXE_yaml-lint"))
        .args(["--format", "json", "--preset", preset, "--jobs", "2"])
        .args(args)
        .args(files.iter().map(|file| fixtures.join(file)))
        .output()
        .unwrap()
}

#[test]
fn json_output_contains_diagnostics_from_multiple_files() {
    let output = lint(
        "default",
        &["--color", "always"],
        &[
            "invalid/trailing-spaces.yaml",
            "invalid/duplicate-keys.yaml",
        ],
    );
    assert_eq!(output.status.code(), Some(1));
    let diagnostics: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
    let first = diagnostics
        .iter()
        .find(|problem| problem["rule"] == "trailing-spaces")
        .unwrap();
    assert!(
        first["file"]
            .as_str()
            .unwrap()
            .ends_with("trailing-spaces.yaml")
    );
    assert_eq!(first["line"], 2);
    assert!(first["column"].as_u64().unwrap() > 0);
    assert_eq!(first["rule"], "trailing-spaces");
    assert_eq!(first["level"], "error");
    assert!(!first["message"].as_str().unwrap().is_empty());
    let last = diagnostics.last().unwrap();
    assert!(
        last["file"]
            .as_str()
            .unwrap()
            .ends_with("duplicate-keys.yaml")
    );
    assert_eq!(last["rule"], "key-duplicates");
    assert!(String::from_utf8_lossy(&output.stderr).contains("Found"));
}

#[test]
fn json_output_is_an_empty_array_for_clean_files() {
    let output = lint("default", &[], &["valid/with-document-start.yaml"]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"[]\n");
}

#[test]
fn json_output_preserves_warning_exit_codes() {
    for (args, expected) in [(&[][..], 0), (&["--strict"][..], 2)] {
        let output = lint("default", args, &["valid/no-document-start.yaml"]);
        assert_eq!(output.status.code(), Some(expected));
        let diagnostics: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
        assert!(!diagnostics.is_empty());
        assert!(
            diagnostics
                .iter()
                .all(|problem| problem["level"] == "warning")
        );
    }
}

#[test]
fn json_output_requires_lint_mode() {
    for flag in ["--list-files", "--fix", "--dry-run"] {
        let output = lint("default", &[flag], &["valid/simple.yaml"]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("JSON output is only available for linting")
        );
    }
}
