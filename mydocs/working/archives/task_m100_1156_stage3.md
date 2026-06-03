# Task M100-1156 — Stage 3 정정 보고서 (차트 겹침 + 워터마크 회귀)

- 이슈: [#1156](https://github.com/edwardkim/rhwp/issues/1156)
- 일시: 2026-05-29
- 단계: Stage 3 (자리차지 텍스트 겹침 + 워터마크 효과 회귀) — 시각 통과

## 1. 작업지시자 시각 판정 (Stage 2 후) 잔여 결함

1. 차트 단1 이동은 확인. 그러나:
2. **차트 y 위치** — 단 시작 기준이어야 (placeholder는 단1 상단 113px 정상이나 텍스트 겹침으로 잘못 보임)
3. **자리차지(TopAndBottom) 겹침** — 차트 영역에 텍스트가 겹침

추가 발견 (작업지시자 제보):
4. **워터마크 효과 회귀** — SVG/PNG 배경 워터마크 반투명(opacity) 누락 (CanvasKit는 정상)

## 2. 차트 자리차지 겹침 (layout.rs)

`layout_shape_item` 의 `if common.treat_as_char` 블록 밖이라 비-TAC 차트는 `result_y`(후속 콘텐츠 시작 y)를 진행 안 시킴 → 텍스트가 차트 영역(y 113~415) 겹침 (텍스트 첫 줄 y=154).

정정: 비-TAC TopAndBottom vert=Para Shape 에 else 분기 추가 — `result_y = y_offset + 차트높이 + margin_bottom`.

검증: 단1 텍스트 첫 줄 y 154 → **486** (차트 bottom 415 아래). 겹침 해소.

## 3. 워터마크 효과 회귀 (svg/web_canvas/skia 3경로)

### 원인

PR #1019(`e26f159b`) + #975 RealPic 톤 commit 들이 RealPic 경로 추가하며, 배경 워터마크(effect=RealPic, brightness=contrast=0, watermark=none)가:
- `is_real_picture_watermark_tone_preset` = RealPic && b==-50 && c==70 → false
- `is_watermark_image` = !RealPic && (b≠0 ‖ c≠0) → false
- → opacity 누락 (배경 워터마크 불투명)

정답지 PDF: 배경 "인천대학교" 워터마크 반투명 → opacity 적용이 정답.

### 정정 (3 렌더러 경로 정합)

페이지 배경 이미지는 본질적으로 워터마크 → RealPic 톤 프리셋 아닌 PageBackground 도 워터마크 opacity:
- `svg.rs` `render_page_background_image`: `is_watermark_image` 에 `(!preserve_color_watermark && RealPic)` 추가
- `web_canvas.rs` PageBackground: 동일
- `skia/renderer.rs` `PaintOp::PageBackground`: `save_layer_alpha(opacity)` 로 이미지 합성 (RealPic 톤 프리셋 0.26 / 그 외 0.17)

## 4. 시각 검증 (작업지시자 통과)

- 차트 단1 상단 + 자리차지 텍스트 차트 아래 (겹침 없음)
- SVG/PNG 배경 워터마크 반투명 효과 복원 ("SVG, PNG 에는 적용된 것을 확인")

## 5. 회귀

- `tests/issue_1156_chart_column_flow.rs` 2 passed
- `svg_snapshot` 8 passed, `cargo test --tests` 전수 통과
- `clippy --lib` 0 warnings, `fmt` 정합

## 6. 다음 단계 (Stage 4)

최종 보고서 + commit + merge + close.
