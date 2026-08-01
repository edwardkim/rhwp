#!/usr/bin/env python3
"""계획 실행 — 검사하고, 통과하면 원자적으로 적용한다.

여러 편집 중 하나라도 불가능하면 아무것도 저장하지 않는다.

    python examples/04_plan_runner.py 서식.hwp 제출본.hwp
"""

from __future__ import annotations

import sys

import rhwp


def main(source: str, target: str) -> int:
    form_fields = [f.name for f in rhwp.fields(source).fields]

    plan = rhwp.Plan(source, target)
    if form_fields:
        plan.fill_fields({form_fields[0]: "계획으로 입력"})
    plan.verify()

    # 1) 검사 — 디스크를 건드리지 않는다.
    preview = plan.check()
    if not preview.ok:
        print("계획에 문제가 있습니다:")
        print(preview.describe_violations())
        return 2

    print("검사 통과. 실행 예정:")
    for step in preview.preview:
        print(f"  {step.raw}")

    # 2) 실행 — 전 step 이 메모리에서 통과해야 저장한다.
    journal = plan.run()
    print(f"\n적용한 step: {len(journal.steps)}")
    verify = journal.verify
    if verify and verify.identical:
        print(f"검증 통과 — {target}")
        return 0
    print(f"검증 실패: {verify.raw if verify else '(보고 없음)'}")
    return 3


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print(__doc__)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1], sys.argv[2]))
