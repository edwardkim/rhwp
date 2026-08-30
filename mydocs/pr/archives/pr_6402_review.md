---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6402
issue: 5184
author: kevin9327
---

# PR #6402 review - HWP3 빈 셀 vertsize 보존

## Metadata

| 항목 | 확인값 |
| --- | --- |
| 원 PR head / 통합 적용 | `c0841510724a7ef0966923e75184904fd7b126e7` / `cd5fcec` |
| 규모 | 3 files, `+64/-1`, 1 commit |
| 작성 시점 원 PR 상태 | Open, non-draft, check `SUCCESS`, `mergeStateStatus=UNKNOWN` |

## 검토와 판단

- HWP3에서 저장된 빈 셀 `vertsize=1000`을 TAC 표 높이로 덮어쓰지 않도록 document 변환 경로를 보정하고, `issue_5184_hwp3_empty_cell_vertsize` integration contract를 추가했다.
- 새 기준 PDF나 시각 fixture를 추가하지 않아 visual sweep 강제 조건에는 해당하지 않는다. 통합 후보 full nextest에 해당 contract가 포함돼 통과했다.
- 원 PR comment는 Codex quota 자동 안내 1건뿐이다.

**수용.** 저장값 보존과 regression contract가 직접 대응한다. merge 전 최신 head CI를 재확인한다.
