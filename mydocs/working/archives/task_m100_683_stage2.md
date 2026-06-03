# Stage 2 — 산식 구현 (Task #683)

## 변경 내역

### `src/renderer/layout.rs::layout_shape_item` Picture 비-TAC + Para-relative 분기

`result_y = self.layout_body_picture(...)` 직후 가드 분기 추가:

```rust
if matches!(pic.common.text_wrap, TextWrap::TopAndBottom)
    && matches!(pic.common.vert_rel_to, VertRelTo::Para)
    && pic.caption.is_none()
{
    let has_visible_text = para.text.chars()
        .any(|c| c > '\u{001F}' && c != '\u{FFFC}');
    if !has_visible_text {
        let line_advance = para.line_segs.first()
            .map(|ls| hwpunit_to_px(ls.line_height + ls.line_spacing, self.dpi))
            .unwrap_or(0.0);
        result_y += line_advance;
    }
}
```

### `src/renderer/layout/integration_tests.rs`

신규 테스트 `test_task683_pr149_image_cluster_spacing` — pr-149.hwp SVG 출력의 그림 cluster 거리가 PDF 정합 18864 HU (= 251.52 px @ 96dpi, ±3 px) 인지 검증.

## 검증 결과 (pr-149.hwp)

| 요소 | PDF (한글 2022) | rhwp SVG (Stage 2) | 차이 |
|------|----------------|------------------|------|
| 그림1 | 273..600 | 273..600 | ✓ 0 px |
| 그림2 | 666..993 | 667..994 | ✓ +1 px (sub-pixel) |
| 그림3 | 1059..1387 | 1060..1388 | ✓ +1 px |
| "회색조:" | 634..649 | 634..651 | ✓ 0 px |
| "흑백:" | 1028..1042 | 1027..1044 | ✓ -1 px |
| "입니다." | 1454..1472 | 1454..1473 | ✓ 0 px |

### 그림 cluster 거리

- PDF: 18864 HU (= 393 px @ 150 dpi)
- 수정 전: 17280 HU (= 360 px) — **-1584 HU 부족**
- 수정 후: 18896 HU (= 251.95 px @ 96 dpi → 393.7 px @ 150 dpi) — **+32 HU (sub-pixel rounding)**

**모든 요소 ±1 px 이내 정합.**

## 단위 테스트

```
test renderer::layout::integration_tests::tests::test_task683_pr149_image_cluster_spacing ... ok
```

## 가드 조건

수정이 적용되는 조건 (모두 만족 시):
- `pic.common.treat_as_char == false`
- `pic.common.text_wrap == TextWrap::TopAndBottom`
- `pic.common.vert_rel_to == VertRelTo::Para`
- `pic.caption.is_none()`
- 부모 paragraph 의 visible 글자 0 (단, `\u{001F}` 이하 + `\u{FFFC}` 제외)

위 조건 모두 만족하지 않는 케이스는 기존 동작 유지.

## 영향 범위

| 항목 | 영향 |
|------|------|
| HWP3 / HWPX | 동일 IR 사용 → 자동 적용 |
| 머리말/꼬리말, 바탕쪽 | 별도 layout 경로 → 영향 없음 |
| 표 셀 내부 그림 | `cell_ctx.is_some()` 분기 → 영향 없음 |
| TAC, caption 보유, Square/BehindText/InFrontOfText wrap | 가드로 제외 |
| Skia 네이티브 렌더러 | 페이지네이션/레이아웃 결과 사용 → 자동 적용 |

## 다음 단계

Stage 3 — 시각 검증 및 회귀 테스트.
