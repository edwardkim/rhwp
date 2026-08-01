#!/usr/bin/env python3
"""메일머지 — 서식 1 + CSV N행 → 산출물 N개.

한 건 실패로 전체를 멈추지 않는다. 실패한 행만 보고하고 나머지를 계속한다.

    python examples/08_mail_merge.py 서식.hwp 데이터.csv 출력폴더
"""

from __future__ import annotations

import csv
import sys
from pathlib import Path
from typing import Dict, List, Tuple

import rhwp


def load_rows(csv_path: str) -> List[Dict[str, str]]:
    # utf-8-sig: 엑셀이 저장한 CSV 의 BOM 을 흡수한다.
    with open(csv_path, encoding="utf-8-sig", newline="") as fh:
        return list(csv.DictReader(fh))


def merge(form: str, rows: List[Dict[str, str]], out_dir: Path) -> Tuple[List[Path], List[str]]:
    available = {f.name for f in rhwp.fields(form).fields}
    if not available:
        raise SystemExit(f"누름틀이 없는 서식입니다: {form}")

    made: List[Path] = []
    failed: List[str] = []

    for i, row in enumerate(rows, 1):
        # CSV 열 중 서식에 있는 것만 골라 넘긴다 — 없는 이름은 notFound 로 보고되지만
        # 미리 거르면 봉투가 깨끗하다.
        data = {k: v for k, v in row.items() if k in available}
        if not data:
            failed.append(f"{i}행: 서식과 겹치는 열이 없습니다 (열: {list(row)})")
            continue

        label = row.get("성명") or row.get("이름") or f"row{i}"
        target = out_dir / f"{i:04d}_{label}.hwp"

        try:
            result = rhwp.fill_fields(form, data, out=target, verify=True)
        except rhwp.RhwpError as exc:
            failed.append(f"{i}행: {exc}")
            continue

        verify = result.verify
        if verify is not None and not verify.identical:
            failed.append(f"{i}행: 저장본 검증 실패 (차이 {verify.diff_count})")
            continue
        if result.not_found:
            failed.append(f"{i}행: 채우지 못한 칸 {result.not_found}")
            continue

        made.append(target)
    return made, failed


def main(form: str, csv_path: str, out_dir: str) -> int:
    rows = load_rows(csv_path)
    if not rows:
        print(f"데이터가 없습니다: {csv_path}")
        return 1

    target_dir = Path(out_dir)
    target_dir.mkdir(parents=True, exist_ok=True)

    print(f"{len(rows)}행 처리 중…")
    made, failed = merge(form, rows, target_dir)

    print(f"\n성공 {len(made)} / 실패 {len(failed)}")
    for message in failed[:20]:
        print(f"  {message}")
    return 0 if not failed else 1


if __name__ == "__main__":
    if len(sys.argv) != 4:
        print(__doc__)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1], sys.argv[2], sys.argv[3]))
