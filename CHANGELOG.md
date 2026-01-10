# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-01-09

### Added
- Initial release of yaml-lint-rs
- Core linting engine with modular rule system
- CLI tool with multiple output formats
- 6 core linting rules:
  - **trailing-spaces**: Detects whitespace at line endings
  - **line-length**: Enforces maximum line length (default: 80)
  - **document-start**: Requires or forbids `---` at document start
  - **colons**: Validates spacing around colons in mappings
  - **key-duplicates**: Prevents duplicate keys in mappings
  - **indentation**: Validates consistent indentation
- Configuration system with `.yamllint` file support
- Two preset configurations: default (strict) and relaxed
- Three output formats: standard, colored, parsable
- Exit codes for CI/CD integration (0: success, 1: errors, 2: warnings with --strict)
- Comprehensive test suite with unit and integration tests
- Directory traversal for batch linting
- Rule-level configuration and enable/disable

### Architecture
- Cargo workspace structure with `core` library and `cli` binary
- Extensible rule system using trait-based design
- Support for custom rules via library API

### Documentation
- Complete README with usage examples
- Inline code documentation
- Example configuration file
- Integration test fixtures

## [Unreleased]

### Planned for 0.2.0
- Directive support (`# yamllint disable-line`, `# yamllint disable`)
- GitHub Actions output format
- Config file inheritance with `extends`
- Additional rules: truthy, hyphens, comments, new-line-at-end-of-file
- Performance improvements

### Planned for 0.3.0
- Remaining 17 rules for full yamllint parity
- Parallel file linting
- Watch mode for development
- Editor integration (LSP server)

[0.1.0]: https://github.com/hiromaily/yaml-lint-rs/releases/tag/v0.1.0
