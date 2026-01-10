# Linting Rules

This document describes all the linting rules implemented in yaml-lint-rs.

## Implemented Rules (Phase 1)

### trailing-spaces

**Level**: Error (default)
**Configurable**: No

Detects whitespace (spaces or tabs) at the end of lines.

**Why it matters**: Trailing whitespace is invisible and can cause unexpected diffs in version control.

**Examples**:

```yaml
# Bad
key: value

# Good
key: value
```

### line-length

**Level**: Error (default)
**Configurable**: Yes

Enforces a maximum line length.

**Configuration**:
```yaml
rules:
  line-length:
    max: 80  # Default: 80
```

**Why it matters**: Long lines are harder to read and may cause horizontal scrolling.

**Examples**:

```yaml
# Bad (assuming max: 80)
key: this is a very long value that exceeds the maximum line length of eighty characters

# Good
key: shorter value
# or use multiline
key: |
  this is a very long value
  split across multiple lines
```

### document-start

**Level**: Disable (default)
**Configurable**: Yes

Requires or forbids the `---` document start marker.

**Configuration**:
```yaml
rules:
  document-start: error    # Require ---
  # or
  document-start: disable  # Don't check
```

**Why it matters**: The `---` marker explicitly marks the start of a YAML document, which is important for multi-document files.

**Examples**:

```yaml
# With document-start: error

# Bad
key: value

# Good
---
key: value
```

### colons

**Level**: Error (default)
**Configurable**: Yes (future)

Validates spacing around colons in key-value mappings.

**Default**: 0 spaces before colon, 1 space after colon

**Why it matters**: Consistent spacing improves readability.

**Examples**:

```yaml
# Bad
key : value    # Space before colon
key:  value    # Two spaces after colon

# Good
key: value
```

### key-duplicates

**Level**: Error (default)
**Configurable**: No

Detects duplicate keys in YAML mappings.

**Why it matters**: Duplicate keys in YAML can lead to unexpected behavior. Most parsers will silently use only the last value.

**Examples**:

```yaml
# Bad
key: value1
another: test
key: value2  # Duplicate!

# Good
key: value1
another: test
different: value2
```

**Note**: Keys in different scopes are allowed:

```yaml
# Good - same key name in different scopes
parent1:
  key: value1
parent2:
  key: value2  # OK, different scope
```

### indentation

**Level**: Error (default)
**Configurable**: Yes

Validates consistent indentation throughout the document.

**Configuration**:
```yaml
rules:
  indentation:
    spaces: 2           # Or 4, or "consistent" to auto-detect
```

**Why it matters**: Inconsistent indentation can make YAML difficult to read and may lead to parsing errors.

**Examples**:

```yaml
# Bad
root:
  level1: value
   level2: wrong  # Inconsistent indentation
  level3: correct

# Good
root:
  level1: value
  level2: correct
  level3: correct
```

**Special cases**:

- Tabs are never allowed in indentation
- Indentation must be a multiple of the configured spaces
- List items follow special indentation rules

```yaml
# Good list indentation
list:
  - item1
  - item2
    nested: value
  - item3
```

### new-line-at-end-of-file

**Level**: Error (default)
**Configurable**: No

Requires files to end with a newline character.

**Why it matters**: POSIX standard requires text files to end with a newline. This also prevents unnecessary diffs in version control when adding content to file end.

**Examples**:

```yaml
# Bad - file ends without newline
key: value
```

```yaml
# Good - file ends with newline
key: value
⏎
```

**Note**: Empty files are considered valid.

### empty-lines

**Level**: Error (default)
**Configurable**: Yes

Limits consecutive empty lines in YAML files.

**Configuration**:
```yaml
rules:
  empty-lines:
    max: 2         # Maximum consecutive empty lines (default: 2)
    max-start: 0   # Maximum empty lines at file start (default: 0)
    max-end: 0     # Maximum empty lines at file end (default: 0)
```

**Why it matters**: Excessive blank lines reduce readability and make files harder to navigate.

**Examples**:

```yaml
# Bad (with max: 2)
key1: value1



key2: value2  # 3 empty lines - error

# Good
key1: value1

key2: value2  # 1 empty line - ok
```

**Note**: Lines containing only whitespace are considered empty.

## Rule Levels

Each rule can be configured with one of three levels:

- **error**: Problem causes non-zero exit code (1)
- **warning**: Problem is reported but doesn't fail (exit code 2 with --strict)
- **disable**: Rule is not checked

## Presets

### default (strict)

All rules enabled as errors, except:
- `document-start`: disabled

Suitable for production code.

### relaxed

Most rules as warnings:
- `trailing-spaces`: warning
- `line-length`: warning
- `colons`: warning
- `indentation`: warning
- `new-line-at-end-of-file`: warning
- `empty-lines`: warning
- `key-duplicates`: error (kept as error)
- `document-start`: disabled

More permissive for development and experimentation.

## Future Rules (Planned)

The following rules are planned for future releases:

### truthy
Restrict boolean representations to avoid YAML 1.1 vs 1.2 ambiguities.

### hyphens
Control spacing after list item hyphens.

### comments
Enforce comment formatting and spacing.

### braces / brackets
Control spacing in flow collections `{}` and `[]`.

### comments-indentation
Enforce that comments are indented like content.

### document-end
Require or forbid `...` document end marker.

### quoted-strings
Manage quote usage (single vs double quotes, when to quote).

### float-values / octal-values
Prevent problematic number representations.

### key-ordering
Enforce alphabetical key sorting.

And more for full yamllint compatibility (23 rules total).

## Custom Rules

The `yaml-lint-core` library provides a `Rule` trait that you can implement for custom rules:

```rust
use yaml_lint_core::{Rule, LintContext, LintProblem, LintLevel, RuleLevel};

pub struct MyCustomRule;

impl Rule for MyCustomRule {
    fn name(&self) -> &'static str {
        "my-custom-rule"
    }

    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        let mut problems = Vec::new();
        // Your linting logic here
        problems
    }

    fn default_level(&self) -> RuleLevel {
        RuleLevel::Error
    }
}
```

Then register your rule:

```rust
let mut registry = RuleRegistry::new();
registry.register(Box::new(MyCustomRule));
```
