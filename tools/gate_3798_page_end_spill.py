#!/usr/bin/env python
"""#3798 쪽 끝 스필 한도 — COM 없는 게이트 (`mydocs/report/task3798/README.md`).

r29 서베이(`output/poc/survey10k_r29_20260802/results/*.tsv`)가 문서마다 한글 쪽수와
어긋난 문단(`detail` 컬럼: `piN rhwp_pX|hwp_pY`)을 남겨 뒀다. 그 기록을 정답지로 삼아
**한글 COM 없이** 한도별 이득·회귀를 잰다.

두 가지를 잰다.

1. `--mode pi` — `n=1` 코호트(문단 하나만 어긋난 문서). 그 한 문단을 뺀 나머지 pi 는
   r29 에서 전부 한글과 맞았으므로, 기준선(한도 무한 = 현행 동작) 대비 **다른 pi 가
   움직이면 회귀**, 어긋났던 pi 가 한글 쪽으로 가면 **해소**다.
2. `--mode pages` — 쪽 총수. r29 의 `hwp_pages` 가 정답지다. 기준선과 한도 적용본의
   쪽수를 비교해 정답지에 가까워지면 해소, 멀어지면 회귀로 센다.

기준선은 같은 바이너리에 `RHWP_EXP_TRIMCAP` 을 아주 크게 줘서 만든다(현행 동작과
동치). r29 당시 바이너리와 devel 이 벌어졌을 수 있으므로, 기준선 쪽수가 r29 의
`rhwp_pages` 와 다른 문서는 `drift` 로 따로 세어 이득·회귀 집계에서 뺀다.
"""
from __future__ import annotations

import argparse
import csv
import glob
import os
import re
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
R29 = ROOT / "output/poc/survey10k_r29_20260802/results"
PG = re.compile(r"=== 페이지 (\d+) \(global_idx=\d+, section=(\d+),")
PI = re.compile(r"\bpi=(\d+)")
DETAIL = re.compile(r"pi(\d+) rhwp_p(\d+)\|hwp_p(\d+)(\[empty-caret\?\])?")

INF = "1000000000"


def rhwp_pi_pages(exe: Path, path: str, cap: str):
    """rhwp dump-pages → (절대 pi -> 시작쪽, 총쪽수) 또는 (None, 오류)."""
    env = dict(os.environ, RHWP_EXP_TRIMCAP=cap)
    try:
        out = subprocess.run([str(exe), "dump-pages", path], capture_output=True, text=True,
                             encoding="utf-8", errors="replace", timeout=300, env=env)
    except Exception as e:  # noqa: BLE001
        return None, f"rhwp:{e}"
    if out.returncode != 0:
        return None, "rhwp:rc"
    start: dict[tuple[int, int], int] = {}
    pages: set[int] = set()
    cur_page = cur_sec = 0
    max_pi: dict[int, int] = {}
    for ln in out.stdout.splitlines():
        m = PG.search(ln)
        if m:
            cur_page, cur_sec = int(m.group(1)), int(m.group(2))
            pages.add(cur_page)
            continue
        if "[미주]" in ln:
            continue
        q = PI.search(ln)
        if q and cur_page:
            pi = int(q.group(1))
            key = (cur_sec, pi)
            if key not in start or cur_page < start[key]:
                start[key] = cur_page
            max_pi[cur_sec] = max(max_pi.get(cur_sec, 0), pi)
    if not pages:
        return None, "rhwp:nopages"
    offsets, acc = {}, 0
    for s in sorted({s for s in max_pi}):
        offsets[s] = acc
        acc += max_pi[s] + 1
    absmap = {offsets.get(s, 0) + pi: pg for (s, pi), pg in start.items()}
    return (absmap, len(pages)), None


def load_r29() -> list[dict]:
    paths: dict[str, str] = {}
    for f in glob.glob(str(ROOT / "output/poc/survey10k_r29_20260802/chunks/*.txt")):
        for line in open(f, encoding="utf-8", errors="replace"):
            p = line.strip()
            if p:
                paths.setdefault(os.path.basename(p), p)
    rows = []
    for f in glob.glob(str(R29 / "chunk_*.tsv")):
        with open(f, encoding="utf-8", errors="replace") as fh:
            r = csv.reader(fh, delimiter="\t")
            next(r, None)
            for row in r:
                if len(row) < 12:
                    continue
                p = paths.get(row[0])
                if not p:
                    continue
                rows.append({"path": p, "verdict": row[1], "rhwp_pages": row[2],
                             "hwp_pages": row[3], "n": row[4], "detail": row[11]})
    return rows


