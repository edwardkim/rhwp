---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-22
---

# PR #5902 검토 - continuation table fragment grid

| 항목 | 확인값 |
| --- | --- |
| PR / 작성자 | [#5902](https://github.com/edwardkim/rhwp/pull/5902) / `@planet6897` |
| 관련 issue | #5877 |
| source head | `968785dfed9853e197799a47bc761d3fca5bdc0d` |
| 상태 | non-draft, `CLEAN`, source CI 완료 |
| 적용 commit | `0cb9cbe4f0`, `afd5a55189` |

## 검토

- 쪽을 넘은 표 조각이 조각-지역 행 인덱스로 원본 전체 행의 `row_col_x`를 읽어, 제목 행의
  균등 보간 격자를 유령 세로 괘선으로 그리던 문제를 조각 순서의 grid로 고친다.
- `issue_5877_fragment_ghost_vrules`는 15쪽 조각 상단에서 유령 x 좌표
  `58.9/439.3/481.6/523.9/566.1`가 없어지고 실제 cell 경계가 남는지를 검사한다.
- source CI의 Full Rust archive, Native Skia, Canvas visual diff, CodeQL, Adapter, Proptest가
  성공했고, 통합 focused contract 1건과 전체 nextest를 다시 통과했다.

## 판정

**통합 후보 수용.** 테두리 좌표만 바꾸며 fixture의 쪽수에는 영향이 없고, 실제 cell 경계를
보존하는 계약으로 regressions를 막는다.
