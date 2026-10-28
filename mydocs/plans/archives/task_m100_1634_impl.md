# Task M100 #1634 구현 계획서 — 선택 셀 Transpose

## 구현 방침

1차 구현은 정적 행/열 바꿈 복사/붙여넣기다.

- `copy`: 현재 F5 셀 선택 범위를 Rust `DocumentCore` 내부 버퍼에 저장한다.
- `paste`: 현재 커서 셀을 좌상단으로 삼아 버퍼 내용의 행/열을 바꿔 대상 셀에 덮어쓴다.
- undo/redo는 Studio의 `SnapshotCommand`를 사용한다.

## Rust 모델

`src/model/table.rs`

- `Table::copy_transpose_range(start_row, start_col, end_row, end_col)` 추가
  - 범위 유효성 검사
  - 모든 셀이 `row_span == 1 && col_span == 1`인지 검사
  - 범위 내 모든 좌표가 실제 앵커 셀인지 검사
  - `Vec<Vec<Vec<Paragraph>>>` 형태로 셀 문단을 clone
- `Table::paste_transposed_cells(start_row, start_col, data)` 추가
  - 대상 크기: `source_cols × source_rows`
  - 대상 범위 유효성 검사
  - 병합 셀 거부
  - 대상 셀 `paragraphs` 교체
  - `dirty`, `rebuild_grid`는 호출자가 처리하거나 메서드에서 안정적으로 처리

## DocumentCore / WASM

`src/document_core/mod.rs` 또는 관련 구조체

- `transpose_clipboard: Option<TableTransposeClipboard>` 필드 추가
- `TableTransposeClipboard`에는 원본 행/열 수와 문단 매트릭스를 저장한다.

`src/document_core/commands/table_ops.rs`

- `copy_table_cells_transposed_native(...) -> Result<String, HwpError>`
- `paste_table_cells_transposed_native(...) -> Result<String, HwpError>`

반환 JSON:

```json
{"ok":true,"sourceRows":4,"sourceCols":2,"targetRows":2,"targetCols":4}
```

`src/wasm_api.rs`

- `copyTableCellsTransposed(...)`
- `pasteTableCellsTransposed(...)`
- `hasTableTransposeClipboard()`

## Studio

`rhwp-studio/src/core/wasm-bridge.ts`

- WASM 메서드 래퍼 추가
- 행/열 바꿈 복사 버퍼 보유 여부를 `EditorContext`에 노출

`rhwp-studio/src/command/commands/table.ts`

- `table:transpose-copy`
  - 셀 선택 모드에서만 실행
  - Ctrl 제외 셀이 있으면 실행하지 않음
  - 복사 후 대상 셀 지정이 쉽도록 셀 선택 모드를 종료
- `table:transpose-paste`
  - 표 내부 커서가 있을 때 실행
  - 행/열 바꿈 복사 버퍼가 있을 때만 활성화
  - 스냅샷 작업으로 WASM paste 호출

`rhwp-studio/index.html`, `rhwp-studio/src/engine/input-handler.ts`

- 표 메뉴와 표 셀 컨텍스트 메뉴에 행/열 바꿈 복사/붙여넣기 항목 추가

`rhwp-studio/src/engine/input-handler-keyboard.ts`

- 셀 선택 모드 중 단축키는 1차에서 추가하지 않는다. 메뉴/커맨드 팔레트 경유로 우선 제공한다.

## 테스트

`src/model/table/tests.rs`

- 4×2 범위 복사 후 다른 위치에 2×4 행/열 바꿈 붙여넣기
- 대상 범위 초과 실패
- 병합 셀 포함 실패
- 원본 유지 확인

Studio 정적 테스트:

- 표 메뉴/컨텍스트 메뉴에 행/열 바꿈 복사/붙여넣기 명령이 노출되는지 확인
