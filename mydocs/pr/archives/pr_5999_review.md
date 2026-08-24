---
kind: pr-review
status: accepted-pending-integration-pr-approval
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-24
---

# PR #5999 review - 직접 HWPX RowBreak 칸 조각 회계 정합 (#5880)

## 접수 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#5999](https://github.com/edwardkim/rhwp/pull/5999) |
| 작성자 | [@planet6897](https://github.com/planet6897) |
| 원 head | `c8da82ba7222ecfc6f7e80a523b7063e706851d8` |
| 통합 commit | `420f6e27c` |
| GitHub 상태 | non-draft, `MERGEABLE/CLEAN`, CI·CodeQL 성공 |
| 판정 | **수용 권고** |

## 변경 검토

직접 HWPX RowBreak 칸에서 저장 사다리와 조각 회계를 맞춰 2737927 fixture의 소실 줄과 쪽수 기준을
복구한다. 변경은 table layout partial/row accounting과 해당 sample·계약 테스트에 한정된다.

## 로컬 검증

- `issue_5880_rowbreak_fragment_overfill`: 3 passed
- 통합 후보 전체 검증에서 `cargo fmt`, suite manifest, unit-tier, clippy, 전체 nextest가 통과했다.
- 전체 nextest: `8304 tests run: 8304 passed (3 slow), 42 skipped`

## 권고

독립 focused test와 통합 전체 회귀가 모두 통과했다. #5970 제외 후에도 회귀가 재발하지 않아 이번 통합
PR에서 수용 가능하다.
