# Stage 2 보고 — Task #849 (M100) — 구현: `start_new_column_band` 를 배분 단으로 한정

상태: **구현 완료**.

## 수정 (`src/renderer/typeset.rs`)

`paginate` 의 명시적 `Column` break 경로(`para.column_type == ColumnBreakType::Column` → `!has_diff_col_def` → `!current_items.is_empty()`):

```rust
let is_last_column = st.current_column + 1 >= st.col_count;
if is_last_column
    && st.col_count > 1
    && st.current_zone_column_type == ColumnType::Distribute   // [Task #849] 추가
{
    self.start_new_column_band(&mut st, para_idx, paragraphs);
} else {
    st.advance_column_or_new_page();
}
```

- `Distribute`(배분) zone 에서만 마지막 단의 단나누기 → 같은 페이지 새 밴드.
- `Normal`(일반/신문형) zone 은 기존 `advance_column_or_new_page`(→ 마지막 단이면 `push_new_page`) 유지.
- `Parallel`(평행) zone 도 현 동작 유지 (별도 의미 — 범위 밖).
- #846 의 `start_new_column_band` 본체 / `upcoming_band_has_floating_object` / 밴드 높이 산출 / `process_multicolumn_break` 는 변경 없음.

## 검증

| 샘플 | 한컴 PDF | baseline | #846 단독 | #849 적용 |
|------|----------|----------|-----------|-----------|
| `basic/shortcut.hwp` | 7 (2022) | 8 | 7 | **7** ✅ (pi=94/95 `<편집 화면 분할에서>` \| `화면 이동 ⟶ Ctrl+W,N` 페이지 3) |
| `exam_math.hwp` | 20 | 18 | 11 | **20** ✅ (PDF 정합, 문제 11·12 페이지 4) |
| `21_언어_기출_편집가능본.hwp` | 15(2020)/16(2010) | 15 | 15(콘텐츠 시프트) | **15** ✅ (시프트 해소) |
| `cargo test` 전건 | — | 1232 pass | 1229/3 fail | **1232 pass / 0 fail** ✅ |

`cargo test` 의 `test_exam_math_page_count`(≥18), `test_539_partial_paragraph_after_overlay_shape`, `test_548_cell_inline_shape_first_line_indent_p8` 모두 통과.

다음: Stage 3 광역 회귀 검증 (다단 샘플 전수 SVG diff).
