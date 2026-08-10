"""10k 코퍼스에 rhwp 쪽수를 매겨 TSV 로 남긴다 — ASCII 표 교정의 쪽수 영향 A/B 용.

오라클(COM) 없이 **rhwp 쪽수만** 잰다. 표를 바꾸기 전후로 두 번 돌려 diff 하면 "그 변경이
쪽을 옮기나"를 곧장 안다(ASCII 폭은 쪽수 레버리지가 ~0 이라는 주장을 실측으로 검증).

    python tools/hwpctrl_compat/pagecount_sweep.py --out output/poc/hwpctrl/pagecount/before.tsv

`--limit N` 으로 앞 N 개만. 코퍼스 루트는 `C:/Users/planet/hwpdocs`.
"""

from __future__ import annotations

import argparse
import io
import json
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
EXE = REPO / "target" / "release" / "rhwp.exe"
CORPUS = Path("C:/Users/planet/hwpdocs")
FINAL = REPO / "output" / "poc" / "pi_page_hwpdocs_10k" / "final.tsv"


def samples(limit: int | None) -> list[str]:
    rows = []
    for line in io.open(FINAL, encoding="utf-8"):
        if line.startswith("#") or line.startswith("sample\t"):
            continue
        rel = line.split("\t", 1)[0].strip()
        if rel:
            rows.append(rel)
        if limit and len(rows) >= limit:
            break
    return rows


def page_count(path: Path) -> int | None:
    proc = subprocess.run(
        [str(EXE), "info", str(path), "--json"], capture_output=True, check=False
    )
    if proc.returncode != 0:
        return None
    try:
        return json.loads(proc.stdout.decode("utf-8", "replace")).get("pageCount")
    except json.JSONDecodeError:
        return None


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--out", required=True)
    ap.add_argument("--limit", type=int)
    args = ap.parse_args()

    rows = samples(args.limit)
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    n = ok = 0
    with io.open(out, "w", encoding="utf-8", newline="\n") as fh:
        fh.write("sample\tpages\n")
        for rel in rows:
            n += 1
            pc = page_count(CORPUS / rel)
            if pc is not None:
                ok += 1
            fh.write(f"{rel}\t{pc}\n")
            if n % 500 == 0:
                print(f"  {n}/{len(rows)} · 성공 {ok}", flush=True)
    print(f"끝 — {n}개 중 {ok}개 쪽수 산출 → {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
