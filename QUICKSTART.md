# Quick Start Guide

Get started with yaml-lint-rs in 5 minutes.

## 1. Install Rust (if not already installed)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

## 2. Build yaml-lint-rs

```bash
cd yaml-lint-rs

# Build the project
cargo build --release

# The binary is at: ./target/release/yaml-lint
```

## 3. Run Your First Lint

```bash
# Create a test YAML file with some issues
cat > test.yaml << 'EOF'
key: value
another : value
verylongkey: this is a very long line that exceeds the default maximum line length of eighty characters
key: duplicate
EOF

# Run the linter
./target/release/yaml-lint test.yaml
```

You should see output like:

```
test.yaml
  2:8      error    too many spaces before colon (1 > 0)  (colons)
  3:1      error    line too long (108 > 80 characters)  (line-length)
  4:1      error    found duplicate key "key"  (key-duplicates)

Found 3 problem(s) in 1 file(s)
```

## 4. Create a Configuration File

```bash
# Copy the example config
cp .yamllint.example .yamllint

# Or create your own
cat > .yamllint << 'EOF'
extends: default

rules:
  line-length:
    max: 120
  document-start: error
EOF
```

## 5. Try Different Output Formats

```bash
# Standard output (default)
./target/release/yaml-lint test.yaml

# Colored output (errors in red, warnings in yellow)
./target/release/yaml-lint -f colored test.yaml

# Parsable output (for editors and CI)
./target/release/yaml-lint -f parsable test.yaml
# Output: test.yaml:2:8: [error] too many spaces before colon...
```

## 6. Lint a Directory

```bash
# Lint all YAML files in current directory
./target/release/yaml-lint .

# Lint a specific directory
./target/release/yaml-lint src/
```

## 7. Use in CI/CD

```bash
# Fail on errors
./target/release/yaml-lint src/
exit_code=$?
if [ $exit_code -ne 0 ]; then
    echo "Linting failed!"
    exit 1
fi

# Fail on errors AND warnings
./target/release/yaml-lint --strict src/
```

## 8. Install Globally (Optional)

```bash
cargo install --path cli

# Now you can use it from anywhere
yaml-lint ~/projects/my-app/config.yaml
```

## Common Use Cases

### Lint with Relaxed Rules

```bash
# Use relaxed preset (warnings instead of errors)
./target/release/yaml-lint -d relaxed file.yaml
```

### List Files Without Linting

```bash
# See what files would be linted
./target/release/yaml-lint --list-files src/
```

### Disable Specific Rules

Create `.yamllint`:

```yaml
extends: default

rules:
  line-length: disable
  document-start: disable
```

### Ignore Files

```yaml
extends: default

ignore: |
  /vendor/
  /node_modules/
  *.generated.yaml
```

## Understanding Output

### Standard Format

```
file.yaml
  12:3      error    trailing spaces  (trailing-spaces)
  ^^:^      ^^^^^    ^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^
  │ │       │        │                └─ Rule name
  │ │       │        └─ Problem description
  │ │       └─ Level (error or warning)
  │ └─ Column number
  └─ Line number
```

### Exit Codes

- **0** = Success (no errors)
- **1** = Errors found
- **2** = Warnings found (only with --strict)

## Next Steps

- Read [USAGE.md](docs/USAGE.md) for detailed CLI options
- Read [RULES.md](docs/RULES.md) for rule descriptions
- Check [examples/sample.yaml](examples/sample.yaml) for YAML examples
- See [PROJECT_SUMMARY.md](PROJECT_SUMMARY.md) for architecture details

## Need Help?

```bash
# Show help
./target/release/yaml-lint --help

# Show version
./target/release/yaml-lint --version
```

## Troubleshooting

### "cargo: command not found"

Install Rust: https://rustup.rs/

### "No YAML files found"

Make sure files have `.yaml` or `.yml` extension.

### "Config not found"

Create `.yamllint` in your project root or use `-c` to specify config path.

---

Happy linting! 🎉
