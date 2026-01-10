# ==============================================================================
# yaml-lint-rs Makefile
# ==============================================================================
#
# This Makefile includes modular makefiles from the make/ directory:
#   - make/dev.mk   : Development targets (build, lint, format, etc.)
#   - make/test.mk  : Testing targets (test, validate-fixtures, etc.)
#   - make/cli.mk   : CLI usage examples (yaml-lint commands)
#   - make/help.mk  : Help documentation
#
# Run 'make help' to see all available targets.
# ==============================================================================

# Default target
.PHONY: all
all: check build test

# Include modular makefiles
include make/dev.mk
include make/test.mk
include make/cli.mk
include make/release.mk
include make/help.mk
