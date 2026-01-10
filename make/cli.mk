# ==============================================================================
# CLI Usage Examples
# ==============================================================================

# Run the CLI with sample file
.PHONY: run
run:
	cargo run --package yaml-lint -- examples/sample.yaml

# Run the CLI with arguments
# Usage: make run-args ARGS="--help"
.PHONY: run-args
run-args:
	cargo run --package yaml-lint -- $(ARGS)

# Run with colored output
.PHONY: run-colored
run-colored:
	cargo run --package yaml-lint -- -f colored examples/sample.yaml

# ==============================================================================
# YAML Linting Commands
# ==============================================================================

# Lint a single YAML file
# Usage: make yaml-lint FILE=path/to/file.yaml
.PHONY: yaml-lint
yaml-lint:
	cargo run --package yaml-lint -- $(FILE)

# Lint a directory recursively
# Usage: make yaml-lint-dir DIR=path/to/dir
.PHONY: yaml-lint-dir
yaml-lint-dir:
	cargo run --package yaml-lint -- $(DIR)

# Lint examples directory
.PHONY: yaml-lint-examples
yaml-lint-examples:
	cargo run --package yaml-lint -- examples/

# Lint with strict mode (warnings cause non-zero exit)
.PHONY: yaml-lint-strict
yaml-lint-strict:
	cargo run --package yaml-lint -- --strict examples/

# Lint with custom config file
# Usage: make yaml-lint-config CONFIG=.yamllint FILE=examples/
.PHONY: yaml-lint-config
yaml-lint-config:
	cargo run --package yaml-lint -- -c $(CONFIG) $(FILE)

# Lint with relaxed preset
.PHONY: yaml-lint-relaxed
yaml-lint-relaxed:
	cargo run --package yaml-lint -- -d relaxed examples/

# Lint with parsable output (for CI/editors)
.PHONY: yaml-lint-parsable
yaml-lint-parsable:
	cargo run --package yaml-lint -- -f parsable examples/

# Lint with colored output
.PHONY: yaml-lint-colored
yaml-lint-colored:
	cargo run --package yaml-lint -- -f colored examples/

# List files that would be linted
.PHONY: yaml-lint-list
yaml-lint-list:
	cargo run --package yaml-lint -- --list-files examples/

# Lint test fixtures (valid files - should pass)
.PHONY: yaml-lint-valid
yaml-lint-valid:
	cargo run --package yaml-lint -- tests/fixtures/valid/

# Lint test fixtures (invalid files - expected errors)
.PHONY: yaml-lint-invalid
yaml-lint-invalid:
	cargo run --package yaml-lint -- tests/fixtures/invalid/ || true

# Show CLI help
.PHONY: yaml-lint-help
yaml-lint-help:
	cargo run --package yaml-lint -- --help
