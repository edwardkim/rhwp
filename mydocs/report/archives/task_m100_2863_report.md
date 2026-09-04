# task-m100-2863 처리 결과 보고

## 이슈

#2863 — 표 열 삭제/셀 분할 시 `local_resize_cell_widths`/`heights` stale 인덱스 미정리
(#2832/#2843/#2853 동일 버그 클래스의 마지막 인스턴스)

## 원인

`src/document_core/commands/table_ops.rs`의 `delete_table_column_native()`,
`split_table_cell_native()`, `split_table_cell_into_native()`가 셀 인덱스 배치를 바꾸는
모델 연산(`Table::delete_column()`, `Table::split_cell()`, `Table::split_cell_into()`) 호출
직후 `table.local_resize_cell_widths` / `table.local_resize_cell_heights`를 비우지 않아,
연산 이전 `cell_idx`를 그대로 가리키는 stale 참조가 남는 문제.

이미 수정된 `insert_table_row_native()`/`insert_table_column_native()`(#2853/#2859),
`delete_table_row_native()`(#2843/#2849), `merge_table_cells_native()`(#2832/#2841)와
동일한 근본 원인이다.

## 수정

`table_ops.rs`의 세 함수에서 모델 호출 직후 `table.dirty = true;` 부근에
`table.local_resize_cell_widths.clear();` / `table.local_resize_cell_heights.clear();`를
추가했다(기존 4개 수정과 동일 패턴).

- `delete_table_column_native()`
- `split_table_cell_native()`
- `split_table_cell_into_native()`

## 테스트

`src/wasm_api/tests.rs`에 `test_delete_table_column_and_split_cell_clear_stale_local_resize`
1개를 추가. 2×2 표에서 열 삭제 전/셀 분할 전 각각 `local_resize_cell_widths/heights`에
값을 채운 뒤 연산을 수행하고, 두 벡터가 비워졌는지 단언한다.

- Red: 수정 전 코드(`.clear()` 제거)로 실행 시 열 삭제 단언에서 FAILED 확인.
- Green: 수정 후 코드로 실행 시 PASS 확인.

## 검증

- `cargo build --lib`: 통과
- `cargo test --lib test_delete_table_column_and_split_cell_clear_stale_local_resize`: 통과
- `cargo clippy --all-targets --profile release-test -- -D warnings`: 경고 없음
- `rustfmt --edition 2021` (변경 파일만): 적용

## 변경 파일

- `src/document_core/commands/table_ops.rs`
- `src/wasm_api/tests.rs`
- `mydocs/report/task_m100_2863_report.md` (본 문서)
