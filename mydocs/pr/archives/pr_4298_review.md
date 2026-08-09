---
kind: pr-review
status: review-ready
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-09
---

# PR #4298 검토 - Issue workflow assignee 세션 잠금 문서화

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#4298](https://github.com/edwardkim/rhwp/pull/4298) / `kevin9327` |
| 관련 근거 | #3902, #3903, track J R93 |
| base / source head | `devel` / `e23f26d173d3445555637cd96fdd67ef4bd51a04` |
| 누적 적용 | `9aa5be010` |
| 접수 참고 상태 | MERGEABLE / CLEAN, 2 files, +24 / -4 |

## 판정

**수용 권고.** 이미 분산돼 있던 assignee=세션잠금 규약을 모든 작업 유형이 참조하는 canonical
workflow에 편입한다. 외부 기여자가 assignee를 설정할 수 없는 경우 comment 잠금 경로도 빠지지
않았다. 문서 변경만이며 source 링크와 roadmap generator 결과를 대조했다.

merge 전 최신 remote 상태를 확인한다. 공통 검증은 [통합 검토 계획](pr_4282_review_impl.md)에 있다.
