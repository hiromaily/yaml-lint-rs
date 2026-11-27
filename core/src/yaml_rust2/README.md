The scanner, parser and character helpers in this directory are from
[yaml-rust2 0.13.2](https://crates.io/crates/yaml-rust2/0.13.2), under the
included MIT and Apache 2.0 licenses. The scanner and parser share token
and marker types, and the upstream parser constructs its own scanner.
These modules are compiled into the published core crate so workspace
and crates.io builds use the same behavior.

Local scanner changes

- Allow outdented closing flow delimiters to reach the parser, which
  checks that the delimiter matches the open collection.
- End alias keys before a mapping separator using the scanner's existing
  separator rules.
- Retain the dash location for block entries and mark implicit block
  mappings at their first key. Identify a separator rejected after a
  continued plain scalar so the linter can avoid a duplicate diagnosis.
- Allow outdented quoted continuations so the indentation rule can
  diagnose them using their original source positions.

Local parser changes

- Require an explicit start marker for subsequent documents, including
  after an explicit end marker, to match yamllint's syntax diagnostics.
- Report unexpected mapping starts and sequence entries at the block
  boundary where they occurred.
