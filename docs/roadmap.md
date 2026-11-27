# Roadmap

- [x] Phase 1: Core infrastructure + 6 priority rules
- [x] Phase 2: Additional output formats, directives, more rules
  - [x] JSON diagnostics output
  - [x] Line-level comment directives
  - [x] Additional lint rules (`braces`, `brackets`, `new-lines`)
- [ ] Phase 3: Full yamllint parity (23 rules)
  - [x] Implement and register all 23 yamllint rule names
  - [x] Support locale-aware key ordering on Linux and macOS
  - [x] Support PCRE2 in rule pattern options
  - [x] Match duplicate-key diagnostics for scalar keys and support the merge-key option
  - [x] Apply top-level ignore patterns to CLI file discovery
  - [ ] Match the remaining upstream configuration and diagnostics, including `yaml-files`, rule-level ignores, and syntax errors. Audit all 23 rules against upstream fixtures.
- [x] Phase 4: Performance optimizations, parallel linting
- [ ] Phase 5: Editor integration (LSP?)
