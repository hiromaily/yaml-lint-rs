# Project Rules

## Architecture
- `core/` = library crate (reusable)
- `cli/` = binary crate (thin wrapper)
- Rules go in `core/src/rules/`

## Adding Rules
1. Create `core/src/rules/<name>.rs`
2. Implement `Rule` trait
3. Register in `mod.rs`
4. Add to config presets

## Commands
- `cargo test --all` - run tests
- `cargo clippy --all` - lint
- `make help` - show targets

## Dependencies
- `yaml-rust2` for YAML parsing
- `serde` for config deserialization
- `thiserror` for errors
