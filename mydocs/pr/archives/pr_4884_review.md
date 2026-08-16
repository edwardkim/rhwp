---
kind: pr-review
status: code-ci-running
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4884 검토 — HWP3 출처 추정과 파일 여백 분리

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4884](https://github.com/edwardkim/rhwp/pull/4884), @planet6897 |
| 원 head | `613239a4b42362c7801b8cc077e132a6422be8c3` |
| 통합 적용 | `b7195286b` |
| 기준 | `upstream/devel@ae5f2a345` |

HWP3 식별 휴리스틱이 직렬화될 `margin_bottom`을 수정하던 경로를, 렌더 페이지네이션에만 쓰는
`pagination_bottom_tolerance`로 옮겼다. 따라서 출처 추정은 유지하면서 원본 파일 여백 값은 보존한다.
기존 HWP3 origin 회귀 3건과 통합 `nextest` 전체 게이트가 통과했다.

저장 값 보존 수정이며 별도 SVG/PDF fixture를 새로 만들지 않았다. 통합 PR
[#4936](https://github.com/edwardkim/rhwp/pull/4936)의 최초 코드 후보 CI는 녹색이었다. 최신 devel 동기화 뒤
docs head의 필수 CI와 head 동일성을 다시 확인하면 **수용 가능**이다.
