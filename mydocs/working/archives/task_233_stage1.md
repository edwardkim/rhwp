# Task 233 1단계 완료 보고서: WASM API 구현

## 완료 내용

### FormObjectNode 문서 위치 정보 추가
- `src/renderer/render_tree.rs`: FormObjectNode에 `section_index`, `para_index`, `control_index`, `name` 필드 추가
- `src/renderer/layout/paragraph_layout.rs`: FormObjectNode 생성 시 위치 정보 전달 (인라인 렌더링 2곳)

### 네이티브 API 구현
- `src/document_core/queries/form_query.rs` (신규): 4개 네이티브 메서드
  - `get_form_object_at_native(page_num, x, y)` — 렌더 트리 재귀 순회 + bbox 좌표 충돌 검사
  - `get_form_value_native(sec, para, ci)` — 문서 트리에서 Control::Form 값 조회
  - `set_form_value_native(sec, para, ci, value_json)` — value/text/caption 설정 + recompose_section
  - `get_form_object_info_native(sec, para, ci)` — properties HashMap 포함 상세 정보

### WASM 바인딩
- `src/wasm_api.rs`: 4개 wasm_bindgen API 추가
  - `getFormObjectAt(pageNum, x, y)` → JSON
  - `getFormValue(sec, para, ci)` → JSON
  - `setFormValue(sec, para, ci, valueJson)` → JSON
  - `getFormObjectInfo(sec, para, ci)` → JSON

## 검증 결과
- `cargo check`: 컴파일 성공
- `cargo test`: 716개 통과, 0개 실패
- `samples/form-01.hwp` SVG 내보내기 정상
