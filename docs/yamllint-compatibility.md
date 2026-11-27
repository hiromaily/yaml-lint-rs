# Yamllint compatibility audit

The audit runs all 23 rule test modules and the syntax tests from
[yamllint v1.38.0](https://github.com/adrienverge/yamllint/tree/v1.38.0/tests).
It compares each fixture's CLI JSON diagnostics with upstream yamllint.

```sh
cargo build -p yaml-lint
python3 -m venv /tmp/yamllint-audit-env
/tmp/yamllint-audit-env/bin/python -m pip install 'PyYAML==6.0.3' 'pathspec==0.12.1'
git clone --depth 1 --branch v1.38.0 https://github.com/adrienverge/yamllint.git /tmp/yamllint-v1.38.0
/tmp/yamllint-audit-env/bin/python tests/audit_yamllint.py target/debug/yaml-lint /tmp/yamllint-v1.38.0
```

The script prints mismatch details to stderr and exits with status 1 while
any fixture differs. It tests the upstream rule and syntax fixtures that
produce diagnostic lists. Cases that expect config or decoding exceptions
run in the upstream suite without a Rust CLI comparison.

| Fixture group | Compared | Target locations match | All locations match | Locations, severity, and messages match |
| --- | ---: | ---: | ---: | ---: |
| `anchors` | 8 | 8 | 8 | 8 |
| `braces` | 74 | 72 | 72 | 72 |
| `brackets` | 71 | 69 | 69 | 69 |
| `colons` | 39 | 37 | 35 | 35 |
| `commas` | 36 | 36 | 36 | 36 |
| `comments` | 24 | 14 | 13 | 12 |
| `comments-indentation` | 24 | 24 | 24 | 12 |
| `document-end` | 13 | 13 | 12 | 12 |
| `document-start` | 21 | 13 | 12 | 10 |
| `empty-lines` | 32 | 22 | 22 | 18 |
| `empty-values` | 62 | 54 | 54 | 54 |
| `float-values` | 5 | 5 | 5 | 5 |
| `hyphens` | 20 | 20 | 20 | 20 |
| `indentation` | 270 | 114 | 113 | 113 |
| `key-duplicates` | 33 | 32 | 32 | 28 |
| `key-ordering` | 21 | 17 | 17 | 17 |
| `line-length` | 50 | 41 | 39 | 37 |
| `new-line-at-end-of-file` | 9 | 9 | 9 | 9 |
| `new-lines` | 30 | 23 | 12 | 12 |
| `octal-values` | 33 | 33 | 33 | 33 |
| `quoted-strings` | 77 | 75 | 75 | 75 |
| `syntax` | 10 | 9 | 9 | 6 |
| `trailing-spaces` | 12 | 10 | 6 | 6 |
| `truthy` | 18 | 18 | 18 | 15 |
| **Total** | 992 | 768 | 745 | 714 |

Location comparisons include diagnostic rule IDs. The target column considers
only the rule named by the fixture group. The other columns compare all
diagnostics emitted for that fixture. The run used Rust 1.100.0 on macOS
and had zero CLI config or file errors for the compared fixtures.

- `indentation` has the most location differences. Upstream fixtures cover
  sequence indentation, block scalars, and nested flows.
- `yaml-rust2` reports `invalid indentation` for some multiline empty flow
  collections whose closing delimiter is at column one. Yamllint accepts
  those fixtures. Parser wording can also differ even when locations match.
- The `new-lines` rule checks every line ending. The upstream fixtures expect
  yamllint to check the first ending.
- The CLI currently reads `ignore-from-file` as UTF-8. Upstream also supports
  UTF-16 and UTF-32. File-based `extends` and the full upstream preset and
  rule option sets remain open.
