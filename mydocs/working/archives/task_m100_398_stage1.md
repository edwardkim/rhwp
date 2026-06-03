# Task #398 Stage 1 — `MeasuredTable` rowspan 묶음 메타데이터 추가

## 변경 사항

### `src/renderer/height_measurer.rs`

1. **`MeasuredTable` 구조체** 에 두 필드 추가:
   - `row_block_start: Vec<usize>`
   - `row_block_end: Vec<usize>`
   - 길이는 `row_heights.len()` 와 동일. 빈 vec이면 모든 행을 단일 블록으로 간주(폴백).

2. **모듈 수준 `compute_row_blocks(table, row_count)` 헬퍼** 추가:
   - 모든 셀의 `row` / `row_span` 을 검사하여 행별 블록 (start, end_exclusive) 산출
   - 겹치는 rowspan 묶음 통합 (전이 폐포)
   - 같은 블록 내 모든 행이 동일 (start, end) 갖도록 정규화

3. **`MeasuredTable` 메서드** 추가:
   - `row_block_for(row) -> (start, end_exclusive, height)`
     - height = `range_height(start, end)`
     - row_block_* 비어있으면 단일 행 폴백
   - `snap_to_block_boundary(end_row) -> usize`
     - end_row가 블록 중간이면 블록 시작 행으로 후퇴
     - end_row >= row_count 이면 그대로 반환

4. **모든 `MeasuredTable` 인스턴스화 지점** 갱신:
   - `measure_table_impl` 정상 분기 (`row_count` 결정 후 `compute_row_blocks` 호출)
   - `measure_table_impl` MAX_NESTED_DEPTH 조기 반환 분기 (table.cells 기준)
   - 기존 단위 테스트 7개 (`row_block_start: vec![], row_block_end: vec![]` 추가)

## 신규 단위 테스트 (7개)

| 테스트 | 검증 내용 |
|--------|-----------|
| `test_compute_row_blocks_all_single` | rowspan 모두 1: `s=[0,1,2]`, `e=[1,2,3]` |
| `test_compute_row_blocks_rs2_at_row0` | 행 0에 rs=2 셀: `s=[0,0,2]`, `e=[2,2,3]` |
| `test_compute_row_blocks_overlapping` | 겹치는 rowspan: 통합되어 전체가 단일 블록 |
| `test_compute_row_blocks_disjoint` | 비인접 rowspan: 별개 블록 유지 |
| `test_row_block_for_basic` | 같은 블록 내 모든 행이 동일 (start, end, height) 반환 |
| `test_row_block_for_empty_metadata` | row_block_* 빈 vec → 단일 행 폴백 |
| `test_snap_to_block_boundary` | 블록 중간 → 시작으로 후퇴, 경계는 그대로, 행 끝은 그대로 |

## 빌드/테스트 결과

```
cargo build --lib  → ok (17.80s)
cargo test --lib renderer::height_measurer
  → 18 passed; 0 failed (기존 11 + 신규 7)
```

## 호출부 변경 없음

이 단계는 자료구조와 헬퍼만 추가했으며 `split_table_rows`/`find_break_row` 호출부는 변경하지 않았다 (단계 2). 기존 동작에 영향 없음.

## 다음 단계

- 단계 2: `split_table_rows` 첫 행 판정에 `row_block_for(0)` 적용 + `find_break_row` 결과 스냅 + 인트라-로우 분할 가드.
