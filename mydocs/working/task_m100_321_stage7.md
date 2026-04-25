# Task #321 v6 Stage 7 — 시각 병합 + border_spacing inset 적용

## Stage 7a — visual stroke signature 병합

**수정 위치**: `src/renderer/layout.rs::build_single_column` 의 문단 테두리 병합.

bf_id 동일 외에, **visible stroke signature (line_type/width/color) 가 동일** 한
인접 range 도 병합하도록 확장. invisible (`any_w=false`) 끼리는 병합하지 않음.

```rust
use crate::model::style::BorderLineType;
type StrokeSig = Option<(BorderLineType, u8, u32)>;
let stroke_sig = |bf_id: u16| -> StrokeSig {
    let idx = (bf_id as usize).saturating_sub(1);
    let bs = styles.border_styles.get(idx)?;
    let top = &bs.borders[2];
    let any_w = bs.borders.iter().any(|b|
        !matches!(b.line_type, BorderLineType::None) && b.width > 0);
    if any_w { Some((top.line_type, top.width, top.color)) } else { None }
};
let same_visual = if last.0 == bf_id { true }
    else { let l = stroke_sig(last.0); l.is_some() && l == stroke_sig(bf_id) };
```

## Stage 7b — border_spacing inset 정식 반영

**수정 1** (`paragraph_layout.rs:2369`): push tuple 에 top/bottom inset 전달

```rust
let top_inset = para_style.map(|s| s.border_spacing[2]).unwrap_or(0.0);
let bottom_inset = para_style.map(|s| s.border_spacing[3]).unwrap_or(0.0);
self.para_border_ranges.borrow_mut().push(
    (para_border_fill_id, ..., y, top_inset, bottom_inset)
);
```

**수정 2** (`layout.rs`): 그룹 렌더 시 inset 적용 + 인접 touches 검사

```rust
const DEFAULT_MIN_INSET: f64 = 2.0;
let prev_touches = gi > 0 && (y_start - groups[gi-1].4) < 4.0;
let next_touches = gi+1 < groups_len && (groups[gi+1].2 - y_end) < 4.0;
let top_pad = if stroke_width > 0.0 && !prev_touches {
    top_inset.max(DEFAULT_MIN_INSET) } else { top_inset };
let bot_pad = if stroke_width > 0.0 && !next_touches {
    bottom_inset.max(DEFAULT_MIN_INSET) } else { bottom_inset };
let rect_y = y_start - top_pad;
let rect_h = height + top_pad + bot_pad;
```

`para_border_ranges` 튜플은 7-tuple 로 확장 (`(bf_id, x, y_start, w, y_end, top_inset, bottom_inset)`).

## 21_언어 page 1 측정 (전후)

| 항목 | 수정 전 (v5) | 수정 후 (v6) | 비고 |
|------|--------------|--------------|------|
| col 0 stroke rect 수 | 2 (pi=6 별도 + pi=7-9 main) | **1 (병합)** | ✓ PDF 일치 |
| col 0 main rect | y=572.24 h=871.68 | y=558.32 h=887.60 | pi=6 흡수 + 2px inset |
| pi=7 border-text 간격 | 0 px (touch) | **~2 px** | ✓ inset 효과 |
| col 1 stroke rect | y=329.94 h=460.05 | y=327.94 h=462.05 | inset 2px 적용 |
| 페이지 수 21_언어 | 16 | 16 | 유지 |

## Stage 7c — 회귀 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | **992 passed**, 0 failed, 1 ignored |
| `cargo test --test svg_snapshot` (golden 6) | **6 passed**, 0 failed |
| `cargo clippy --release` | **clean** |
| 21_언어 page count | 16 (유지) |
| exam_math / exam_kor / exam_eng | 20 / 24 / 10 (유지) |
| exam_math_8 / exam_science / exam_social | 1 / 5 / 5 (유지) |

## 시각 검증

- 21_언어 p1 col 0: 단일 bordered rect 로 "비즈니스 프로세스..." 부터 "...줄어든다." 까지 둘러싸고
  border 안쪽 top/bottom 에 ~2 px inset 확보 → PDF 와 일치
- "[1~3]-다음 글을 읽고..." 는 border 외부 평문으로 유지 (이전과 동일)
- 다른 sample: form-002 / table-text / issue-147 / issue-267 골든 SVG 무변경

## 영향 범위

- 시각 stroke 가 다른 인접 그룹은 영향 없음 (None 그룹 끼리 병합 안 함)
- `ResolvedBorderStyle` 구조체 변경 없음 (필드 유지)
- 다른 코드 path (table/cell border) 무영향
- v5 의 pi=0 block-table drift 보정과 충돌 없음
