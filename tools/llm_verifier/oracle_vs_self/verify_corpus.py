#!/usr/bin/env python3
"""Verify every committed corpus row against decide()."""

from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from pathlib import Path

try:
    from .corpus_io import iter_axis_table, iter_corpus, load_manifest
    from .decide import VERDICT_CLASSES, decide
    from .schema import parse_bool
except ImportError:
    from corpus_io import iter_axis_table, iter_corpus, load_manifest
    from decide import VERDICT_CLASSES, decide
    from schema import parse_bool

HERE = Path(__file__).resolve().parent
MIN_ROWS = 100_000


def verify(corpus_dir: Path | None = None) -> dict:
    errors: list[str] = []
    seen_ids: set[str] = set()
    seen_keys: set[tuple] = set()
    counts: Counter[str] = Counter()
    rows = 0
    for case in iter_corpus(corpus_dir):
        rows += 1
        if case.case_id in seen_ids:
            errors.append(f"duplicate case_id {case.case_id}")
        seen_ids.add(case.case_id)
        key = case.identity_key()
        if key in seen_keys:
            errors.append(f"duplicate identity {key}")
        seen_keys.add(key)
        got = decide(
            case.has_hangul_pdf,
            case.versions,
            case.page_count_match,
            case.render_self_pass,
            case.cheap_ok,
        )
        if got.verdict_class != case.expected_verdict_class:
            errors.append(
                f"{case.case_id} expected {case.expected_verdict_class} got {got.verdict_class}"
            )
        if got.honest_claim != case.honest_claim:
            errors.append(f"{case.case_id} honest_claim drift")
        if case.expected_verdict_class not in VERDICT_CLASSES:
            errors.append(f"{case.case_id} unknown class")
        counts[case.expected_verdict_class] += 1

    axis_rows = 0
    for row in iter_axis_table():
        axis_rows += 1
        got = decide(
            parse_bool(row["has_hangul_pdf"]),
            row["versions"],
            parse_bool(row["page_count_match"]),
            parse_bool(row["render_self_pass"]),
            parse_bool(row["cheap_ok"]),
        )
        if got.verdict_class != row["expected_verdict_class"]:
            errors.append(
                f"axis {row} expected {row['expected_verdict_class']} got {got.verdict_class}"
            )

    manifest = load_manifest((corpus_dir or HERE / "corpus") / "manifest.json")
    if manifest.get("rowCount") != rows:
        errors.append(f"manifest rowCount {manifest.get('rowCount')} != scanned {rows}")
    if rows < MIN_ROWS:
        errors.append(f"row count {rows} < {MIN_ROWS}")

    return {
        "ok": not errors,
        "rows": rows,
        "axisRows": axis_rows,
        "byVerdict": dict(sorted(counts.items())),
        "errorCount": len(errors),
        "errors": errors[:20],
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--corpus-dir", type=Path, default=None)
    args = parser.parse_args(argv)
    report = verify(args.corpus_dir)
    json.dump(report, sys.stdout, ensure_ascii=False, indent=2)
    sys.stdout.write("\n")
    return 0 if report["ok"] else 1


if __name__ == "__main__":
    if str(HERE) not in sys.path:
        sys.path.insert(0, str(HERE))
    raise SystemExit(main())
