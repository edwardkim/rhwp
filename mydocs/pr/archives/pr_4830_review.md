---
kind: pr-review
status: code-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4830 검토 - HWP5 문단·표·셀 상호재귀 깊이 상한

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4830](https://github.com/edwardkim/rhwp/pull/4830) · @kevin9327 |
| 원 head | `4772eb251cffc5f1648a1bcada6b5dcf5e070fd3` |
| 누적 순서·적용 SHA | 5/27 · `317449e87` |
| 통합 기준선 | `upstream/devel@6631e7057` |
| 충돌·의존성 | `body_text` 충돌 1건 해소 · 최신 orphan field-end 연결과 공존 |

문단→표→셀 재귀에 TLS 깊이 상한과 RAII guard를 넣어 export-structure 스택 오버플로를 차단한다.
리베이스 중 최신 `link_orphan_field_ends`를 보존하고 깊이 guard를 함께 적용했다. 두 회귀
`nested_table_recursion_is_depth_capped`, `orphan_field_end_links_to_the_open_field_id`가 각각 통과했고
전체 GitHub code CI도 성공했다. **수용 가능**으로 판정한다.
