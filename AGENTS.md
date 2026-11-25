# Agent instructions for yaml-lint-rs

yaml-lint-rs is a Rust YAML linter inspired by Python yamllint. The workspace uses Rust edition 2024. CI checks Rust 1.85 compatibility, so keep new code and dependencies compatible with that version.

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

## Adding or changing a lint rule

1. Implement `Rule` in [core/src/rules/](core/src/rules/) with a snake_case module and a kebab-case rule name. Return `Vec<LintProblem>` from `check`. Use nearby rules as examples for diagnostic positions and YAML edge cases.
2. Declare the module and register the rule in `RuleRegistry::with_defaults()`. Wire up construction in `Config::create_registry()` too, since configurations with rule entries use that path. Set the intended enablement and severity in the default and relaxed presets.
3. For configurable rules, wire the options through `RuleOptions`, parsing, and construction. Check that explicit options and omitted options both reach the expected behavior.
4. Cover valid input, violations, and relevant edge cases in unit tests. Add a fixture and a test in [integration_tests.rs](core/tests/integration_tests.rs) for a new rule. Assert the specific rule and its expected diagnostics, since other rules can report problems on the same fixture. Use [config_options.rs](core/tests/config_options.rs) for configuration behavior and exercise the fixer if the rule supports auto-fix.
5. Update [docs/RULES.md](docs/RULES.md) and the [README rule list](README.md#implemented-rules) for rule additions. Update [docs/USAGE.md](docs/USAGE.md) when CLI or configuration usage changes.
6. Run `make ci` and `make validate-fixtures` for rule changes.

## GitHub issues

When asked to create an issue, check for duplicates and inspect the current implementation first. Follow the [GitHub issue conventions](.claude/rules/github-issue.md) for titles, labels, and the body structure. Use the current code and docs for implementation status. For a new rule, include config and valid and invalid examples, intended presets, and the equivalent yamllint rule when applicable. For a bug, include reproduction steps and expected and actual behavior.
