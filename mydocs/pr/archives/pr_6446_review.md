---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6446
author: kevin9327
---

# PR #6446 review - 묶음 Image hit-test의 표 셀 진입

## Metadata

| 항목 | 확인값 |
| --- | --- |
| 원 PR head / 통합 적용 | `ac91035bbfee66895bfd86407ca934e67c49c323` / `3628324` |
| 규모 | 2 files, `+82/-0`, 1 commit |
| 작성 시점 원 PR 상태 | Open, non-draft, check `SUCCESS`, `mergeStateStatus=UNKNOWN` |

## 검토와 판단

- group child Image의 page-wide bbox가 table cell cursor hit를 가로채지 않도록 cursor-rect query를 제한하고 `issue_4753_group_image_hittest`로 table cell 진입을 고정한다.
- interactive hit-test semantic 변경이며 HWP/PDF fidelity fixture를 추가하지 않는다. full nextest와 clippy 사전 검증이 통과했고 comment는 자동 quota 안내뿐이다.

**수용.** hit-test 영역의 과도한 포획을 regression contract가 직접 막는다.
