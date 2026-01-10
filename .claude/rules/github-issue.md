# GitHub Issue Creation Rules

When the user asks to create a GitHub issue (e.g., "issueを作成して", "create an issue", "issue追加"), follow these guidelines.

## Project Context

- **Repository**: hiromaily/yaml-lint-rs
- **Purpose**: Fast YAML linter written in Rust, inspired by Python yamllint
- **Structure**: Cargo workspace with `core` (library) and `cli` (binary) crates

### Current Implementation Status

**Implemented Rules (6):**
- trailing-spaces, line-length, document-start, colons, key-duplicates, indentation

**Roadmap:**
- Phase 2: Additional output formats, more rules
- Phase 3: Full yamllint parity (23 rules)
- Phase 4: Parallel linting
- Phase 5: Editor integration (LSP)

## Issue Guidelines

### Title Conventions

| Type | Format | Example |
|------|--------|---------|
| New Rule | `feat: Add \`rule-name\` rule` | `feat: Add \`truthy\` rule` |
| New Feature | `feat: Add feature description` | `feat: Add colored output format` |
| CLI Option | `feat: Add \`--flag\` option` | `feat: Add \`--fix\` auto-fix option` |
| Bug Fix | `fix: Brief description` | `fix: Incorrect line number in error` |
| Docs | `docs: Description` | `docs: Add examples for indentation rule` |

### Labels

- `enhancement` - New features and rules
- `bug` - Bug fixes
- `documentation` - Documentation improvements

### Required Sections

1. **Summary** - 1-2 sentences describing the change
2. **Motivation** - Why it's needed, who benefits
3. **Proposed Implementation** - Configuration, CLI usage, examples
4. **Implementation Notes** - Technical considerations
5. **Acceptance Criteria** - Checklist of requirements
6. **Priority** - 🔥 High / ⚡ Medium / 📋 Low with reason

### For New Rules, Include:

- YAML configuration format
- Good/Bad code examples
- Reference to yamllint equivalent rule if applicable
- Which presets it should be added to (default, relaxed)

## Issue Template

```markdown
## Summary

[1-2 sentence description]

## Motivation

- [Why this is needed]
- [Who benefits]
- [Reference to yamllint or other tools if applicable]

## Proposed Implementation

### Configuration / Usage

```yaml
rules:
  rule-name:
    option: value
```

### Examples

```yaml
# Bad
...

# Good
...
```

## Implementation Notes

- [Technical considerations]
- [Dependencies needed]
- [Files to modify]

## Acceptance Criteria

- [ ] Core functionality implemented
- [ ] Configuration options work
- [ ] Unit tests added
- [ ] Documentation updated in docs/RULES.md or docs/USAGE.md
- [ ] Added to presets in config.rs

## Priority

[🔥 High / ⚡ Medium / 📋 Low] - [Brief reason]
```

## Command to Use

```bash
gh issue create \
  --title "feat: Title here" \
  --label "enhancement" \
  --body "$(cat <<'EOF'
## Summary
...

## Motivation
...

## Proposed Implementation
...

## Acceptance Criteria
- [ ] ...

## Priority
...
EOF
)"
```

## Before Creating Issues

1. Check existing issues: `gh issue list`
2. Understand current implementation by reading relevant files
3. Reference yamllint documentation for rule compatibility
4. Consider implementation complexity and user value
