---
kind: pr-review
status: accepted-pending-integration-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-23
---

# PR #5909 검토 - 빈 host 표의 선언 초과 tail 흡수 (#5906)

## 접수와 범위

| 항목 | 확인값 |
| --- | --- |
| PR / 작성자 | [#5909](https://github.com/edwardkim/rhwp/pull/5909) / [@kevin9327](https://github.com/kevin9327) |
| base / source head | `devel` / `2c60993db5e2d05c47db2b0435b4c73b57535df6` |
| 규모 | 7 files, +429 / -19, 1 commit |
| 접수 상태 | non-draft, reviewer `@jangster77` 지정, 작성 시점 `MERGEABLE/CLEAN` |

빈 host가 자리만 차지하는 표에서 마지막 행이 저장 선언보다 큰 높이를 흡수하도록 하여 2쪽 문서가
불필요하게 3쪽으로 갈라지는 layout 결함을 고친다. source commit 1/1이 통합 후보에 적용됐다.

## 검증과 증적

- 최신 source head의 check는 23 success, 2 skipped, failure 0이다.
- 통합 code candidate의 전체 nextest 8,201 passed와 fmt, clippy 결과를 재사용했다. 현 head의
  merge-tree, `git diff --check`, fmt, unit-tier도 통과했다.
- [전·후·한글 2022 대조](../../report/edit_demo_5906/float_stack_defer_p2.png)는 표가 기준과 같은 2쪽
  흐름으로 유지됨을 보여 준다.

## 판정

**수용 권고.** renderer 회귀시험과 source CI, 통합 회귀 및 보존된 시각 증적이 같은 결론이다. 통합 PR
최신 head의 CI 성공과 작업지시자 승인 전에는 merge하지 않는다.
