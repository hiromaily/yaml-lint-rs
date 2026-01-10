---
paths: "**/*.rs"
---

# Rust Conventions

## Style
- Use `rustfmt` defaults
- Follow clippy suggestions
- Prefer `impl Trait` over generics when simple

## Error Handling
- Use `thiserror` for custom errors
- Return `Result<T, E>` for fallible ops
- Use `?` operator for propagation

## Testing
- Unit tests in same file (`#[cfg(test)]`)
- Integration tests in `tests/`
- Use `#[test]` attribute

## Documentation
- Doc comments (`///`) for public items
- Examples in doc comments when useful

## Make Commands

```bash
make build       # Build debug version
make release     # Build release version
make test        # Run all tests
make lint        # Run clippy linter
make fmt         # Format code
make ci          # Run all quality checks (fmt-check, lint, test)
```
