# Task #902 Stage 10 보고서 — B2 raster 구현 (WMF SVG → PNG)

**Stage**: 10 / 12 (v2 확장)
**상태**: 완료

## 1. 변경 영역

- `Cargo.toml` — `resvg` 0.47 을 native target 영역에서 non-optional 화
- `src/renderer/svg.rs` — `rasterize_wmf_svg_to_png()` 추가 + WMF embed 경로 (`render_image_node`, `draw_image`) 가 raster 우선 시도 → 실패 시 SVG fallback

## 2. 처리 흐름

```
WMF binary  →  convert_wmf_to_svg()  →  SVG bytes
                                            │
                                            ▼
                            rasterize_wmf_svg_to_png()  ──── 성공 ──── PNG bytes
                                            │                                │
                                            └──── 실패 (WASM 등) ──── SVG bytes
                                                                              │
                                            <image href="data:image/{png,svg+xml};base64,...">
```

## 3. 핵심 구현

### 3.1 rasterize_wmf_svg_to_png

```rust
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn rasterize_wmf_svg_to_png(
    svg_bytes: &[u8],
    target_width_px: f32,
    target_height_px: f32,
) -> Option<Vec<u8>> {
    use resvg::{tiny_skia, usvg};
    
    const SCALE: f32 = 2.0;  // supersampling 으로 crispness
    let raster_w = (target_width_px * SCALE).ceil() as u32;
    let raster_h = (target_height_px * SCALE).ceil() as u32;
    
    let svg_str = std::str::from_utf8(svg_bytes).ok()?;
    let mut options = usvg::Options::default();
    {
        let fontdb = options.fontdb_mut();
        fontdb.load_system_fonts();
        fontdb.set_sans_serif_family("Nanum Gothic");
        fontdb.set_serif_family("Nanum Myeongjo");
    }
    let tree = usvg::Tree::from_str(svg_str, &options).ok()?;
    
    let mut pixmap = tiny_skia::Pixmap::new(raster_w, raster_h)?;
    let scale_x = raster_w as f32 / tree.size().width();
    let scale_y = raster_h as f32 / tree.size().height();
    let transform = tiny_skia::Transform::from_scale(scale_x, scale_y);
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    pixmap.encode_png().ok()
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn rasterize_wmf_svg_to_png(...) -> Option<Vec<u8>> {
    None  // WASM: SVG embed fallback
}
```

### 3.2 핵심 효과

- **폰트 일관성**: usvg fontdb 가 시스템 폰트 + Nanum Gothic 명시적 매핑 — 브라우저 fontconfig 의존 제거
- **2x supersampling**: 1217×824 PNG (bbox 608×412 의 2배) — 디스플레이 줌 시도 crisp
- **파일 크기 감소**: SVG 4.4 MB → PNG 292 KB (외부 SVG 4.4 MB → 534 KB total, 약 87% 감소)
- **렌더링 성능 향상**: 브라우저가 SVG 텍스트/패스 수천 개를 매번 렌더 → 단순 PNG 표시

## 4. Cargo.toml 변경

```toml
# Before:
resvg = { version = "0.47", optional = true }
native-skia = ["dep:resvg", "dep:skia-safe"]

# After:
resvg = "0.47"  # B2 raster 위해 항상 native 빌드 포함
native-skia = ["dep:skia-safe"]
```

WASM 빌드는 영향 없음 (`[target.'cfg(not(target_arch = "wasm32"))'`).

## 5. 검증 결과

### 5.1 회귀 테스트

```
cargo test --release --all-targets — 1412 passed / 0 failed
cargo test --release --test svg_snapshot — 8 / 8 passed
```

### 5.2 SVG/PNG 출력

| Sample | 출력 |
|--------|------|
| sample14 (Task #860 fixture) | 11 페이지 ✓ |
| sample16 page 18 (#902 본 대상) | SVG 534 KB, 내부 WMF PNG 1217×824 ✓ |
| sample17 | 12 페이지 ✓ |
| sample18 | 69 페이지 ✓ |

### 5.3 시각 효과

- WMF 텍스트가 **rsvg 가 직접 렌더한 polygon-based glyph** 으로 변환 → 브라우저 별 폰트 차이 제거
- usvg 의 fontdb 가 Nanum Gothic 우선 매핑 → cross-platform 일관 렌더
- 한컴 viewer 와 동일 폰트 (Apple SD Gothic Neo / Nanum Gothic / Malgun Gothic 등 시스템 한국어 폰트) 사용 → 더 가까운 시각 정합

## 6. 산출물

- 소스 수정: `Cargo.toml`, `src/renderer/svg.rs`
- 본 보고서: `mydocs/working/task_m100_902_stage10.md`
- 검증 SVG/PNG: `/tmp/task902_s10/`, `/tmp/task902_s10_s{14,17,18}/`

## 7. 다음 단계

Stage 11: 광범위 회귀 검증 + 시각 정합 비교
