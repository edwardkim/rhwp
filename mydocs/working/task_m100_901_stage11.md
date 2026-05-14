# Task #901 Stage 11 보고서 — flow-around base 정밀 계산

**Stage**: 11
**상태**: page 2 paragraph 22 한컴 정밀 정합 ✅ (1402 test 회귀 없음)

## 1. 진단

Stage 10 (anchor first_vpos 를 base 로 사용) 으로 paragraph 22 y=678 (이전 959 대비 -281 px 개선). 하지만 한컴 (paragraph 22 iris 직하 ~y=520-540) 대비 +130 px 추가 차이.

원인: anchor_first_vpos 를 base 로 쓰면 file vpos 누적이 visual y 와 정확히 매핑되지만, anchor paragraph 의 text 가 iris 위에 flow-around 된 경우 base 가 다른 값이어야 함.

정확한 base = next_para_vpos - (bottom_y - col_area.y) × 7200/dpi

## 2. Fix

post_jump 시점에 후속 paragraph item 의 first vpos 를 peek 하여 base 직접 계산:

```rust
let next_para_vpos: Option<i32> = col_content.items.iter()
    .skip_while(|it| match it {
        PageItem::FullParagraph { para_index } => *para_index != anchor_pi,
        PageItem::PartialParagraph { para_index, .. } => *para_index != anchor_pi,
        _ => true,
    })
    .skip(1)
    .find_map(|it| match it {
        PageItem::FullParagraph { para_index }
        | PageItem::PartialParagraph { para_index, .. } => {
            paragraphs.get(*para_index)
                .and_then(|p| p.line_segs.first())
                .map(|s| s.vertical_pos)
        }
        _ => None,
    });
let base_for_post = if let Some(npv) = next_para_vpos {
    let visual_diff_hu = ((bottom_y - col_area.y) / self.dpi * 7200.0).round() as i32;
    npv - visual_diff_hu
} else {
    anchor_para.line_segs.first().map(|s| s.vertical_pos).unwrap_or(0)
};
```

## 3. 결과

VPOS_CORR trace:
```
pi=20: base=93440 vpos_end=120391 end_y=491.61 ✓ (iris bottom)
pi=21: base=93440 vpos_end=121991 end_y=512.95 ✓
pi=22: base=93440 vpos_end=123591 end_y=534.28 ✓
```

paragraph 22 "집" y 비교:

| Stage | y |
|-------|---|
| Baseline (pre-#901) | 1045 (page 2 overflow) |
| Stage 7 | 1045 |
| Stage 8 (flow-around) | 959 |
| Stage 10 (vpos_lazy_base set) | 678 |
| **Stage 11 (base 정밀 계산)** | **545** ✓ |

한컴 PDF paragraph 22 ~y=520-540 — Stage 11 한컴 거의 완전 정합.

## 4. 회귀 검증

- ✅ `cargo test --release --all-targets`: **1402 passed, 0 failed**
- ✅ page 2 paragraph 19 top (143) + iris (237-491) + paragraph 22 (545) 한컴 정합
- ✅ pic2.hwp page 1 layout 유지

## 5. Task #901 최종 종합

pic2.hwp 정합 progress (Stage 1+2+3+5+6+7+8+10+11):

| 항목 | Baseline | 최종 |
|------|----------|------|
| paragraph 0 "우" | 좌측 (잘못) | 우측 세로 ✅ |
| paragraph 0 line gap | 119 px | 60 px (한컴) ✅ |
| paragraph 7 SK하이닉스 y | 788 | 571 |
| paragraph 11 올해 확정 | page 2 | page 1 ✅ |
| paragraph 19 page 2 | iris 아래 | iris 위 ✅ |
| paragraph 22 page 2 y | 1045 (overflow) | 545 (한컴 정합) ✅ |
| 1402 test | passed | passed ✅ |
