# -*- coding: utf-8 -*-
"""두 roundtrip fidelity TSV(기준 vs 후보 바이너리)를 대조해 verdict 전이를 분류.

roundtrip_fidelity_harness.py 를 서로 다른 두 바이너리(예: 기준 devel vs 후보 수정본)로
동일 코퍼스에 실행한 두 TSV 를 sample 기준으로 join 하여, 각 문서의 verdict 전이를
분류한다. 후보 수정이 roundtrip fidelity 를 개선/회귀시키는지 정량화하는 데 쓴다.

분류(--base=기준, --new=후보):
  IMPROVED    base=PI_MOVED/PAGE_DELTA → new=SAME  (회귀 해소)
  REGRESSED   base=SAME → new=PI_MOVED/PAGE_DELTA  (신규 회귀)
  STILL_MOVED 양쪽 모두 non-SAME (수정 무관 divergence)
  STILL_SAME  양쪽 모두 SAME
  ERR_*       한쪽/양쪽 ERR

사용: python tools/roundtrip_fidelity_diff.py --base base.tsv --new candidate.tsv -o diff.tsv
"""
from __future__ import annotations

import argparse
import csv
from pathlib import Path


def load(p: Path) -> dict[str, dict]:
    rows = {}
    with open(p, encoding="utf-8", newline="") as fh:
        for r in csv.DictReader(fh, delimiter="\t"):
            rows[r["sample"]] = r
    return rows


NONSAME = {"PI_MOVED", "PAGE_DELTA"}


def classify(dv: str, nv: str) -> str:
    if dv == "ERR" or nv == "ERR":
        return "ERR_BOTH" if dv == nv else ("ERR_BASE" if dv == "ERR" else "ERR_NEW")
    if dv in NONSAME and nv == "SAME":
        return "IMPROVED"
    if dv == "SAME" and nv in NONSAME:
        return "REGRESSED"
    if dv in NONSAME and nv in NONSAME:
        return "STILL_MOVED"
    return "STILL_SAME"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--base", "--devel", dest="base", type=Path, required=True,
                    help="기준 바이너리 TSV")
    ap.add_argument("--new", type=Path, required=True, help="후보 바이너리 TSV")
    ap.add_argument("-o", "--out", type=Path, required=True)
    args = ap.parse_args()

    d, n = load(args.base), load(args.new)
    keys = sorted(set(d) | set(n))
    counts: dict[str, int] = {}
    with open(args.out, "w", encoding="utf-8", newline="") as fh:
        w = csv.writer(fh, delimiter="\t")
        w.writerow(["sample", "class", "base_verdict", "new_verdict",
                    "base_pages", "new_pages", "base_detail", "new_detail"])
        for k in keys:
            dr, nr = d.get(k), n.get(k)
            dv = dr["verdict"] if dr else "MISSING"
            nv = nr["verdict"] if nr else "MISSING"
            cls = classify(dv, nv) if dv != "MISSING" and nv != "MISSING" else "MISSING"
            counts[cls] = counts.get(cls, 0) + 1
            w.writerow([k, cls, dv, nv,
                        (dr or {}).get("hwp_pages", ""), (nr or {}).get("hwp_pages", ""),
                        (dr or {}).get("detail", ""), (nr or {}).get("detail", "")])
    total = sum(counts.values())
    print(f"[fidelity-diff] samples={total}")
    for cls in ["IMPROVED", "REGRESSED", "STILL_MOVED", "STILL_SAME",
                "ERR_BASE", "ERR_NEW", "ERR_BOTH", "MISSING"]:
        if counts.get(cls):
            print(f"  {cls:12s} {counts[cls]}")
    print(f"  → {args.out}")
    return 1 if counts.get("REGRESSED") else 0


if __name__ == "__main__":
    raise SystemExit(main())
