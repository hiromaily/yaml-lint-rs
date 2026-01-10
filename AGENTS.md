# Agent Instructions

This document provides guidance for AI coding assistants working on yaml-lint-rs.

## Project Overview

- **Purpose**: Fast YAML linter written in Rust, inspired by Python yamllint
- **Language**: Rust (edition 2024)
- **Structure**: Cargo workspace with `core` (library) and `cli` (binary) crates

## Architecture

```
yaml-lint-rs/
├── core/                 # Library crate
│   └── src/
│       ├── lib.rs        # Public API
│       ├── config.rs     # Configuration system
│       ├── linter.rs     # Main orchestration
│       ├── problem.rs    # LintProblem type
│       ├── rules/        # Rule implementations
│       └── output/       # Output formatters
├── cli/                  # Binary crate
│   └── src/main.rs       # CLI entry point
└── tests/                # Integration tests
```

## Key Commands (Makefile)

### CI Commands (run before commit)
```bash
make ci          # Run all CI checks (fmt-check + lint + test)
make fmt-check   # Check formatting (same as CI)
make lint        # Run clippy (same as CI)
make test        # Run all tests (same as CI)
```

### Development
```bash
make build       # Build debug version
make release     # Build release version
make fmt         # Format code
make clean       # Clean build artifacts
make help        # Show all available targets
```

### Fixture Validation
```bash
make validate-fixtures        # Validate test fixtures
make validate-fixtures-detail # Detailed validation
```

## Adding New Rules

1. Create `core/src/rules/<rule_name>.rs`
2. Implement `Rule` trait (see existing rules for examples)
3. Register in `core/src/rules/mod.rs`
4. Add to presets in `core/src/config.rs`
5. Add tests in the rule file and `tests/`

## Code Style

- Follow Rust idioms and clippy suggestions
- Use `thiserror` for error types
- Prefer `Vec<LintProblem>` return type for rule checks
- Keep functions small and focused
- Add doc comments for public APIs

## Documentation

- `docs/RULES.md` - Rule documentation
- `docs/USAGE.md` - Usage guide
- `README.md` - Project overview
