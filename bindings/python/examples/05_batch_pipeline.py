#!/usr/bin/env python3
"""대량 처리 — 부분 실패를 잃지 않고 집계한다.

    python examples/05_batch_pipeline.py 폴더
"""

from __future__ import annotations

import sys
from pathlib import Path

import rhwp


def main(folder: str) -> int:
    root = Path(folder)
    paths = sorted(p for p in root.rglob("*") if p.suffix.lower() in (".hwp", ".hwpx"))
    if not paths:
        print(f"처리할 문서가 없습니다: {root}")
        return 1

    print(f"{len(paths)}개 문서 처리 중…")
    records = rhwp.batch("export-text", paths)

    ok, failed = [], []
    for r in records:
        (failed if "error" in r else ok).append(r)

    print(f"\n성공 {len(ok)} / 실패 {len(failed)}")
    total_pages = sum(r.get("pageCount", 0) for r in ok)
    print(f"총 쪽수: {total_pages}")

    # 실패해도 성공분은 남는다 — 스트림을 통째로 버리지 않는다.
    for r in failed[:10]:
        print(f"  실패: {r.get('source')} — {r.get('error')}")
    return 0 if not failed else 1


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print(__doc__)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1]))
