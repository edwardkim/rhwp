# Task #901 Stage 1 보고서 — paragraph 0 layout 정밀 진단 + 부분 Fix

**Stage**: 1 / 5
**상태**: 부분 정합 — paragraph 0 fix 성공, paragraph 1+ 미정합

## 1. 진단 결과

### 1.1 SVG 좌표 추출 (rhwp vs 한컴 PDF)

**rhwp (fix 전)** paragraph 0 "우/리/나/라":
- 모두 x=113.39 (col_area.x, 좌측 margin)
- 한컴 PDF: x=651 부근 (우측 세로)

### 1.2 ROOT CAUSE 식별

`src/renderer/layout/paragraph_layout.rs:938~948` 의 cs/sw 분기:

```rust
let line_avail_hu = comp_line.segment_width.saturating_add(comp_line.column_start);
let (effective_col_x, effective_col_w) = if (has_picture_shape_square_wrap || line_has_inline_tac_table)
    && comp_line.segment_width > 0
    && line_avail_hu < col_area_w_hu - 200
{
    (col_area.x + cs_px, sw_px)  // effective_col_x = wrap zone 좁은 영역
} else {
    (col_area.x, col_area.width)  // col_area.x = 좌측 margin
};
```

paragraph 0 의 line_segs 별 분기 결과:

| line | ts | cs | sw | avail | col_w | 분기 | effective_col_x |
|------|----|----|----|-------|-------|------|------------------|
| 0 (공백) | 0 | 24470 | 2570 | 27040 | 42520 | ✅ | 439.65 |
| **1 ("우")** | 33 | 39123 | 3397 | **42520** | 42520 | ❌ avail == col_w | **113.39 (좌측)** |
| 2 | 34 | 24470 | 2570 | 27040 | 42520 | ✅ | 439.65 |
| **3 ("리")** | 34 | 39123 | 3397 | **42520** | 42520 | ❌ | **113.39** |
| ... | | | | | | | |

paragraph 0 의 한글 char ("우/리/나/라") 가 ls[1], ls[3], ls[5], ls[7] 에 emit. 모두 avail=col_w 라 기존 `avail < col_w - 200` 조건 실패 → 좌측 margin 으로 그려짐.

### 1.3 Fix

조건 확장 — `cs > 0 + sw > 0 + sw < col_w` (cs_significant) 추가:

```rust
let cs_significant = comp_line.column_start > 0
    && comp_line.segment_width > 0
    && comp_line.segment_width < col_area_w_hu;
let (effective_col_x, effective_col_w) = if (has_picture_shape_square_wrap || line_has_inline_tac_table)
    && comp_line.segment_width > 0
    && (line_avail_hu < col_area_w_hu - 200 || cs_significant)
{
    ...
}
```

### 1.4 Fix 결과

paragraph 0 "우/리/나/라" SVG x:
- 이전: 113.39 (좌측)
- **이후: 635.03 (우측, 한컴 정합)** ✅

cargo test --release --all-targets: **1402 passed**, 0 failed.

## 2. 부분 정합 — 잔존 차이

### 2.1 paragraph 1 "대한민국" 미정합

- text " 대한민국" (5 chars), controls=0
- ls[0] cs=24470 sw=18050 (= wrap zone 가운데 영역)
- avail = 42520 = col_w
- **paragraph 자신의 controls 없음 → `has_picture_shape_square_wrap = false` → 분기 미발동**

SVG 결과 paragraph 1 "대한민국" 가 좌측 (x=132) 에 그려짐. 한컴 PDF 는 우측 중앙.

### 2.2 본문 paragraph 의 wrap zone

paragraph 7+ 본문 + 무용수 그림 (paragraph 7 의 그림) wrap layout 차이.

### 2.3 ROOT CAUSE — has_picture_shape_square_wrap 한계

`has_picture_shape_square_wrap` 는 paragraph 자신의 controls 만 검사:
```rust
let has_picture_shape_square_wrap = para
    .map(|p| p.controls.iter().any(|c| { ... }))
```

paragraph 0 의 그림 wrap 이 paragraph 1+ 의 layout 도 영향 미치는데, paragraph 1+ 는 자신의 controls 가 없어 분기 미발동.

## 3. 추가 Fix 방향 후보

| 후보 | 처리 | 위험 |
|------|------|------|
| α | `cs > 0 + sw < col_w` 만으로 분기 발동 (paragraph 자신의 wrap 검사 무시) | 매우 높 — hanging indent 등 정상 case 영향 |
| β | wrap_anchor (wrap zone 의 anchor paragraph) 검사 확장 | 중 — typeset 의 wrap_anchor 추적 필요 |
| γ | 선행 paragraph 의 그림 wrap 영향을 후속 paragraph 까지 전파 | 매우 높 — wrap zone 알고리즘 큰 변경 |

## 4. 다음 단계 결정 요청

paragraph 1+ 추가 fix 시도 또는 본 Stage 결과 (paragraph 0 부분 정합) 로 진행.
