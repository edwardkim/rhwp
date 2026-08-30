---
kind: pr-review
status: visual-hold
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6479
issue: 6465
author: planet6897
---

# PR #6479 검토 기록

## 결정

| 구분 | 현재 결정 |
|---|---|
| 원 PR head 직접 병합 | 하지 않음 |
| 수용 대상 | 기존 overlap baseline 행을 보존한 #6481의 `2a014091410a1d4f93d24af804b8382e55103bdc` |
| 현재 상태 | 수용 보류: footer/logo line 배치 visual sweep과 최신 CI가 아직 완료되지 않음 |
| 승인 뒤 처리 | #6481 수용·병합 뒤 #6479를 포함 수용 근거와 함께 close |

## 식별과 provenance

| 항목 | 값 |
|---|---|
| 원 PR | https://github.com/edwardkim/rhwp/pull/6479 |
| 원 head | `13cac599fd35e80b7a55a1a1019cc637a90d691e` |
| 통합 기준 | `upstream/devel@8a150f9a8bb19a9918e195da3a646690f68f4328` |
| 통합 commit | `2a014091410a1d4f93d24af804b8382e55103bdc` |
| 통합 순서 | 8/8 |

## 검토

inline object가 footer logo와 같은 line을 잘못 공유하는 #6465 회귀를 보정한다. 원 PR CI는 최초 수집 시점에 진행 중이었으므로 원 head의 완료 상태를 통합 판정 근거로 사용하지 않는다. text-overlap baseline 충돌에서는 기존 #6310 행과 #6465 신규 행을 함께 보존했다.

통합 후보에서 `footer_logos_sit_on_their_own_line`이 통과했다. 공통 필수 native·WASM·workspace clippy, workspace build, manifest와 format 검증도 통과했다.

번들 PNG는 통합 head 직접 산출물이 아니므로 최종 시각 판정은 통합 PR Render Diff 성공을 조건으로 한다.

## 현재 결론

원 PR 최초 수집 시 CI가 진행 중이었고, footer/logo의 line placement는 사용자-visible 범위다. focused 계약만 통과한 현재 상태에서는 수용을 확정하지 않는다. 승인 뒤 #6481의 CI와 visual 증빙을 확인한 경우에만 포함 수용한다.
