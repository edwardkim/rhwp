#!/usr/bin/env python
"""#3738 트리거 census — 한 문단에 자리차지 개체가 둘 이상인 문서를 센다 (COM 불필요).

`rhwp info` 에 `RHWP_DIAG_TACSIB=1` 을 걸면 조판이 그 문단을 만날 때마다 한 줄 찍는다.

    DIAG_TACSIB pi=1920 ci=1 line_idx=1 add=222.1 first_seg=449.9
                                        └ 이 개체의 줄   └ 예전에 쓰던 첫 줄

`add` 와 `first_seg` 가 크게 벌어진 문서가 #3738 의 영향권이다. `export-svg` 없이 돌기
때문에 모집단 전수에 쓸 수 있다.

이 진단은 #3738 수정과 같이 들어간다 — 그 이전 바이너리로 돌리면 전건 0 이 나온다.

사용:
  python tools/tac_sibling_census.py --list <경로목록.txt> --exe <rhwp.exe> --out <tsv>
"""
import argparse
import csv
import io
import os
import subprocess
import sys
from pathlib import Path

sys.stdout.reconfigure(encoding="utf-8", errors="replace")


def probe(exe: str, src: Path, timeout: int):
    env = dict(os.environ)
    env["RHWP_DIAG_TACSIB"] = "1"
    try:
        r = subprocess.run([exe, "info", str(src)], capture_output=True,
                           timeout=timeout, env=env)
    except subprocess.TimeoutExpired:
        return None, None
    if r.returncode != 0:
        return None, None
    err = r.stderr.decode("utf-8", "replace")
    hits = [l for l in err.splitlines() if l.startswith("DIAG_TACSIB")]
    return len(hits), hits[:3]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--list", required=True)
    ap.add_argument("--exe", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--timeout", type=int, default=900)
    a = ap.parse_args()

    paths = [Path(l.strip()) for l in io.open(a.list, encoding="utf-8") if l.strip()]
    print(f"모수 {len(paths)}건", flush=True)
    hit_docs = 0
    with io.open(a.out, "w", encoding="utf-8", newline="") as fh:
        w = csv.writer(fh, delimiter="\t")
        w.writerow(["sample", "tacsib_paras", "sample_lines"])
        for i, p in enumerate(paths, 1):
            n, sample = probe(a.exe, p, a.timeout)
            w.writerow([p.name, "" if n is None else n, " | ".join(sample or [])])
            if n:
                hit_docs += 1
            if i % 50 == 0:
                print(f"  {i}/{len(paths)}  트리거 문서 {hit_docs}", flush=True)
                fh.flush()
    print(f"트리거 문서 {hit_docs}/{len(paths)} — 기록: {a.out}")


if __name__ == "__main__":
    main()
