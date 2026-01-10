---
name: adding-rule-validation
description: Guide for adding new YAML linting rules to yaml-lint-rs. Use when implementing a new validation rule, creating a linting feature, or when the user mentions adding rules like "truthy", "comments", "braces", etc.
---

# Adding New Validation Rules

This skill provides step-by-step guidance for implementing new YAML linting rules in the yaml-lint-rs project.

## Quick Checklist

When adding a new rule, complete these steps in order:

1. [ ] Create rule file: `core/src/rules/<rule_name>.rs`
2. [ ] Register in `core/src/rules/mod.rs`
3. [ ] Add to presets in `core/src/config.rs`
4. [ ] Add unit tests (minimum 10+ test cases)
5. [ ] Create test fixture: `tests/fixtures/invalid/<rule-name>.yaml`
6. [ ] Add integration test in `core/tests/integration_tests.rs`
7. [ ] Update `docs/RULES.md`
8. [ ] Update `README.md` rule list
9. [ ] Run `make ci` and `make validate-fixtures` to verify
10. [ ] Create PR

## Step 1: Create Rule File

Create `core/src/rules/<rule_name>.rs` with this template:

```rust
//! <Rule Name> rule - <brief description>

use crate::problem::{LintLevel, LintProblem};
use crate::rules::{LintContext, Rule, RuleLevel};

/// Rule that <description>
#[derive(Debug)]
pub struct <RuleName>Rule {
    // Configuration fields (if any)
}

impl <RuleName>Rule {
    /// Create a new rule with default settings
    pub fn new() -> Self {
        Self {
            // Default configuration
        }
    }

    /// Create a new rule with custom settings (if configurable)
    pub fn with_config(/* params */) -> Self {
        Self {
            // Custom configuration
        }
    }
}

impl Default for <RuleName>Rule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for <RuleName>Rule {
    fn name(&self) -> &'static str {
        "<rule-name>"  // kebab-case
    }

    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        let mut problems = Vec::new();

        for (line_idx, line) in context.lines.iter().enumerate() {
            let line_num = line_idx + 1;
            
            // Rule logic here
            // if problem_found {
            //     problems.push(LintProblem::new(
            //         line_num,
            //         column,
            //         "error message",
            //         self.name(),
            //         LintLevel::Error,
            //     ));
            // }
        }

        problems
    }

    fn default_level(&self) -> RuleLevel {
        RuleLevel::Error  // or Warning, Disable
    }

    // Optional: implement if rule supports auto-fixing
    fn is_fixable(&self) -> bool {
        false
    }

    fn fix(&self, _content: &str, _problem: &LintProblem) -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_case() {
        let yaml = "valid: content\n";
        let context = LintContext::new(yaml.to_string());
        let rule = <RuleName>Rule::new();
        let problems = rule.check(&context);
        assert!(problems.is_empty());
    }

    #[test]
    fn test_invalid_case() {
        let yaml = "invalid content\n";
        let context = LintContext::new(yaml.to_string());
        let rule = <RuleName>Rule::new();
        let problems = rule.check(&context);
        assert_eq!(problems.len(), 1);
    }

    // Add more tests: edge cases, configuration options, etc.
}
```

## Step 2: Register in mod.rs

Edit `core/src/rules/mod.rs`:

```rust
// Add module declaration (alphabetical order)
pub mod <rule_name>;

// In RuleRegistry::with_defaults(), add:
registry.register(Box::new(<rule_name>::<RuleName>Rule::new()));
```

## Step 3: Add to Presets in config.rs

Edit `core/src/config.rs`:

```rust
// In with_default_preset():
config.rules.insert("<rule-name>".to_string(), RuleLevel::Error);
// or RuleLevel::Warning for less critical rules

// In with_relaxed_preset():
config.rules.insert("<rule-name>".to_string(), RuleLevel::Warning);
```

## Step 4: Add Unit Tests

Minimum test coverage should include:

- Valid cases (no problems detected)
- Invalid cases (problems correctly detected)
- Edge cases (empty files, comments, quoted strings)
- Configuration options (if any)
- Auto-fix functionality (if fixable)

## Step 5: Create Test Fixture

Create an invalid YAML fixture file at `tests/fixtures/invalid/<rule-name>.yaml`:

```yaml
# File demonstrating <rule-name> violations
# This file should trigger the rule multiple times

# Example for truthy rule:
enabled: yes
disabled: no
flag: on

# Example for trailing-spaces rule:
key: value   
another: test  
```

**Fixture Guidelines:**
- Include a comment at the top explaining the violations
- Include multiple violations (3+ recommended)
- Cover different scenarios the rule checks
- Keep it focused on ONE rule per fixture file

**Directory Structure:**
```
tests/fixtures/
├── invalid/           # Files with lint errors
│   ├── <rule-name>.yaml
│   ├── trailing-spaces.yaml
│   └── ...
└── valid/             # Files that should pass all rules
    ├── simple.yaml
    └── ...
```

## Step 6: Add Integration Test

