---
kind: pr-review
status: accepted-pending-integration-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-23
---

# PR #5928 검토 - 하단 고정 앵커의 불확실 마진 (#5924)

## 접수와 범위

| 항목 | 확인값 |
| --- | --- |
| PR / 작성자 | [#5928](https://github.com/edwardkim/rhwp/pull/5928) / [@kevin9327](https://github.com/kevin9327) |
| base / source head | `devel` / `27da1cf35623534a4209da96165fa3848fe8595e` |
| 규모 | 6 files, +169 / -14, 1 commit |
| 접수 상태 | non-draft, reviewer `@jangster77` 지정, 작성 시점 `MERGEABLE/CLEAN` |

복원 과정이 위로 이동하지 않은 하단 고정 앵커에 불확실 마진을 잘못 더해 ghost page를 만드는 renderer
결함을 고친다. source commit 1/1이 통합 후보에 적용됐다.

## 검증과 증적

- source head의 check는 23 success, 2 skipped, failure 0이다.
- 통합 code candidate 전체 nextest 8,201 passed와 현 head merge-tree·공백·fmt·unit-tier 검사를 통과했다.
- [수정 전 ghost page](../../report/edit_demo_5924/task2098_before_ghost_page2.png)와
  [전·후·한글 2020 대조](../../report/edit_demo_5924/task2098_margin_split_before_after.png)는 수정 후
  여분 페이지가 없어지고 footer/frame이 기준의 1쪽 흐름으로 들어오는 것을 보인다.

## 판정

**수용 권고.** ghost page 제거와 anchored footer 위치가 원 PR의 renderer 계약 및 기준 증적에 부합한다.
통합 PR 최신 CI와 작업지시자 승인이 merge 전 조건이다.
