# Linting Rules

This document describes all the linting rules implemented in yaml-lint-rs.

## Implemented Rules (Phase 1)

### trailing-spaces

**Level**: Error (default)
**Configurable**: No
**Fixable**: ✅ Yes

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
**Fixable**: ❌ No (requires semantic understanding)

Enforces a maximum line length in Unicode characters.

**Configuration**:
```yaml
rules:
  line-length:
    max: 80
    allow-non-breakable-words: true
    allow-non-breakable-inline-mappings: false
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
**Fixable**: ❌ No

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
**Configurable**: Yes
**Fixable**: ❌ No (planned)

Validates spacing around colons in key-value mappings.

`max-spaces-before` defaults to 0 and `max-spaces-after` defaults to 1. Set either
option to -1 to disable its limit.

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
**Fixable**: ❌ No (ambiguous which to keep)

Detects duplicate scalar keys in block and flow mappings. Set
`forbid-duplicated-merge-keys` to true to report repeated `<<` merge keys.
The option defaults to false.

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
**Fixable**: ❌ No (planned)

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
**Fixable**: ✅ Yes

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
**Fixable**: ✅ Yes

Limits consecutive empty lines in YAML files and reports the last line of each
excess run. The `trailing-spaces` rule handles whitespace on otherwise empty lines.

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


### hyphens

**Level**: Error (default)
**Configurable**: Yes
**Fixable**: ❌ No (planned)

Controls spacing after list item hyphens in YAML sequences. Set
`max-spaces-after` to -1 to disable the limit.

**Configuration**:
```yaml
rules:
  hyphens:
    max-spaces-after: 1  # Maximum spaces after hyphen (default: 1)
```

**Why it matters**: Consistent list formatting improves readability.

**Examples**:

```yaml
# Bad (with max-spaces-after: 1)
list:
  -  item1   # 2 spaces after hyphen
  -   item2  # 3 spaces after hyphen

# Good
list:
  - item1    # 1 space after hyphen
  - item2
```

**Note**: Empty list items (`-` alone) and block scalar indicators (`- |`, `- >`) are allowed.

### comments

**Level**: Warning (default)
**Configurable**: Yes
**Fixable**: ✅ Yes

Enforces consistent comment formatting in YAML files.

**Configuration**:
```yaml
rules:
  comments:
    require-starting-space: true    # Require space after # (default: true)
    ignore-shebangs: true           # Ignore a shebang on the first line (default: true)
    min-spaces-from-content: 2      # Minimum spaces before inline comment (-1 disables this check)
```

**Why it matters**: Consistent comment formatting improves readability.

**Examples**:

```yaml
# Bad
#This comment has no space after #
key: value # Inline comment too close

# Good
# This comment has proper spacing
key: value  # Inline comment with proper spacing
```

**Notes**:
- Shebangs (`#!/bin/bash`) are ignored by default
- Section headers (`##`) are allowed
- Hash inside strings is not treated as a comment

### truthy

**Level**: Warning (default)
**Configurable**: Yes
**Fixable**: ❌ No

Restricts boolean representations to avoid YAML 1.1 vs 1.2 ambiguities.

**Configuration**:
```yaml
rules:
  truthy:
    allowed-values: ['true', 'false']  # Default: only true/false
    check-keys: true                    # Also check mapping keys (default: true)
```

**Why it matters**: YAML 1.1 boolean spellings include `yes`, `no`, `on`, and `off`. With a `%YAML 1.2` directive, the rule checks only variants of `true` and `false`. The allowlist is case-sensitive. An empty allowlist flags every checked spelling.

**Examples**:

```yaml
# Bad - YAML 1.1 ambiguous values
debug: yes
enabled: on
feature_flag: True

# Good - explicit boolean values
debug: true
enabled: true
feature_flag: true

# Good - quoted strings when you mean strings
country_code: "NO"  # Norway, not boolean false
answer: "yes"       # String, not boolean true
```

**Notes**:
- Case-insensitive detection (YES, Yes, yes all detected)
- Quoted values are not flagged (they are strings)
- Critical for Kubernetes/Helm users

## Implemented Rules (Phase 2)

### braces

**Level**: Error (default)
**Configurable**: Yes
**Fixable**: ❌ No

Controls spacing inside flow mappings (braces `{}`).

**Configuration**:
```yaml
rules:
  braces:
    forbid: false              # true forbids all flow mappings, non-empty permits {}
    min-spaces-inside: 0       # Minimum spaces inside braces (default: 0)
    max-spaces-inside: 0       # Maximum spaces inside braces (default: 0)
    min-spaces-inside-empty: -1  # For empty braces, -1 uses min-spaces-inside
    max-spaces-inside-empty: -1  # For empty braces, -1 uses max-spaces-inside
```

**Why it matters**: Enforces consistent formatting in flow-style mappings for readability.

**Examples**:

```yaml
# Bad (with max-spaces-inside: 0)
mapping: { key: value }     # spaces inside braces

# Good
mapping: {key: value}

# With forbid: true, require block style
mapping:
  key: value
```

**Notes**:

- Braces inside quoted strings are ignored
- Braces in comments are ignored
- Empty braces (`{}`) have separate spacing options

### brackets

**Level**: Error (default)
**Configurable**: Yes
**Fixable**: ❌ No

