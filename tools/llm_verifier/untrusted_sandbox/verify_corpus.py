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
    from .schema import parse_bool
    from .slot import SLOT_VALUES, Slot
except ImportError:  # python tools/.../verify_corpus.py
    _pkg_parent = str(Path(__file__).resolve().parent.parent)
    if _pkg_parent not in sys.path:
        sys.path.insert(0, _pkg_parent)
    from untrusted_sandbox.corpus_io import (  # type: ignore
        iter_axis_table,
        iter_corpus,
        load_manifest,
    )
    from untrusted_sandbox.decide import decide  # type: ignore
    from untrusted_sandbox.schema import parse_bool  # type: ignore
    from untrusted_sandbox.slot import SLOT_VALUES, Slot  # type: ignore

HERE = Path(__file__).resolve().parent
MIN_ROWS = 100_000


def verify(corpus_dir: Path | None = None) -> dict:
    errors: list[str] = []
    seen_ids: set[str] = set()
    seen_keys: set[tuple] = set()
    slot_counts: Counter[str] = Counter()
    block_counts: Counter[str] = Counter()
    rows = 0
    for case in iter_corpus(corpus_dir):
        rows += 1
        if case.case_id in seen_ids:
            errors.append(f"duplicate case_id {case.case_id}")
        seen_ids.add(case.case_id)
        key = case.contract_tuple()
        if key in seen_keys:
            errors.append(f"duplicate contract {case.case_id}")
        seen_keys.add(key)
        got = decide(
            case.slot,
            case.leaked_into_criteria,
            case.nonce,
            case.excerpt,
            case.source_label_kind,
            case.wrap_state,
            case.untrusted_content,
        )
        if got.expected_block != case.expected_block:
            errors.append(
                f"{case.case_id} expected_block {case.expected_block} got {got.expected_block}"
            )
        if got.fail_kinds_cell() != case.fail_kinds:
            errors.append(
                f"{case.case_id} fail_kinds {case.fail_kinds!r} got {got.fail_kinds_cell()!r}"
            )
        if case.slot not in SLOT_VALUES:
            errors.append(f"{case.case_id} unknown slot {case.slot}")
        if case.leaked_into_criteria and not case.expected_block:
            errors.append(f"{case.case_id} leak must block")
        if case.slot == Slot.CRITERIA.value and not case.expected_block:
            errors.append(f"{case.case_id} criteria slot must block")
        if "\t" in case.excerpt or "\n" in case.excerpt:
            errors.append(f"{case.case_id} excerpt must be a single TSV cell")
        slot_counts[case.slot] += 1
        block_counts["block" if case.expected_block else "allow"] += 1
        if len(errors) >= 40:
            break

    axis_rows = 0
    for row in iter_axis_table():
        axis_rows += 1
        got = decide(
            row["slot"],
            parse_bool(row["leaked_into_criteria"]),
            _axis_nonce(row["nonce_kind"]),
            _axis_excerpt(row["nonce_kind"]),
            row["source_label_kind"],
            row["wrap_state"],
            True,
        )
        if got.expected_block != parse_bool(row["expected_block"]):
            errors.append(
                f"axis {row} expected {row['expected_block']} got {got.expected_block}"
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
        "bySlot": dict(slot_counts),
        "byVerdict": dict(block_counts),
        "errors": errors[:40],
        "errorCount": len(errors),
    }


def _axis_nonce(kind: str) -> str:
    if kind == "collision":
        return "TOKEN16axis"
    if kind == "static":
        return "DOCUMENT"
    if kind == "empty":
        return ""
    if kind == "reused":
        return "reuse00deadbeef"
    return "0123456789abcdef"


def _axis_excerpt(kind: str) -> str:
    if kind == "collision":
        return "axis-collision-TOKEN16axis"
    if kind == "static":
        return "axis-static"
    if kind == "empty":
        return "axis-empty"
    if kind == "reused":
        return "axis-reused"
    return "axis-fresh"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--corpus", type=Path, default=None)
    args = parser.parse_args(argv)
    result = verify(args.corpus)
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0 if result["ok"] else 1


if __name__ == "__main__":
    sys.exit(main())
