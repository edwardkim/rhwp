# Task #321 v6 구현 계획서 — 시각 병합 + border_spacing inset

## 사전 조사

`src/renderer/layout.rs::build_single_column` 의 문단 테두리 병합 로직 (line ~1512):

```rust
let mut groups: Vec<(u16, f64, f64, f64, f64)> = Vec::new();
for &(bf_id, x, y_start, w, y_end) in ranges.iter() {
    if let Some(last) = groups.last_mut() {
        if last.0 == bf_id && (y_start - last.4) < 30.0 {
            last.4 = y_end; continue;
        }
    }
    groups.push((bf_id, x, y_start, w, y_end));
}
```

문제:
- `last.0 == bf_id` 만 검사 → bf_id 가 다르면 시각적 stroke 동일이라도 분리
- `ResolvedBorderStyle::borders` 의 stroke (line_type/width/color) 시각 비교 없음
- `ParaShape::border_spacing` 가 push tuple 에 포함 안 됨 → 렌더 시 inset 적용 불가

## Stage 7a — 시각 병합 + visibility 규칙

**수정 위치**: `src/renderer/layout.rs::build_single_column` 의 merge 로직.

**수정 시안**:

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
// 병합 조건: bf_id 동일 OR (None 아닌 동일 visible stroke)
let same_visual = if last.0 == bf_id { true }
    else { let l = stroke_sig(last.0); l.is_some() && l == stroke_sig(bf_id) };
if same_visual && (y_start - last.4) < 30.0 { ... }
```

**검증** (Stage 7a):
1. 21_언어 p1 SVG 의 stroke rect 개수 확인: 4 → 4 유지 (form 표 2 + col 0 main 1 + col 1 main 1)
2. pi=6 의 작은 rect (y=558.32 h=13.92) 가 pi=7 main rect 에 흡수되어 사라지는지 확인

## Stage 7b — border_spacing inset

**수정 위치**:
1. `src/renderer/layout.rs::para_border_ranges` 튜플 7-tuple 로 확장 (top_inset, bottom_inset 추가)
2. `src/renderer/layout/paragraph_layout.rs:2369` push 에 `para_style.border_spacing[2]/[3]` 전달
3. `src/renderer/layout.rs` 의 group 렌더 루프: 인접 touches 검사 + default 2px 적용

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

**검증** (Stage 7b):
1. pi=7 border top 과 "비즈니스" 첫 글자 사이 시각 간격 확인 (zoom)
2. 인접 다른 border group 과 겹침 없는지 확인

## Stage 7c — 회귀 + 보고

**확인 항목**:
1. `cargo test --lib`: 992 passed
2. `cargo test --test svg_snapshot`: 6 passed (form-002 포함)
3. `cargo clippy --release`: clean
4. 페이지 수 (7 sample): 21_언어 16, exam_math 20, exam_kor 24, exam_eng 10,
   exam_math_8 1, exam_science 5, exam_social 5
5. PDF 와 시각 비교 (pdftoppm + side-by-side crop)

**산출**:
- `mydocs/working/task_m100_321_stage7.md`: 수정 전후 stroke rect / 시각 비교
- `mydocs/report/task_m100_321_v6_report.md`: 최종 결과

## 회피 사항

- ✋ bf_id indexing (`bf_id - 1`) 은 다른 코드와 일관 유지 — 변경 금지
- ✋ invisible stroke (None signature) 끼리 병합 금지 — golden form-002 회귀 우려
- ✋ inset 이 인접 그룹과 겹치면 시각 회귀 → touches 검사 필수
- ✋ table/cell 의 border_fill_id 는 본 수정과 무관 (별도 코드 path)

## 승인 요청

본 구현 계획서 승인 후 Stage 7a 부터 순차 진행.
