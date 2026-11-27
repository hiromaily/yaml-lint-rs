#!/usr/bin/env python3
"""Compare the Rust CLI with the rule and syntax fixtures in yamllint v1.38.0."""

import argparse
from collections import Counter, defaultdict
import json
import locale
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch


def audit(binary, upstream):
    # Load the upstream tests from the checkout without copying their fixtures.
    sys.path.insert(0, str(upstream))
    os.chdir(upstream)
    import yaml
    from tests.common import RuleTestCase
    from yamllint import linter
    from yamllint.rules import new_lines

    counts = defaultdict(Counter)
    upstream_check = RuleTestCase.check

    with tempfile.TemporaryDirectory() as directory:
        config = Path(directory, "config")
        source = Path(directory, "fixture.yaml")

        def compare(case, data, settings, **expected):
            # Preserve upstream assertions, including cases that expect exceptions.
            upstream_check(case, data, settings, **expected)
            rule = case.rule_id or "syntax"
            upstream_diagnostics = list(linter.run(data, case.build_fake_config(settings)))
            rules = yaml.safe_load(settings) if settings else {}
            new_lines_settings = rules.get("new-lines")
            if isinstance(new_lines_settings, dict) and new_lines_settings.get("type") == "platform":
                # The upstream fixture mocks both Windows and Unix line endings.
                new_lines_settings["type"] = "dos" if new_lines.linesep == "\r\n" else "unix"

            rust_settings = {"extends": "default", "rules": rules}
            collation = locale.setlocale(locale.LC_COLLATE)
            if rule == "key-ordering" and collation not in {"C", "POSIX"}:
                # The upstream tests set the Python process locale directly.
                rust_settings["locale"] = collation
            config.write_text(yaml.safe_dump(rust_settings), encoding="utf-8")
            source.write_bytes(data.encode("utf-8") if isinstance(data, str) else data)
            result = subprocess.run(
                [binary, "-j", "1", "-c", config, "-f", "json", source],
                capture_output=True,
                text=True,
                check=False,
            )
            row = counts[rule]
            row["fixtures"] += 1
            try:
                rust = json.loads(result.stdout)
            except json.JSONDecodeError:
                row["cli_errors"] += 1
                print(f"{case.id()} CLI error {result.stderr.strip()}", file=sys.stderr)
                return

            reference = sorted((p.line, p.column, p.rule or "syntax") for p in upstream_diagnostics)
            actual = sorted((p["line"], p["column"], p["rule"]) for p in rust)
            if actual == reference:
                row["all_locations"] += 1
            else:
                print(f"{case.id()} {data!r}\n  expected {reference}\n  actual   {actual}", file=sys.stderr)
            if [p for p in actual if p[2] == rule] == [p for p in reference if p[2] == rule]:
                row["rule_locations"] += 1

            reference_details = sorted(
                (p.line, p.column, p.rule or "syntax", p.level, p.desc.removesuffix(" (syntax)") if not p.rule else p.desc)
                for p in upstream_diagnostics
            )
            actual_details = sorted(
                (p["line"], p["column"], p["rule"], p["level"], p["message"]) for p in rust
            )
            if actual_details == reference_details:
                row["severity_and_message"] += 1
            elif actual == reference:
                print(
                    f"{case.id()} {data!r}\n  expected {reference_details}\n  actual   {actual_details}",
                    file=sys.stderr,
                )

        loader = unittest.defaultTestLoader
        with patch.object(RuleTestCase, "check", compare):
            suite = unittest.TestSuite(
                [
                    loader.discover("tests/rules", top_level_dir="."),
                    loader.loadTestsFromName("tests.test_syntax_errors"),
                ]
            )
            result = unittest.TextTestRunner().run(suite)

    columns = ("fixtures", "rule_locations", "all_locations", "severity_and_message", "cli_errors")
    print("rule", *columns, sep="\t")
    for rule, row in sorted(counts.items()):
        print(rule, *(row[column] for column in columns), sep="\t")
    return result.wasSuccessful() and all(row["fixtures"] == row["severity_and_message"] for row in counts.values())


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=lambda path: Path(path).resolve(strict=True))
    parser.add_argument("upstream", type=lambda path: Path(path).resolve(strict=True))
    args = parser.parse_args()
    sys.exit(0 if audit(args.binary, args.upstream) else 1)
