---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6426
issue: 6358
author: kevin9327
---

# PR #6426 review - 음수 셀 padding 폴백

## Metadata

| 항목 | 확인값 |
| --- | --- |
| 원 PR head / 통합 적용 | `7aced56388a36c9cdec20741edd63ee6474cf5d8` / `30b162e` |
| 규모 | 3 files, `+66/-9`, 1 commit |
| 작성 시점 원 PR 상태 | Open, non-draft, check `SUCCESS`, `mergeStateStatus=UNKNOWN` |

## 검토와 판단

- 음수 cell pad를 결측 sentinel으로 보고 표 기본 0으로 clamp하되, 정상 양수 vertical pad는 보존한다. 두 경우를 `issue_6358_negative_cell_pad_clamp`가 고정한다.
- test는 in-memory model만 사용하며 HWP/HWPX/PDF fixture나 특정 문서 fidelity 주장이 없다. 따라서 visual sweep 강제 조건은 아니다. full nextest가 사전 통과했고 comment는 자동 quota 안내뿐이다.

**수용.** 결측값 처리와 정상값 보존이 같은 contract에 있다.
