# ==============================================================================
# Test targets
# ==============================================================================

# Run all tests (same as CI)
.PHONY: test
test:
	cargo test --all

# Run tests with output
.PHONY: test-verbose
test-verbose:
	cargo test --all -- --nocapture

# Run specific test
# Usage: make test-one TEST=test_name
.PHONY: test-one
test-one:
	@echo "Usage: make test-one TEST=test_name"
	cargo test $(TEST) -- --nocapture

# ==============================================================================
# Fixture Validation
# ==============================================================================

# Validate all fixtures - valid files should pass, invalid should fail
.PHONY: validate-fixtures
validate-fixtures: release
	@echo "=== Validating test fixtures ==="
	@echo ""
	@echo "--- Valid fixtures (should have 0 errors) ---"
	@./target/release/yaml-lint tests/fixtures/valid/ && echo "✓ All valid fixtures passed" || (echo "✗ Valid fixtures had unexpected errors" && exit 1)
	@echo ""
	@echo "--- Invalid fixtures (should have errors) ---"
	@./target/release/yaml-lint tests/fixtures/invalid/ 2>&1; \
	if [ $$? -eq 0 ]; then \
		echo "✗ Invalid fixtures should have errors but passed"; \
		exit 1; \
	else \
		echo "✓ Invalid fixtures correctly detected errors"; \
	fi
	@echo ""
	@echo "=== All fixture validations passed ==="

# Show detailed results for each invalid fixture
.PHONY: validate-fixtures-detail
validate-fixtures-detail: release
	@echo "=== Detailed fixture validation ==="
	@echo ""
	@echo "--- Valid fixtures ---"
	@for f in tests/fixtures/valid/*.yaml; do \
		printf "  %-40s " "$$f:"; \
		if ./target/release/yaml-lint "$$f" > /dev/null 2>&1; then \
			echo "✓ PASS"; \
		else \
			echo "✗ FAIL (unexpected)"; \
		fi; \
	done
	@echo ""
	@echo "--- Invalid fixtures ---"
	@for f in tests/fixtures/invalid/*.yaml; do \
		printf "  %-45s " "$$f:"; \
		output=$$(./target/release/yaml-lint "$$f" 2>&1); \
		if [ $$? -ne 0 ]; then \
			count=$$(echo "$$output" | grep -c "error\|warning" || echo "0"); \
			echo "✓ DETECTED ($$count issues)"; \
		else \
			echo "✗ MISSED (no errors detected)"; \
		fi; \
	done
	@echo ""

# ==============================================================================
# Individual Rule Validation
# ==============================================================================

.PHONY: validate-rule-trailing-spaces
validate-rule-trailing-spaces: release
	@echo "Testing trailing-spaces rule:"
	@./target/release/yaml-lint tests/fixtures/invalid/trailing-spaces.yaml || true

.PHONY: validate-rule-line-length
validate-rule-line-length: release
	@echo "Testing line-length rule:"
	@./target/release/yaml-lint tests/fixtures/invalid/long-lines.yaml || true

.PHONY: validate-rule-colons
validate-rule-colons: release
	@echo "Testing colons rule:"
	@./target/release/yaml-lint tests/fixtures/invalid/bad-colons.yaml || true

.PHONY: validate-rule-duplicates
validate-rule-duplicates: release
	@echo "Testing key-duplicates rule:"
	@./target/release/yaml-lint tests/fixtures/invalid/duplicate-keys.yaml || true

.PHONY: validate-rule-indentation
validate-rule-indentation: release
	@echo "Testing indentation rule:"
	@./target/release/yaml-lint tests/fixtures/invalid/bad-indentation.yaml || true
	@echo ""
	@echo "Testing tabs-indentation:"
	@./target/release/yaml-lint tests/fixtures/invalid/tabs-indentation.yaml || true

# Run all rule validations
.PHONY: validate-all-rules
validate-all-rules: validate-rule-trailing-spaces validate-rule-line-length validate-rule-colons validate-rule-duplicates validate-rule-indentation
	@echo ""
	@echo "=== All rule validations complete ==="
