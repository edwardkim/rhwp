---
kind: pr-review
status: review-ready
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-09
---

# PR #4289 검토 - 표 캡션 sentinel 문단 편집

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#4289](https://github.com/edwardkim/rhwp/pull/4289) / `kevin9327` |
| 관련 이슈 | #4288 |
| base / source head | `devel` / `aa735f70bc286be45eef26d9620e49389a06c5e6` |
| 누적 적용 | `ec2733665` |
| 접수 참고 상태 | MERGEABLE / BLOCKED, 1 file, +92 / -6 |

## 판정

**수용 권고.** caption sentinel은 일반 셀 paragraph index가 아니므로 split/merge가 이를
일반 index처럼 처리하면 panic이 났다. 변경은 caption 존재를 검증한 뒤 전용 문단을 처리하고,
없는 caption은 오류로 반환한다. split과 merge focused regression, 전체 Rust 5,499건이 통과했다.

GitHub의 `BLOCKED` 표시는 작성 시점 required-check 참고값이다. 실제 merge 전 최신 head와
checks를 재확인한다. 공통 검증은 [통합 검토 계획](pr_4282_review_impl.md)에 있다.
