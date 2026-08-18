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
    from .decide import decide
    from .schema import VERDICTS, AbstainCase
except ImportError:
    from corpus_io import iter_axis_table, iter_corpus, load_manifest
    from decide import decide
    from schema import VERDICTS, AbstainCase

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
            errors.append(f"duplicate field tuple {key}")
        seen_keys.add(key)
        got = decide(case.fields)
        if got.verdict != case.expected:
            errors.append(f"{case.case_id} expected {case.expected} got {got.verdict}")
        if case.expected == "abstain" and not got.abstained:
            errors.append(f"{case.case_id} failed to abstain")
        if case.expected in {"pass", "fail"} and got.verdict == "abstain":
            errors.append(f"{case.case_id} invented abstain on consistent fields")
        if case.expected not in VERDICTS:
            errors.append(f"{case.case_id} unknown expected {case.expected}")
        if case.expected == "abstain" and not case.contradiction_id:
            errors.append(f"{case.case_id} abstain without contradiction_id")
        counts[case.expected] += 1
        if len(errors) >= 40:
            break

    axis_rows = 0
    for row in iter_axis_table():
        axis_rows += 1
        case = AbstainCase.from_mapping(row)
        got = decide(case.fields)
        if got.verdict != case.expected:
            errors.append(
                f"axis {case.case_id} expected {case.expected} got {got.verdict}"
            )

    manifest = load_manifest((corpus_dir or HERE / "corpus") / "manifest.json")
    if manifest.get("rowCount") != rows and len(errors) < 40:
        errors.append(f"manifest rowCount {manifest.get('rowCount')} != scanned {rows}")
    if rows < MIN_ROWS:
        errors.append(f"row count {rows} < {MIN_ROWS}")
    for needed in ("abstain", "pass", "fail"):
        if counts[needed] == 0:
            errors.append(f"missing {needed} rows")

    return {
        "ok": not errors,
        "rows": rows,
        "axisRows": axis_rows,
        "byVerdict": dict(sorted(counts.items())),
        "errors": errors[:40],
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--corpus", type=Path, default=None)
    args = parser.parse_args(argv)
    result = verify(args.corpus)
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0 if result["ok"] else 1


if __name__ == "__main__":
    sys.exit(main())
