---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-11-03
---

# PR #8235 검토 — 양식의 논리 편집·선택

## 최종 판정

**로컬 보정과 집중·시각 검증 진행 — 최종 전체 검증이 끝나기 전까지 머지 보류.**
원 head를 보존한 누적 후보에 보정과 실제 API·시각 대조를 적용했다. [현재 검증 명세](assets/semanticist21-20261103/validation.md)의 중간 제품 head와 최종 정규 검증을 구분한다. 모든 필수 검증이 끝난 뒤 최종 판정을 갱신한다.

## 접수 정보

| 항목 | 고정값 |
| --- | --- |
| 원 PR / 작성자 | [#8235](https://github.com/edwardkim/rhwp/pull/8235) / semanticist21 |
| 원 repository·branch | `semanticist21/rhwp:codex/radio-outline-contrast` |
| 원 head | `96c066b2dd267e76e282d1dccfa3b5e66efeaa20` |
| 현재 기준 devel | `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4` |
| Reviewer | edwardkim, 2026-11-03T19:19:26.028937+09:00 API 확인 |
| 원 head CI | commit status success; skipped 5, success 34 |
| 실제 merge-tree | 충돌 20파일 |
| 로컬 branch | `review/integration-20261103-semanticist21` |
| 경로 | maintainer_general + intake_and_review, local_validation, multi_pr_update_branch, visual_fixture_evidence, post_merge |

## 주장·현재 devel 대조와 보정 방침

양식의 같은 위치 형제 개체·범위 삭제·물리 행 선택과 라디오 외곽선을 보완한다.

기본 Form 슬롯 분류는 #8093 → #8319에, 공개 논리 편집·범위 재조판·model_para_range는 #8187에 이미 반영됐다. 이 중복과 양식 선택·복사·삭제의 잔여를 분리한다.

기존 #8187 API와 NBSP·필드 경계·공통 삭제를 유지하고 Form의 정확한 형제·물리 행 소유를 연결한다. schema 35·Print 기하 계약을 되돌리지 않는다. composer와 빈 중첩 HTML 표는 현행 Storage 전달·취소·자원 제한 경로에 연결한다.

이 절의 현재 관측은 코드·이력 대조이며 현재 base/head의 실제 재현 결과는 검증 단계에서 추가한다.

충돌 파일:
- `src/document_core/commands/clipboard.rs`
- `src/document_core/commands/html_import.rs`
- `src/document_core/commands/text_editing.rs`
- `src/document_core/queries/cursor_rect.rs`
- `src/paint/builder.rs`
- `src/paint/json.rs`
- `src/paint/schema.rs`
- `src/renderer/composer/line_breaking.rs`
- `src/renderer/composer/stored_cell_reflow.inc.rs`
- `src/renderer/composer/stored_lines_runs.inc.rs`
- `src/renderer/layout/table_partial.rs`
- `src/renderer/render_tree.rs`
- `src/wasm_api.rs`
- `tests/cases/form_object_caret_slot.rs`
- `tests/cases/issue_6520_svg_layer_contract.rs`
- `tests/cases/issue_7444_cut_boundary_object.rs`
- `tests/cases/reflow_body_paragraph_range.rs`
- `tests/cases/styled_text_clipboard_field_affinity.rs`
- `tests/cases/styled_text_html_field_affinity.rs`
- `tests/common/passthrough_exemptions.rs`

## 검증 범위

form_inline_editing, form_object_caret_slot; 본문/셀·같은 위치 형제·양쪽 경계·범위 태그·거부 원자성·HWP/HWPX 왕복; 기존 #8093/#7444/#8187 대조와 실제 WASM 선택 API

제품 경로:
- `src/diagnostics/layout_anomaly.rs`
- `src/document_core/commands/clipboard.rs`
- `src/document_core/commands/formatting.rs`
- `src/document_core/commands/html_import.rs`
- `src/document_core/commands/text_editing.rs`
- `src/document_core/html_table_import.rs`
- `src/document_core/html_table_width.rs`
- `src/document_core/queries/cursor_nav.rs`
- `src/document_core/queries/cursor_rect.rs`
- `src/document_core/queries/doc_tree_nav.rs`
- `src/document_core/queries/hwpctrl_sets.rs`
- `src/model/control.rs`
- `src/model/paragraph.rs`
- `src/paint/builder.rs`
- `src/paint/json.rs`
- `src/paint/schema.rs`
- `src/renderer/canvas.rs`
- `src/renderer/composer/line_breaking.rs`
- `src/renderer/composer/stored_cell_reflow.inc.rs`
- `src/renderer/composer/stored_lines_runs.inc.rs`
- `src/renderer/form_appearance.rs`
- `src/renderer/layout/paragraph_layout.rs`
- `src/renderer/layout/table_cell_content.rs`
- `src/renderer/layout/table_layout.rs`
- `src/renderer/layout/table_layout/wrapper_cell_viewport.rs`
- `src/renderer/layout/table_partial.rs`
- `src/renderer/layout/tests.rs`
- `src/renderer/render_tree.rs`
- `src/wasm_api.rs`
- `src/wasm_api/tests.rs`

정식 focused 대상:
- `tests/cases/blank_nested_table_html.rs`
- `tests/cases/form_inline_editing.rs`
- `tests/cases/form_object_caret_slot.rs`
- `tests/cases/issue_6520_svg_layer_contract.rs`
- `tests/cases/issue_7444_cut_boundary_object.rs`
- `tests/cases/layout_anomaly_m02f_fatten.rs`
- `tests/cases/layout_anomaly_off_canvas_contract.rs`
- `tests/cases/layout_anomaly_text_overlap_contract.rs`
- `tests/cases/reflow_body_paragraph_range.rs`
- `tests/cases/styled_text_clipboard_field_affinity.rs`
- `tests/cases/styled_text_html_field_affinity.rs`
- `tests/cases/table_fragment_source_range.rs`
- `tests/common/passthrough_exemptions.rs`

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

## 승인 범위의 로컬 통합 보정

#8093/#8187에서 이미 통합한 Form 기본 슬롯·논리 API·문단 메타데이터·필드 원자성·셀 삭제와
schema 35는 현행 구현을 보존했다. Form 전용 형제 선택은 정확한 논리 슬롯을 공통 삭제에
전달하고 변경 전 범위 태그를 검증한다. 양식의 프레임 투영·재조판은 현재 Storage 전달
경로를 사용한다. TextLine 소유와 실제 양식 노드 조회를 함께 유지했다.
기존 실물 form-slot 관계 검사는 보존했고, 이미 통합한 API 회귀의 오래된 중복본은 복원하지 않았다.
이 항목은 충돌 보정의 설명이며, 새 후보의 실행·시각 검증 완료 판정은 후속 결과로 기록한다.

## 첫 집중 실행과 추가 보정

양식의 복사·삭제·유니코드·필드 거부·원자성·기존 Form 슬롯은 통과했다. 여덟 양식을
붙여넣은 전체 흐름에서는 모델 개수와 실제 paint 개수가 달랐다. 현행의 원시 제어 전용 줄
소유 인증이 저장 줄만 받아, 프레임이 새로 발행한 증가하는 제어 슬롯 컷을 배제한 것을
확인했다. 원본을 추측해 완화하지 않고 완전한 원시 스트림과 실제 증가하는 생성 줄 컷을
함께 요구해 같은 줄 소유 결과를 측정·paint·선택에 전달한다. 새 실행으로 확인 중이다.

생성 줄 소유 보정 뒤 실제 양식 16개가 각각 한 번 표시되는 것을 집중 검사로 확인했다.
셀의 줄 경계에서 앞줄 끝 앵커가 다음 줄 양식의 leading 경계를 선점하지 않게 하고,
선택도 저장 줄 유무와 무관하게 실제 Form/TextLine의 공통 소유 결과를 소비하게 했다.
빠른 진단 profile(해당 rhwp package opt-level 0)에서 8개 붙여넣기·본문/셀·양쪽 저장
재열기와 다각형 두 검사를 합친 3건이 통과했다. 이 진단을 정규 release-test 전체 검증으로
대신하지 않으며 Native/fresh WASM 시각 검증과 전체 회귀를 이어 간다.
