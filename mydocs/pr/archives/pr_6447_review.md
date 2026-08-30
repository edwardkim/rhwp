---
kind: pr-review
status: visual-hold
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6447
issue: 6300
author: planet6897
---

# PR #6447 검토 기록

## 결정

| 구분 | 현재 결정 |
|---|---|
| 원 PR head 직접 병합 | 하지 않음 |
| 수용 대상 | 통합 PR #6481에 포함된 `16ee5f750f8ce76ba9879eed00c2597423b1b0c6` |
| 현재 상태 | 수용 보류: 페이지 경계 visual sweep과 최신 CI가 아직 완료되지 않음 |
| 승인 뒤 처리 | #6481 수용·병합 뒤 #6447을 포함 수용 근거와 함께 close |

## 식별과 provenance

| 항목 | 값 |
|---|---|
| 원 PR | https://github.com/edwardkim/rhwp/pull/6447 |
| 원 head | `73f906ca5a88434a2740580ac7b4499b741be83d` |
| 통합 기준 | `upstream/devel@8a150f9a8bb19a9918e195da3a646690f68f4328` |
| 통합 commit | `16ee5f750f8ce76ba9879eed00c2597423b1b0c6` |
| 통합 순서 | 4/8 |

## 검토

line 끝의 forced break가 다음 행과 합쳐지는 #6300 페이지 회귀를 보정한다. 원 PR CI는 수집 시점에 비성공 check 없이 완료됐고 cherry-pick 충돌도 없었다.

통합 후보에서 `forced_break_at_line_end_does_not_merge_two_rows`, `page_count_moves_toward_the_hangul_oracle`가 통과했다. 공통 필수 clippy·build·manifest·format 검증도 통과했다.

원 PR의 비교 이미지는 변경 의도의 보조 근거로만 확인했다. 통합 head의 최종 시각 수용은 Render Diff 성공을 조건으로 한다.

## 현재 결론

forced break와 페이지 수는 사용자-visible 결과이므로 contract test만으로 수용을 확정하지 않는다. 현재는 visual sweep 미실행에 따른 수용 보류이며, 승인 뒤 #6481의 CI와 visual 증빙을 확인한 경우에만 포함 수용한다.
