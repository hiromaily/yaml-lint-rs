#!/bin/bash
# Benchmark yaml-lint-rs vs Python yamllint

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "=== YAML Linter Benchmark ==="
echo ""

# Check dependencies
if ! command -v hyperfine &> /dev/null; then
    echo "Error: hyperfine is not installed. Install with: brew install hyperfine"
    exit 1
fi

if ! command -v yamllint &> /dev/null; then
    echo "Error: yamllint (Python) is not installed. Install with: pip install yamllint"
    exit 1
fi

# Build yaml-lint-rs in release mode
echo "Building yaml-lint-rs in release mode..."
cd "$PROJECT_ROOT"
cargo build --release --quiet
YAML_LINT_RS="$PROJECT_ROOT/target/release/yaml-lint"

if [ ! -f "$YAML_LINT_RS" ]; then
    echo "Error: yaml-lint binary not found at $YAML_LINT_RS"
    exit 1
fi

# Generate test files
echo "Generating test files..."
cd "$SCRIPT_DIR"
python3 generate_test_files.py
echo ""

# Run benchmarks
echo "=== Benchmark Results ==="
echo ""

for size in small medium large xlarge; do
    file="files/${size}.yaml"
    if [ -f "$file" ]; then
        lines=$(wc -l < "$file" | tr -d ' ')
        echo "--- ${size}.yaml (${lines} lines) ---"
        echo ""
        
        hyperfine \
            --warmup 3 \
            --min-runs 10 \
            --export-markdown "results_${size}.md" \
            --command-name "yaml-lint-rs" "$YAML_LINT_RS $file" \
            --command-name "yamllint (Python)" "yamllint $file"
        
        echo ""
    fi
done

# Summary
echo "=== Summary ==="
echo ""
echo "Detailed results saved to:"
for size in small medium large xlarge; do
    if [ -f "results_${size}.md" ]; then
        echo "  - results_${size}.md"
    fi
done
