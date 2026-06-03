# Task #398: 표 페이지 분할 시 rowspan>1 셀 누락 — 구현계획서

## 설계 결정

### 1. 데이터 구조

`MeasuredTable`에 사전 계산된 블록 경계 캐시를 추가한다:

```rust
pub struct MeasuredTable {
    // ... 기존 필드
    /// 각 행이 속한 rowspan 묶음 블록의 시작 행 인덱스.
    /// row_block_start[r] = r 행을 포함하는 모든 rowspan 셀의 최소 시작 행.
    /// 단순 행(rowspan=1만 포함)이면 row_block_start[r] = r.
    pub row_block_start: Vec<usize>,
    /// 각 행이 속한 rowspan 묶음 블록의 종료 행 (exclusive).
    /// row_block_end[r] = max(c.row + c.row_span) for cells c such that
    ///                    c.row <= r < c.row + c.row_span
    pub row_block_end: Vec<usize>,
}
```

산출 규칙 (build 시):
```
1. 모두 row_block_start[r] = r, row_block_end[r] = r+1 로 초기화.
2. 각 셀 c (rs=row_span)에 대해:
   for r in c.row .. c.row+rs:
       row_block_start[r] = min(row_block_start[r], c.row)
       row_block_end[r]   = max(row_block_end[r],   c.row + rs)
3. 전이 폐포: 묶음끼리 겹치면 통합 (예: 셀 A가 행 0~2, 셀 B가 행 2~4 → 블록 0~4).
   while changed: for r: row_block_start[r] = min over r2 in [start[r]..end[r]] of start[r2];
                       row_block_end[r]   = max over r2 in [start[r]..end[r]] of end[r2];
```

`mt.cells`(`page_break == CellBreak`일 때만 채움)에 의존하지 말고 **`table.cells`(model)** 에서 직접 산출. 모든 page_break 모드에서 일관 적용.

### 2. 신규 메서드

`MeasuredTable`:
```rust
/// row가 시작하는 블록의 (start, end_exclusive, height) 반환.
/// height = range_height(start, end). 단일 행이면 (row, row+1, row_heights[row]).
pub fn row_block_for(&self, row: usize) -> (usize, usize, f64);

/// 종료 행 후보가 블록 중간이면 블록 시작으로 후퇴.
pub fn snap_to_block_boundary(&self, end_row: usize) -> usize;
```

### 3. 호출부 변경

**`pagination/engine.rs::split_table_rows` (line 1492~1507):**
```rust
// before
let first_row_h = if row_count > 0 { mt.row_heights[0] } else { 0.0 };
if remaining_on_page < first_row_h && !st.current_items.is_empty() { ... }

// after
let (_, _, first_block_h) = if row_count > 0 {
    mt.row_block_for(0)
} else { (0, 0, 0.0) };
if remaining_on_page < first_block_h && !st.current_items.is_empty() { ... }
```

`first_row_h`를 사용하는 다른 위치도 동일하게 교체 (find via grep).

**`height_measurer.rs::find_break_row` (line 1179):**
- 기존 partition_point 결과 `pos` 산출 후, `snap_to_block_boundary(pos)` 적용.
- 단, snap 결과가 `cursor_row`보다 작거나 같으면 (즉, cursor_row 자체가 블록 중간) 다음 블록 시작으로 전진하여 cursor_row 자체를 한 블록으로 묶어 처리.

### 4. 인트라-로우 분할과의 상호작용

기존 인트라-로우 분할(`is_row_splittable=true`) 경로는 **블록 내부 행에서는 비활성화**한다. 이유: 블록 중간에서 잘리면 rowspan 셀이 두 페이지 셀 영역을 가로질러 잘려 그려짐.

`split_table_rows::end_row` 결정부 (`engine.rs:1599~1638`):
- `approx_end <= cursor_row` && cursor_row가 블록 중간 → 인트라-로우 분할 시도하지 않고 강제로 새 페이지로 전진.
- 정확한 위치는 단계 2에서 코드 확인 후 확정.

## 단계 분할

### 단계 1: `row_block_start/end` 산출 + 헬퍼 + 단위 테스트

**대상 파일**: `src/renderer/height_measurer.rs`

