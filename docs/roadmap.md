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
  - [x] Support `yaml-files`, rule-level ignores, and `ignore-from-file`
  - [x] Report YAML syntax errors as diagnostics during linting and fixing
  - [x] [Audit all 23 rules against yamllint v1.38.0 fixtures](yamllint-compatibility.md)
  - [ ] Resolve remaining differences in rule options, diagnostic positions and messages, and parser behavior
- [x] Phase 4: Performance optimizations, parallel linting
- [ ] Phase 5: Editor integration (LSP?)
