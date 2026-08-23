---
kind: pr-review
status: accepted-pending-integration-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-23
---

# PR #5926 검토 - 검정 밑줄색 보존 (#5925)

## 접수와 범위

| 항목 | 확인값 |
| --- | --- |
| PR / 작성자 | [#5926](https://github.com/edwardkim/rhwp/pull/5926) / [@kevin9327](https://github.com/kevin9327) |
| base / source head | `devel` / `547ea8a1273185f03ef55b7dd7243313f2ca480a` |
| 규모 | 6 files, +123 / -15, 2 commits |
| 접수 상태 | non-draft, reviewer `@jangster77` 지정, 작성 시점 `MERGEABLE/CLEAN` |

HWP COLORREF `0`을 미지정이 아닌 검정으로 해석해 SVG, Skia, paint JSON이 저장된 밑줄색을 보존한다.
source commit 2/2가 통합 후보에 적용됐다.

## 검증과 증적

- source head의 check는 21 success, 1 neutral, 3 skipped, failure 0이다.
- 통합 code candidate 전체 nextest 8,201 passed와 현재 merge-tree·fmt·unit-tier 검사를 통과했다.
- [hwpx_sample2 12쪽](../../report/assets/issue_5925_underline_color_black/before_after_hwpx_sample2_p12.png)과
  [pr-1674 7쪽](../../report/assets/issue_5925_underline_color_black/before_after_pr1674_p7.png) 모두 수정 후
  `#000000` 밑줄이 기준과 일치하고, 글자색으로 대체되던 수정 전 상태가 사라졌음을 보여 준다.

## 판정

**수용 권고.** 세 backend의 색상 계약과 두 기준 문서 시각 증적, source CI 및 통합 회귀가 일치한다.
통합 PR 최신 CI와 작업지시자 승인 후 merge한다.
