# Task M100 #1413 — 3단계 완료 보고서 (8인자 군 *Ex 8개)

- 브랜치: `local/task1413`
- 작성일: 2026-06-20
- 수정: `src/wasm_api.rs`(+171), `src/wasm_api/tests.rs`(+116)

## 1. 추가한 *Ex (8개)

| *Ex | options 키 비고 |
|-----|----------------|
| `setPageHideEx` | sec/para/hideHeader~hidePageNum (bool 6) |
| `setCharShapeIdInCellEx` | secIdx/.../startOffset/endOffset/charShapeId |
| `insertClickHereFieldByPathEx` | path: string(cell_path JSON) + guide/memo/name/editable |
| `getSelectionRectsInCellEx` | cell 좌표 + start/end para·offset (native 인자 재배열 유지) |
| `exportSelectionInCellHtmlEx` | 동일 |
| `deleteRangeInCellEx` | 동일 (native 재배열) |
| `copySelectionInCellEx` | 동일 |
| `applyCharFormatInCellEx` | props: object(중첩 JSON, `json_object` 추출) |

- native 인자 재배열(get_selection_rects/delete_range)·중첩 props(applyCharFormat)·
  path 문자열(by_path) 등 변형을 positional 과 동일하게 처리.

## 2. 동치 테스트 (8건)

각 *Ex == positional (반환 동일). 표 기반은 `create_doc_with_table()`, setPageHide 는 본문.

### 정정 사항 (native 에러→JsValue 패닉 회피)
- native 테스트에서 positional API 가 `Err(HwpError)` 를 반환하면 `.map_err(|e| e.into())`
  의 wasm JsValue 변환이 패닉(`__wbindgen_describe`)한다. 따라서 **에러 안 나는 정상 입력**
  을 줘야 한다:
  - `setCharShapeIdInCellEx`: char_shape_id=0 유효하려면 `char_shapes` 1개 등록 필요
    (없으면 "범위 초과" Err) → 테스트에서 `CharShape::default()` push.
  - `insertClickHereFieldByPathEx`: 빈 `"[]"` path 는 Err → 유효 cell path
    `[{"controlIndex":0,"cellIndex":0,"cellParaIndex":0}]` 사용. *Ex options 의 path 값은
    escape 된 JSON 문자열(`json_str` 가 정상 디코드).

## 3. 검증

- `cargo test --lib task1413`: **14/14 passed** (1단계 2 + 2단계 4 + 3단계 8).
- `cargo fmt --check`: CLEAN. `cargo clippy --lib`: **0**.

## 4. 다음 단계

- 4단계: 7인자 군 13개 + 설계 관행 가이드(`mydocs/manual/wasm_api_options_convention.md`)
  + CHANGELOG breaking change 규약 + WASM 빌드(rhwp.d.ts *Ex 노출) + 최종 보고서.