Controls spacing inside flow sequences (brackets `[]`).

**Configuration**:
```yaml
rules:
  brackets:
    forbid: false              # true forbids all flow sequences, non-empty permits []
    min-spaces-inside: 0       # Minimum spaces inside brackets (default: 0)
    max-spaces-inside: 0       # Maximum spaces inside brackets (default: 0)
    min-spaces-inside-empty: -1  # For empty brackets, -1 uses min-spaces-inside
    max-spaces-inside-empty: -1  # For empty brackets, -1 uses max-spaces-inside
```

**Why it matters**: Enforces consistent formatting in flow-style sequences for readability.

**Examples**:

```yaml
# Bad (with max-spaces-inside: 0)
sequence: [ 1, 2, 3 ]       # spaces inside brackets

# Good
sequence: [1, 2, 3]

# With forbid: true, require block style
sequence:
  - 1
  - 2
  - 3
```

**Notes**:

- Brackets inside quoted strings are ignored
- Brackets in comments are ignored
- Empty brackets (`[]`) have separate spacing options

## Rule Levels

Each rule can be configured with one of three levels:

- **error**: Problem causes non-zero exit code (1)
- **warning**: Problem is reported but doesn't fail (exit code 2 with --strict)
- **disable**: Rule is not checked

### new-lines

Checks the first line ending in the YAML source. The default preset reports
mismatches as errors, and the relaxed preset reports them as warnings.

| `type` | Expected line ending |
| --- | --- |
| `unix` (default) | LF |
| `dos` | CRLF |
| `platform` | CRLF on Windows and LF elsewhere |

```yaml
rules:
  new-lines:
    type: dos
```

Use `new-line-at-end-of-file` to require a final line ending. Its auto-fix uses
the style configured by `new-lines` when that rule is enabled.

## Presets

### default (strict)

Most enabled rules are errors. The exceptions and disabled rules are:

- `document-start`: disabled
- `comments`, `truthy`, and `comments-indentation`: warning
- `document-end`, `empty-values`, `float-values`, `key-ordering`, `octal-values`, and `quoted-strings`: disabled

Suitable for production code.

### relaxed

Most rules as warnings:

- `trailing-spaces`: warning
- `line-length`: warning
- `colons`: warning
- `indentation`: warning
- `new-line-at-end-of-file`: warning
- `new-lines`: warning
- `empty-lines`: warning
- `hyphens`: warning
- `comments`: warning
- `truthy`: warning
- `braces`: warning
- `brackets`: warning
- `commas`: warning
- `anchors` and `key-duplicates`: error
- `comments-indentation`, `document-start`, `document-end`, `empty-values`, `float-values`, `key-ordering`, `octal-values`, and `quoted-strings`: disabled

More permissive for development and experimentation.

## Phase 3 rules

These rules report diagnostics. Each can be explicitly enabled with `enable`
or a settings mapping such as `{}`.

| Rule | Default preset | Relaxed preset | Options when enabled |
| --- | --- | --- | --- |
| `anchors` | Error | Error | `forbid-undeclared-aliases` defaults to true. `forbid-duplicated-anchors` and `forbid-unused-anchors` default to false. Anchor scope is per YAML document. |
| `commas` | Error | Warning | `max-spaces-before` defaults to 0. `min-spaces-after` and `max-spaces-after` default to 1. Use -1 to turn off a spacing limit. |
| `comments-indentation` | Warning | Disabled | Checks standalone comment indentation. |
| `document-end` | Disabled | Disabled | `present` defaults to true to require `...` at the end of each document. Use false to forbid it. |
| `empty-values` | Disabled | Disabled | `forbid-in-block-mappings`, `forbid-in-flow-mappings`, and `forbid-in-block-sequences` default to true. Explicit nulls and quoted empty strings are accepted. |
| `float-values` | Disabled | Disabled | `require-numeral-before-decimal`, `forbid-scientific-notation`, `forbid-nan`, and `forbid-inf` default to false. |
| `key-ordering` | Disabled | Disabled | `ignored-keys` is a list of regex patterns and defaults to an empty list. Keys are ordered by Unicode code point unless a top-level `locale` is configured. |
| `octal-values` | Disabled | Disabled | `forbid-implicit-octal` and `forbid-explicit-octal` default to true. |
| `quoted-strings` | Disabled | Disabled | `quote-type` defaults to `any` and also accepts `single`, `double`, or `consistent`. `required` defaults to true and also accepts false or `only-when-needed`. |

For `quoted-strings`, set `check-keys` to true to include mapping keys, or
`allow-quoted-quotes` to true to allow the other quote style when it avoids
escaping a quote in the value. Both options default to false. Use
`extra-required` with `required: false` or `required: only-when-needed` to
require quotes on matching strings. Use `extra-allowed` with
`required: only-when-needed` to allow quotes on matching strings.

The `ignored-keys`, `extra-required`, and `extra-allowed` options use PCRE2
with Unicode character classes. Each pattern can match anywhere in the
scalar. For example, `'^(\w+)\1$'` matches a scalar made of a repeated word.

On Linux and macOS, top-level `locale: en_US.UTF-8` selects an installed system
locale for `key-ordering`. Use `locale: ""` to select the environment's locale.
An unavailable locale produces a configuration error.

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
