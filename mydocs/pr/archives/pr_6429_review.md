---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6429
issue: 6104
author: kevin9327
---

# PR #6429 review - 자리차지 표 밴드와 TAC 제목

## Metadata

| 항목 | 확인값 |
| --- | --- |
| 원 PR head / 통합 적용 | `2070fb7f7ed3ea070fd29634fc80d3305e230bc1` / `103b4b5` |
| 규모 | 2 files, `+168/-0`, 1 commit |
| 작성 시점 원 PR 상태 | Open, non-draft, check `SUCCESS`, `mergeStateStatus=UNKNOWN` |

## 검토와 판단

- 이미 그린 `TopAndBottom` 표 exclusion band를 후속 TAC 제목 상자가 피하도록 layout을 보정한다. `issue_6104_tac_title_over_table`은 in-memory 문서로 표 하단 아래 제목 상단을 직접 측정한다.
- 실제 HWP/HWPX sample이나 canonical PDF가 PR에 추가되지 않았고, test가 정확한 geometry contract를 제공한다. visual sweep 강제 조건은 아니다. full nextest가 사전 통과했고 comment는 자동 quota 안내뿐이다.

**수용.** 선행 밴드만 회피해 anchor reservation을 이중 계상하지 않는 범위가 test로 고정됐다.
