#!/usr/bin/env python3
"""변환 감사 — 폴더 전체를 HWPX 로 바꾸고 손실을 집계한다.

판정은 예외가 아니라 데이터다. 실패한 것만 세는 게 아니라 **무엇이 달라졌는지**
범주별로 모은다.

    python examples/09_convert_audit.py 입력폴더 출력폴더
"""

from __future__ import annotations

import sys
from collections import Counter
from pathlib import Path
from typing import Dict, List

import rhwp


def audit(source_dir: Path, target_dir: Path) -> Dict[str, List[str]]:
    target_dir.mkdir(parents=True, exist_ok=True)
    buckets: Dict[str, List[str]] = {"통과": [], "차이": [], "재파싱실패": [], "오류": []}
    categories: Counter[str] = Counter()

    docs = sorted(p for p in source_dir.rglob("*") if p.suffix.lower() == ".hwp")
    if not docs:
        print(f"변환할 .hwp 가 없습니다: {source_dir}")
        return buckets

    for doc in docs:
        out = target_dir / f"{doc.stem}.hwpx"
        try:
            result = rhwp.export_hwpx(doc, out=out, verify=True)
        except rhwp.RhwpError as exc:
            buckets["오류"].append(f"{doc.name}: {exc}")
            continue

        verify = result.verify
        if verify is None:
            buckets["오류"].append(f"{doc.name}: verify 를 요청했는데 보고가 없음")
        elif verify.reparse_error:
            buckets["재파싱실패"].append(f"{doc.name}: {verify.reparse_error}")
        elif verify.identical:
            buckets["통과"].append(doc.name)
        else:
            buckets["차이"].append(f"{doc.name}: {verify.diff_count}건")
            # 무엇이 달라졌는지 범주로 모은다.
            try:
                diff = rhwp.ir_diff(doc, out)
                for name in (diff.raw.get("categories") or {}):
                    categories[name] += 1
            except rhwp.RhwpError:
                pass

    if categories:
        print("\n차이 범주 (문서 수):")
        for name, count in categories.most_common():
            print(f"  {name}: {count}")
    return buckets


def main(source: str, target: str) -> int:
    buckets = audit(Path(source), Path(target))

    print("\n집계:")
    for label, items in buckets.items():
        print(f"  {label}: {len(items)}")

    for label in ("오류", "재파싱실패", "차이"):
        for item in buckets[label][:10]:
            print(f"  [{label}] {item}")

    problems = sum(len(buckets[k]) for k in ("차이", "재파싱실패", "오류"))
    return 0 if problems == 0 else 3   # 판정 실패는 exit 3 (도구 사전과 같은 어휘)


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print(__doc__)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1], sys.argv[2]))
