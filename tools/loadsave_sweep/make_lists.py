#!/usr/bin/env python3
"""코퍼스를 스캔해 스윕 마스터 목록(master.tsv)을 만든다.

행 형식: docid \t format(hwp|hwpx) \t abspath
docid 는 정렬된 relpath 순서의 5자리 순번 — 모든 후속 산출물(변환본·텍스트·판정)이
이 id 로 연결되므로, 같은 코퍼스 루트라면 재실행해도 id 가 변하지 않는다.

파일럿 표본은 --take-hwp/--take-hwpx 로 뽑는다. 앞에서 N개가 아니라 등간격 추출이라
코퍼스 전체의 다양성이 표본에 실린다.

    python make_lists.py --root D:\\hwpdocs_10k_share --out master.tsv
    python make_lists.py --root D:\\hwpdocs_10k_share --out pilot.tsv --take-hwp 20 --take-hwpx 20
"""
from __future__ import annotations

import argparse
import sys
from pathlib import Path


def spaced_sample(items: list, n: int) -> list:
    if n <= 0 or n >= len(items):
        return items
    step = len(items) / n
    return [items[int(i * step)] for i in range(n)]


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--take-hwp", type=int, default=0, help="hwp 등간격 표본 수 (0=전체)")
    ap.add_argument("--take-hwpx", type=int, default=0, help="hwpx 등간격 표본 수 (0=전체)")
    args = ap.parse_args()

    root = Path(args.root)
    if not root.is_dir():
        print(f"root not found: {root}", file=sys.stderr)
        return 2

    # docid 는 전체 코퍼스의 정렬 순서에서 나온다 — 표본을 떠도 id 가 전수 목록과 일치해,
    # 파일럿에서 발견한 문서를 전수 스윕 결과에서 같은 id 로 찾을 수 있다.
    by_fmt: dict[str, list[tuple[str, str, str]]] = {"hwp": [], "hwpx": []}
    all_files = sorted(
        (p for p in root.rglob("*") if p.is_file() and p.suffix.lower() in (".hwp", ".hwpx")),
        key=lambda p: str(p.relative_to(root)).lower(),
    )
    for i, p in enumerate(all_files):
        fmt = p.suffix.lower().lstrip(".")
        by_fmt[fmt].append((f"{i:05d}", fmt, str(p)))

    rows = spaced_sample(by_fmt["hwp"], args.take_hwp) + spaced_sample(by_fmt["hwpx"], args.take_hwpx)
    rows.sort(key=lambda r: r[0])

    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    with out.open("w", encoding="utf-8", newline="\n") as f:
        for docid, fmt, path in rows:
            f.write(f"{docid}\t{fmt}\t{path}\n")
    print(f"{out}: {len(rows)} rows (hwp {sum(1 for r in rows if r[1]=='hwp')}, "
          f"hwpx {sum(1 for r in rows if r[1]=='hwpx')})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
