---
kind: pr_review
status: accepted-for-integrated-merge
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-08
---

# PR #4258 검토 - 셀 나누기 뒤 stale line_segs 재래핑

## 대상과 범위

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4258](https://github.com/edwardkim/rhwp/pull/4258) / @humdrum00001010 |
| contributor 원 head | `2e1ad89f2393c68f50ebe2dfcb9a2d44ea4dd73b` |
| base / 규모 | `devel`, 7개 파일, +500/-0 |
| 관련 이슈 | [#4138](https://github.com/edwardkim/rhwp/issues/4138) |
| 작성 시점 원격 상태 | `mergeable=MERGEABLE`, `mergeStateStatus=CLEAN` |

표 셀 분할 뒤 이전 폭 기준 `line_segs`가 남는 문제를 현재 폭으로 재래핑하고, vpos 사다리를
단조롭게 재구축한다. #4259 → #4260 → #4261의 선행 pagination stack이다.

## 검증과 판단

- 원 head의 `Build & Test`가 통과했다.
- `issue_4138_split_cell_stale_linesegs` 2건과 전체 `release-test --tests`가 통과했다.
- renderer/layout 변경이므로 #4265의 최신 Full CI 및 Native Skia 결과를 merge 전에 확인한다.

**통합 수용 권고.** 적용 순서는 #4258 → #4259 → #4260 → #4261이다.
