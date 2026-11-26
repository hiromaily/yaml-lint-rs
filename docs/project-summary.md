# Project Summary: yaml-lint-rs

## Overview

**yaml-lint-rs** is an extensible YAML linter written in Rust and inspired by Python yamllint. It provides 13 lint rules, configurable presets, parallel file linting, and auto-fix for common formatting issues.

## Project Statistics

- **Workspace version** 0.2.0
- **Language** Rust (Edition 2024)
- **Minimum Rust version** 1.100, checked in CI
- **Architecture** Cargo workspace with two crates
- **License** MIT
- **Author** Hiroki Yasui

See [Cargo.toml](../Cargo.toml) for the workspace settings and [CI](../.github/workflows/ci.yml) for the compatibility check.

## Architecture

- [yaml-lint-core](../core/src/lib.rs) provides the reusable linter, configuration, rule registry, diagnostics, auto-fix, and output formatters.
- [yaml-lint](../cli/src/main.rs) handles command-line arguments, file discovery, parallel execution, and exit codes.
- Integration tests live in [core/tests](../core/tests), and YAML fixtures live in [tests/fixtures](../tests/fixtures).

## Implemented Features

The [default rule registry](../core/src/rules/mod.rs) includes:

- `trailing-spaces`, `line-length`, `document-start`, `colons`, `key-duplicates`, and `indentation`
- `new-line-at-end-of-file`, `empty-lines`, `hyphens`, `comments`, and `truthy`
- `braces` and `brackets`

The CLI supports recursive linting of YAML files, parallel execution with `--jobs`, and standard, colored, and parsable output. Use `--fix` to correct trailing spaces, a missing final newline, or excess blank lines. Use `--dry-run` to preview those fixes.

Configuration files support the `default` and `relaxed` presets, per-rule severity levels, and rule-specific options. See the [CLI and library guide](cli-and-library-guide.md) and [rule documentation](lint-rules.md).

## Build and Test

Install Rust 1.100 or later from [rustup.rs](https://rustup.rs/), then run these commands from the repository root.

```bash
cargo build --release
cargo test --workspace
./target/release/yaml-lint examples/sample.yaml
```

## Usage Examples

```bash
yaml-lint file.yaml
yaml-lint -c .yamllint src/
yaml-lint -f parsable file.yaml
yaml-lint --strict file.yaml
yaml-lint --dry-run file.yaml
```

### Library Usage

```rust
use yaml_lint_core::Linter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let linter = Linter::with_defaults();

    for problem in linter.lint_string("key: value\n")? {
        println!("{}:{} - {}", problem.line, problem.column, problem.message);
    }

    Ok(())
}
```
