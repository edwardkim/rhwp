"""HWPX roundtrip 통제 비교 — 수정 전/후 배치 stdout 을 파일 단위로 대조.

Task #1591 v2 채택 게이트 (개선−회귀>0, 악화 0). `rhwp hwpx-roundtrip --batch` 의
per-file stdout 라인(`[ STATUS] diff= N r2= M ...ms 파일명`)을 파싱해 상태 전이를 집계한다.

사용:
    python tools/roundtrip_control_compare.py before.txt after.txt [--list-transitions]

판정 (파일별 before → after):
- 개선: severity 감소 (IR_DIFF→PASS, diff 감소, FAIL→IR_DIFF/PASS 등)
- 악화: severity 증가 (PASS→IR_DIFF, diff 증가, →FAIL 등)  ← 1건이라도 있으면 exit 1
- 공통 파일만 비교(한쪽에만 있는 파일은 별도 카운트).
"""
from __future__ import annotations

import argparse
import re
import sys

LINE = re.compile(r"^\[\s*([A-Z0-9_]+)\]\s+diff=\s*(\d+)\s+r2=\s*(\d+)\s+\d+ms\s+(.+)$")

SEVERITY_TIER = {
    "PASS": 0,
    "IR_DIFF": 1,
    "REPARSE_FAIL": 2,
    "SERIALIZE_FAIL": 3,
    "PARSE_FAIL": 4,
}


def parse(path: str) -> dict[str, tuple[str, int]]:
    out: dict[str, tuple[str, int]] = {}
    with open(path, encoding="utf-8", errors="replace") as f:
        for line in f:
            m = LINE.match(line.strip())
            if not m:
                continue
            status, diff, _r2, name = m.group(1), int(m.group(2)), m.group(3), m.group(4)
            out[name.strip()] = (status, diff)
    return out


def severity(status: str, diff: int) -> tuple[int, int]:
    return (SEVERITY_TIER.get(status, 5), diff if status == "IR_DIFF" else 0)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("before")
    ap.add_argument("after")
    ap.add_argument("--list-transitions", action="store_true", help="전이 파일 목록 출력")
    a = ap.parse_args()

    b = parse(a.before)
    f = parse(a.after)
    common = sorted(set(b) & set(f))
    only_b = len(set(b) - set(f))
    only_f = len(set(f) - set(b))

    improved: list[str] = []
    worsened: list[str] = []
    for name in common:
        sb, sf = severity(*b[name]), severity(*f[name])
        if sf < sb:
            improved.append(f"{name}: {b[name][0]}/{b[name][1]} → {f[name][0]}/{f[name][1]}")
        elif sf > sb:
            worsened.append(f"{name}: {b[name][0]}/{b[name][1]} → {f[name][0]}/{f[name][1]}")

    print(f"공통 {len(common)}건 (before-only {only_b}, after-only {only_f})")
    print(f"개선 {len(improved)} / 악화 {len(worsened)} / 순효과 {len(improved) - len(worsened)}")
    if a.list_transitions or worsened:
        for line in improved:
            print(f"  개선  {line}")
        for line in worsened:
            print(f"  악화  {line}")

    return 1 if worsened else 0


if __name__ == "__main__":
    sys.exit(main())
