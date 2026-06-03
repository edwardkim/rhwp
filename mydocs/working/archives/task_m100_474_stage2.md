# Task #474 Stage 2 — 정정 적용

## 정정 요약

`MeasuredTable::allows_row_break_split()` 메서드 추가 + `snap_to_block_boundary` + 호출부 3개소 (typeset.rs, engine.rs) 에 RowBreak 가드 적용.

## 변경 파일 목록

| 파일 | 변경 내용 |
|------|----------|
| `src/renderer/height_measurer.rs` | `allows_row_break_split()` 메서드 추가 + `snap_to_block_boundary` RowBreak 조기반환 |
| `src/renderer/typeset.rs` | `paginate_table` 의 `first_block_protected` / `cur_block_protected` / `next_block_protected` 가드 (3개소) |
| `src/renderer/pagination/engine.rs` | `split_table_rows` 의 `first_block_protected` / `cur_block_protected` / `next_block_protected` 가드 (3개소) |

## 정정 핵심

```rust
// height_measurer.rs
pub fn snap_to_block_boundary(&self, end_row: usize) -> usize {
    let rc = self.row_heights.len();
    if end_row >= rc { return end_row.min(rc); }
    // [Task #474] RowBreak 표는 보호 블록 정책 비적용
    if self.allows_row_break_split() { return end_row; }
    // ... (기존 보호 후퇴 로직)
}

pub fn allows_row_break_split(&self) -> bool {
    matches!(self.page_break, crate::model::table::TablePageBreak::RowBreak)
}

// typeset.rs / engine.rs (3개소 동일 패턴)
let first_block_protected = !mt.allows_row_break_split()
    && first_block_size >= 2
    && first_block_size <= BLOCK_UNIT_MAX_ROWS;
```

## dump-pages 회귀 정정 확인

### k-water-rfp.hwp 페이지 5 (대상 결함)

| 정정 전 (Stage 1 baseline) | 정정 후 (Stage 2) |
|---------------------------|------------------|
| `PartialTable rows=0..2` (4행 중 2행), used=196.3px (21%) | `PartialTable rows=0..4` (4행 모두), 표가 1페이지에 완성 |

→ 회귀 해소 ✅

### k-water-rfp.hwp 페이지 6 (이전 보호 블록 잔여)

| 정정 전 | 정정 후 |
|---------|---------|
| `PartialTable rows=2..4` + 잔여 컨텐츠 | `PartialTable rows=3..4` (표 1행 잔여) + 다음 컨텐츠 정상 배치 |

### synam-001.hwp 페이지 5/6 (회귀 점검)

| 페이지 | 결과 |
|--------|------|
| 5 | `PartialTable rows=0..5` (PR #401 v2 정정 보존) ✅ |
| 6 | `PartialTable rows=4..8` (PR #401 v2 정정 보존) ✅ |

→ synam-001 의 표 pi=69 는 RowBreak 이지만 block_size=5 > BLOCK_UNIT_MAX_ROWS=3 이라 기존 동작 그대로 유지. RowBreak 가드 적용 후에도 정정 영향 없음 (이미 분할 허용 상태).

## 검증 게이트

| 검증 | 결과 |
|------|------|
| `cargo test --lib` | **1077 passed** (이전 1075 + 신규 단위 테스트 2) |
| `cargo test --test svg_snapshot` | 6/6 ✅ |
| `cargo test --test issue_418` | 1/1 ✅ (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | 0건 ✅ |

## 단위 테스트 추가

`src/renderer/height_measurer.rs`:

- `test_snap_to_block_boundary_row_break_skipped` — RowBreak 표는 보호 후퇴 비적용
- `test_allows_row_break_split` — None / CellBreak / RowBreak 정책별 인지 확인

## 다음 단계

- **Stage 3**: 광범위 회귀 검증 (10 샘플 SVG byte 비교)
- **Stage 4**: 작업지시자 시각 검증 — k-water-rfp 5쪽/6쪽 + synam-001 5쪽/6쪽 + 다른 RowBreak 표 점검
- **Stage 5**: 최종 결과보고서 + 오늘할일 갱신
