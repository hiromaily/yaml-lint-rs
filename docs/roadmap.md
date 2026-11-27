# Roadmap

- [x] Phase 1: Core infrastructure + 6 priority rules
- [x] Phase 2: Additional output formats, directives, more rules
  - [x] JSON diagnostics output
  - [x] Line-level comment directives
  - [x] Additional lint rules (`braces`, `brackets`, `new-lines`)
- [ ] Phase 3: Full yamllint parity (23 rules)
  - [x] Implement and register all 23 yamllint rule names
  - [ ] Match upstream configuration and diagnostics, including locale-aware key ordering and PCRE support
- [x] Phase 4: Performance optimizations, parallel linting
- [ ] Phase 5: Editor integration (LSP?)
