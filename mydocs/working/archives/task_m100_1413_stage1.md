# Task M100 #1413 — 1단계 완료 보고서 (insertPictureEx 하이브리드 패턴 확립)

- 브랜치: `local/task1413`
- 작성일: 2026-06-20
- 수정: `src/wasm_api.rs`(+57), `src/wasm_api/tests.rs`(+90)

## 1. insertPictureEx 추가 (하이브리드 *Ex 패턴)

```rust
#[wasm_bindgen(js_name = insertPictureEx)]
pub fn insert_picture_ex(&mut self, options_json: &str, image_data: &[u8])
    -> Result<String, JsValue>
```

- 이미지 바이너리는 별도 `image_data`(Uint8Array) 인자, 나머지는 JSON options.
- options 키(camelCase): sectionIdx/paraIdx/charOffset?/cellPath?/width/height/
  naturalWidthPx/naturalHeightPx/extension?/description?/paperOffsetXHu?/paperOffsetYHu?.
- json 헬퍼(`json_u32`/`json_i32`/`json_str`)로 추출 후 **positional `insert_picture_native`
  를 그대로 호출** — 로직 중복 없는 얇은 어댑터. paperOffset 키 부재 시 `None`(positional
  Option 동작과 동일).

## 2. 동치 테스트 (2건)

- `task1413_insert_picture_ex_equivalent_to_positional`: 같은 입력으로 positional
  `insertPicture` 와 `insertPictureEx` 각각 삽입 → **반환 JSON 동일 + 렌더 SVG 이미지 수
  동일**(1개).
- `task1413_insert_picture_ex_optional_keys_default`: optional 키(extension/description/
  paperOffset/cellPath) 생략 시 default 처리(본문 inline 삽입) 성공.

## 3. 검증

- `cargo test --lib task1413_insert_picture_ex`: **2/2 passed**.
- `cargo fmt --check`: 정렬 1건 적용 후 CLEAN.
- `cargo clippy --lib`: **0**.
- native lib 빌드 OK (`#[wasm_bindgen]` impl 이 native 에서도 컴파일 — *Ex 를 native
  테스트로 직접 검증 가능).

## 4. 확립된 패턴 (2~4단계 적용)

- `fooEx(options_json[, binary])` → json 헬퍼로 필드 추출 → 기존 positional native 재호출.
- 키 = positional 인자명 camelCase, 누락 시 positional default.
- 동치 테스트: positional == Ex (반환/문서 상태).

## 5. 다음 단계

- 2단계: 고인자 9~11 (insertClickHereFieldInCell/splitTableCellsInRange/splitTableCellInto/
  moveVertical) — 순수 스칼라 *Ex + 동치 테스트.
