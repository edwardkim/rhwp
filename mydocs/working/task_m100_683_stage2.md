# Stage 2 — 산식 구현 (Task #683)

**브랜치**: `local/task683`
**관련**: `mydocs/plans/task_m100_683.md`, `mydocs/plans/task_m100_683_impl.md`, `mydocs/working/task_m100_683_stage1.md`

## 요약

Stage 1 진단 결과를 기반으로 두 곳을 함께 수정:

1. **`src/renderer/layout.rs::layout_shape_item` Picture 분기** — 빈 paragraph + Para-relative TopAndBottom 그림(caption 없음) 의 `result_y` 에 `line_height + line_spacing` 추가.
2. **`src/renderer/layout/paragraph_layout.rs` trailing-ls 제외 분기** — 직전 paragraph 가 빈 image-paragraph 였으면 trailing line_spacing 보존 (Task #479 의 trailing 제외와 충돌 회피).

## 변경 내역

### `src/renderer/layout.rs`

1. `Layout` 구조체에 `prev_para_was_empty_topandbottom_pic: Cell<bool>` 필드 추가 + `new()` 초기화
2. 신규 헬퍼 `paragraph_is_empty_topandbottom_pic(pi, paragraphs) -> bool` — 텍스트 0 + Para-relative TopAndBottom Picture (caption 없음) 판정
3. `layout_column_item::FullParagraph` 분기에서 caller 가 플래그 set/reset:
   ```rust
   self.prev_para_was_empty_topandbottom_pic.set(
       *para_index > 0
           && self.paragraph_is_empty_topandbottom_pic(*para_index - 1, paragraphs));
   y_offset = self.layout_paragraph(...);
   self.prev_para_was_empty_topandbottom_pic.set(false);
   ```
4. `layout_shape_item` Picture 비-TAC 분기 (Para-relative path):
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

### `src/renderer/layout/paragraph_layout.rs`

trailing-ls 제외 분기 (`is_full_paragraph_end && !next_starts_border && !next_continues_border`) 에 신규 가드 `&& !prev_was_empty_pic` 추가. true 면 trailing line_spacing 보존.

### `src/renderer/layout/integration_tests.rs`

신규 테스트 `test_task683_pr149_image_cluster_spacing` — pr-149.hwp 의 SVG 출력에서 그림 cluster 간 거리가 PDF 정합 18864 HU (= 251.52 px @ 96dpi, ±3px) 인지 검증.

## 검증 결과 (pr-149.hwp)

### 측정 (150 dpi)

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

## 회귀 검증 (Stage 3 에서 광범위 검증 예정)

`cargo test --release` 전체 통과:
- 18개 테스트 스위트 모두 0 failures
- 1125 + 1 (신규) + 다수 통과
- 기존 회귀 테스트 모두 통과 (visible_border, table layout, footnote 등 영향 없음)

## 가드 조건 (재현 위험 최소화)

수정이 적용되는 조건:
- `pic.common.treat_as_char == false`
- `pic.common.text_wrap == TextWrap::TopAndBottom`
- `pic.common.vert_rel_to == VertRelTo::Para`
- `pic.caption.is_none()`
- 부모 paragraph 의 text 가 visible 글자 0 (단, `\u{001F}` 이하 + `\u{FFFC}` 제외)

위 조건 모두 만족하지 않는 케이스는 기존 동작 유지.

## 영향 범위

| 항목 | 영향 |
|------|------|
| HWP3 / HWPX | 동일 IR 사용 → 자동 적용 (Stage 3 에서 검증) |
| 머리말/꼬리말, 바탕쪽 | 별도 레이아웃 경로 → 영향 없음 |
| 표 셀 내부 그림 | `cell_ctx.is_some()` 분기 → 영향 없음 |
| TAC 그림 (글자처럼) | 가드로 제외 |
| caption 보유 그림 | 가드로 제외 (별도 케이스) |
| Square / BehindText / InFrontOfText wrap | 가드로 제외 |

## 다음 단계

Stage 3 — 시각 검증 및 회귀 테스트 (다른 wrap=TopAndBottom 그림 보유 샘플 시각 확인).

**작업지시자 승인 대기**.
