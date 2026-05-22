# Stage 3 보고서 — Task #1062: 미주 누적 vpos 정합 구현

- 브랜치: `local/task1062`
- 변경: `src/renderer/typeset.rs` 미주 루프(1436-1453 영역) 한정, +34/−6

## 구현

다단(col_count>1) 미주 누적/판정을 렌더러 vpos 전진과 통일:

```rust
let (en_fit, en_advance) = if st.col_count > 1 {
    let advance = match (en_para.line_segs.first(), en_para.line_segs.last()) {
        (Some(f), Some(l)) => {
            let span_hu = (l.vertical_pos + l.line_height + l.line_spacing) - f.vertical_pos;
            if span_hu > 0 { hwpunit_to_px(span_hu, self.dpi) } else { fmt.total_height }
        }
        _ => fmt.total_height,
    };
    let trailing_ls = en_para.line_segs.last()
        .map(|l| hwpunit_to_px(l.line_spacing.max(0), self.dpi)).unwrap_or(0.0);
    ((advance - trailing_ls).max(0.0), advance)
} else {
    (fmt.height_for_fit, fmt.total_height)   // 단단: 종전 유지
};
// fit 판정 → advance_column_or_new_page, 누적 += en_advance
```

- 누적 = 미주 문단 vpos 전진(렌더러 일치), fit = 전진 − trailing_ls(마지막 항목 의미).
- **단단(col_count==1)은 종전 그대로** → 회귀면 최소.
- line_segs 부재 시 total_height fallback.

## Smoke 결과 (devel baseline 대비)

| 대상 | overflow devel→변경 | 쪽수 우리/PDF |
|------|------|------|
| 3-09 2022 | 155 → **27** | 22 / 23 (1쪽 부족, 잔여 max 277px) |
| 3-09 2023 | 116 → **31** | 20 / 20 ✓ |
| 3-10 2022 | 110 → **22** | 18 / 18 ✓ |
| 3-11 2022 | 94 → **9** | 21 / 21 ✓ |

| 비회귀 | devel→변경 |
|------|------|
| exam_eng | 12 → 11 |
| exam_kor | 19 → 19 |
| k-water-rfp | 3 → 3 |
| 복학원서 | 2 → 2 |
| footnote-01 / endnote-01 | 0 → 0 |

3종은 PDF 쪽수 정확 일치. 3-09 2022는 대폭 개선(155→27)되었으나 1쪽 부족·잔여 277px — 별도 잔류 요인(Stage 4 분석).

## 다음 (Stage 4)
전 251 샘플 회귀 점검 + cargo test + 골든 SVG + 3-09 2022 잔여 분석.
