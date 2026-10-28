# Task M100 #1413 — 2단계 완료 보고서 (고인자 9~11 *Ex 4개)

- 브랜치: `local/task1413`
- 작성일: 2026-06-20
- 수정: `src/wasm_api.rs`(+100), `src/wasm_api/tests.rs`(+82)

## 1. 추가한 *Ex (4개, 순수 스칼라 options)

1단계 패턴(positional native 재호출 얇은 어댑터)대로:

| *Ex | 인자수 | options 키 |
|-----|--------|-----------|
| `insertClickHereFieldInCellEx` | 11 | sectionIdx/parentParaIdx/controlIdx/cellIdx/cellParaIdx/charOffset?/isTextbox?/guide?/memo?/name?/editable? |
| `splitTableCellsInRangeEx` | 10 | sectionIdx/parentParaIdx/controlIdx/startRow/startCol/endRow/endCol/nRows/mCols/equalRowHeight? |
| `splitTableCellIntoEx` | 9 | sectionIdx/parentParaIdx/controlIdx/row/col/nRows/mCols/equalRowHeight?/mergeFirst? |
| `moveVerticalEx` | 9 | sectionIdx/paraIdx/charOffset?/delta/preferredX/parentParaIdx?/controlIdx?/cellIdx?/cellParaIdx? |

- `moveVerticalEx`: cell 컨텍스트 키 생략 시 본문 이동(parentParaIdx=`u32::MAX` 동작과 동일).
- `json_f64`(preferredX), `json_i32`(delta) 헬퍼 활용 — f64 포함 타입 모두 커버.

## 2. 동치 테스트 (4건)

각 *Ex 가 같은 입력으로 positional 과 **동일 반환** (`task1413_*_ex_equivalent`):
- split_table_cell_into / split_table_cells_in_range / insert_click_here_field_in_cell:
  `create_doc_with_table()` 셋업 후 positional·*Ex 반환 동일.
- move_vertical: 본문 텍스트(다줄)에서 positional·*Ex 반환 동일.

## 3. 검증

- `cargo test --lib task1413`: **6/6 passed** (1단계 2 + 2단계 4).
- `cargo fmt --check`: CLEAN. `cargo clippy --lib`: **0**.

## 4. 다음 단계

- 3단계: 8인자 군 8개 (setPageHide, setCharShapeIdInCell, insertClickHereFieldByPath,
  getSelectionRectsInCell, exportSelectionInCellHtml, deleteRangeInCell, copySelectionInCell,
  applyCharFormatInCell) + 동치 테스트.
