---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6407
issue: 6368
author: kevin9327
---

# PR #6407 review - 표 행 컷 0.1px 관용 contract

## Metadata

| 항목 | 확인값 |
| --- | --- |
| 원 PR head / 통합 적용 | `bf6858e8fc7a6ee951cfbdd83c2ade9e75f74ece` / `fbddb00` |
| 규모 | 1 file, `+72/-0`, 1 commit |
| 작성 시점 원 PR 상태 | Open, non-draft, check `SUCCESS`, `mergeStateStatus=UNKNOWN` |

## 검토와 판단

- `issue_6368_row_cut_fp_epsilon`이 0.07px 코드줄 이월을 0.1px tolerance 안에서 고정한다. 구현 변경이 아닌 regression contract 추가다.
- 기준 HWP/HWPX/PDF나 renderer source가 바뀌지 않아 visual sweep은 불필요하다. 통합 후보 full nextest에서 통과했고, comment는 자동 quota 안내뿐이다.

**수용.** 부동소수점 경계의 의도된 관용을 명시적 test로 남긴다.
