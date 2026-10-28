# Task #1716 Stage 1 완료보고서 — 공유 헬퍼 `leading_header_rows`

## 작업
`src/model/table.rs` `impl Table` 에 반복 제목행 산출 헬퍼 추가:
```rust
pub fn leading_header_rows(&self) -> Vec<usize>
```
- 각 행의 제목여부를 rowspan 덮개까지 반영해 계산(header 셀이 `row..row+row_span` 덮음).
- 행 0부터 제목행이 **연속되는 최대 구간 `0..H`** 만 반환. 표 중간·하단에 흩어진 `is_header`
  행은 제외.

## 단위 테스트 (`src/model/table/tests.rs`, 4개)
- `test_leading_header_rows_scattered_body_headers`: 상단 1행 + 본문 흩어진 header → `[0]`.
- `test_leading_header_rows_contiguous_multi`: 상단 연속 2행 → `[0,1]` (#1022 다중 머리행 보존).
- `test_leading_header_rows_rowspan_header`: rs=2 header 셀 덮개 → `[0,1]`.
- `test_leading_header_rows_none_and_all`: header 없음 → `[]`, 전 행 header → 전체.

## 검증
- `cargo test --release leading_header_rows` **통과**(exit 0).
- 대표 파일 실제 분포 확인: header 행 169개(0,15,…,182)지만 상단 연속 H=**1** → 접근 유효.

## 상태
완료. 이 헬퍼를 Stage 2(페이지네이터)·Stage 3(렌더러)에서 공유해 desync 차단.