Edit `core/tests/integration_tests.rs` to add a test for the new rule:

```rust
#[test]
fn test_<rule_name>_detected() {
    let linter = Linter::with_defaults();
    let path = fixture_path("invalid/<rule-name>.yaml");
    let problems = linter.lint_file(&path).expect("Failed to lint file");

    assert!(!problems.is_empty(), "Expected to find <rule-name> violations");
    
    // Verify expected number of problems
    let rule_problems: Vec<_> = problems
        .iter()
        .filter(|p| p.rule == "<rule-name>")
        .collect();
    assert_eq!(
        rule_problems.len(),
        3,  // Adjust based on your fixture
        "Expected 3 <rule-name> problems"
    );

    // Verify rule name
    for problem in &rule_problems {
        assert_eq!(problem.rule, "<rule-name>");
    }
}
```

**Additional Integration Tests (if fixable):**

```rust
#[test]
fn test_fix_<rule_name>() {
    let config = Config::with_default_preset();
    let registry = config.create_registry();
    let fixer = Fixer::new(&registry);

    let content = "<content with violations>";
    let result = fixer.fix("test.yaml", content);

    assert!(result.has_fixes(), "Expected fixes to be applied");
    assert!(
        result.fixes_by_rule.contains_key("<rule-name>"),
        "Expected <rule-name> fixes"
    );
    
    // Verify fixed content
    let fixed = result.fixed_content.unwrap();
    assert!(!fixed.contains("<violation pattern>"));
}
```

**Helper Function (already exists):**

```rust
fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("tests")
        .join("fixtures")
        .join(name)
}
```

## Step 7: Update docs/RULES.md

Add rule documentation in appropriate section:

```markdown
### <rule-name>

**Level**: Error/Warning (default)
**Configurable**: Yes/No
**Fixable**: ✅ Yes / ❌ No

<Description of what the rule checks>

**Configuration** (if applicable):
```yaml
rules:
  <rule-name>:
    option1: value1
    option2: value2
```

**Why it matters**: <Motivation>

**Examples**:

```yaml
# Bad
<invalid example>

# Good
<valid example>
```
```

Also update:
- Preset documentation (if rule is in default/relaxed)
- Remove from "Future Rules" section if listed there

## Step 8: Update README.md

Add to the "Implemented Rules" section:

```markdown
N. **<rule-name>** - <Brief description>
   - Level: Error/Warning
   - Options: `option1`, `option2` (if configurable)
```

Update "Future Rules" section if needed.

## Step 9: Run CI

```bash
make ci                    # fmt-check + clippy + test
make validate-fixtures     # Validate test fixtures
```

Fix any issues:
- `make fmt` for formatting
- Address clippy warnings
- Fix failing tests

**Fixture Validation:**

`make validate-fixtures` runs the linter on all fixtures and verifies:
- Valid fixtures produce 0 errors
- Invalid fixtures produce expected errors

## Step 10: Create PR

```bash
git checkout -b feature/<issue-number>-<rule-name>-rule
git add -A
git commit -m "feat: Add \`<rule-name>\` rule

<Description of what the rule does>

Closes #<issue-number>"
git push -u origin feature/<issue-number>-<rule-name>-rule
gh pr create --title "feat: Add \`<rule-name>\` rule" --body "..."
```

## Common Patterns

### Handling Quoted Strings

Use peekable iterator for proper YAML string parsing:

```rust
fn find_in_line(line: &str) -> Option<usize> {
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut chars = line.char_indices().peekable();

    while let Some((idx, ch)) = chars.next() {
        if !in_single_quote && !in_double_quote {
            match ch {
                '\'' => in_single_quote = true,
                '"' => in_double_quote = true,
                // Your check here
                _ => {}
            }
        } else if in_single_quote {
            if ch == '\'' {
                // Handle escaped single quote ''
                if chars.peek().is_some_and(|&(_, c)| c == '\'') {
                    chars.next();
                } else {
                    in_single_quote = false;
                }
            }
        } else {
            // in_double_quote
            if ch == '\\' {
                chars.next(); // Skip escaped char
            } else if ch == '"' {
                in_double_quote = false;
            }
        }
    }
    None
}
```

### MSRV Compatibility

For Rust 1.85 compatibility, avoid `if let ... && let ...` chains:

```rust
// Instead of (unstable in 1.85):
if let Some(x) = opt && x > 0 { ... }

// Use nested ifs with allow attribute:
#[allow(clippy::collapsible_if)]
fn check(...) {
    if let Some(x) = opt {
        if x > 0 { ... }
    }
}
```

## Reference

- Existing rules: `core/src/rules/*.rs`
- Rule trait: `core/src/rules/mod.rs`
- Problem type: `core/src/problem.rs`
- Config system: `core/src/config.rs`
- Integration tests: `core/tests/integration_tests.rs`
- Test fixtures: `tests/fixtures/invalid/*.yaml`, `tests/fixtures/valid/*.yaml`
- Fixture validation: `make validate-fixtures`
