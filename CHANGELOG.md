# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned

- File-level `# yamllint disable` directives.
- GitHub Actions output format.
- Remaining rules for full yamllint parity.
- Watch mode for development.
- Editor integration (LSP server).

## [0.3.0] - 2026-11-26

### Added

- JSON diagnostics via `--format json`.
- Inline and standalone `# yamllint disable-line` directives for individual rules.
- `braces` and `brackets` rules for spacing in flow collections.
- `new-lines` rule for LF, CRLF, or platform-native line endings.
- Parallel file linting with `-j` / `--jobs`.

### Changed

- Raised the minimum supported Rust version to 1.100.
- Both presets now expect LF by default. The default preset reports mismatches as errors, and the relaxed preset reports them as warnings.

### Fixed

- Auto-fix preserves existing LF and CRLF line endings on retained lines and uses the configured style when adding a final line ending.

## [0.2.1] - 2026-01-15

### Fixed

- Apply rule-specific options from `.yamllint` configuration files and validate their values.

## [0.2.0] - 2026-01-10

### Added
- **truthy**: New rule for YAML 1.1 vs 1.2 boolean disambiguation (detects ambiguous truthy values like `yes`, `no`, `on`, `off`)
- **comments**: New rule for validating comment formatting (spacing after `#`, minimum spaces before inline comments)

### Changed
- Improved YAML string escape handling in comments rule
- Improved truthy rule performance and reduced code duplication

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

[0.3.0]: https://github.com/hiromaily/yaml-lint-rs/releases/tag/v0.3.0
[0.2.1]: https://github.com/hiromaily/yaml-lint-rs/releases/tag/v0.2.1
[0.2.0]: https://github.com/hiromaily/yaml-lint-rs/releases/tag/v0.2.0
[0.1.0]: https://github.com/hiromaily/yaml-lint-rs/releases/tag/v0.1.0
