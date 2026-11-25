# Agent instructions for yaml-lint-rs

yaml-lint-rs is a Rust YAML linter inspired by Python yamllint. The workspace uses Rust edition 2024. CI checks Rust 1.100 compatibility, so keep new code and dependencies compatible with that version.

## Where to work

| Task | Entry points |
| --- | --- |
| CLI arguments, file discovery, exit codes | [cli/src/main.rs](cli/src/main.rs) in the `yaml-lint` package |
| Library API and lint orchestration | [lib.rs](core/src/lib.rs), [linter.rs](core/src/linter.rs) in the `yaml-lint-core` package |
| Config discovery, presets, options, configured registry | [config.rs](core/src/config.rs) |
| Rule trait, context, and default registry | [rules/mod.rs](core/src/rules/mod.rs) |
| Diagnostics, output, and auto-fix | [problem.rs](core/src/problem.rs), [output/](core/src/output/), [fixer.rs](core/src/fixer.rs) |
| Integration tests and fixtures | [core/tests/](core/tests/), [tests/fixtures/](tests/fixtures/) |

Use `yaml-rust2` for YAML parsing and the existing Serde code for config deserialization. Use `thiserror` for library error types and `?` to propagate errors. Follow rustfmt and Clippy, keep functions focused, and document public APIs with doc comments. Put unit tests alongside the code they exercise.

## Development checks

Run these commands from the repository root.

| Command | Purpose |
| --- | --- |
| `make build` | Build the debug CLI |
| `make fmt` | Format Rust code |
| `make test-one TEST=test_name` | Run tests matching a name |
| `cargo test -p yaml-lint-core --test config_options` | Run rule configuration integration tests |
| `cargo run -p yaml-lint -- --help` | Inspect the CLI |
| `make ci` | Run formatting checks, Clippy with warnings denied, and workspace tests before committing |
| `make validate-fixtures` | Build the release CLI and check that the valid fixture directory passes and the invalid directory fails |
| `make validate-fixtures-detail` | Print per-file fixture results |

`make help` lists the available targets. See [GitHub Actions CI](.github/workflows/ci.yml) for additional build, release-mode test, and MSRV checks. Use `--dry-run` or copies of fixtures when exercising auto-fix.

## Codex skills

Use these repository skills from the repository root. In Codex, enter the skill name with `$` followed by the task details.

| Task | Invocation |
| --- | --- |
| Add or update a lint rule | [`$adding-rule-validation`](.agents/skills/adding-rule-validation/SKILL.md) |
| Draft or create a GitHub issue | [`$create-issue`](.agents/skills/create-issue/SKILL.md) |
