---
kind: pr-review
status: visual-hold
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
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
| 현재 상태 | 수용 보류: golden SVG와 비용 상세 열 visual sweep이 아직 완료되지 않음 |
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

golden SVG와 번들 이미지 변경은 통합 PR Render Diff에서 다시 확인해야 하며, contributor 산출물만으로 시각 sweep 통과를 주장하지 않는다.

## 현재 결론

저장된 condensed width와 텍스트 계약은 통과했지만 golden SVG 변경은 직접 visual 확인이 필요하다. 현재는 visual sweep 미실행에 따른 수용 보류이며, 승인 뒤 #6481의 CI와 visual 증빙을 확인한 경우에만 포함 수용한다.
