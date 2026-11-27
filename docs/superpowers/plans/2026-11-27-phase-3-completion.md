# Phase 3 Compatibility Implementation Plan

> **For agentic workers:** Use superpowers:executing-plans or, if delegation is requested, superpowers:subagent-driven-development. Steps use checkboxes for progress tracking.

**Goal**

Close the 24 diagnostic mismatches in the yamllint v1.38.0 audit and complete Phase 3 on `feat/phase-3-yamllint-parity`.

**Architecture**

Fix rule-local behavior in the existing rules. Trace scanner differences to their first divergent token before changing shared behavior. Preserve original source indices and use yaml-rust2 for YAML parsing. Use one reviewable commit per compatibility behavior and keep the commits on the same PR branch.

**Tech stack**

Rust edition 2024, Rust 1.100, yaml-rust2, Python fixture audit.

**Spec**

[Phase 3 roadmap](../../roadmap.md) and [compatibility audit](../../yamllint-compatibility.md). Baseline commit `f88ee64` matches 968 of 992 fixtures exactly.

## Verification workflow

Run the audit setup in the compatibility document. Record the mismatch report before editing. The same upstream test method can have multiple fixture calls, so compare the source and expected diagnostic tuples as well as the method name.

```sh
cargo build -p yaml-lint
/tmp/yamllint-audit-env/bin/python tests/audit_yamllint.py \
  target/debug/yaml-lint /tmp/yamllint-v1.38.0 > /tmp/phase3-audit.tsv 2> /tmp/phase3-audit.log
```

For each task, add a regression at the normal lint entry point or beside the rule, verify it fails, apply the fix, rerun the regression and full audit, and review any newly mismatching cases. Compare positions, severity, messages, and rule IDs. Use `make ci` and `make validate-fixtures` before committing rule changes. The fixture count remains 992. Targets below are incremental from the baseline and assume tasks run in order.

## Rule-local changes

### Task 1. Comment indentation after a folded scalar (1 case, target 969)

- [ ] Reproduce `test_comments.CommentsTestCase.test_multi_line_scalar` in `core/src/rules/comments_indentation.rs`. For `"---\n- string: >\n    content\n  \n  # comment\n"`, expect no `comments-indentation` diagnostic.
- [ ] Use the enclosing sequence mapping indentation when anchoring a standalone comment after a block scalar. Retain a neighboring case where a comment outside the scalar is misaligned.
- [ ] Verify all 24 cases in each of the `comments` and `comments-indentation` groups match. Commit the rule and regression.

### Task 2. Block scalar header after a bare dash (1 case, target 970)

- [ ] Reproduce `test_indentation.IndentationTestCase.test_clear_sequence_item` in `core/src/rules/indentation.rs` with `"---\n-\n  -\n     >\n      multi\n      line\n...\n"`. Expect only `indentation` at line 4, column 6.
- [ ] Include the header line in the block-entry child check. Resolve the header from the source span before the body marker. Use the existing inferred step and leave the body to `check-multi-line-strings`. Cover both `|` and `>` and a correctly indented header.
- [ ] Verify the fixture and the existing scalar indentation tests. Commit separately.

## Scanner and syntax compatibility

Before changing scanner behavior in Tasks 3–8, capture the yaml-rust2 tokens or error marker and compare with the reference. If the dependency exposes no suitable hook, first validate a minimal change in a temporary copy, then determine how to integrate it while retaining yaml-rust2. A correction must preserve source coordinates, keep invalid neighbors rejected, and work through both lint and fix. Do not count a match achieved by filtering out diagnostics in the audit.

### Task 3. Closing delimiters in multiline flow collections (8 cases, target 978)

- [ ] Reproduce the two `test_forbid` cases each in `braces` and `brackets`, the three flow indentation cases in `test_indentation`, and `test_syntax_errors.test_empty_flows`. Start with `"---\ndict: {\n}\n"`, which should have no diagnostics, and an indented mapping whose closing delimiter is outdented, which should report `indentation` at the delimiter and continue scanning.
- [ ] Trace the scanner's indentation guard at closing tokens. Verify that matching close tokens can reach the parser and rules at their original positions. Wrong-kind delimiters, missing closures, and lines outside flow collections must still produce syntax diagnostics.
- [ ] Verify all eight reference cases, including the one with later indentation errors. Review `core/src/syntax.rs`, the scanner construction in `core/src/rules/tokens.rs`, and the flow rule entry points for consistent behavior. Commit separately.

