# Task m100-2843 — 표 행 삭제(delete_row) 후 local_resize_cell_widths/heights stale 인덱스 수정

## 이슈

edwardkim/rhwp#2843

## 근거 (문제)

`src/model/table.rs`의 `Table::delete_row()`(1056-1121행)는 삭제 대상 행에 앵커가 있고
`row_span == 1`인 셀을 `self.cells.retain(...)`으로 실제 제거하고, 이어서
`self.cells.sort_by_key(|c| (c.row, c.col))`으로 남은 셀을 재정렬한다. 즉 행 삭제 한 번으로
`cells` 벡터의 길이와 각 셀의 배열 인덱스 배치가 모두 바뀐다 — #2832에서 고친
`Table::merge_cells()`의 `retain()` + `sort_by_key()` 패턴과 정확히 동일하다.

`Table::local_resize_cell_widths: Vec<(usize, u32)>` / `local_resize_cell_heights: Vec<(usize, u32)>`
(`src/model/table.rs:66, 69`)는 바로 이 `cells` 인덱스를 키로 저장하는 필드다. `delete_row()`도,
이를 호출하는 커맨드 `delete_table_row_native()`(`src/document_core/commands/table_ops.rs:105`)도
삭제 후 이 두 필드를 전혀 갱신하지 않았다.

`merge_table_cells_native()`(#2832)가 이미 같은 정리를 병합 경로에 추가했지만, 삭제 경로는
누락된 상태였다.

## 전수 점검 결과

같은 관점으로 `src/model/table.rs`/`src/document_core/commands/table_ops.rs`의 나머지
셀-인덱스-시프팅 연산을 확인했다:

- `insert_row`/`insert_column`: `self.cells.push(...)` 후 `sort_by_key` — 정리 없음(동일 결함).
- `delete_row`(본 이슈): `retain()` + `sort_by_key` — 정리 없음(이번에 수정).
- `delete_column`: `retain()` + `sort_by_key` — 정리 없음(동일 결함).
- `split_cell`/`split_cell_into`: `push(...)` + `sort_by_key` — 정리 없음(동일 결함).
- `merge_cells`: `retain()` + `sort_by_key` — `merge_table_cells_native()`에서 이미 정리(#2832).
- `transpose_unmerged_table_in_place()`: 셀 전면 재구성 — 이미 `clear()` 정리함(기준 패턴).

즉 `merge_cells` 외의 모든 셀-인덱스-시프팅 연산(`insert_row`, `insert_column`, `delete_row`,
`delete_column`, `split_cell`, `split_cell_into`)이 같은 결함을 공유한다. 이번 작업은 속도 우선
정책에 따라 재현이 가장 단순한 `delete_row` 하나로 스코프를 좁혔고, 나머지는 이슈 #2843 본문에
후속 이슈로 명시해 두었다.

또한 `zones: Vec<TableZone>` 필드는 `cell_grid` 배열 인덱스가 아니라 `start_row/col`~`end_row/col`
좌표를 직접 저장하므로, 이번 조사 범위인 "인덱스 키 side table" 결함과는 다른 종류다(좌표가
삭제/삽입으로 의미상 어긋날 수 있는 별개 문제이며, 이번 수정 대상이 아니다).

## 재현 (red)

3×2 표(row 0,1,2 × col 0,1, 총 6셀)에서 셀 인덱스 2(row=1,col=0)에 로컬 resize 높이를 저장한 뒤
row 0을 삭제하면 셀 2개가 제거되어 `cells.len()`이 6→4가 되지만, `local_resize_cell_heights`는
여전히 존재하지 않는 인덱스 2를 가리키는 `[(2, H)]`를 유지한다.

- 테스트: `src/wasm_api/tests.rs::test_delete_table_row_clears_stale_local_resize_heights`
- 수정 전 실행 결과: FAILED —
  `행 삭제 후 셀 인덱스가 재배치되므로 local_resize_cell_heights의 stale 참조(인덱스 2)가 비워져야 한다`
  assertion 실패 (실제로 `[(2, 5678)]`가 그대로 남아 있음을 확인).

## 수정 (green)

`src/document_core/commands/table_ops.rs`의 `delete_table_row_native()`에서
`table.delete_row(...)` 성공 직후, `merge_table_cells_native()`(#2832)와 동일하게
`table.local_resize_cell_widths.clear()` / `table.local_resize_cell_heights.clear()`를 호출한다.
삭제된 행의 로컬 resize 힌트는 더 이상 유효하지 않은 인덱스를 가리키므로 정리가 안전하다.

수정 후 동일 테스트 결과: PASSED.

## 영향

`local_resize_cell_widths`/`heights`를 셀 인덱스로 조회하는 렌더링·직렬화 경로가 있다면 행 삭제
후 (1) 범위 초과 패닉, 또는 (2) 삭제 후 남은 엉뚱한 셀에 잘못된 로컬 resize 폭/높이가 적용되어
표가 찌그러져 보이는 렌더링 회귀로 이어질 수 있다. 사용자가 셀 경계를 드래그해 크기를 조정한
표(견적서·시간표 등)에서 행을 삭제하면 발생 가능.

## 검증

- `cargo build --lib`: 성공
- `cargo test --lib test_delete_table_row`: 1 passed
  (`test_delete_table_row_clears_stale_local_resize_heights`)
- `cargo clippy --all-targets --profile release-test -- -D warnings`: 경고 없음
- `rustfmt --edition 2021` 적용, 변경 파일 대상 추가 diff 없음 확인

## 변경 파일

- `src/document_core/commands/table_ops.rs` (수정)
- `src/wasm_api/tests.rs` (테스트 추가)
- `mydocs/report/task_m100_2843_report.md` (본 문서)
