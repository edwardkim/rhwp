# Stage 1 보고서: Task #898 — 원인 정밀 규명

## 1. 좌표 산출식 추적

`src/renderer/layout/table_layout.rs::compute_table_y_position` (1152~1241행) 분석 결과:

```
[Paper-relative, depth=0, wrap=TopAndBottom 분기 (1180~1188행)]
  ref_y = 0.0
  ref_h = page_h_approx
  vert_align = Top → raw_y = ref_y + v_offset + caption_top_offset
  ⇒ raw_y = v_offset
```

**`outer_margin_top` 가 산식에 포함되지 않음.** (depth>0 중첩 표 분기에서만 적용)

## 2. 측정값 검증

`samples/exam_math.hwp` 페이지 1 바탕쪽 표 (1×3 쪽번호 박스):

| 값 | HU | px(96dpi) |
|----|-----|-----------|
| `vert_rel_to` | Paper | — |
| `vertical_offset` | 101954 | 1359.39 |
| `outer_margin_top` | **1417** | **18.89** |
| `outer_margin_bottom` | 283 | 3.77 |
| `outer_margin_left/right` | 283 | 3.77 |

**기대 표 상단 y** = `v_offset + outer_margin_top` = 101954 + 1417 = 103371 HU = **1378.28 px**

## 3. PDF 측정 비교

`pdf/exam_math-2022.pdf` 페이지 1 (1169×1653 px, A3 100dpi 렌더):
- 페이지번호 박스 상단 y = 1530 → 페이지 비율 92.6%
- SVG 1489 px 환산: 1489 × 0.926 = **1378.1 px**

→ **기대 1378.28 px 와 PDF 측정 1378.1 px 일치** (0.18px 오차)

## 4. 현재 rhwp 출력

`output/svg/exam_math/exam_math_001.svg`:
- 바탕쪽 표 셀 클립 y = **1359.39 px** (= v_offset 만 적용)
- PDF 대비 **18.9 px (=outer_margin_top) 위쪽으로 잘못 배치**

## 5. 결론

가설 (a) **`outer_margin_top` 미적용** 이 정답. Paper-relative + depth=0 + TopAndBottom 분기에 `outer_margin_top` 추가 필요.

## 6. 수정 후보 위치

`src/renderer/layout/table_layout.rs:1180~1205`:

```rust
let (ref_y, ref_h) = match vert_rel_to {
    ...
    crate::model::shape::VertRelTo::Paper => (0.0, page_h_approx),
};
// 현재:
let raw_y = match vert_align {
    VertAlign::Top => ref_y + v_offset + caption_top_offset,
    VertAlign::Center => ref_y + (ref_h - table_height) / 2.0 + v_offset + caption_top_offset,
    VertAlign::Bottom => ref_y + ref_h - table_height - v_offset + caption_top_offset,
};
```

수정 방향:
- `VertAlign::Top` 분기에 `+ om_top` 추가
- `VertAlign::Bottom` 분기에서 `- om_bottom` 추가 (대칭)
- `VertAlign::Center` 는 외곽 박스 중앙 정렬이므로 표 본체 위치 = `ref_y + (ref_h - (table_h + om_top + om_bottom)) / 2 + om_top` ≈ 영향 작음

영향 범위:
- **Paper-positioned table** (주로 바탕쪽 + 일부 머리말/꼬리말 fixed 표)
- 본문 표 (Para-positioned) 는 영향 없음
- HWPX와 HWP5 동일 적용

## 7. 다음 단계

Stage 2 — 수정 구현 + 단위 테스트 신설
- `compute_table_y_position` 수정
- `tests/master_page_table_outer_margin.rs` 등 단위 테스트 추가
- 회귀 확인용 데이터: `exam_math.hwp` 외 바탕쪽 사용 문서 목록 작성
