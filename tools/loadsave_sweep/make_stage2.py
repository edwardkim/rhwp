#!/usr/bin/env python3
"""2단계(버전 확대) 대상 목록 생성 — 1단계 실패군 전체 + OK군 등간격 표본.

실패군: verdict 가 OK/ORACLE_ORIG_FAIL 이 아닌 (문서×경로)가 하나라도 있는 문서.
단, 코퍼스 자체 문제(DRM·암호·비지원 포맷의 CONVERT_FAIL)만 있는 문서는 제외 —
다른 한글 버전으로 열어도 rhwp 판정이 달라질 수 없다.

산출: stage2 master(문서 목록) + 해당 문서의 oracle_tasks 부분집합.

사용:
    python make_stage2.py --master master.tsv --verdicts <v.tsv> --phase-a <a.ndjson> \
        --tasks oracle_tasks.tsv --out-master stage2.tsv --out-tasks stage2_tasks.tsv --sample-ok 400
"""
from __future__ import annotations

import argparse
import csv
import json
import sys
from pathlib import Path

CORPUS_ERR = ("DRM", "암호", "UNSUPPORTED")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--master", required=True)
    ap.add_argument("--verdicts", required=True)
    ap.add_argument("--phase-a", required=True)
    ap.add_argument("--tasks", required=True)
    ap.add_argument("--out-master", required=True)
    ap.add_argument("--out-tasks", required=True)
    ap.add_argument("--sample-ok", type=int, default=400)
    args = ap.parse_args()

    stderr_by_key: dict[tuple[str, str], str] = {}
    for l in open(args.phase_a, encoding="utf-8"):
        if l.strip():
            r = json.loads(l)
            if r.get("kind") != "meta":
                stderr_by_key[(r["docid"], r["route"])] = r.get("stderr", "")

    defect: set[str] = set()
    ok_docs: set[str] = set()
    for x in csv.DictReader(open(args.verdicts, encoding="utf-8"), delimiter="\t"):
        v = x["verdict"]
        if v in ("OK", "ORACLE_ORIG_FAIL"):
            ok_docs.add(x["docid"])
            continue
        if v == "CONVERT_FAIL":
            e = stderr_by_key.get((x["docid"], x["route"]), "")
            if any(t in e for t in CORPUS_ERR):
                continue  # 코퍼스 문제 — 버전 확대 무의미
        defect.add(x["docid"])
    ok_docs -= defect

    rows = []
    picked = set(defect)
    ok_sorted = sorted(ok_docs)
    if args.sample_ok > 0 and ok_sorted:
        step = max(len(ok_sorted) / args.sample_ok, 1)
        picked |= {ok_sorted[int(i * step)] for i in range(min(args.sample_ok, len(ok_sorted)))}

    n = 0
    with open(args.out_master, "w", encoding="utf-8", newline="\n") as f:
        for line in open(args.master, encoding="utf-8"):
            if line.split("\t", 1)[0] in picked:
                f.write(line)
                n += 1
    m = 0
    with open(args.out_tasks, "w", encoding="utf-8", newline="\n") as f:
        for line in open(args.tasks, encoding="utf-8"):
            if line.split(".", 1)[0] in picked:
                f.write(line)
                m += 1
    print(f"defect docs: {len(defect)}, ok-sample: {len(picked) - len(defect)}, "
          f"master rows: {n}, oracle tasks: {m}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
