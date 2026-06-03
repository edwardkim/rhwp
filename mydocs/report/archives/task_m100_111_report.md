# 최종 결과 보고서 — Task #111 + #112

**이슈**: [#111](https://github.com/edwardkim/rhwp/issues/111) + [#112](https://github.com/edwardkim/rhwp/issues/112)
**타이틀**: 양식 컨트롤 인터랙션 — 셀 커서 진입 + 체크박스 클릭 토글
**마일스톤**: M100
**완료일**: 2026-04-13
**브랜치**: `local/task111`

---

## 결과 요약

`form-002.hwpx`의 체크박스가 있는 표 셀에 마우스 클릭으로 커서가 진입하고,
체크박스 클릭 시 체크 상태가 정상 토글됨을 E2E 테스트로 확인하였다.

---

## 원인 분석

### #111 — 셀 커서 진입 불가

`cursor_rect.rs`의 `collect_runs()`가 `TableCell` bbox를 등록할 때
`parent_para_index`/`control_index`를 0으로 초기화하고, 자식 `TextRun`에서 보완하는 구조였다.
FormObject만 있는 셀은 TextRun이 없으므로 보완이 이루어지지 않았고,
`cell_runs` 필터링(`parent_para_index == cb.parent_para_index && control_index == cb.control_index`)이
항상 실패하여 셀 진입 경로에 도달하지 못했다.

### #112 — 체크박스 클릭 토글 불가

`get_form_object_at_native()`가 `para`로 셀 내 문단 인덱스(`cp_idx`)를 반환했고,
`set_form_value_native(sec, para, ci)`는 `sections[sec].paragraphs[para].controls[ci]`로
flat 접근하여 셀 내부 Form 컨트롤을 찾지 못했다.

---

## 수정 내용

### `src/document_core/queries/cursor_rect.rs` (#111)

- `CellBboxInfo`에 `section_index`, `has_meta` 필드 추가
- `collect_runs()`에 `current_table_meta: Option<(usize, usize, usize)>` 파라미터 추가
  - `RenderNodeType::Table` 진입 시 `(section_index, para_index, control_index)` 추출하여 하위 전파
  - `TableCell` 등록 시 `table_meta`로 즉시 meta 채움 → `has_meta=true`
- `cell_runs`가 비어있고 `has_meta==true`이면 `charOffset=0`으로 셀 진입 fallback 반환

### `src/renderer/render_tree.rs`

- `FormObjectNode`에 `cell_location: Option<(usize, usize, usize, usize)>` 필드 추가
  - `(table_para_index, table_control_index, cell_index, cell_para_index)` 저장

### `src/renderer/layout/paragraph_layout.rs`

- 셀 내부 FormObject 렌더링 시 `cell_ctx`에서 `cell_location` 추출하여 저장 (2곳)

### `src/document_core/queries/form_query.rs` (#112)

- `get_form_object_at_native()`: `cell_location`이 있으면 `para=table_para_index` 반환,
  JSON에 `inCell`, `tablePara`, `tableCi`, `cellIdx`, `cellPara` 추가
- `set_form_value_in_cell_native()` 신규 추가:
  표 내부 셀 문단의 Form 컨트롤에 직접 접근하여 값 설정
- `apply_form_value()` 헬퍼 함수 추출 (공통 로직)

### `src/wasm_api.rs`

- `setFormValueInCell` WASM 바인딩 추가

### `rhwp-studio/src/core/types.ts`

- `FormObjectHitResult`에 `inCell?`, `tablePara?`, `tableCi?`, `cellIdx?`, `cellPara?` 필드 추가

### `rhwp-studio/src/core/wasm-bridge.ts`

- `setFormValueInCell()` 바인딩 추가

### `rhwp-studio/src/engine/input-handler.ts`

- `handleFormObjectClick()`에 `setFormVal` 헬퍼 추가:
  `inCell==true`이면 `setFormValueInCell()`, 아니면 기존 `setFormValue()` 호출

---

## E2E 테스트

신규: `rhwp-studio/e2e/form-control.test.mjs`

| TC | 내용 | 결과 |
|----|------|------|
| TC-1 | form-002.hwpx 로드 (10페이지) | ✅ PASS |
| TC-2 | 렌더 트리에서 FormObject bbox 추출 | ✅ PASS |
| TC-3 | hitTest() 체크박스 셀 커서 진입 (#111) | ✅ PASS |
| TC-4 | getFormObjectAt() 체크박스 감지 (#112) | ✅ PASS |
| TC-5 | setFormValueInCell() 체크박스 토글 (#112) | ✅ PASS |

---

## 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✅ 성공 |
| `cargo test` (785개) | ✅ 전체 통과 |
| WASM 빌드 | ✅ 성공 |
| E2E `form-control.test.mjs` (5개) | ✅ 전체 통과 |

---

## 커밋 목록

| 커밋 | 내용 |
|------|------|
| (이번 커밋) | Task #111+#112: 양식 컨트롤 셀 커서 진입 + 체크박스 클릭 토글 |
