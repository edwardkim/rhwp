---
kind: pr-review
status: approved-via-integration
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-31
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
| 현재 상태 | 승인: #6481 통합 후보 기준으로 카드·페이지 배치 시각 증적과 계약 검증에 차단 finding 없음 |
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

번들 PNG는 변경 의도의 보조 자료로만 사용했다. 통합 code head에서 HWP MCP 2020 기준 PDF와 전 3쪽 sweep을 직접 실행했고, claim page인 2쪽의 `pixel_match=89.70238`, `visual_accuracy_proxy_percent=40.26297`, 자동 후보 0건을 기록했다. review PNG에서 앞면과 뒷면 카드 4개 모두 내용이 있고, 이번 결함인 뒷면 카드 공백은 보이지 않았다. 대표 증적은 [p2 review PNG](../assets/pr_6481_issue6442_p002_review.png)이며, 재현 명령과 원본/PDF SHA는 [PR #6481 visual sweep 기록](pr_6481_planet6897_visual_sweep.md)에 있다.

## #6481 당시 결론 (역사)

**당시 판정: 승인.** baseline 충돌은 기존 회귀 행을 보존해 해소됐고 focused test와 claim page 시각 증적에도 차단 finding이 없다. proxy 수치는 카드 공백 결함 해소의 제한된 근거다. 원 PR은 직접 merge하지 않고 #6481 통합 결과로만 수용한다. remote push, merge, #6455 close는 별도 지시가 있을 때만 수행한다.

## #6485 최신 통합 판정

현재 수용 대상은 PR #6485 code candidate `f47d5b3586d470c99ed38f155af18175801f3c85`의 `8f384e51b`이며 원 PR head는 직접 병합하지 않는다. focused 3건, 전체 nextest 8,785건, Native Skia, WASM, lint가 실제 통과했다. HWP MCP 2020 기준 PDF `pdf/pr6485-visual/pr6485-issue6442-access-pass-form-2020.pdf`(SHA-256 `b32a32fbe04bdeca83b702bea27317deed6fc1eb8c6b131969acabfbd38fdf58`)와 p2 direct sweep의 `pixel_match=89.70238`, proxy `40.26297`, 후보 0건을 확인했다. 대표 PNG는 [p2 review PNG](../assets/pr_6485_issue6442_p002_review.png)다.

**최종 판정: 승인.** #6485 최신 trailing head CI와 명시적 merge 승인이 남은 조건이다.

## Merge 후 contributor PR comment 계획

#6485 merge SHA와 실제 PR/devel CI, p2 후보 0건과 위 수치를 [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment), `<merge-commit-sha>` 고정 raw PNG URL로 한 번 게시한다.
