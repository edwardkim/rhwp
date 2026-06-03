# 단계별 완료 보고서 — Task #111 + #112 Stage 1

**이슈**: [#111](https://github.com/edwardkim/rhwp/issues/111) + [#112](https://github.com/edwardkim/rhwp/issues/112)
**단계**: 1단계 + 2단계 (전 단계 완료)
**완료일**: 2026-04-13
**브랜치**: `local/task111`

---

## 구현 결과

### #111 — 셀 커서 진입 수정 (`cursor_rect.rs`)

#### 원인

`collect_runs()`가 `TableCell` bbox를 `parent_para_index=0, control_index=0`으로 등록한 뒤,
자식 `TextRun`에서 보완하는 구조였다. FormObject만 있는 셀은 TextRun이 없으므로 보완되지 않아
`cell_bboxes`의 meta가 0으로 남았다. 이로 인해 `cell_runs` 필터링이 실패하여 셀 진입이 불가했다.

#### 수정

1. **`CellBboxInfo`에 `section_index`, `has_meta` 추가**

2. **`collect_runs()`에 `current_table_meta: Option<(usize, usize, usize)>` 파라미터 추가**
   - `RenderNodeType::Table` 진입 시 `(section_index, para_index, control_index)` 추출하여 하위 전파
   - `TableCell` 노드 등록 시 `table_meta`로 즉시 `section_index`, `parent_para_index`, `control_index` 채움

3. **TextRun 보완 로직에 `section_index`, `has_meta=true` 추가** (TextRun이 있는 셀은 runs로 재확인)

4. **빈 `cell_runs` fallback 추가**
   - `cell_runs`가 비어있고 `has_meta==true`이면 `charOffset=0`으로 셀 진입 JSON 반환
   - `parentParaIndex`, `controlIndex`, `cellIndex`, `cellParaIndex`, `cellPath`, `cursorRect` 모두 포함

---

### #112 — 체크박스 클릭 토글 수정

#### 원인

`get_form_object_at_native()`가 반환하는 `para`/`ci`가 셀 내 문단 인덱스(`cp_idx`)와 셀 내 컨트롤 인덱스로,
`set_form_value_native(sec, para, ci)`가 `sections[sec].paragraphs[para].controls[ci]`로
flat 접근하여 Form 컨트롤을 찾지 못했다.

#### 수정

1. **`FormObjectNode`에 `cell_location: Option<(usize, usize, usize, usize)>` 추가** (`render_tree.rs`)
   - `(table_para_index, table_control_index, cell_index, cell_para_index)` 저장

2. **`paragraph_layout.rs`에서 `cell_location` 채우기**
   - 셀 내부 렌더링 시 `cell_ctx`에서 추출하여 `FormObjectNode.cell_location` 설정 (2곳)

3. **`get_form_object_at_native()` 수정** (`form_query.rs`)
   - `cell_location`이 있으면 `para=table_para_index` 반환 (최상위 문단)
   - JSON에 `inCell`, `tablePara`, `tableCi`, `cellIdx`, `cellPara` 추가 필드 포함

4. **`set_form_value_in_cell_native()` 신규 추가** (`form_query.rs`)
   - `sections[sec].paragraphs[table_para].controls[table_ci]` → `Table.cells[cell_idx].paragraphs[cell_para].controls[form_ci]` 체인으로 셀 내부 Form에 직접 접근

5. **WASM 바인딩 `setFormValueInCell` 추가** (`wasm_api.rs`)

6. **TypeScript 수정**
   - `FormObjectHitResult`에 `inCell?`, `tablePara?`, `tableCi?`, `cellIdx?`, `cellPara?` 필드 추가 (`types.ts`)
   - `wasm-bridge.ts`에 `setFormValueInCell()` 바인딩 추가
   - `input-handler.ts`의 `handleFormObjectClick()`에 `setFormVal` 헬퍼 추가:
     `inCell==true`이면 `setFormValueInCell()`, 아니면 기존 `setFormValue()` 호출

---

### E2E 테스트 신규 작성

**`rhwp-studio/e2e/form-control.test.mjs`**

| TC | 내용 | 결과 |
|----|------|------|
| TC-1 | form-002.hwpx 로드 (10페이지) | ✅ PASS |
| TC-2 | 렌더 트리에서 FormObject bbox 추출 | ✅ PASS |
| TC-3 | hitTest() 체크박스 셀 커서 진입 (#111) | ✅ PASS (`parentParaIndex=0, controlIndex=2`) |
| TC-4 | getFormObjectAt() 체크박스 감지 (#112) | ✅ PASS (`inCell:true, tableCi:2, cellIdx:6`) |
| TC-5 | setFormValueInCell() 체크박스 토글 (#112) | ✅ PASS (`value: 0→1`) |

---

## 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✅ 성공 |
| `cargo test` (785개) | ✅ 전체 통과 |
| WASM 빌드 | ✅ 성공 |
| E2E `form-control.test.mjs` | ✅ 5/5 PASS |

---

## 수정 파일 목록

| 파일 | 변경 내용 |
|------|----------|
| `src/document_core/queries/cursor_rect.rs` | #111: `collect_runs` table_meta 전파 + FormObject 셀 fallback |
| `src/renderer/render_tree.rs` | `FormObjectNode.cell_location` 필드 추가 |
| `src/renderer/layout/paragraph_layout.rs` | `cell_location` 채우기 (2곳) |
| `src/document_core/queries/form_query.rs` | #112: `cell_location` 반환 + `set_form_value_in_cell_native` 추가 |
| `src/wasm_api.rs` | `setFormValueInCell` WASM 바인딩 추가 |
| `rhwp-studio/src/core/types.ts` | `FormObjectHitResult` 셀 경로 필드 추가 |
| `rhwp-studio/src/core/wasm-bridge.ts` | `setFormValueInCell` 바인딩 추가 |
| `rhwp-studio/src/engine/input-handler.ts` | `setFormVal` 헬퍼로 셀 내부 분기 |
| `rhwp-studio/e2e/form-control.test.mjs` | E2E 테스트 신규 작성 |
| `rhwp-studio/public/samples/form-002.hwpx` | E2E 테스트용 샘플 추가 |

---

## 승인 요청

위 1단계 + 2단계 완료 결과를 검토 후 승인해주시면 최종 결과보고서를 작성하겠습니다.
