---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-11-03
---

# PR #8298 검토 — 넘친 표의 저장 높이

## 최종 판정

**머지 보류 — 접수 완료, 현재 devel에서의 실행·시각 검증 진행 전.**
현재 단계는 원 head·기존 CI·코드와 실제 merge-tree의 접수 결과다. 원 head의 과거 통과를
최신 devel 통합 후보의 실행 완료로 보고하지 않는다. 아래 보정과 검증이 끝나면 판정을 갱신한다.

## 접수 정보

| 항목 | 고정값 |
| --- | --- |
| 원 PR / 작성자 | [#8298](https://github.com/edwardkim/rhwp/pull/8298) / semanticist21 |
| 원 repository·branch | `semanticist21/rhwp:fix/save-grown-table-height` |
| 원 head | `b46dde6798da1fde750bdf43a4a3078af1c139b9` |
| 현재 기준 devel | `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4` |
| Reviewer | edwardkim, 2026-11-03T19:19:50.028226+09:00 API 확인 |
| 원 head CI | commit status success; skipped 4, success 31 |
| 실제 merge-tree | 충돌 1파일 |
| 로컬 branch | `review/integration-20261103-semanticist21` |
| 경로 | maintainer_general + intake_and_review, local_validation, multi_pr_update_branch, visual_fixture_evidence, post_merge |

## 주장·현재 devel 대조와 보정 방침

재조판한 셀 내용의 실제 행 높이를 표 sz에 쓰고 cellSz는 보존한다.

현재 export는 try_prepare_hwp_export_snapshot·writeback_reflowed_table_frames_with_storage와 HWPX lineage를 사용한다. 원 PR의 이전 clone/export 경로를 그대로 적용하지 않는다.

성장 높이를 기존 측정 결과와 Storage 기반 snapshot 경로에 연결하고 HWPX lineage·되감김 프레임·cellSz를 보존한다. 손대지 않은 표·중첩 표의 적용 범위와 HU 반올림 근거도 검증한다.

이 절의 현재 관측은 코드·이력 대조이며 현재 base/head의 실제 재현 결과는 검증 단계에서 추가한다.

충돌 파일:
- `src/document_core/commands/document.rs`

## 검증 범위

hancom_table_grow_saves_table_height; grow3 전후·본문/중첩 표·무편집·HWP/HWPX 왕복·저장 취소/예산 실패 원자성

제품 경로:
- `src/document_core/commands/document.rs`
- `src/document_core/commands/document/grown_table_heights.rs`

정식 focused 대상:
- `tests/cases/hancom_table_grow_saves_table_height.rs`

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

## 로컬 충돌 보정

성장 높이를 현행 Storage 기반 HWP snapshot 경로에 전달하고 HWPX lineage 분기를 보존했다.
본문 표는 기존 측정 결과를 소비하며, 중첩 표는 동일한 LayoutProfile·정규화 정책의 측정기를
사용한다. 편집하지 않은 중첩 표는 원본 높이를 보존한다. 임의의 10 HU 문턱은 제거하고
공통 정수 단위 변환 결과의 실제 증가를 사용한다. 이 결정의 무편집·중첩·한도·시각 검증은
누적 후보에서 진행하며, 실행 전 완료로 판정하지 않는다.
