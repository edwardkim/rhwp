# Task #902 Stage 13 보고서 — raster Player 핵심 records 구현

**Stage**: 13 / 17 (v2 확장)
**상태**: 완료 (drawing records 1차 + object table 구현)

## 1. 변경 영역

`src/wmf/converter/raster/player.rs` — Player trait 의 핵심 drawing/object records 구현.

## 2. 구현 추가

### 2.1 Drawing records (tiny-skia path 기반)

| Record | 처리 |
|--------|------|
| `line_to` | PathBuilder::move_to + line_to, stroke_path |
| `polyline` | 다중 점 line 연결, stroke_path |
| `polygon` | 다중 점 close 후 fill_path (poly_fill_mode 적용) + stroke_path |
| `poly_polygon` | LO mtftools.cxx 의 DrawPolyPolygon 포팅 — 단일 path 의 다중 서브경로, fill-rule (winding/alternate) 적용 |
| `rectangle` | 4 점 rectangle path, fill + stroke |

### 2.2 Object table

| Record | 처리 |
|--------|------|
| `create_brush_indirect` | LogBrush enum 분기 (Solid/Hatched/Null) → BrushInfo |
| `create_pen_indirect` | Pen.style/width/color_ref → PenInfo (PS_NULL 검출) |
| `create_font_indirect` | Font → FontInfo |
| `create_region` | RasterObject::Region stub |
| `delete_object` | object_table 제거 + selected 해제 |
| `select_object` | 객체 종류 별 selected_pen/brush/font 설정 |

### 2.3 build_*_paint 헬퍼

```rust
fn build_stroke_paint(&self) -> Option<(Paint<'static>, Stroke)>
fn build_fill_paint(&self) -> Option<Paint<'static>>
```

- selected_pen/brush 의 NULL 상태 + 색상 + width 를 tiny-skia Paint/Stroke 로 변환
- canvas 스케일 적용

### 2.4 logical_to_pixel 변환

```rust
fn logical_to_pixel(&self, x, y) -> (f32, f32) {
    let (dx, dy) = self.state.logical_to_device(x, y);  // MM_ANISOTROPIC 변환
    let scale_x = canvas_width / extent.0;
    let scale_y = canvas_height / extent.1;
    (dx * scale_x, dy * scale_y)
}
```

- WMF logical 좌표 → MM_ANISOTROPIC device 좌표 → canvas pixel

## 3. 미구현 (Stage 14+)

- text rendering (ext_text_out, text_out) — fontdue + LO DrawText 알고리즘
- bitmap records (bit_blt, dib_stretch_blt 등) — image crate decode + pixmap copy
- arc, chord, ellipse, pie, round_rect — geometry path
- flood fill, region ops — 우선순위 낮음

## 4. 검증

```
cargo build --release           — Finished
cargo test --release --all-targets — (1412 passed 기대)
```

기존 SVGPlayer 는 그대로 사용 (디스패치 미통합) — RasterPlayer 는 별도 모듈, Stage 16 에서 통합/디스패치 결정.

## 5. 산출물

- 소스: `src/wmf/converter/raster/player.rs` (확장)
- 본 보고서: `mydocs/working/task_m100_902_stage13.md`

## 6. 다음 단계

Stage 14: text rendering — fontdue + LO DrawText 알고리즘 + 폰트 매칭
