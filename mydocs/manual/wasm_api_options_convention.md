# WASM API 설계 관행 — options object(*Ex) (#1413)

`@rhwp/core` 의 WASM public API 설계 규약. downstream(앱·라이브러리) 의 업그레이드 비용을
줄이기 위한 관행이다.

## 배경

0.x 단계라 API 변경이 잦다. positional 인자가 많은 API 는 **중간 삽입형 변경**(예:
`insertPicture` 의 `cell_path_json` 4번째 인자 추가)에서 기존 호출이 타입 오류로 깨진다
(`Expected 11-13 arguments, but got 10`). 인자 순서만 바뀌어도 모든 호출부를 수동 점검해야
한다(#1413).

## 규약

### 1. 고인자(3+) API 는 options object(`*Ex`) 병행 제공

인자가 많고(특히 7+) 확장 가능성이 높은 API 는 기존 positional 을 유지하면서
`fooEx(options_json: &str)` 변형을 함께 제공한다(하위 호환 + 안정적 옵션). 이미 확립된
`createTableEx` 패턴을 따른다.

```rust
#[wasm_bindgen(js_name = insertTextInCellEx)]
pub fn insert_text_in_cell_ex(&mut self, options_json: &str) -> Result<String, JsValue> {
    use crate::document_core::helpers::{json_str, json_u32};
    self.insert_text_in_cell_native(/* JSON 에서 추출한 positional 인자 */)
        .map_err(|e| e.into())
}
```

- **얇은 어댑터**: *Ex 는 JSON 필드를 추출해 **기존 positional native 를 그대로 재호출**한다.
  로직 중복 없음. positional 과 동치.
- **반환 타입 동일**: positional 이 `Result<String,JsValue>`/`bool`/`String` 무엇이든 *Ex 도
  동일하게 맞춘다.

### 2. 바이너리/특수 인자는 하이브리드

이미지 바이트 같은 바이너리는 JSON(base64)에 넣지 않고 **별도 인자**로 받는다.

```rust
pub fn insert_picture_ex(&mut self, options_json: &str, image_data: &[u8]) -> Result<String, JsValue>
```

### 3. JSON 키 = positional 인자명 camelCase

`section_idx` → `sectionIdx`. TS 친화. 누락 시 positional default 와 동일하게 처리
(`json_u32(..).unwrap_or(0)` 등). 중첩 객체(서식 props 등)는 `json_object` 로 추출.

### 4. positional 변경 시 — 중간 삽입 금지, 끝에 추가

부득이 positional 시그니처를 바꿔야 하면 **인자를 중간에 끼우지 말고 끝에 추가**한다(기존
호출이 덜 깨짐). 동시에 *Ex 를 제공해 downstream 이 options 로 옮겨가게 한다.

## breaking change 표기 규약 (CHANGELOG)

WASM public API 의 positional 시그니처가 바뀌면 CHANGELOG 에 다음을 명시한다.

- `[API]` 머리말 + 대상 함수명.
- "positional arg index changed" 또는 "arg added at index N".
- before/after migration snippet (TS).
- 가능하면 `*Ex` options 대안 안내.

예:
```
### API
- `insertPicture`: cell_path_json 인자가 index 3 에 추가됨. 기존 positional 호출은
  인자 위치를 조정해야 한다. options 방식 `insertPictureEx` 를 권장한다 (#1413).
```

## 현재 *Ex 제공 API (#1413 기준, 27개)

`createTableEx`(#기존) + insertPicture / insertClickHereFieldInCell /
splitTableCellsInRange / splitTableCellInto / moveVertical / setPageHide /
setCharShapeIdInCell / insertClickHereFieldByPath / getSelectionRectsInCell /
exportSelectionInCellHtml / deleteRangeInCell / copySelectionInCell / applyCharFormatInCell /
setNoteEquationProperties / setFormValueInCell / setActiveFieldInCell / removeFieldAtInCell /
pasteHtmlInCell / moveLineEndpoint / mergeTableCells / insertTextInCell /
insertClickHereField / getTextInCell / getFieldInfoAtInCell / evaluateTableFormula /
deleteTextInCell — 각 `...Ex(options_json[, binary])`.

확장 대상: 7인자 이상 고인자 API 우선 적용(#1413). 나머지 3~6인자는 점진 확대.
