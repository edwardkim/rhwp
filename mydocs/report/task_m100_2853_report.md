# 완료 보고서 — Task M100-2853

- 이슈: #2853
- 제목: 표 행/열 삽입 시 local_resize_cell_widths/heights stale 인덱스 미정리 (#2832/#2843 동일 버그 클래스)
- 작성일: 2026-07-22
- 브랜치: `task/m100-2853-insert-row-col-stale-local-resize`

## 1. 완료 내용

`insert_table_row_native`/`insert_table_column_native`(`src/document_core/commands/table_ops.rs`)에서
`Table::insert_row()`/`Table::insert_column()` 호출 직후 `local_resize_cell_widths`/
`local_resize_cell_heights`를 `clear()`하도록 수정했다.

두 모델 함수 모두 새 셀을 `push()`한 뒤 `self.cells.sort_by_key(|c| (c.row, c.col))`로 전체 셀
배열을 재정렬한다. `local_resize_cell_widths/heights`는 `Vec<(usize, u32)>`로 cell 배열 인덱스를
직접 키로 쓰기 때문에, 재정렬 이전 인덱스가 재정렬 이후에는 다른 셀을 가리키거나 범위를
벗어나는 stale 참조가 된다. 이는 #2832(PR #2841, 셀 병합)·#2843(PR #2849, 행 삭제)에서 이미
확인·수정된 것과 동일한 버그 클래스이며, 이번 건은 그 세 번째·네 번째 사례(행 삽입/열 삽입)다.

## 2. 주요 변경

- `src/document_core/commands/table_ops.rs`
  - `insert_table_row_native`: `table.insert_row(...)` 호출 직후 `local_resize_cell_widths`/
    `local_resize_cell_heights` clear 추가
  - `insert_table_column_native`: `table.insert_column(...)` 호출 직후 동일하게 clear 추가
- `src/wasm_api/tests.rs`
  - `test_insert_table_row_and_column_clear_stale_local_resize` 추가 (3×2 표에서 셀 인덱스 2에
    로컬 resize 폭/높이를 기록한 뒤 행 삽입, 이어서 열 삽입을 수행하며 각 단계 후 두 벡터가
    비어 있는지 확인)

## 3. 검증 결과

통과:

- `cargo build --lib`
- `cargo test --lib test_insert_table_row_and_column_clear_stale_local_resize` (1 passed)
- `cargo clippy --all-targets --profile release-test -- -D warnings`
- `rustfmt --edition 2021` (변경 파일만)

## 4. 잔여

같은 버그 클래스로 코드 확인은 되었으나 이번 PR 범위 밖으로 남겨둔 항목 (별도 이슈/PR 필요):

- `delete_table_column_native` (열 삭제)
- `split_table_cell_native`/`split_cell_into` (셀 분할)