### Task 4. Aliases immediately before a mapping colon (1 case, target 979)

- [ ] Reproduce `test_colons.ColonTestCase.test_with_alias_as_key` with both `- *a: 42` and `- {*a: 42}` after declaring `&a`. Expect the original fixture's only diagnostics at (7, 6) and (8, 7), both `colons`.
- [ ] Trace the alias scan boundary. Use the same mapping-separator recognition as the corresponding block or flow scanner path. Preserve ordinary aliases, declaration matching, and source positions. Verify that a genuinely undeclared alias still reports `anchors`.
- [ ] Verify `anchors` and `colons` audit groups, then commit.

### Task 5. Implicit document after an explicit end (1 case, target 980)

- [ ] Reproduce `test_document_start.DocumentStartTestCase.test_multiple_documents` with `"---\nfirst: document\n...\nsecond: document\n---\nthird: document\n"`. Expect `syntax` at (4, 1) with the reference message from the audit.
- [ ] In `core/src/syntax.rs`, trace how the parser reports document transitions. Distinguish explicit marker tokens from implicit boundaries. Verify that a first implicit document and a later explicitly started document still behave correctly.
- [ ] Verify missing-marker warnings outside this case and run lint/fix regression checks. Document the intentional compatibility behavior and commit.

### Task 6. Malformed block collections (8 cases, target 988)

- [ ] Reproduce the one malformed `test_indent_sequences_whatever` fixture, one `test_nested_collections`, two `test_over_indented`, two `test_return`, and two `test_under_indented` fixtures. For `"---\na:\n  b:\n    c:\n   d:\n...\n"`, expect `syntax` at (5, 4), with the message verified against upstream.
- [ ] In `core/src/syntax.rs` and `core/src/rules/indentation.rs`, identify where the parser lost the expected block boundary. Attribute errors to the unexpected token and remove competing indentation diagnostics only where the same malformed structure produced both. The over-indented fixtures expect the mapping separator location, while the under-indented fixtures expect the start of the unexpected key.
- [ ] Verify all eight cases and a valid neighbor for each failure shape. Commit this task in smaller commits if location correction and diagnostic suppression can be verified independently.

### Task 7. Outdented continuation in quoted scalars (2 cases, target 990)

- [ ] Reproduce the one each from `ScalarIndentationTestCase.test_paragraph_double_quoted` and `test_paragraph_single_quoted`. With `check-multi-line-strings: true`, expect `indentation` at (2, 1), (4, 1), and (5, 1) in each reference fixture.
- [ ] Trace the scanner's quote continuation check. Preserve original scalar spans so `core/src/rules/indentation.rs` can report continuation lines. Verify the case with the option disabled and an unterminated-quote neighbor.
- [ ] Run the quoted-strings, comments, and indentation audit groups and commit.

### Task 8. Tabs at token boundaries (2 cases, target 992)

- [ ] Reproduce `test_trailing_spaces.TrailingSpacesTestCase.test_enabled` for `"\t\t\t\n"` and `"---\nsome: text\t\n"`. Expect one `syntax` diagnostic instead of `trailing-spaces`, at (1, 1) and (2, 11), respectively.
- [ ] Trace when tabs occur outside content. Check quoted and block scalars, comments, and legal separators before deciding where to diagnose in `core/src/syntax.rs`. Do not scan every tab indiscriminately. Verify the fixer keeps the same syntax result on these inputs.
- [ ] Verify both reference cases, run the full audit, and commit.

## Close Phase 3

- [ ] Verify the audit exits 0 with all 992 cases matching locations, severity, and messages, and zero CLI config or file errors. Run `make ci` and `make validate-fixtures` on Rust 1.100.0.
- [ ] Update the audit table and Phase 3 roadmap only after verification. If an unresolved parser difference requires a project decision, leave that item open with the exact fixture, diagnostic, and dependency behavior recorded.
- [ ] Review the full branch diff against the PR scope and check that all task commits are present on the same branch.
