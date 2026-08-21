---
kind: pr-review
status: approved-integration-candidate
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-21
---

# PR #5829 검토 - renderer bughunt r5

## 판정

- source head `f8653e535a61aa7ff871ab88e7bdee6e4533b741`를 적용했다.
- TAC baseline, PUA 이중선, 중첩표 horizontal offset, TAC 표 anchor line spacing의 네 회귀를 보정한다.

## 검증

- `issue_5789_tac_line_shape_baseline`, `issue_5793_pua_f0827_double_rule`, `issue_5787_nested_square_table_horz_offset`, `issue_5788_tac_table_anchor_line_spacing`은 각각 1/1 통과했고, 통합 전체 nextest도 통과했다.
- HWP 2020 PDF p1 visual sweep의 #5787은 pixel 94.99474%/proxy 13.39698%, #5788은 pixel 94.42491%/proxy 41.08884%였다. 사람 대조에서 표 기하 구조는 유지되고 텍스트 raster 차이가 남는 것을 확인했다. 대표 asset은 #5824 검토 문서에 보존했다.

## 최종 판단과 GitHub 기록

- **수용**: #5844 전체 CI와 r5의 네 직접 회귀 계약이 모두 성공했다. 관찰된 텍스트 raster 차이는 기준 PDF와 SVG의 기존 차이로 기록했으며 이 PR의 보류 사유나 추가 코드 보정은 아니다.
- merge 뒤 원 PR에는 4개 renderer 회귀와 CI 수용 근거를 comment로 남기고 close한다.
