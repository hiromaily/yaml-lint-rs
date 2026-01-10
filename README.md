# yaml-lint-rs

A fast YAML linter written in Rust, inspired by [yamllint](https://github.com/adrienverge/yamllint).

Built with [Claude Code](https://www.anthropic.com/claude-code) and [Cursor](https://cursor.sh/).

## Why yaml-lint-rs?

- ⚡ **20-60x faster** than Python yamllint (see [benchmarks](#performance))
- 📦 **Single binary** - no runtime dependencies required
- 🔧 **Drop-in replacement** - compatible with yamllint config format
- 🦀 **Memory safe** - written in Rust

## Features

- ✅ Fast and efficient YAML linting
- ✅ Multiple output formats (standard, colored, parsable)
- ✅ Configurable rules with preset configurations
- ✅ Support for `.yamllint` configuration files
- ✅ Directory traversal for batch linting
- ✅ Exit codes for CI/CD integration

## Performance

yaml-lint-rs is significantly faster than Python yamllint:

| File Size | Lines  | yaml-lint-rs | yamllint (Python) | Speedup |
|-----------|--------|--------------|-------------------|---------|
| Small     | 112    | 2.1 ms       | 61 ms             | **29x** |
| Medium    | 1,100  | 4.8 ms       | 104 ms            | **22x** |
| Large     | 11,000 | 12 ms        | 532 ms            | **43x** |
| X-Large   | 55,000 | 42 ms        | 2,535 ms          | **60x** |

*Benchmarks run on Apple M1, macOS. Results may vary. See `benchmarks/` for reproduction.*

## Installation

### From crates.io (Recommended)

```bash
cargo install yaml-lint
```

### Homebrew (macOS/Linux)

```bash
brew tap hiromaily/tap
brew install yaml-lint
```

### From Source

Requires Rust 1.85+ (install from [rustup.rs](https://rustup.rs/)):

```bash
git clone https://github.com/hiromaily/yaml-lint-rs.git
cd yaml-lint-rs
cargo build --release
# The binary will be at ./target/release/yaml-lint
```

## Usage

### Basic usage

```bash
# Lint a single file
yaml-lint file.yaml

# Lint multiple files
yaml-lint file1.yaml file2.yaml

# Lint all YAML files in a directory
yaml-lint src/

# Lint with specific config file
yaml-lint -c .yamllint file.yaml
```

### Output formats

```bash
# Standard human-readable output (default in non-TTY)
yaml-lint file.yaml

# Colored output (default in TTY)
yaml-lint -f colored file.yaml

# Machine-parsable output
yaml-lint -f parsable file.yaml
```

### Color control

```bash
# Auto-detect based on terminal (default)
yaml-lint --color auto file.yaml

# Always use colors
yaml-lint --color always file.yaml

# Never use colors
yaml-lint --color never file.yaml
```

Colors are automatically enabled when output is to a terminal. The `NO_COLOR` environment variable is respected (see https://no-color.org/).

### Presets

```bash
# Use default preset (strict)
yaml-lint -d default file.yaml

# Use relaxed preset
yaml-lint -d relaxed file.yaml
```

### Options

```bash
# Treat warnings as errors
yaml-lint --strict file.yaml

# List files that would be linted
yaml-lint --list-files src/
```

## Configuration

Create a `.yamllint` or `.yamllint.yml` file in your project root:

```yaml
# Extend a preset
extends: default

# Configure individual rules
rules:
  trailing-spaces: error
  line-length:
    max: 120
  document-start: disable
  indentation:
    spaces: 2

# Ignore patterns
ignore: |
  /vendor/
  *.generated.yaml
```

### Available Presets

#### default (strict)
- All rules enabled as errors
- Suitable for production code

#### relaxed
- Most rules as warnings
- More permissive for development

## Implemented Rules

### ✅ Phase 1 (Complete)

1. **trailing-spaces** - Detects whitespace at line endings
   - Level: Error
   - No configuration

2. **line-length** - Enforces maximum line length
   - Level: Error
   - Options: `max` (default: 80)

3. **document-start** - Requires or forbids `---` at document start
   - Level: Disable (by default)
   - Options: `present` (true/false)

4. **colons** - Validates spacing around colons in mappings
   - Level: Error
   - Options: `max-spaces-before` (0), `max-spaces-after` (1)

5. **key-duplicates** - Prevents duplicate keys in mappings
   - Level: Error
   - Critical for YAML correctness

6. **indentation** - Validates consistent indentation
   - Level: Error
   - Options: `spaces` (2/4/consistent), `indent-sequences`

7. **new-line-at-end-of-file** - Requires newline at end of file
   - Level: Error
   - POSIX standard compliance

8. **empty-lines** - Limits consecutive blank lines
   - Level: Error
   - Options: `max` (default: 2), `max-start` (0), `max-end` (0)

### 📋 Future Rules

Additional 15 rules planned for full yamllint compatibility:
- truthy, hyphens, comments, new-lines
- braces, brackets, commas, empty-values
- comments-indentation, document-end
- float-values, octal-values, quoted-strings, key-ordering

## Exit Codes

- **0**: Success (no errors, or only warnings without `--strict`)
- **1**: Errors detected
- **2**: Warnings detected with `--strict` flag

## Examples

### Example 1: Trailing spaces

```yaml
# Bad
key: value

# Good
key: value
```

### Example 2: Line length

```yaml
# Bad (> 80 characters)
key: this is a very long value that exceeds the maximum line length of eighty characters

# Good
key: shorter value
# or use multiline
key: |
  this is a very long value
  split across multiple lines
```

## Roadmap

- [x] Phase 1: Core infrastructure + 6 priority rules
- [ ] Phase 2: Additional output formats, directives, more rules
- [ ] Phase 3: Full yamllint parity (23 rules)
- [ ] Phase 4: Performance optimizations, parallel linting
- [ ] Phase 5: Editor integration (LSP?)

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

MIT

## Acknowledgments

Inspired by [yamllint](https://github.com/adrienverge/yamllint) by Adrien Vergé.