작업:
1. `MeasuredTable` 구조체에 `row_block_start`, `row_block_end` 필드 추가
2. `MeasuredTable::build` (또는 측정 코드 끝부분)에서 두 필드 채우기
3. `row_block_for(row)`, `snap_to_block_boundary(end_row)` 메서드 추가
4. 기존 모든 `MeasuredTable { ... }` 생성자(테스트 포함)에 새 필드 기본값 추가
5. 단위 테스트 추가 (`height_measurer.rs` `#[cfg(test)]`):
   - 단순 표 (rowspan 모두 1): 각 행이 자기 자신만 포함하는 블록
   - rs=2 셀이 행 0+1을 묶음: `row_block_for(0) == (0, 2, h0+h1)`
   - 겹치는 rowspan: 셀 A가 행 0~2, 셀 B가 행 1~3 → 블록 0~3
   - 비인접 rowspan은 별개 블록 유지

빌드/테스트:
- `cargo build --lib`
- `cargo test --lib renderer::height_measurer`

### 단계 2: 분할 호출부 적용 + `find_break_row` 스냅

**대상 파일**:
- `src/renderer/pagination/engine.rs::split_table_rows` (1445~)
- `src/renderer/height_measurer.rs::find_break_row` (1179)

작업:
1. `split_table_rows` 첫 행 판정에 `row_block_for(0)` 적용 (1492~)
2. 루프 내 `cursor_row` 진행 시 블록 중간 진입 방지 (cursor_row가 블록 시작이 아니면 블록 시작으로 보정 또는 advance)
3. `find_break_row` 결과에 `snap_to_block_boundary` 적용
4. 인트라-로우 분할 경로에 "블록 중간 시 비활성" 가드 추가
5. 빌드 + 단위 테스트
   - `cargo build`
   - `cargo test --lib renderer::pagination`

### 단계 3: 회귀 검증 (테스트 + 수동 비교)

**대상**: 전체 테스트 + 본 샘플

작업:
1. `cargo test` 전체 실행
   - 골든 샘플 회귀 테스트(`re_sample_gen`) 통과 여부 확인
   - 페이지네이션 테스트(`pagination/tests.rs::test_typeset_page_break` 등) 통과
2. 본 샘플 SVG 재생성:
   ```
   ./target/debug/rhwp export-svg "samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx"
   ```
3. 1쪽 SVG에서 "분석보고서 요약" 텍스트 부재 확인:
   ```
   grep -c "기부\|답례\|분석보고서\|요약" output/2025*양식_001.svg  # → 0 기대
   ```
4. 2쪽 SVG에 표 전체가 시작되었는지 확인 (dump-pages -p 1)
5. 회귀가 발견된 다른 샘플이 있으면 단계 2로 회귀 검토

## 회귀 시 대응

골든 샘플 출력이 변경된 경우:
1. 변경 페이지 수동 검토 (PDF 비교 가능하면 비교).
2. **개선**(rowspan 셀이 잘리지 않게 됨)이면 골든 갱신.
3. **악화**(불필요하게 표가 다음 페이지로 밀림)이면 알고리즘 재검토.

## 단계 외 작업

- **샘플 파일 커밋**: `samples/2025년 기부·답례품 실적 지자체 보고서_양식.{hwpx,pdf}` 단계 1 시작 시 함께 커밋 (회귀 자료).
- **package-lock.json**: 본 타스크에서 무시.

## 예상 변경 파일

| 파일 | 변경 내용 |
|------|-----------|
| `src/renderer/height_measurer.rs` | `row_block_start/end` 필드, `row_block_for`/`snap_to_block_boundary` 메서드, `find_break_row` 스냅, 단위 테스트 |
| `src/renderer/pagination/engine.rs` | `split_table_rows`에서 블록 기반 분할 판정 |
| `src/renderer/pagination/tests.rs` | (필요 시) 신규 회귀 테스트 케이스 |
| `samples/2025년 기부·답례품 실적 지자체 보고서_양식.{hwpx,pdf}` | 신규 추가 (회귀 자료) |
| `mydocs/working/task_m100_398_stage{1,2,3}.md` | 단계별 보고서 |
| `mydocs/report/task_m100_398_report.md` | 최종 보고서 |
| `mydocs/orders/{오늘날짜}.md` | 오늘 할일 갱신 |
