---
kind: pr-review
status: approved-via-integration
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-31
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
| 현재 상태 | 승인: #6481 통합 후보 기준으로 footer/logo line 시각 증적과 계약 검증에 차단 finding 없음 |
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

번들 PNG는 변경 의도의 보조 자료로만 사용했다. 통합 code head에서 HWP MCP 2020 기준 PDF와 전 13쪽 sweep을 실행했고, footer/logo claim page인 13쪽의 `pixel_match=93.64131`, `visual_accuracy_proxy_percent=20.50308`, 자동 후보 0건을 기록했다. review PNG에서 footer logo와 설명 block은 자체 줄에 남아 inline object와 같은 줄을 공유하지 않았다. 대표 증적은 [p13 review PNG](../assets/pr_6481_issue6465_p013_review.png)이며, 재현 명령과 원본/PDF SHA는 [PR #6481 visual sweep 기록](pr_6481_planet6897_visual_sweep.md)에 있다.

## 현재 결론

**최종 판정: 승인.** footer/logo의 line placement는 사용자-visible 범위이므로 focused 계약과 claim page 시각 증적을 함께 확인했고 차단 finding이 없다. proxy 수치는 이번 줄 배치 주장에 한정한다. 원 PR은 직접 merge하지 않고 #6481 통합 결과로만 수용한다. remote push, merge, #6479 close는 별도 지시가 있을 때만 수행한다.
