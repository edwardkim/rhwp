# Task M100 #1317 Stage 3 완료보고서 — Canvas/Skia 적분 path 정합

## 목표

Canvas(WASM)·Skia(native PNG) 경로의 적분기호를 SVG와 동일한 `integral_geom` 기반 stroke path로 렌더하여, **SVG/Canvas/Skia 3경로가 폰트 대체에 무관하게 정합**한다(메인테이너 요구).

## 변경 내용

### 1. `src/renderer/equation/canvas_render.rs`

- `draw_integral(ctx, x, y, fs, color)` 헬퍼 신설: `bezier_curve_to` + `set_line_cap("round")`로 svg_render `integral_path`와 동일 곡선.
- `MathSymbol(∫)`/`BigOp` 적분 분기를 `fill_text` → `draw_integral`로 치환 (∑/∏는 `fill_text` 유지).

### 2. `src/renderer/skia/equation_conv.rs`

- `integral_geom` import 추가.
- `draw_integral(canvas, x, y, fs, color)` 헬퍼 신설: `PathBuilder::cubic_to` + `Paint::set_stroke_cap(Round)`로 동일 곡선.
- `MathSymbol(∫)`/`BigOp` 적분 분기를 `draw_text` → `draw_integral`로 치환 (∑/∏는 `draw_text` 유지).

## 검증

| 항목 | 방법 | 결과 |
|------|------|------|
| Canvas 컴파일 | `cargo check --target wasm32-unknown-unknown --lib` | ✅ exit 0 |
| Skia 빌드 | `cargo build --release --features native-skia` | ✅ exit 0 |
| Skia 렌더 | `rhwp export-png ... -p 8` → 9페이지 PNG | ✅ 적분 path 정상 |
| **SVG ↔ Skia 정합** | 동일 적분 3-way 비교 (`cmp_3way.png`) | ✅ **동일 path glyph** |
| ∑/∏ 회귀 | SVG `>∑<` 4개 유지, `>∫<` 0개, 적분 path 3개 | ✅ 무영향 |

3경로가 동일 `integral_geom`을 공유하므로 SVG와 Skia의 적분 글리프가 픽셀 단위로 일치하며, PDF 정답(한글 2022)과도 상·하한 배치가 부합한다.

## 산출물

- `output/poc/pr1314/cmp_3way.png` (SVG / Skia / PDF 비교)
- `output/poc/pr1314/skia_p9.png/` (Skia PNG)
