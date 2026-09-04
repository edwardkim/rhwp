# Task m100-2832 — 표 셀 병합 후 local_resize_cell_widths/heights stale 인덱스 수정

## 이슈

edwardkim/rhwp#2832

## 근거 (문제)

`src/model/table.rs`의 `Table::merge_cells()`는 병합 범위 안의 비주 셀을 `self.cells.retain(...)`
으로 실제 제거하고 `self.cells.sort_by_key(|c| (c.row, c.col))`으로 재정렬한다. 즉 병합 한 번으로
`cells` 벡터의 길이와 각 셀의 배열 인덱스 배치가 모두 바뀐다.

그런데 `Table::local_resize_cell_widths: Vec<(usize, u32)>` /
`local_resize_cell_heights: Vec<(usize, u32)>` (`src/model/table.rs:75, 78`)는 바로 이 `cells`
인덱스를 키로 저장하는 필드다. `merge_cells()`도, 이를 호출하는 커맨드
`merge_table_cells_native()`(`src/document_core/commands/table_ops.rs:167`)도 병합 후 이 두
필드를 전혀 갱신하지 않았다.

셀 배치를 전면 재구성하는 형제 함수 `transpose_unmerged_table_in_place()`
(`src/model/table.rs:775-778`)는 정확히 이 문제를 이미 인지하고 재구성 직후 두 필드를 `clear()`
하는데, 병합 경로만 같은 정리를 누락한 상태였다.

## 재현 (red)

2×2 표에서 셀 인덱스 3(row=1,col=1)에 로컬 resize 폭을 저장한 뒤 (0,0)~(0,1)을 병합하면
비주 셀 1개가 제거되어 `cells.len()`이 4→3이 되지만, `local_resize_cell_widths`는 여전히
존재하지 않는 인덱스 3을 가리키는 `[(3, W)]`를 유지한다.

- 테스트: `src/wasm_api/tests.rs::test_merge_table_cells_clears_stale_local_resize_widths`
- 수정 전 실행 결과: FAILED —
  `병합 후 셀 인덱스가 재배치되므로 local_resize_cell_widths의 stale 참조(인덱스 3)가 비워져야 한다`
  assertion 실패 (실제로 `[(3, 1234)]`가 그대로 남아 있음을 확인).

## 수정 (green)

`src/document_core/commands/table_ops.rs`의 `merge_table_cells_native()`에서
`table.merge_cells(...)` 성공 직후, `transpose_unmerged_table_in_place()`와 동일하게
`table.local_resize_cell_widths.clear()` / `table.local_resize_cell_heights.clear()`를 호출한다.
병합 범위의 로컬 resize 힌트는 합쳐진 셀 하나로 대체되어 더 이상 유효하지 않으므로 정리가 안전하다.

수정 후 동일 테스트 결과: PASSED.

## 영향

`local_resize_cell_widths`/`heights`를 셀 인덱스로 조회하는 렌더링·직렬화 경로가 있다면 병합 후
(1) 범위 초과 패닉, 또는 (2) 병합 후 남은 엉뚱한 셀에 잘못된 로컬 resize 폭/높이가 적용되어 표가
찌그러져 보이는 렌더링 회귀로 이어질 수 있다. 사용자 정의 열 너비 표(견적서·시간표 등)에서
발생 가능.

## 검증

- `cargo build --lib`: 성공
- `cargo test --lib test_merge_table_cells`: 2 passed
  (`test_merge_table_cells`, `test_merge_table_cells_clears_stale_local_resize_widths`)
- `cargo clippy --all-targets --profile release-test -- -D warnings`: 경고 없음
- `rustfmt --edition 2021` 적용 후 `git diff --name-only` 추가 변경 없음 확인

## 변경 파일

- `src/document_core/commands/table_ops.rs` (수정)
- `src/wasm_api/tests.rs` (테스트 추가)
- `mydocs/report/task_m100_2832_report.md` (본 문서)
