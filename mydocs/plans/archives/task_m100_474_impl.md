# Task #474 구현 계획서 — RowBreak 표 보호 블록 정책 비적용

## 정정 위치

### 1. `MeasuredTable` 의 RowBreak 인지 메서드 추가

`src/renderer/height_measurer.rs`:

```rust
impl MeasuredTable {
    /// 표 정책이 RowBreak 인지 (행 단위 분할 허용 — rowspan 보호 블록 비적용 대상)
    pub fn allows_row_break_split(&self) -> bool {
        matches!(self.page_break, crate::model::table::TablePageBreak::RowBreak)
    }
    // ...
}
```

### 2. `snap_to_block_boundary` 에 RowBreak 가드

```rust
pub fn snap_to_block_boundary(&self, end_row: usize) -> usize {
    let rc = self.row_heights.len();
    if end_row >= rc {
        return end_row.min(rc);
    }
    // ★ Task #474: RowBreak 표는 보호 블록 정책 비적용 — 원래 end_row 그대로 반환
    if self.allows_row_break_split() {
        return end_row;
    }
    let block_start = self.row_block_start.get(end_row).copied().unwrap_or(end_row);
    let block_end = self.row_block_end.get(end_row).copied().unwrap_or(end_row + 1);
    if end_row == block_start {
        return end_row;
    }
    let block_size = block_end.saturating_sub(block_start);
    if block_size <= BLOCK_UNIT_MAX_ROWS {
        block_start
    } else {
        end_row
    }
}
```

### 3. 호출부 보호 블록 가드 (typeset.rs + engine.rs)

`paginate_table` / `split_table_rows` 에서:

```rust
// Before
let first_block_protected = first_block_size >= 2 
    && first_block_size <= BLOCK_UNIT_MAX_ROWS;

// After (RowBreak 표 비적용)
let first_block_protected = !mt.allows_row_break_split()
    && first_block_size >= 2 
    && first_block_size <= BLOCK_UNIT_MAX_ROWS;
```

같은 패턴으로 `cur_block_protected`, `next_block_protected` 가드도 정정.

## Stage 별 작업

### Stage 1: 베이스라인 측정

```bash
# 현재 회귀 정황 dump-pages 결과 보존
cargo run --release --quiet --bin rhwp -- dump-pages samples/k-water-rfp.hwp -p 4 > /tmp/baseline_p5.txt
cargo run --release --quiet --bin rhwp -- dump-pages samples/k-water-rfp.hwp -p 5 > /tmp/baseline_p6.txt

# synam-001 페이지 5 (PR #401 v2 정정 보존 확인) baseline
cargo run --release --quiet --bin rhwp -- dump-pages samples/synam-001.hwp -p 4 > /tmp/baseline_synam001_p5.txt

# 광범위 회귀 baseline (10 샘플 SVG)
mkdir -p output/svg/task474-baseline
# ... export-svg 10 샘플
```

### Stage 2: 정정 적용

1. `MeasuredTable::allows_row_break_split` 메서드 추가
2. `snap_to_block_boundary` 에 RowBreak 가드
3. `paginate_table` (typeset.rs) 의 `first_block_protected` / `cur_block_protected` / `next_block_protected` 가드
4. `split_table_rows` (engine.rs) 동일 패턴

### Stage 3: 회귀 검증

```bash
# k-water-rfp 페이지 5 정정 확인
cargo run --release --quiet --bin rhwp -- dump-pages samples/k-water-rfp.hwp -p 4
# 기대: PartialTable rows=0..4 (또는 더 많은 행) — 회귀 해소

# synam-001 페이지 5 보존 확인
cargo run --release --quiet --bin rhwp -- dump-pages samples/synam-001.hwp -p 4
# 기대: PR #401 v2 정정 그대로 (synam-001 의 표는 RowBreak 가 아닌 정책일 가능성)

# 광범위 byte 비교 (10 샘플)
mkdir -p output/svg/task474-test
# ... export-svg 10 샘플 + cmp
```

### Stage 4: 시각 검증 (작업지시자 직접)

- k-water-rfp 5쪽 / 6쪽 debug-overlay SVG 비교
- synam-001 5쪽 회귀 없음 확인
- 다른 RowBreak 표 샘플 점검

### Stage 5: 최종 결과보고서 + 오늘할일 갱신

## 검증 게이트

- `cargo test --lib`: 회귀 0건 (1075 passed 유지)
- `cargo test --test svg_snapshot`: 6/6
- `cargo test --test issue_418`: 1/1 (Task #418 보존)
- `cargo clippy --lib -- -D warnings`: 0건
- `MeasuredTable::allows_row_break_split` 단위 테스트 추가

## 위험 정황 + 회피

- **PR #401 v2 의 synam-001 정정 보존**: synam-001 표가 RowBreak 가 아니면 본 정정의 영향 받지 않음 → Stage 1 에서 synam-001 표 정책 확인 필수
- **다른 RowBreak 표 회귀**: rowspan>1 셀이 있는 RowBreak 표가 본 정정으로 행 단위 분할 → 시각상 잘림 위험. 그러나 RowBreak 의 의미가 "행 경계 분할 허용" 이라 정합

## 단위 테스트 추가

```rust
#[test]
fn snap_to_block_boundary_skipped_for_row_break_table() {
    // RowBreak 표는 보호 블록 정책 비적용 → end_row 그대로
    let mt = MeasuredTable {
        // rowspan=2 보호 블록이 있어도 RowBreak 면 그대로
        page_break: TablePageBreak::RowBreak,
        row_block_start: vec![0, 1, 2, 2],
        row_block_end: vec![1, 2, 4, 4],
        // ...
    };
    assert_eq!(mt.snap_to_block_boundary(3), 3); // 후퇴 없음
}

#[test]
fn snap_to_block_boundary_protects_for_non_row_break_table() {
    // None / CellBreak 표는 기존 동작 유지
    let mt = MeasuredTable {
        page_break: TablePageBreak::None,
        row_block_start: vec![0, 1, 2, 2],
        row_block_end: vec![1, 2, 4, 4],
        // ...
    };
    assert_eq!(mt.snap_to_block_boundary(3), 2); // 보호 후퇴
}
```
