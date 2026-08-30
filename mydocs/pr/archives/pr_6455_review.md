---
kind: pr-review
status: visual-hold
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6455
issue: 6442
author: planet6897
---

# PR #6455 검토 기록

## 결정

| 구분 | 현재 결정 |
|---|---|
| 원 PR head 직접 병합 | 하지 않음 |
| 수용 대상 | 기존 baseline 행을 보존한 #6481의 `1b8a5deff57cef226fb89e587128335fb767d86e` |
| 현재 상태 | 수용 보류: 카드·페이지 배치 visual sweep과 최신 CI가 아직 완료되지 않음 |
| 승인 뒤 처리 | #6481 수용·병합 뒤 #6455를 포함 수용 근거와 함께 close |

## 식별과 provenance

| 항목 | 값 |
|---|---|
| 원 PR | https://github.com/edwardkim/rhwp/pull/6455 |
| 원 head | `c8783f11933a329054c42607ba7d27d63c16d03c` |
| 통합 기준 | `upstream/devel@8a150f9a8bb19a9918e195da3a646690f68f4328` |
| 통합 commit | `1b8a5deff57cef226fb89e587128335fb767d86e` |
| 통합 순서 | 5/8 |

## 검토

사용되지 않는 cell inner margin을 레이아웃 비용으로 청구하지 않는 #6442 보정이다. 원 PR CI는 수집 시점에 비성공 check 없이 완료됐다. baseline TSV 충돌은 기존 행과 #6442 신규 행을 함께 보존해 해소했다.

통합 후보에서 `unused_inner_margin_field_is_not_charged`, `page3_control_group_is_unchanged`, `both_back_side_cards_on_page2_carry_their_content`가 통과했다. 공통 필수 clippy·build·manifest·format 검증도 통과했다.

번들 PNG는 통합 head 직접 재생성 결과가 아니므로 최종 시각 판정은 통합 PR Render Diff 성공을 조건으로 한다.

## 현재 결론

baseline 충돌은 기존 회귀 행을 보존해 해소됐고 focused test도 통과했다. 그러나 cell margin 변경은 카드와 페이지 배치에 영향을 주므로 visual sweep 없이 수용 확정을 하지 않는다. 현재는 수용 보류이며, 승인 뒤 #6481의 CI와 visual 증빙을 확인한 경우에만 포함 수용한다.
