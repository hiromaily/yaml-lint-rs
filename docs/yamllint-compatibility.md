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
| `colons` | 39 | 39 | 38 | 38 |
| `commas` | 36 | 36 | 36 | 36 |
| `comments` | 24 | 24 | 23 | 23 |
| `comments-indentation` | 24 | 24 | 24 | 24 |
| `document-end` | 13 | 13 | 13 | 13 |
| `document-start` | 21 | 20 | 20 | 15 |
| `empty-lines` | 32 | 32 | 32 | 32 |
| `empty-values` | 62 | 62 | 62 | 62 |
| `float-values` | 5 | 5 | 5 | 5 |
| `hyphens` | 20 | 20 | 20 | 20 |
| `indentation` | 270 | 222 | 221 | 219 |
| `key-duplicates` | 33 | 33 | 33 | 33 |
| `key-ordering` | 21 | 21 | 21 | 21 |
| `line-length` | 50 | 50 | 50 | 50 |
| `new-line-at-end-of-file` | 9 | 9 | 9 | 9 |
| `new-lines` | 30 | 30 | 30 | 30 |
| `octal-values` | 33 | 33 | 33 | 33 |
| `quoted-strings` | 77 | 77 | 77 | 77 |
| `syntax` | 10 | 9 | 9 | 9 |
| `trailing-spaces` | 12 | 10 | 10 | 10 |
| `truthy` | 18 | 18 | 18 | 18 |
| **Total** | 992 | 936 | 933 | 926 |

Location comparisons include diagnostic rule IDs. The target column considers
only the rule named by the fixture group. The other columns compare all
diagnostics emitted for that fixture. The run used Rust 1.100.0 on macOS
and had zero CLI config or file errors for the compared fixtures.

- `indentation` has the most location differences. The remaining fixtures
  include sequence indentation, block scalars, and nested flow collections.
- The YAML scanner rejects some multiline empty flow collections whose closing
  delimiter is at column one. It also differs on aliases adjacent to a colon
  and on tabs outside quoted scalars.
- The audit forwards the collation locale selected by the upstream key-ordering
  fixtures to the Rust CLI.
- The built-in presets still differ from upstream. The default preset keeps
  `document-start` disabled, and the relaxed preset enables a different set of
  warnings.
