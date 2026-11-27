The scanner, parser and character helpers in this directory are from
[yaml-rust2 0.13.2](https://crates.io/crates/yaml-rust2/0.13.2), under the
included MIT and Apache 2.0 licenses. The scanner and parser share token
and marker types, and the upstream parser constructs its own scanner.
These modules are compiled into the published core crate so workspace
and crates.io builds use the same behavior.

Local scanner changes

- Allow outdented closing flow delimiters to reach the parser, which
  checks that the delimiter matches the open collection.
