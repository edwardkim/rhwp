#!/usr/bin/env python3
"""TEXT_MISMATCH 를 원인군으로 분류한다.

U+2007(FIGURE SPACE, COM 추출에서 &#8199;) 삽입이 파일럿에서 지배적이었다 —
양쪽에서 &#8199; 와 U+2007 을 제거했을 때 같아지면 그 군으로 묶는다.
공백류 전체를 무시하면 같아지는 군, 그래도 다른 군(실질 텍스트 차이)을 나눈다.

사용: python classify_mismatch.py --verdicts <verdicts.tsv> --texts <texts dir>
"""
from __future__ import annotations

import argparse
import re
import sys
from collections import Counter
from pathlib import Path

FIGSP = re.compile(r"&#8199;| ")
ANYSP = re.compile(r"&#\d+;|\s+")


def norm(raw: bytes) -> str:
    s = raw.decode("utf-8", "replace")
    s = s.replace("\r\n", "\n").replace("\r", "\n")
    return "\n".join(line.rstrip() for line in s.split("\n")).rstrip("\n")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--verdicts", required=True)
    ap.add_argument("--texts", required=True)
    ap.add_argument("--out", help="분류 결과 TSV (기본: verdicts 옆 mismatch_classes.tsv)")
    args = ap.parse_args()

    texts = Path(args.texts)
    rows = Path(args.verdicts).read_text(encoding="utf-8").splitlines()
    header = rows[0].split("\t")
    idx = {h: i for i, h in enumerate(header)}

    classes: Counter = Counter()
    by_route: dict[str, Counter] = {}
    out_rows = []
    for line in rows[1:]:
        c = line.split("\t")
        if "TEXT_MISMATCH" not in c[idx["verdict"]]:
            continue
        docid, route = c[idx["docid"]], c[idx["route"]]
        a = texts / f"{docid}.orig.txt"
        b = texts / f"{docid}.{route}.txt"
        if not a.is_file() or not b.is_file():
            cls = "text-missing"
        else:
            ta, tb = norm(a.read_bytes()), norm(b.read_bytes())
            if FIGSP.sub("", ta) == FIGSP.sub("", tb):
                cls = "figure-space-only"
            elif ANYSP.sub("", ta) == ANYSP.sub("", tb):
                cls = "whitespace-only"
            elif len(ANYSP.sub("", tb)) < len(ANYSP.sub("", ta)):
                cls = "content-loss"
            else:
                cls = "content-diff"
        classes[cls] += 1
        by_route.setdefault(route, Counter())[cls] += 1
        out_rows.append(f"{docid}\t{route}\t{cls}")

    out = Path(args.out) if args.out else Path(args.verdicts).with_name("mismatch_classes.tsv")
    out.write_text("\n".join(out_rows) + "\n", encoding="utf-8")
    print("전체:", dict(classes))
    for r in sorted(by_route):
        print(f"  {r}: {dict(by_route[r])}")
    print(f"-> {out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
