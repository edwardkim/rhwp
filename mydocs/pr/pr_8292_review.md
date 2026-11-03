---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-11-03
---

# PR #8292 검토 — 중복 표 ID의 HWPX 저장

## 최종 판정

**머지 보류 — 접수 완료, 현재 devel에서의 실행·시각 검증 진행 전.**
현재 단계는 원 head·기존 CI·코드와 실제 merge-tree의 접수 결과다. 원 head의 과거 통과를
최신 devel 통합 후보의 실행 완료로 보고하지 않는다. 아래 보정과 검증이 끝나면 판정을 갱신한다.

## 접수 정보

| 항목 | 고정값 |
| --- | --- |
| 원 PR / 작성자 | [#8292](https://github.com/edwardkim/rhwp/pull/8292) / semanticist21 |
| 원 repository·branch | `semanticist21/rhwp:fix/hwpx-table-grid-duplicate-ids` |
| 원 head | `1f26297e9cae86955dc36f2f100585cae4b06784` |
| 현재 기준 devel | `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4` |
| Reviewer | edwardkim, 2026-11-03T19:19:42.122439+09:00 API 확인 |
| 원 head CI | commit status success; neutral 1, skipped 6, success 32 |
| 실제 merge-tree | 충돌 1파일 |
| 로컬 branch | `review/integration-20261103-semanticist21` |
| 경로 | maintainer_general + intake_and_review, local_validation, multi_pr_update_branch, visual_fixture_evidence, post_merge |

## 주장·현재 devel 대조와 보정 방침

빈 논리 행 보존 대상의 ID 중복 때문에 저장 전체가 실패하는 경로를 보완한다.

현행 Writer는 project_with_storage·예산 적용 clone/rebuild/records push를 사용한다. 원 PR은 이전 무한도 경로를 수정해 충돌한다.

ID 충돌 처리를 project_with_storage에 연결하고 tree_entry·clone_table·grid rebuild·fingerprint·record push의 예산과 오류 전달을 유지한다. 뒤에 나오는 기존 ID와 재할당 값 충돌도 검증한다.

이 절의 현재 관측은 코드·이력 대조이며 현재 base/head의 실제 재현 결과는 검증 단계에서 추가한다.

충돌 파일:
- `src/model/table_row_grid.rs`

## 검증 범위

hwpx_table_grid_duplicate_ids; 빈 논리 행·0/같은 ID·중첩·비중복 ID 보존·최소/정상 자원 예산·기존 #7998 저장 대조

제품 경로:
- `src/model/table_row_grid.rs`

정식 focused 대상:
- `tests/cases/hwpx_table_grid_duplicate_ids.rs`
- `tests/cases/issue_7998_pdf_retained_border_structure.rs`

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
