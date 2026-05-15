# Stage 2 보고서: Task #898 — 수정 구현

## 1. 수정 내용

### `src/renderer/layout/table_layout.rs::compute_table_y_position`

Paper-relative 분기 (depth=0, wrap=TopAndBottom/BehindText/InFrontOfText) 의 `raw_y` 산식에 `outer_margin_top` (Top/Inside 정렬) 및 `outer_margin_bottom` (Bottom/Outside 정렬) 가산.

```rust
// [Task #898] Paper-relative 표는 v_offset 이 외곽 박스 (outer_margin 포함) 기준이므로
// 가시 표 상단 = v_offset + outer_margin_top.
let om_top_px = if matches!(vert_rel_to, VertRelTo::Paper) {
    hwpunit_to_px(table.outer_margin_top as i32, self.dpi)
} else { 0.0 };
let om_bottom_px = if matches!(vert_rel_to, VertRelTo::Paper) {
    hwpunit_to_px(table.outer_margin_bottom as i32, self.dpi)
} else { 0.0 };
let raw_y = match vert_align {
    Top | Inside  => ref_y + v_offset + caption_top_offset + om_top_px,
    Center        => ref_y + (ref_h - table_height) / 2.0 + v_offset + caption_top_offset,
    Bottom|Outside => ref_y + ref_h - table_height - v_offset + caption_top_offset - om_bottom_px,
};
```

영향 범위: `vert_rel_to=Paper` 인 표만. Page/Para 기준 표는 변화 없음.

## 2. 테스트

신규: `tests/issue_898.rs::master_page_table_includes_outer_margin_top`
- exam_math.hwp 페이지 1 SVG 에서 바탕쪽 표 셀 y=1378.28 px 확인
- 회귀 가드: y=1359.x (outer_margin_top 미적용) 위치 검출 시 실패

결과: **pass**

## 3. 측정 검증

`output/svg/exam_math/exam_math_001.svg`:
- 직선 끝 y = 1358.41 px (변화 없음)
- 바탕쪽 표 셀 y = **1378.28 px** (수정 전 1359.39 px → +18.89 px = outer_margin_top)
- 두 객체 간격 = **19.87 px ≈ 5.26 mm**
- PDF 측정 간격 = 5.3 mm (1378.1 - 1358.4 환산) → **일치**

시각 확인: `/tmp/em_fixed_p1.png` 페이지 1 — 가운데 세로선과 1/20 박스 사이 명확한 여백 확보.

## 4. 회귀

`cargo test --release --lib`: **1257 passed, 0 failed**

## 5. 다음 단계

Stage 3 — 시각 회귀 검증:
- exam_math.hwp 전 20쪽 PDF 대비 측정
- 바탕쪽 사용 다른 샘플 (있다면) 회귀
- 골든 SVG 영향 확인
