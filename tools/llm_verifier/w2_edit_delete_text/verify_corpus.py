from __future__ import annotations
import csv
from collections import Counter
from pathlib import Path
from .logic import decide_row, COMMAND, FAMILY, CLAIM_ID

HERE = Path(__file__).resolve().parent
CORPUS = HERE / "corpus"
MIN_ROWS = 15000

def verify() -> dict:
    rows = 0
    seen = set()
    verdicts: Counter[str] = Counter()
    for path in sorted(CORPUS.glob("shard_*.tsv")):
        with path.open(encoding="utf-8", newline="") as fh:
            reader = csv.DictReader(fh, delimiter="\t")
            for rec in reader:
                rows += 1
                key = (rec["sample"], rec["f0"], rec["f1"], rec["f2"], rec["f3"], rec["verdict"])
                if key in seen:
                    raise SystemExit(f"duplicate {rec['case_id']}")
                seen.add(key)
                got = decide_row(rec["f0"], rec["f1"], rec["f2"], rec["f3"])
                if got != rec["verdict"]:
                    raise SystemExit(f"{rec['case_id']}: {got} != {rec['verdict']}")
                if rec["command"] != COMMAND or rec["family"] != FAMILY:
                    raise SystemExit(f"{rec['case_id']}: command/family drift")
                verdicts[got] += 1
    if rows < MIN_ROWS:
        raise SystemExit(f"{CLAIM_ID} rows {rows} < {MIN_ROWS}")
    return {"ok": True, "claim": CLAIM_ID, "rows": rows, "distinct": len(seen), "byVerdict": dict(sorted(verdicts.items()))}
