---
kind: pr-review
status: accepted-pending-integration-pr-approval
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-24
---

# PR #6006 review - HWP5 개체 번호 범주 attr bits 26-28 파싱 (#5864)

## 접수 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#6006](https://github.com/edwardkim/rhwp/pull/6006) |
| 작성자 | [@planet6897](https://github.com/planet6897) |
| 원 head | `39bf9b22b297732524a37902d09bb193ff64ee14` |
| 통합 commit | `077e46c0c` |
| GitHub 상태 | non-draft, CI·CodeQL 성공 |
| 판정 | **수용 권고** |

## 변경 검토

HWP5 shape attribute bits 26-28의 numbering category를 읽어 HWPX `numberingType`이 전량 `NONE`으로
붕괴하는 문제를 막는다. parser tag/shape와 해당 roundtrip 계약 테스트가 추가됐다.

## 로컬 검증

- `issue_5864_hwp5_numbering_category`: 1 passed
- 통합 후보 전체 검증에서 `cargo fmt`, suite manifest, unit-tier, clippy, 전체 nextest가 통과했다.
- 전체 nextest: `8304 tests run: 8304 passed (3 slow), 42 skipped`

## 권고

focused test와 전체 회귀가 모두 통과했다. 이번 통합 PR에서 수용 가능하다.
