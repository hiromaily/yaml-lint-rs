---
name: create-issue
description: Draft or create a GitHub issue for yaml-lint-rs when asked to report a bug or propose a feature, lint rule, or documentation change.
---

# Create a GitHub issue

1. Use `hiromaily/yaml-lint-rs` as the target. Search existing open and closed issues with `gh issue list --repo hiromaily/yaml-lint-rs --state all --search '<keywords>'`. If an issue already covers the request, return its link.
2. Inspect the relevant source and docs before describing current behavior. Use a `feat:`, `fix:`, or `docs:` title and the corresponding `enhancement`, `bug`, or `documentation` label.
3. Include a summary, motivation, proposed implementation, implementation notes, acceptance criteria, and priority with a reason when applicable. For a bug, include reproduction steps, expected and actual behavior, and relevant environment details. For a new lint rule, include YAML configuration, valid and invalid examples, intended presets, and the yamllint equivalent when applicable.
4. For a draft request, return the title and Markdown body. When the user asks to create the issue, save the body to a temporary file and run `gh issue create --repo hiromaily/yaml-lint-rs --title '<title>' --label '<label>' --body-file '<path>'`. Return the issue URL.
