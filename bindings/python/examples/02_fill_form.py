#!/usr/bin/env python3
"""서식 채우기 — 검증까지 한 번에.

판정은 예외가 아니라 반환값이다. 저장본이 의도한 문서인지 봉투로 확인한다.

    python examples/02_fill_form.py 서식.hwp 제출본.hwp
"""

from __future__ import annotations

import sys

import rhwp


def main(source: str, target: str) -> int:
    available = {f.name for f in rhwp.fields(source).fields}
    if not available:
        print("누름틀이 없는 문서입니다.")
        return 1

    print(f"채울 수 있는 누름틀: {', '.join(sorted(available))}")

    data = {name: f"자동입력-{i}" for i, name in enumerate(sorted(available), 1)}
    result = rhwp.fill_fields(source, data, out=target, verify=True)

    print(f"\n채운 칸: {result.filled_count}")
    if result.not_found:
        print(f"못 찾은 이름: {result.not_found}")

    verify = result.verify
    if verify is None:
        print("검증을 요청하지 않았습니다.")
    elif verify.identical:
        print(f"검증 통과 — 저장본이 의도한 문서와 같습니다: {target}")
    else:
        # 도구는 정상 동작했다. 실패한 것은 문서에 대한 단언이다.
        print(f"검증 실패 — 차이 {verify.diff_count}건")
        return 3

    if result.changed_pages:
        print(f"눈으로 확인할 쪽: {result.changed_pages}")
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print(__doc__)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1], sys.argv[2]))
