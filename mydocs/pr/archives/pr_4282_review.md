---
kind: pr-review
status: review-ready
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-09
---

# PR #4282 검토 - 표 span u16 overflow 방어

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#4282](https://github.com/edwardkim/rhwp/pull/4282) / `kevin9327` |
| 관련 이슈 | #4264 |
| base / source head | `devel` / `416e62acad9c9f2712892abfef31e250b1cf9e5c` |
| 누적 적용 / 보정 | `e5965dc2a`, `83b5dd93e` |
| 접수 참고 상태 | MERGEABLE / CLEAN, 2 files, +101 / -14 |

## 판정

**수용 권고, 메인터너 보정 포함.** contributor가 발견한 span 끝 계산과 행·열 편집의
overflow DoS 경로는 실제로 닫혔다. 다만 `saturating_add`만 사용하면 table count와 span 변경이
포화한 상태로 mutation을 계속해 불일치가 남을 수 있었다. 보정은 edit 시작 전에 모든 `u16`
증가와 span 끝을 검사해 오류로 중단하므로 partial mutation을 만들지 않는다.

`model::table::tests` 79건과 누적 Rust 회귀 5,499건이 통과했다. renderer/fixture 변경이 없어
별도 시각 sweep은 대상이 아니다. 실제 merge 전 최신 source head와 required checks를 재확인한다.

공통 누적 순서·검증: [통합 검토 계획](pr_4282_review_impl.md).
