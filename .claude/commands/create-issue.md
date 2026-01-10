# Create GitHub Issue

Create a well-structured GitHub issue for this project.

## Project Context

- **Repository**: hiromaily/yaml-lint-rs
- **Type**: Fast YAML linter written in Rust (inspired by Python yamllint)
- **Architecture**: Cargo workspace with `core` (library) and `cli` (binary) crates

## Issue Creation Guidelines

### For New Rules

When creating an issue for a new linting rule:

1. **Title format**: `feat: Add \`rule-name\` rule`
2. **Label**: `enhancement`
3. **Include**:
   - Summary of what the rule does
   - Motivation (why it's needed)
   - Configuration options (YAML format)
   - Good/Bad examples
   - Implementation notes
   - Reference to yamllint if applicable
   - Acceptance criteria checklist

### For New Features

When creating an issue for a new feature:

1. **Title format**: `feat: Add feature-name` or `feat: Add \`--flag\` option`
2. **Label**: `enhancement`
3. **Include**:
   - Summary
   - Motivation and use cases
   - Proposed CLI usage examples
   - Implementation approach
   - Acceptance criteria

### For Bug Fixes

1. **Title format**: `fix: Brief description of the bug`
2. **Label**: `bug`
3. **Include**:
   - Steps to reproduce
   - Expected behavior
   - Actual behavior
   - Environment info if relevant

## Issue Template Structure

```markdown
## Summary

[1-2 sentence description]

## Motivation

- [Why this is needed]
- [Who benefits]
- [Reference to yamllint or other tools if applicable]

## Proposed Implementation

### Configuration / Usage

\`\`\`yaml
rules:
  rule-name:
    option: value
\`\`\`

### Examples

\`\`\`yaml
# Bad
...

# Good
...
\`\`\`

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

## Command

Use `gh issue create` with the `--label enhancement` flag. Format the body using a HEREDOC for proper markdown rendering.

## User Request

$ARGUMENTS
