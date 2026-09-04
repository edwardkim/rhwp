# Task M100 #1413 구현계획서 — *Ex(options) API 추가 (고인자 26개)

- 이슈: #1413, 마일스톤 M100, 브랜치 `local/task1413`
- 작성일: 2026-06-20
- 수행계획서: `mydocs/plans/task_m100_1413.md`

## 0. 인자 타입 분류 (조사 완료)

- json 헬퍼 충분: `json_u32`/`json_i32`/`json_u16`/`json_f64`/`json_usize`/`json_bool`/
  `json_str`/`json_object`/배열류 — 모든 스칼라·문자열·bool 커버.
- **insertPicture(1)**: `image_data: &[u8]`(바이너리) + `Option<i32>` 2개 → 하이브리드.
- **나머지 25**: 순수 스칼라/문자열/bool → 순수 JSON options 로 표현 가능. 콜백/특수 타입
  없음.

## 1. 패턴 — *Ex 는 positional 의 얇은 어댑터

각 `fooEx(options_json)` 는 JSON 에서 필드를 뽑아 **기존 positional `foo_native`/내부 호출을
그대로 호출**한다(로직 중복 없음). `createTableEx` 동형.

```rust
#[wasm_bindgen(js_name = insertTextInCellEx)]
pub fn insert_text_in_cell_ex(&mut self, options_json: &str) -> Result<String, JsValue> {
    use crate::document_core::helpers::{json_u32, json_str};
    let section_idx = json_u32(options_json, "sectionIdx").unwrap_or(0);
    // ... 필드 추출 ...
    self.insert_text_in_cell_native(/* positional */).map_err(|e| e.into())
}
```

### 1.1 하이브리드 — insertPictureEx

```rust
#[wasm_bindgen(js_name = insertPictureEx)]
pub fn insert_picture_ex(&mut self, options_json: &str, image_data: &[u8])
    -> Result<String, JsValue>
```
- image_data 는 Uint8Array 별도 인자. options JSON: sectionIdx/paraIdx/charOffset/
  cellPath(string)/width/height/naturalWidthPx/naturalHeightPx/extension/description/
  paperOffsetXHu?/paperOffsetYHu?. 내부 `insert_picture_native` 호출.

### 1.2 키 명명 규약

positional 인자명을 camelCase 로 (`section_idx`→`sectionIdx`). JSON 키 = TS 친화.
누락 시 기존 default(positional 동작과 동일)로 처리.

## 2. 동치 보장 — 테스트

각 *Ex 는 **같은 입력으로 positional 과 결과 동일**해야 한다. 네이티브 테스트(`DocumentCore`
직접)로 positional 경로와 *Ex 경로(options_json 구성 → 파싱 → 동일 native 호출)의 반환·
문서 상태를 대조한다. wasm_bindgen 함수 자체는 native 호출의 얇은 래퍼라, native 동치 +
JSON 파싱 단위 테스트로 커버.

## 3. 단계 분할 (26개 — 4단계)

### 1단계 — insertPictureEx (하이브리드 패턴 확립)
- 가장 복잡한 1개. 하이브리드 *Ex + JSON 파싱(cellPath/Option 포함) + 동치 테스트.
- 필요 시 json 헬퍼 보강.

### 2단계 — 고인자 순수 스칼라 (9~11인자, 4개)
- insertClickHereFieldInCell(11), splitTableCellsInRange(10), splitTableCellInto(9),
  moveVertical(9, f64 포함). + 동치 테스트.

### 3단계 — 8인자 군 (8개)
- setPageHide, setCharShapeIdInCell, insertClickHereFieldByPath, getSelectionRectsInCell,
  exportSelectionInCellHtml, deleteRangeInCell, copySelectionInCell, applyCharFormatInCell.

### 4단계 — 7인자 군 (13개) + 가이드 문서
- setNoteEquationProperties, setFormValueInCell, setActiveFieldInCell, removeFieldAtInCell,
  pasteHtmlInCell, moveLineEndpoint, mergeTableCells, insertTextInCell, insertClickHereField,
  getTextInCell, getFieldInfoAtInCell, evaluateTableFormula, deleteTextInCell.
- 설계 관행 가이드(`mydocs/manual/wasm_api_options_convention.md`) + CHANGELOG breaking
  change 표기 규약 + WASM 빌드(rhwp.d.ts 에 *Ex 노출 확인).

(각 단계 완료 후 보고 + 승인. 단계 내 일괄이 과하면 추가 분할.)

## 4. 검증

- 단계별 동치 테스트(native positional == Ex).
- `cargo test --profile release-test --tests` + fmt + clippy.
- WASM 빌드(Docker) + rhwp.d.ts 에 *Ex 26개 노출.

## 5. 위험과 대응

| 위험 | 대응 |
|------|------|
| *Ex 가 positional 과 미묘하게 다른 동작 | 얇은 어댑터(native 재호출) + 동치 테스트 |
| JSON 키 명명 불일치(downstream 혼란) | camelCase 규약 + 가이드 문서에 키 표 |
| 26개 대형 diff | 4단계 분할, 단계별 승인 |
| 기존 positional 호출 회귀 | positional 불변(추가만) — 기존 테스트 그린 |

## 6. 산출물

- `src/wasm_api.rs`(*Ex 26개) + 동치/파싱 테스트
- `mydocs/manual/wasm_api_options_convention.md`(설계 관행) + CHANGELOG 규약
- 단계별/최종 보고서