def cohort_pi(rows) -> list[dict]:
    """n=1 · non-caret · rhwp 가 한 쪽 일찍 얹은 문서."""
    out = []
    for r in rows:
        if r["verdict"] != "PI_MISMATCH" or r["n"] != "1":
            continue
        m = DETAIL.search(r["detail"] or "")
        if not m or m.group(4):
            continue
        pi, rp, hp = int(m.group(1)), int(m.group(2)), int(m.group(3))
        if hp - rp != 1:
            continue
        out.append({**r, "pi": pi, "rhwp_p": rp, "hwp_p": hp})
    return out


def measure(exe, docs, cap, jobs):
    def one(d):
        res, err = rhwp_pi_pages(exe, d["path"], cap)
        return d["path"], (res, err)
    with ThreadPoolExecutor(max_workers=jobs) as ex:
        return dict(ex.map(one, docs))


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--mode", choices=["pi", "pages"], required=True)
    ap.add_argument("--cap", required=True, help="RHWP_EXP_TRIMCAP 값(px)")
    ap.add_argument("--exe", default=str(ROOT / "target/debug/rhwp.exe"))
    ap.add_argument("--jobs", type=int, default=4)
    ap.add_argument("--limit", type=int, default=0, help="pages 모드 표본 수(0=전체)")
    ap.add_argument("--out", default="")
    a = ap.parse_args()

    exe = Path(a.exe)
    rows = load_r29()
    docs = cohort_pi(rows) if a.mode == "pi" else [
        r for r in rows if r["verdict"] in ("MATCH", "PAGE_DELTA", "PI_MISMATCH",
                                            "PI_MISMATCH_CARET") and r["hwp_pages"].isdigit()
    ]
    if a.mode == "pages" and a.limit:
        docs = sorted(docs, key=lambda r: r["path"])[:: max(1, len(docs) // a.limit)][:a.limit]
    print(f"대상 {len(docs)}건 · 한도 {a.cap}px · exe {exe}", flush=True)

    base = measure(exe, docs, INF, a.jobs)
    cur = measure(exe, docs, a.cap, a.jobs)

    recs, fixed, broke, drift, err = [], 0, 0, 0, 0
    for d in docs:
        (b, be), (c, ce) = base[d["path"]], cur[d["path"]]
        if be or ce:
            err += 1
            continue
        bmap, bpg = b
        cmap, cpg = c
        if str(bpg) != d["rhwp_pages"]:
            drift += 1
            continue
        if a.mode == "pi":
            moved = {k for k in set(bmap) | set(cmap) if bmap.get(k) != cmap.get(k)}
            target = d["pi"]
            hit = cmap.get(target) == d["hwp_p"]
            others = moved - {target}
            if hit and not others:
                fixed += 1
                verdict = "FIXED"
            elif others:
                broke += 1
                verdict = f"MOVED_OTHERS({len(others)})"
            else:
                verdict = "SAME"
            recs.append((d["path"], verdict, d["pi"], bmap.get(target), cmap.get(target),
                         d["hwp_p"], bpg, cpg))
        else:
            hwp = int(d["hwp_pages"])
            if bpg == cpg:
                continue
            if cpg == hwp and bpg != hwp:
                fixed += 1
                verdict = "FIXED"
            elif bpg == hwp and cpg != hwp:
                broke += 1
                verdict = "BROKE"
            else:
                verdict = "MOVED"
            recs.append((d["path"], verdict, "", "", "", hwp, bpg, cpg))

    print(f"해소 {fixed} · 회귀 {broke} · 기준선drift {drift} · 오류 {err}")
    if a.out:
        with open(a.out, "w", encoding="utf-8", newline="") as fh:
            w = csv.writer(fh, delimiter="\t")
            w.writerow(["path", "verdict", "pi", "base_p", "cap_p", "hwp", "base_pages", "cap_pages"])
            w.writerows(recs)
        print(f"기록 {a.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
