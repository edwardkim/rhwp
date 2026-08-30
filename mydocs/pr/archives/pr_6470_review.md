---
kind: pr-review
status: approved-via-integration
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-31
pr: 6470
issue: 6443
author: planet6897
---

# PR #6470 검토 기록

## 결정

| 구분 | 현재 결정 |
|---|---|
| 원 PR head 직접 병합 | 하지 않음 |
| 수용 대상 | 통합 PR #6481에 포함된 `421f3846d14720a9cfd3b2fc8f4a42afea80fc90` |
| 현재 상태 | 승인: #6481 통합 후보 기준으로 비용 상세 열 시각 증적과 계약 검증에 차단 finding 없음 |
| 승인 뒤 처리 | #6481 수용·병합 뒤 #6470을 포함 수용 근거와 함께 close |

## 식별과 provenance

| 항목 | 값 |
|---|---|
| 원 PR | https://github.com/edwardkim/rhwp/pull/6470 |
| 원 head | `465a56293aa0181f231cfb0c27e56ff32fdf5405` |
| 통합 기준 | `upstream/devel@8a150f9a8bb19a9918e195da3a646690f68f4328` |
| 통합 commit | `421f3846d14720a9cfd3b2fc8f4a42afea80fc90` |
| 통합 순서 | 6/8 |

## 검토

비용 상세 열의 저장된 condensed width와 텍스트 보존을 맞추는 #6443 보정이다. 원 PR CI는 수집 시점에 비성공 check 없이 완료됐고 cherry-pick 충돌도 없었다.

통합 후보에서 `cost_detail_column_text_is_intact`, `cost_detail_column_keeps_its_stored_condensed_width`가 통과했다. 공통 필수 clippy·build·manifest·format 검증도 통과했다.

golden SVG와 번들 이미지는 contributor 산출물만으로 판단하지 않았다. 통합 code head에서 HWP MCP 2020 기준 PDF와 전 8쪽 sweep을 실행했고, 비용 상세 열 claim page인 8쪽의 `pixel_match=90.08795`, `visual_accuracy_proxy_percent=25.08413`, 자동 후보 0건을 기록했다. review PNG에서 비용 상세 텍스트가 우측 cell line을 넘거나 잘려 보이지 않았다. 대표 증적은 [p8 review PNG](../assets/pr_6481_issue6443_p008_review.png)이며, 재현 명령과 원본/PDF SHA는 [PR #6481 visual sweep 기록](pr_6481_planet6897_visual_sweep.md)에 있다.

## 현재 결론

**최종 판정: 승인.** 저장된 condensed width와 텍스트 계약은 통과했고 claim page의 우측 경계도 직접 확인했다. proxy 수치는 overflow/clip 여부의 제한된 근거다. 원 PR은 직접 merge하지 않고 #6481 통합 결과로만 수용한다. remote push, merge, #6470 close는 별도 지시가 있을 때만 수행한다.
