---
kind: pr-review
status: local-accept-recommendation
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-11
---

# PR #4556 리뷰 - 다중 NewNumber 마지막 채택 회귀 고정

| 항목 | 검토 기록 |
| --- | --- |
| 원 PR | [#4556](https://github.com/edwardkim/rhwp/pull/4556) · @planet6897 |
| base / 원 head | `devel` / `f53da07c670102c1cc4d7ec389a89d69cd1b4b0e` |
| 규모 | 1 file, `+22/-0`, 1 commit |
| 작성 시점 상태 | OPEN, `MERGEABLE`, `CLEAN` (merge 직전 재확인 필요) |

## 검토와 판정

같은 문단의 여러 NewNumber에서 문서 순서상 마지막 컨트롤을 채택한다는 현행 규칙을 회귀 test로
고정한다. 코드 경로를 바꾸지 않으며, 새 test는 누적 branch에서 통과했다.

한컴의 첫/마지막 채택 규칙 자체는 재현 fixture가 제공될 때 별도로 대조해야 한다는 한계를 PR 설명과
일치하게 유지한다. 이 한계는 test 고정 PR의 차단 사유가 아니다.

**최종 권고: 최신 통합 head의 CI와 작업지시자 승인 후 수용.**
