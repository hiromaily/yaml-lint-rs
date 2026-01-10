# ==============================================================================
# Release targets
# ==============================================================================

# Project metadata
PROJECT_NAME := yaml-lint
VERSION := $(shell grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
RELEASE_DIR := target/release
DIST_DIR := dist

# Supported targets for cross-compilation
TARGETS := x86_64-apple-darwin aarch64-apple-darwin x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu

# ==============================================================================
# Version Management
# ==============================================================================

# Show current version
.PHONY: version
version:
	@echo "Current version: $(VERSION)"

# Bump version (requires cargo-edit)
# Usage: make bump-patch / make bump-minor / make bump-major
.PHONY: bump-patch
bump-patch:
	cargo set-version --bump patch
	@echo "Version bumped to patch release"

.PHONY: bump-minor
bump-minor:
	cargo set-version --bump minor
	@echo "Version bumped to minor release"

.PHONY: bump-major
bump-major:
	cargo set-version --bump major
	@echo "Version bumped to major release"

# Set specific version
# Usage: make set-version V=0.2.0
.PHONY: set-version
set-version:
	@if [ -z "$(V)" ]; then echo "Usage: make set-version V=0.2.0"; exit 1; fi
	cargo set-version $(V)
	@echo "Version set to $(V)"

# ==============================================================================
# Release Build
# ==============================================================================

# Build optimized release binary
.PHONY: release-build
release-build:
	cargo build --release

# Build with maximum optimizations (LTO)
.PHONY: release-build-optimized
release-build-optimized:
	CARGO_PROFILE_RELEASE_LTO=true cargo build --release

# Build for specific target
# Usage: make release-target TARGET=x86_64-apple-darwin
.PHONY: release-target
release-target:
	@if [ -z "$(TARGET)" ]; then echo "Usage: make release-target TARGET=x86_64-apple-darwin"; exit 1; fi
	cargo build --release --target $(TARGET)

# Build for all supported targets (requires cross or appropriate toolchains)
.PHONY: release-all-targets
release-all-targets:
	@for target in $(TARGETS); do \
		echo "Building for $$target..."; \
		cargo build --release --target $$target || echo "Failed to build for $$target (may need cross or toolchain)"; \
	done

# ==============================================================================
# Distribution Packaging
# ==============================================================================

# Create dist directory
.PHONY: dist-dir
dist-dir:
	mkdir -p $(DIST_DIR)

# Package release binary for current platform
.PHONY: dist-package
dist-package: release-build dist-dir
	@ARCH=$$(uname -m); \
	OS=$$(uname -s | tr '[:upper:]' '[:lower:]'); \
	if [ "$$OS" = "darwin" ]; then \
		if [ "$$ARCH" = "arm64" ]; then TARGET="aarch64-apple-darwin"; \
		else TARGET="x86_64-apple-darwin"; fi; \
	else \
		if [ "$$ARCH" = "aarch64" ]; then TARGET="aarch64-unknown-linux-gnu"; \
		else TARGET="x86_64-unknown-linux-gnu"; fi; \
	fi; \
	ARCHIVE_NAME="$(PROJECT_NAME)-v$(VERSION)-$$TARGET.tar.gz"; \
	echo "Packaging $$ARCHIVE_NAME..."; \
	tar -czvf $(DIST_DIR)/$$ARCHIVE_NAME -C $(RELEASE_DIR) $(PROJECT_NAME)

# Package for specific target
# Usage: make dist-target TARGET=x86_64-apple-darwin
.PHONY: dist-target
dist-target: dist-dir
	@if [ -z "$(TARGET)" ]; then echo "Usage: make dist-target TARGET=x86_64-apple-darwin"; exit 1; fi
	@if [ ! -f "target/$(TARGET)/release/$(PROJECT_NAME)" ]; then \
		echo "Binary not found. Building for $(TARGET)..."; \
		cargo build --release --target $(TARGET); \
	fi
	ARCHIVE_NAME="$(PROJECT_NAME)-v$(VERSION)-$(TARGET).tar.gz"; \
	echo "Packaging $$ARCHIVE_NAME..."; \
	tar -czvf $(DIST_DIR)/$$ARCHIVE_NAME -C target/$(TARGET)/release $(PROJECT_NAME)

# Generate checksums for all archives in dist/
.PHONY: dist-checksums
dist-checksums:
	@cd $(DIST_DIR) && \
	rm -f checksums.txt && \
	for f in *.tar.gz *.zip 2>/dev/null; do \
		if [ -f "$$f" ]; then \
			shasum -a 256 "$$f" >> checksums.txt; \
		fi; \
	done && \
	echo "Checksums generated:" && \
	cat checksums.txt

# Clean dist directory
.PHONY: dist-clean
dist-clean:
	rm -rf $(DIST_DIR)

# ==============================================================================
# Git Tagging
# ==============================================================================

# Create git tag for current version
.PHONY: tag
tag:
	@if git rev-parse "v$(VERSION)" >/dev/null 2>&1; then \
		echo "Tag v$(VERSION) already exists"; \
		exit 1; \
	fi
	git tag -a "v$(VERSION)" -m "Release v$(VERSION)"
	@echo "Created tag v$(VERSION)"
	@echo "Run 'make tag-push' to push the tag to remote"

# Push tag to remote
.PHONY: tag-push
tag-push:
	git push origin "v$(VERSION)"
	@echo "Pushed tag v$(VERSION) to origin"

# Delete local tag
# Usage: make tag-delete V=0.1.0
.PHONY: tag-delete
tag-delete:
	@if [ -z "$(V)" ]; then echo "Usage: make tag-delete V=0.1.0"; exit 1; fi
	git tag -d "v$(V)"
	@echo "Deleted local tag v$(V)"

# Delete remote tag
# Usage: make tag-delete-remote V=0.1.0
.PHONY: tag-delete-remote
tag-delete-remote:
	@if [ -z "$(V)" ]; then echo "Usage: make tag-delete-remote V=0.1.0"; exit 1; fi
	git push origin --delete "v$(V)"
	@echo "Deleted remote tag v$(V)"

# List all tags
.PHONY: tags
tags:
	git tag -l --sort=-v:refname | head -20

# ==============================================================================
# Release Workflow
# ==============================================================================

# Full release preparation (dry run - no git operations)
.PHONY: release-prepare
release-prepare: fmt-check lint test release-build
	@echo ""
	@echo "=============================================="
	@echo "Release preparation complete for v$(VERSION)"
	@echo "=============================================="
	@echo ""
	@echo "Next steps:"
	@echo "  1. Update CHANGELOG.md"
	@echo "  2. Commit changes: git add -A && git commit -m 'Prepare release v$(VERSION)'"
	@echo "  3. Create tag: make tag"
	@echo "  4. Push changes: git push && make tag-push"
	@echo ""

# Create release (runs CI checks, builds, tags, and pushes)
.PHONY: release-create
release-create: release-prepare
	@echo "Creating release v$(VERSION)..."
	@read -p "Continue with release? [y/N] " confirm && \
	if [ "$$confirm" = "y" ] || [ "$$confirm" = "Y" ]; then \
		make tag && make tag-push; \
		echo "Release v$(VERSION) created and pushed!"; \
		echo "GitHub Actions will now build and publish the release."; \
	else \
		echo "Release cancelled."; \
	fi

# Show release checklist
.PHONY: release-checklist
release-checklist:
	@echo "=============================================="
	@echo "Release Checklist for v$(VERSION)"
	@echo "=============================================="
	@echo ""
	@echo "Pre-release:"
	@echo "  [ ] All tests pass (make test)"
	@echo "  [ ] Code is formatted (make fmt-check)"
	@echo "  [ ] No clippy warnings (make lint)"
	@echo "  [ ] CHANGELOG.md is updated"
	@echo "  [ ] Version is correct in Cargo.toml"
	@echo ""
	@echo "Release:"
	@echo "  [ ] Create git tag (make tag)"
	@echo "  [ ] Push tag to trigger CI (make tag-push)"
	@echo ""
	@echo "Post-release:"
	@echo "  [ ] Verify GitHub release is created"
	@echo "  [ ] Verify binaries are attached"
	@echo "  [ ] Verify Homebrew formula is updated"
	@echo "  [ ] Announce release (if applicable)"
	@echo ""

# ==============================================================================
# Install from Release
# ==============================================================================

# Install release binary to /usr/local/bin
.PHONY: release-install
release-install: release-build
	@echo "Installing $(PROJECT_NAME) to /usr/local/bin..."
	sudo cp $(RELEASE_DIR)/$(PROJECT_NAME) /usr/local/bin/
	@echo "Installed successfully!"
	@$(PROJECT_NAME) --version

# Uninstall from /usr/local/bin
.PHONY: release-uninstall
release-uninstall:
	@echo "Removing $(PROJECT_NAME) from /usr/local/bin..."
	sudo rm -f /usr/local/bin/$(PROJECT_NAME)
	@echo "Uninstalled successfully!"

# ==============================================================================
# Binary Info
# ==============================================================================

# Show binary size
.PHONY: release-size
release-size:
	@if [ -f "$(RELEASE_DIR)/$(PROJECT_NAME)" ]; then \
		ls -lh $(RELEASE_DIR)/$(PROJECT_NAME) | awk '{print "Binary size: " $$5}'; \
	else \
		echo "Release binary not found. Run 'make release-build' first."; \
	fi

# Show binary info (macOS)
.PHONY: release-info
release-info:
	@if [ -f "$(RELEASE_DIR)/$(PROJECT_NAME)" ]; then \
		echo "Binary: $(RELEASE_DIR)/$(PROJECT_NAME)"; \
		ls -lh $(RELEASE_DIR)/$(PROJECT_NAME); \
		echo ""; \
		file $(RELEASE_DIR)/$(PROJECT_NAME); \
	else \
		echo "Release binary not found. Run 'make release-build' first."; \
	fi

# Strip debug symbols (reduces binary size)
.PHONY: release-strip
release-strip:
	@if [ -f "$(RELEASE_DIR)/$(PROJECT_NAME)" ]; then \
		strip $(RELEASE_DIR)/$(PROJECT_NAME); \
		echo "Debug symbols stripped."; \
		make release-size; \
	else \
		echo "Release binary not found. Run 'make release-build' first."; \
	fi
