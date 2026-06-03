# Task #101: 1단계 완료보고서

> **이슈**: [#101](https://github.com/edwardkim/rhwp/issues/101)
> **브랜치**: `local/task101`
> **작성일**: 2026-04-10

---

## 수행 내용

### 1. `find_break_row` SPLIT_EPSILON 제거 (height_measurer.rs)

v1 구현계획서 기반으로 적용했던 SPLIT_EPSILON=0.5 코드를 제거했다.
(부동소수점 오차 가설 기반이었으나 실제 원인이 아니었음)

**변경 전**:
```rust
const SPLIT_EPSILON: f64 = 0.5;
let target = self.cumulative_heights[cursor_row] + avail + delta + adj_cs - SPLIT_EPSILON;
```

**변경 후**:
```rust
let target = self.cumulative_heights[cursor_row] + avail + delta + adj_cs;
```

### 2. `split_table_rows` spacing_before_px 차감 (engine.rs)

`paginate_table_control`에서 `spacing_before_px`를 계산하여 `split_table_rows`에 전달.
`split_table_rows` 내 첫 분할 조건에서 `avail_for_rows`에서 차감.

**변경 내용**:

1. `host_spacing` 계산 블록에서 `spacing_before_px` 추출 추가:
   ```rust
   let spacing_before_px = before - outer_top;
   (before + sa + outer_bottom + host_line_spacing, host_line_spacing, spacing_before_px)
   ```

2. `split_table_rows` 호출부에 `spacing_before_px` 파라미터 추가

3. `split_table_rows` 함수 시그니처에 `spacing_before_px: f64` 추가

4. `avail_for_rows` 계산 수정:
   ```rust
   let sb_extra = if !is_continuation && cursor_row == 0 && content_offset == 0.0 {
       spacing_before_px
   } else {
       0.0
   };
   let avail_for_rows = (page_avail - header_overhead - sb_extra).max(0.0);
   ```

---

## 검증 결과

### dump-pages -p 18 (페이지 19)

**수정 전**:
```
PartialTable   pi=78 ci=0  rows=0..20  cont=false  26x3
```

**수정 후**:
```
PartialTable   pi=78 ci=0  rows=0..19  cont=false  26x3
```

rows=0..20 → rows=0..19 로 1행 감소. 분할 기준이 정상 조정됨.

### export-svg -p 18

LAYOUT_OVERFLOW 출력 없음. 페이지 초과 해소 확인.

### 페이지 20 연속 확인

```
PartialTable   pi=78 ci=0  rows=19..26  cont=true  26x3
```

rows=19..26 cont=true 로 정상 연속.

---

## 다음 단계

2단계: 226개 샘플 전체 회귀 테스트
