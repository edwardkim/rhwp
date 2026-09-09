# 완료 보고서 — Task M100-3175

- 이슈: #3175
- 제목: 표 범위 분할(split_table_cells_in_range) 시 local_resize_cell_widths/heights 정리 누락
- 작성일: 2026-07-23
- 브랜치: `fix/3175-table-split-range-stale-local-resize`

## 1. 완료 내용

`src/document_core/commands/table_ops.rs`의 `split_table_cells_in_range_native`에서
`table.split_cells_in_range(...)` 호출 직후 `local_resize_cell_widths`/`local_resize_cell_heights`를
비우도록 두 줄을 추가했다.

`Table::split_cells_in_range()`는 범위 내 각 셀에 대해 `split_cell_into()`를 반복 호출하며,
`split_cell_into()`는 새 셀들을 `push()`한 뒤 `cells` 배열을 재정렬한다. 그 결과 분할 이전 셀
인덱스를 물고 있던 `local_resize_cell_widths`/`local_resize_cell_heights` (`Vec<(usize, u32)>`)가
stale 참조로 남는다.

같은 파일의 자매 함수인 `merge_table_cells_native` / `split_table_cell_native` /
`split_table_cell_into_native` (#2853/#2859/#2843/#2832 계열)는 모두 셀 배치를 바꾼 직후 이
두 필드를 `clear()`하지만, `split_table_cells_in_range_native`만 이 정리 코드가 빠져 있었다.

## 2. 주요 변경

- `src/document_core/commands/table_ops.rs`
  - `split_table_cells_in_range_native`에서 `split_cells_in_range()` 호출 직후
    `local_resize_cell_widths.clear()` / `local_resize_cell_heights.clear()` 추가
- `src/wasm_api/tests.rs`
  - `test_split_table_cells_in_range_clears_stale_local_resize` 추가: 2x2 표에서
    stale `(1, 1234)`/`(1, 5678)` 항목을 채운 뒤 범위 분할을 호출하고, 자매 함수들과
    동일하게 두 필드가 비워지는지 검증

## 3. 검증 결과

- 수정 전: 새 테스트 FAIL (`범위 분할 후 local_resize_cell_widths/heights의 stale 참조가
  비워져야 한다`)
- 수정 후:
  - `cargo test --lib split_table` — 4 passed
  - `cargo test --lib table` (표 관련 전체) — 336 passed, 0 failed, 3 ignored
  - `cargo fmt --check -- src/document_core/commands/table_ops.rs src/wasm_api/tests.rs` — 통과
    (저장소 전체 `cargo fmt --check`는 기존에 알려진 Windows CRLF 노이즈만 발생, 변경 파일은 무관)

## 4. 리스크

- 매우 지역적인 수정(2줄)이며 자매 함수 3개와 동일한 패턴을 그대로 따른다.
- `clear()`는 stale 참조뿐 아니라 범위 분할 이전에 유효했던 로컬 리사이즈 값도 함께 지운다.
  이는 자매 함수들의 기존 동작과 일치하며, 병합/분할 후에는 로컬 리사이즈 값을 유지할
  방법이 없으므로(재정렬된 인덱스에 안전하게 매핑할 수단 없음) 동일하게 처리하는 것이 맞다.

## 5. 결론

Task M100-3175 구현과 검증을 완료했다. PR 생성은 별도 승인 후 진행한다.
