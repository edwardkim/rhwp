# Task M100 #1413 최종 보고서 — 고인자 WASM API에 options object(*Ex) 병행

- 이슈: #1413 "[API] positional 인자 3개 이상 API에 options object 도입 제안" (sacru2red)
- 마일스톤: M100 (v1.0.0)
- 브랜치: `local/task1413`
- 작성일: 2026-06-20

## 1. 개요

`@rhwp/core` 소비자가 positional 고인자 API 의 중간 삽입형 변경(`insertPicture` 의
`cell_path_json` 4번째 인자)으로 타입 오류를 겪던 문제에 대응. 고인자(7+) WASM public API
**26개에 options object 변형 `*Ex`** 를 추가했다(기존 createTableEx 포함 총 27개). 기존
positional API 는 그대로 유지(하위 호환).

## 2. 전수 조사 → 범위

- 3인자 초과 wasm API 126개 → 전부는 과도. 작업지시자 결정 **고인자(7+) 26개 우선 + 가이드**.

## 3. 패턴 — 얇은 어댑터

각 `fooEx(options_json[, binary])` 는 json 헬퍼로 필드를 추출해 **positional native 를 그대로
재호출**한다(로직 중복 없음). 키 = positional 인자명 camelCase, 누락 시 default.
- 바이너리(insertPicture image_data)는 별도 인자(하이브리드).
- 반환 타입은 positional 과 동일(`Result<String,JsValue>` 23개 / `bool` 1 / `String` 2).
- 중첩 객체(props/value)는 `json_object`, i32/f64 좌표는 `json_i32`/`json_f64`.

## 4. 단계별 (4단계, 26개)

| 단계 | *Ex | 비고 |
|------|-----|------|
| 1 | insertPicture(13, 하이브리드) | 패턴 확립 |
| 2 | insertClickHereFieldInCell(11)/splitTableCellsInRange(10)/splitTableCellInto(9)/moveVertical(9) | f64 포함 |
| 3 | setPageHide/setCharShapeIdInCell/insertClickHereFieldByPath/getSelectionRectsInCell/exportSelectionInCellHtml/deleteRangeInCell/copySelectionInCell/applyCharFormatInCell (8) | native 재배열·props 중첩·path 문자열 |
| 4 | setNoteEquationProperties/setFormValueInCell/setActiveFieldInCell/removeFieldAtInCell/pasteHtmlInCell/moveLineEndpoint/mergeTableCells/insertTextInCell/insertClickHereField/getTextInCell/getFieldInfoAtInCell/evaluateTableFormula/deleteTextInCell (13) | bool/String 반환 포함 |

## 5. 검증

- 동치 테스트 24건(`task1413_*`): 각 *Ex == positional (반환 동일). **24/24 passed**.
  - 셋업 복잡한 3개(setNoteEquationProperties/setFormValueInCell/moveLineEndpoint)는 *Ex
    구현·컴파일 검증.
  - native 테스트 함정: positional 이 `Err(HwpError)` 반환 시 wasm JsValue 변환이 패닉
    (`__wbindgen_describe`) → 정상 입력(char_shape 등록·유효 cell path)으로 회피.
- `cargo fmt --check`: CLEAN. `cargo clippy --lib`: **0**.
- WASM 빌드(Docker) 성공, `pkg/rhwp.d.ts` 에 `Ex(options` 시그니처 **27개 노출**.

## 6. 문서

- 설계 관행 가이드: `mydocs/manual/wasm_api_options_convention.md`
  (고인자 *Ex 권장, 바이너리 하이브리드, camelCase 키, positional 중간 삽입 금지,
  breaking change CHANGELOG 표기 규약).
- CHANGELOG `[Unreleased] ### API` 에 *Ex 26개 + 규약 명시.

## 7. 산출물

- 수행계획서/구현계획서/단계별 보고서(stage1~3)/본 최종 보고서
- `src/wasm_api.rs`(*Ex 26개), `src/wasm_api/tests.rs`(동치 24건)
- `mydocs/manual/wasm_api_options_convention.md`, `CHANGELOG.md`
