# Benchmarks

This directory contains scripts to benchmark yaml-lint-rs against Python yamllint.

## Prerequisites

```bash
# Install hyperfine (benchmark tool)
brew install hyperfine

# Install Python yamllint
pipx install yamllint
# or: pip install yamllint
```

## Running Benchmarks

```bash
# Generate test files and run benchmarks
./run_benchmark.sh
```

Or manually:

```bash
# Generate test YAML files
python3 generate_test_files.py

# Build yaml-lint-rs in release mode
cd .. && cargo build --release && cd benchmarks

# Run individual benchmark
hyperfine --warmup 3 \
  "../target/release/yaml-lint files/large.yaml" \
  "yamllint files/large.yaml"
```

## Results (Apple M4, macOS)

| File Size | Lines  | yaml-lint-rs | yamllint (Python) | Speedup |
|-----------|--------|--------------|-------------------|---------|
| Small     | 112    | 2.1 ms       | 61 ms             | **29x** |
| Medium    | 1,100  | 4.8 ms       | 104 ms            | **22x** |
| Large     | 11,000 | 12 ms        | 532 ms            | **43x** |
| X-Large   | 55,000 | 42 ms        | 2,535 ms          | **60x** |

## Test Files

Generated test files contain nested YAML structures:

- `small.yaml` - 10 items (~112 lines)
- `medium.yaml` - 100 items (~1,100 lines)
- `large.yaml` - 1,000 items (~11,000 lines)
- `xlarge.yaml` - 5,000 items (~55,000 lines)

## Notes

- Benchmarks measure cold-start performance (including process startup)
- Python yamllint has significant startup overhead due to Python interpreter
- Results may vary based on system configuration and load
- Run benchmarks on a quiet system for accurate results
