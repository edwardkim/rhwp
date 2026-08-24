---
kind: pr-review
status: accepted-pending-integration-pr-approval
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-24
---

# PR #6008 review - HWP3 셀 안여백 2022 사상 규칙 제거 (#5916)

## 접수 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#6008](https://github.com/edwardkim/rhwp/pull/6008) |
| 작성자 | [@planet6897](https://github.com/planet6897) |
| 원 head | `72001e836dc0626d59725015ed86ae180d2086de` |
| 통합 commit | `5bd0ed8aa` |
| GitHub 상태 | non-draft, `MERGEABLE/CLEAN`, CI·CodeQL 성공 |
| 판정 | **수용 권고** |

## 변경 검토

HWP3 셀 안여백을 한글 2022 사상 규칙으로 재해석하던 경로를 제거해 원본 padding을 그대로 보존한다.
`05434_vehicle_log_form.hwp`의 되살린 기호가 2쪽 서식 영역에서 3쪽으로 넘치는 문제를 회귀 테스트로
고정한다.

## 로컬 검증

- `issue_5557_hwp3_cell_margin`: 2 passed
- `issue_5916_hwp3_cell_margin_pagefit`: 2 passed
- #5970 제외 후 `issue_1880` focused 회귀도 2 passed로 복구됨을 확인했다.
- 통합 후보 전체 검증에서 `cargo fmt`, suite manifest, unit-tier, clippy, 전체 nextest가 통과했다.
- 전체 nextest: `8304 tests run: 8304 passed (3 slow), 42 skipped`

## 권고

원 PR CI와 로컬 focused/전체 회귀가 모두 통과했다. #5970 대신 이번 통합 PR에 포함해 수용 가능하다.
