# -*- coding: utf-8 -*-
"""#2279 — footer 판정의 코드 좌표계 슬랙(slack_code) 추출 + S*_code 재계산.

RHWP_DIAG_SCAN FOOTER 진단(typeset.rs)으로 코호트 각 문서의 판정 변수
(sync_h/target_y/avail_after)를 뽑고, 전환점 실측(delta_px)과 결합해
S*_code = slack_code(m0) - delta_px 를 산출한다. 2026-07-15 실측:
S* 는 ~36.9px 균일 군집, 이탈치 = rhwp 슬랙 측정오차 (#2279 코멘트).

usage: python tools/task2279/code_slack_probe.py
       [--cohort tools/task2279/footer_cohort.tsv]
       [--flip tools/task2279/flip_results_20260715.tsv]
       [--exe target/debug/rhwp.exe]
COM 불필요 (전환점은 기준값 TSV 재사용). 전환점 재실측은 footer_flip_run.py.
"""
import argparse
import os
import re
import subprocess
import sys
from pathlib import Path


def default_exe() -> str:
    """플랫폼별 debug 바이너리 기본 경로 (Windows/macOS/Linux)."""
    name = "rhwp.exe" if os.name == "nt" else "rhwp"
    return str(Path("target") / "debug" / name)


sys.stdout.reconfigure(encoding="utf-8")

FOOTER = re.compile(
    r"DIAG_SCAN FOOTER pi=(\d+) anchor_vpos=(-?\d+) cur_h=([\d.-]+) target_y=([\d.-]+) "
    r"sync_h=([\d.-]+) block_h=([\d.-]+) v_off=([\d.-]+) avail=([\d.-]+) "
    r"avail_after=([\d.-]+) slack_code=([\d.-]+)"
)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--cohort", default="tools/task2279/footer_cohort.tsv")
    ap.add_argument("--flip", default="tools/task2279/flip_results_20260715.tsv")
    ap.add_argument("--exe", default=default_exe())
    a = ap.parse_args()

    paths = {}
    for ln in Path(a.cohort).read_text(encoding="utf-8-sig").splitlines():
        ln = ln.strip()
        if ln:
            p = ln.split("\t")[0]
            paths[Path(p).name.split("_")[0]] = p

    flip = {}
    for ln in Path(a.flip).read_text(encoding="utf-8").splitlines()[1:]:
        f = ln.split("\t")
        if len(f) >= 8 and f[4] in ("OK", "NOFLIP"):
            # flip_results_20260715.tsv 컬럼: doc label p0 rhwp flip margin0 m* delta ...
            flip[f[0]] = (f[2], f[4], f[7])

    env = dict(os.environ)
    env["RHWP_DIAG_SCAN"] = "1"
    print("doc\ttruth_p0\tflip\tdelta_px\tslack_code\ts_star_code")
    for doc, (p0, flipst, delta) in sorted(flip.items()):
        path = paths.get(doc)
        if not path:
            continue
        out = subprocess.run(
            [a.exe, "dump-pages", path],
            capture_output=True, text=True, encoding="utf-8", errors="replace",
            timeout=300, env=env,
        )
        hits = FOOTER.findall(out.stderr)
        if not hits:
            print(f"{doc}\t{p0}\t{flipst}\t{delta}\tNO_DIAG")
            continue
        slack_code = float(hits[0][9])
        s_star = f"{slack_code - float(delta):.2f}" if flipst == "OK" else ""
        print(f"{doc}\t{p0}\t{flipst}\t{delta}\t{slack_code:.2f}\t{s_star}")


if __name__ == "__main__":
    main()
