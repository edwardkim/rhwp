---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-11-03
---

# PR #8287 검토 — 다각형 회전·대칭 왕복

## 최종 판정

**머지 보류 — 접수 완료, 현재 devel에서의 실행·시각 검증 진행 전.**
현재 단계는 원 head·기존 CI·코드와 실제 merge-tree의 접수 결과다. 원 head의 과거 통과를
최신 devel 통합 후보의 실행 완료로 보고하지 않는다. 아래 보정과 검증이 끝나면 판정을 갱신한다.

## 접수 정보

| 항목 | 고정값 |
| --- | --- |
| 원 PR / 작성자 | [#8287](https://github.com/edwardkim/rhwp/pull/8287) / semanticist21 |
| 원 repository·branch | `semanticist21/rhwp:fix/polygon-transform-roundtrip` |
| 원 head | `92d19ea0e3690446106a126b747e0e5f611a0837` |
| 현재 기준 devel | `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4` |
| Reviewer | edwardkim, 2026-11-03T19:19:34.155490+09:00 API 확인 |
| 원 head CI | commit status success; skipped 5, success 34 |
| 실제 merge-tree | clean |
| 로컬 branch | `review/integration-20261103-semanticist21` |
| 경로 | maintainer_general + intake_and_review, local_validation, multi_pr_update_branch, visual_fixture_evidence, post_merge |

## 주장·현재 devel 대조와 보정 방침

저장 행렬이 없는 다각형도 첫 편집부터 중심 기준 합성 행렬을 쓴다.

현재 공통 변환 helper에는 저장 affine이 없는 경우의 기존 분기가 남아 있다. 실제 왕복 동작 판정은 아직 실행하지 않았다.

자동 적용 후보에서 기존 affine 참조·회전 전 bbox·각도 소비를 추적한다. 회전·대칭·크기 변경·역연산을 같은 행렬에서 검증한다.

이 절의 현재 관측은 코드·이력 대조이며 현재 base/head의 실제 재현 결과는 검증 단계에서 추가한다.

## 검증 범위

polygon_transform_roundtrip; 한컴 rotated HWP/HWPX·90도/대칭/크기 변경·역연산·저장 재열기

제품 경로:
- `src/document_core/commands/object_ops/common.rs`
- `src/renderer/layout/shape_layout.rs`

정식 focused 대상:
- `tests/cases/polygon_transform_roundtrip.rs`

원본·독립 Print·편집 상태의 대응을 확인하고 현재 base/후보의 Native/fresh WASM 전쪽 TSV와 확대 판독을
확보한다. 입력이 합성 계약인 범위와 실물 한컴 출력은 구분한다. 현재 단계에 새 측정 수치를 적지 않는다.

## 조판 원칙 준수 검토

| 항목 | 접수 판정과 후속 근거 |
| --- | --- |
| 제출 전 로컬 검증과 PR 상태 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |
| 기존 이슈·해결 PR의 선행 분석 | 미검증 — 위 현재 devel 관측과 #8093/#8187/#8517의 선행 연결을 시작했으며 적용 경계의 실행을 남긴다. |
| 구현 근거와 일반성 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |
| 측정·배치 일관성 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |
| 리베이스 반영 PR의 기능 보존 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |
| 최종 출력 소비 계약 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |
| 분할·이어받기 계약 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |
| 줄 소속과 점유 높이 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |
| 사례와 증거의 독립성 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |
| 기준값 변경 | 비해당 — 접수 단계에서 코드·기대값·baseline·golden을 변경하지 않았다. 이후 검사 변경은 독립 출력 gate 뒤에 수행한다. |
| 회귀 추가·변경의 선행 조건 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |
| 90% 통과 후 확대 판독 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |
| 주장과 검증 범위 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |

## Merge 후 contributor PR comment 계획

현재는 maintainer 시각 검증 미실행이다. 검증 완료 뒤 이 원 PR의 실제 결론·source/merge SHA·CI URL,
[Visual Sweep 정본](../manual/verification/visual_sweep_guide.md#github-merge-comment)과 실제 측정한 전쪽 최저값·직접 판독을
연결한다. 수용 시 원 기여와 메인터너 보정 이유를 존댓말로 구분하고 정식 대표 PNG가 devel에 반영된 뒤
승인된 원격 범위에서 게시·API 재확인한다. 아직 없는 수치나 이미지 URL은 만들지 않는다.

[누적 검토 계획](semanticist21_20261103_review_plan.md) · [원 head·CI·충돌 증거](assets/semanticist21-20261103/intake.json).

## 현행 프레임 계약과의 동작 충돌 보정

첫 집중 실행에서 한컴 30도 저장본 왕복은 통과했지만 새 90도 다각형이 변형됐다.
선행 보정 `d55209cc17d2dbd7e58d0d35639abf59098c46d7` → #8319 merge
`af6bb05b26887135fb9da0fa4e54ca7205564fca`의 현재 계약은 회전 후 common 외곽 크기와
회전 전 current 크기를 구분한다. 원 PR의 common 크기 비율을 그대로 쓰면 회전만 한
120×60px 다각형에 0.5×2 배율이 추가된다. 기대 좌표를 바꾸지 않고 이 생산 경로를 보정한다.

`group_geometry`의 동일 배치 프레임에서 현재 affine 행렬을 만들고, 원본 저장 행렬과
이상적인 원본 프레임 사이의 잔여 변환을 보존해 합성한다. 위치만 바뀌면 원본 행렬을 유지하고,
역연산은 최초 잔여 변환에서 계산해 누적 오차를 피한다. 보정 후 focused·독립 Print·fresh
WASM의 실제 검증을 진행 중이며 이 설명만으로 완료 판정하지 않는다.

보정 뒤 새 90도·270도·양방향 대칭·30도+크기 변경 및 한컴 30도 저장본의 두 테스트가
빠른 진단 profile에서 통과했다. 최종 Path bbox도 변환된 실제 꼭짓점 경계를 발행해
paint와 hit-test 상자가 다른 회전 프레임을 쓰지 않게 했다. 정규 lint·최적화 빌드·전체
회귀와 새 편집 상태의 독립 Print 검증은 진행 중이다.
