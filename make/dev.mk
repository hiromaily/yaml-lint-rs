# ==============================================================================
# Development targets
# ==============================================================================

# Build debug version
.PHONY: build
build:
	cargo build

# Build release version
.PHONY: release
release:
	cargo build --release

# Check code without building
.PHONY: check
check:
	cargo check

# Format code
.PHONY: fmt
fmt:
	cargo fmt --all

# Check formatting without changes (same as CI)
.PHONY: fmt-check
fmt-check:
	cargo fmt --all -- --check

# Run clippy linter (same as CI)
.PHONY: lint
lint:
	cargo clippy --all-targets --all-features -- -D warnings

# Clean build artifacts
.PHONY: clean
clean:
	cargo clean

# Generate documentation
.PHONY: doc
doc:
	cargo doc --no-deps --open

# Generate documentation without opening
.PHONY: doc-build
doc-build:
	cargo doc --no-deps

# Install the binary locally
.PHONY: install
install:
	cargo install --path cli

# Uninstall the binary
.PHONY: uninstall
uninstall:
	cargo uninstall yaml-lint

# Update dependencies
.PHONY: update
update:
	cargo update

# Check for outdated dependencies (requires cargo-outdated)
.PHONY: outdated
outdated:
	cargo outdated

# Security audit (requires cargo-audit)
.PHONY: audit
audit:
	cargo audit

# Show dependency tree
.PHONY: deps
deps:
	cargo tree

# Run all quality checks
.PHONY: ci
ci: fmt-check lint test
	@echo "All CI checks passed!"

# Watch for changes and run tests (requires cargo-watch)
.PHONY: watch
watch:
	cargo watch -x test

# Watch for changes and run check (requires cargo-watch)
.PHONY: watch-check
watch-check:
	cargo watch -x check
