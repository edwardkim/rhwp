---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-11-03
---

# PR #8235 검토 — 양식의 논리 편집·선택

## 최종 판정

**머지 보류 — 접수 완료, 현재 devel에서의 실행·시각 검증 진행 전.**
현재 단계는 원 head·기존 CI·코드와 실제 merge-tree의 접수 결과다. 원 head의 과거 통과를
최신 devel 통합 후보의 실행 완료로 보고하지 않는다. 아래 보정과 검증이 끝나면 판정을 갱신한다.

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
