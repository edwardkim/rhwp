#!/usr/bin/env python3
"""Recompute decide() for every committed corpus row."""

from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from pathlib import Path

try:
    from .corpus_io import CORPUS_DIR, load_corpus, load_manifest
    from .decide import VERDICT_CLASSES, decide_observation
except ImportError:
    from corpus_io import CORPUS_DIR, load_corpus, load_manifest
    from decide import VERDICT_CLASSES, decide_observation


def verify(dir_path: Path) -> dict:
    manifest = load_manifest(dir_path)
    cases = load_corpus(dir_path)
    if len(cases) != manifest["rowCount"]:
        raise SystemExit(f"loaded {len(cases)} != manifest {manifest['rowCount']}")
    keys = set()
    counts: Counter[str] = Counter()
    for case in cases:
        key = case.identity_key()
        if key in keys:
            raise SystemExit(f"duplicate identity {case.case_id}")
        keys.add(key)
        got = decide_observation(case.observation())
        if got.verdict != case.verdict:
            raise SystemExit(f"{case.case_id}: {got.verdict} != {case.verdict}")
        if got.chain_accepted != case.chain_accepted:
            raise SystemExit(f"{case.case_id}: chainAccepted drift")
        if case.verdict not in VERDICT_CLASSES:
            raise SystemExit(f"{case.case_id}: unknown verdict {case.verdict}")
        lower = (case.implementer_claim + " " + case.broken_at).lower()
        for marker in ("lorem", "ipsum", "asdf", "qwerty", "padding", "xxx"):
            if marker in lower:
                raise SystemExit(f"{case.case_id}: padding marker {marker}")
        counts[case.verdict] += 1
    if set(counts) != set(VERDICT_CLASSES):
        raise SystemExit(f"verdict coverage {sorted(counts)} != {list(VERDICT_CLASSES)}")
    return {
        "ok": True,
        "rowCount": len(cases),
        "distinct": len(keys),
        "byVerdict": dict(sorted(counts.items())),
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--corpus", type=Path, default=CORPUS_DIR)
    args = parser.parse_args(argv)
    report = verify(args.corpus)
    json.dump(report, sys.stdout, ensure_ascii=False, indent=2)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
