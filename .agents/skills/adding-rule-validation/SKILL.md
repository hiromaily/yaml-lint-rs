---
name: adding-rule-validation
description: Add or change YAML lint rules in yaml-lint-rs, including registration, presets, configuration, rule tests, and documentation.
---

## Adding or changing a lint rule

1. Implement `Rule` in [core/src/rules/](../../../core/src/rules/) with a snake_case module and a kebab-case rule name. Return `Vec<LintProblem>` from `check`. Use nearby rules as examples for diagnostic positions and YAML edge cases.
2. Declare the module and register the rule in `RuleRegistry::with_defaults()`. Wire up construction in `Config::create_registry()` too, since configurations with rule entries use that path. Set the intended enablement and severity in the default and relaxed presets.
3. For configurable rules, wire the options through `RuleOptions`, parsing, and construction. Check that explicit options and omitted options both reach the expected behavior.
4. Cover valid input, violations, and relevant edge cases in unit tests. Add a fixture and a test in [integration_tests.rs](../../../core/tests/integration_tests.rs) for a new rule. Assert the specific rule and its expected diagnostics, since other rules can report problems on the same fixture. Use [config_options.rs](../../../core/tests/config_options.rs) for configuration behavior and exercise the fixer if the rule supports auto-fix.
5. Update [docs/RULES.md](../../../docs/RULES.md) and the [README rule list](../../../README.md#implemented-rules) for rule additions. Update [docs/USAGE.md](../../../docs/USAGE.md) when CLI or configuration usage changes.
6. Run `make ci` and `make validate-fixtures` for rule changes.

For YAML-sensitive checks, exercise relevant cases such as quoted text, comments, block scalars, and flow collections. Match existing severity and diagnostic position conventions.
