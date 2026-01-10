# Usage Guide

Comprehensive guide for using yaml-lint-rs.

## Table of Contents

1. [Installation](#installation)
2. [Basic Usage](#basic-usage)
3. [Configuration](#configuration)
4. [CLI Options](#cli-options)
5. [Output Formats](#output-formats)
6. [Exit Codes](#exit-codes)
7. [Integration](#integration)
8. [Library Usage](#library-usage)

## Installation

### Homebrew (macOS/Linux)

The easiest way to install yaml-lint:

```bash
brew tap hiromaily/tap
brew install yaml-lint
```

### From Source

```bash
# Clone the repository (or navigate to project directory)
cd yaml-lint-rs

# Build and install
cargo install --path cli

# Or just build
cargo build --release
# Binary at: ./target/release/yaml-lint
```

### System Requirements

- Rust 1.85 or later (Edition 2024) - only for building from source
- Works on Linux, macOS, and Windows

## Basic Usage

### Lint a Single File

```bash
yaml-lint file.yaml
```

### Lint Multiple Files

```bash
yaml-lint file1.yaml file2.yaml file3.yaml
```

### Lint All YAML Files in a Directory

```bash
yaml-lint src/
```

This recursively finds all `.yaml` and `.yml` files.

### Lint with Custom Config

```bash
yaml-lint -c .yamllint file.yaml
```

## Configuration

### Config File Locations

yaml-lint-rs looks for configuration in these locations (in order):

1. File specified with `-c` flag
2. `.yamllint` in current directory
3. `.yamllint.yml` in current directory
4. `.yamllint.yaml` in current directory
5. Walk up parent directories looking for config
6. Default preset if no config found

### Config File Format

```yaml
# Extend a preset
extends: default  # or "relaxed"

# Configure rules
rules:
  trailing-spaces: error
  line-length:
    max: 120
  document-start: disable
  colons: error
  key-duplicates: error
  indentation:
    spaces: 2

# Ignore patterns (.gitignore style)
ignore: |
  /vendor/
  /node_modules/
  *.generated.yaml
```

### Presets

#### default

Strict preset suitable for production:

```yaml
extends: default
# All rules enabled as errors except document-start (disabled)
```

#### relaxed

More permissive preset:

```yaml
extends: relaxed
# Most rules as warnings, key-duplicates as error
```

### Rule Levels

- `error`: Causes exit code 1
- `warning`: Reported but doesn't fail (exit code 2 with `--strict`)
- `disable`: Rule is not checked

## CLI Options

### Basic Options

```bash
yaml-lint [OPTIONS] <FILES>...
```

### Options

| Option | Description |
|--------|-------------|
| `-c, --config <PATH>` | Path to config file |
| `-d, --preset <NAME>` | Use preset (default, relaxed) |
| `-f, --format <FORMAT>` | Output format (standard, colored, parsable) |
| `--strict` | Treat warnings as errors (exit code 2) |
| `--list-files` | List files that would be linted |
| `-h, --help` | Show help |
| `-V, --version` | Show version |

### Examples

```bash
# Use specific config
yaml-lint -c my-config.yml src/

# Use relaxed preset
yaml-lint -d relaxed file.yaml

# Colored output
yaml-lint -f colored file.yaml

# Strict mode (warnings cause exit code 2)
yaml-lint --strict file.yaml

# List files without linting
yaml-lint --list-files src/
```

## Output Formats

### standard (default)

Human-readable format with aligned columns:

```
test.yaml
  12:3      error    trailing spaces  (trailing-spaces)
  15:80     warning  line too long (82 > 80 characters)  (line-length)
```

### colored

Same as standard but with ANSI colors:
- Errors in red
- Warnings in yellow
- Dimmed rule names

```bash
yaml-lint -f colored file.yaml
```

### parsable

Machine-readable format for editors and CI tools:

```
test.yaml:12:3: [error] trailing spaces (trailing-spaces)
test.yaml:15:80: [warning] line too long (82 > 80 characters) (line-length)
```

```bash
yaml-lint -f parsable file.yaml
```

## Exit Codes

| Exit Code | Meaning |
|-----------|---------|
| 0 | Success (no errors, or only warnings without --strict) |
| 1 | Errors detected |
| 2 | Warnings detected with --strict flag |

### CI/CD Usage

```bash
# Fail on any errors
yaml-lint src/
if [ $? -ne 0 ]; then
    echo "Linting failed"
    exit 1
fi

# Fail on errors AND warnings
yaml-lint --strict src/
```

## Integration

### GitHub Actions

```yaml
name: Lint YAML

on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install yaml-lint
        run: cargo install --git https://github.com/hiromaily/yaml-lint-rs yaml-lint

      - name: Lint YAML files
        run: yaml-lint --strict .
```

### Pre-commit Hook

Create `.git/hooks/pre-commit`:

```bash
#!/bin/bash
# Lint YAML files before commit

echo "Running YAML linter..."

# Get all staged YAML files
YAML_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep -E '\.(yaml|yml)$')

if [ -n "$YAML_FILES" ]; then
    yaml-lint $YAML_FILES
    if [ $? -ne 0 ]; then
        echo "YAML linting failed. Fix errors before committing."
        exit 1
    fi
fi

exit 0
```

Make it executable:

```bash
chmod +x .git/hooks/pre-commit
```

### Makefile Integration

```makefile
.PHONY: lint
lint:
	yaml-lint src/

.PHONY: lint-strict
lint-strict:
	yaml-lint --strict src/

.PHONY: lint-fix
lint-fix:
	@echo "Auto-fix not yet supported. Please fix issues manually."
	yaml-lint src/
```

### VS Code Integration

Add to `.vscode/tasks.json`:

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Lint YAML",
      "type": "shell",
      "command": "yaml-lint",
      "args": ["${file}"],
      "problemMatcher": {
        "owner": "yaml-lint",
        "fileLocation": ["relative", "${workspaceFolder}"],
        "pattern": {
          "regexp": "^(.+):(\\d+):(\\d+):\\s+\\[(error|warning)\\]\\s+(.+)\\s+\\((.+)\\)$",
          "file": 1,
          "line": 2,
          "column": 3,
          "severity": 4,
          "message": 5,
          "code": 6
        }
      }
    }
  ]
}
```

## Library Usage

You can use yaml-lint-rs as a library in your Rust projects:

### Add Dependency

```toml
[dependencies]
yaml-lint-core = { path = "../yaml-lint-rs/core" }
# or when published:
# yaml-lint-core = "0.1"
```

### Basic Usage

```rust
use yaml_lint_core::{Config, Linter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create linter with default config
    let linter = Linter::with_defaults();

    // Lint a string
    let yaml = "key: value\n";
    let problems = linter.lint_string(yaml)?;

    for problem in problems {
        println!("{}:{} {} - {}",
            problem.line,
            problem.column,
            problem.level,
            problem.message
        );
    }

    Ok(())
}
```

### Custom Configuration

```rust
use yaml_lint_core::{Config, Linter, rules::RuleLevel};

let mut config = Config::with_default_preset();

// Customize rules
config.rules.insert("line-length".to_string(), RuleLevel::Warning);
config.rules.insert("document-start".to_string(), RuleLevel::Error);

let linter = Linter::new(config);
```

### Custom Rules

```rust
use yaml_lint_core::{
    Rule, LintContext, LintProblem, LintLevel, RuleLevel, RuleRegistry
};

struct MyRule;

impl Rule for MyRule {
    fn name(&self) -> &'static str {
        "my-rule"
    }

    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        // Your linting logic
        vec![]
    }

    fn default_level(&self) -> RuleLevel {
        RuleLevel::Error
    }
}

// Register and use
let mut registry = RuleRegistry::new();
registry.register(Box::new(MyRule));
```

## Troubleshooting

### No YAML Files Found

Make sure your files have `.yaml` or `.yml` extensions.

### Config Not Found

Use `--list-files` to debug file discovery:

```bash
yaml-lint --list-files src/
```

### Permission Errors

Ensure you have read permissions for the files and directories.

### Rust Not Installed

Install Rust from [rustup.rs](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Tips

1. **Start with relaxed preset**: If migrating existing YAML files, start with `extends: relaxed` and gradually tighten rules.

2. **Use --strict in CI**: Always use `--strict` in CI/CD to catch warnings.

3. **Ignore generated files**: Add generated or vendored YAML files to the `ignore` list.

4. **Consistent indentation**: Pick 2 or 4 spaces and stick with it across your project.

5. **Document start markers**: For multi-document YAML files, enable `document-start: error`.
