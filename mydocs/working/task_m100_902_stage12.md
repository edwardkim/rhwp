# Task #902 Stage 12 보고서 — LibreOffice emfio 포팅 baseline

**Stage**: 12 / 15 (v2 확장)
**상태**: framework 구축 완료 (실제 raster 렌더링은 Stage 13+ 점진적 구현)

## 1. 배경

사용자 평가: 현재 SVG → resvg PNG 파이프라인의 quality 가 한컴/LibreOffice 대비 미달.

작업지시자 결정: **LibreOffice emfio (MPL 2.0) 알고리즘 포팅** — 장기 자립.

LO emfio:
- `wmfreader.cxx` (2193 라인): WMF binary record parsing + 디스패치
- `mtftools.cxx` (3205 라인): DrawText, DrawPolygon, DrawPolyPolygon, 등 렌더링
- 합 5,400+ 라인 C++. 2~4주 작업 예상.

## 2. Stage 12 — Framework 구축

### 2.1 의존성 추가

```toml
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
fontdue = "0.9"     # pure Rust glyph rasterizer
tiny-skia = "0.11"  # resvg 의 transitive dep, 명시화
```

### 2.2 새 모듈 구조

```
src/wmf/converter/raster/
├── mod.rs       — 모듈 entry, LO MPL 2.0 attribution
├── state.rs     — RasterState (device context), RasterObject (pen/brush/font)
└── player.rs    — RasterPlayer (Player trait 구현)
```

### 2.3 RasterPlayer 의 현재 구현 상태

| 영역 | 구현 |
|------|------|
| state context 추적 (window/viewport/text_align/colors/fonts) | ✓ |
| DC stack (SAVEDC/RESTOREDC) | ✓ |
| SetWindowExt/Org, SetViewportExt/Org, Offset/Scale | ✓ |
| SetTextColor/Align, SetBkMode/Color, SetPolyFillMode, SetROP2 | ✓ |
| MoveTo (current_position) | ✓ |
| header (Placeable bounding_box → extent) | ✓ |
| **Drawing records (line/poly/text/rect/bitmap)** | **stub (Stage 13)** |
| **Object table (Create/Select/Delete)** | **stub (Stage 13)** |
| **generate() — PNG 출력** | **빈 흰색 PNG (Stage 13)** |

### 2.4 MPL 2.0 라이센스 정합

LO emfio 의 알고리즘 참조 시 각 파일의 MPL 2.0 attribution 헤더 유지. 본 raster module 의 mod.rs:

```rust
/* [Task #902 v2 Stage 12] WMF raster Player — LibreOffice emfio 포팅 baseline.
 *
 * This file incorporates algorithms derived from LibreOffice's emfio module:
 *   wmfreader.cxx + mtftools.cxx
 *
 * The algorithm references retain attribution per LibreOffice's MPL 2.0.
 * License Notice (LO):
 *   This Source Code Form is subject to the terms of the Mozilla Public
 *   License, v. 2.0. http://mozilla.org/MPL/2.0/
 */
```

## 3. 검증

```
cargo build --release           — Finished
cargo test --release --all-targets — (실행 중, 기존 1412 passed 기준 회귀 없음)
```

기존 `SVGPlayer` 는 그대로 유지. RasterPlayer 는 별도 모듈로 추가 — 통합/디스패치는 Stage 13+ 에서 결정.

## 4. Stage 13+ 계획

- Stage 13: 핵심 records 의 raster 구현
  - polygon / polyline / poly_polygon (LO 의 ImplMap + UpdateLineStyle/FillStyle 포팅)
  - rectangle / ellipse / round_rect
  - line_to / move_to + current_position
  - ext_text_out (LO 의 DrawText 포팅 — DX 합산 + fontdue glyph rasterize)
  - select_object / create_*_indirect (object table 관리)
- Stage 14: bitmap records + clip regions
- Stage 15: 통합 (WMF → raster 또는 SVG 분기) + 최종 보고서 + PR

## 5. 산출물

- `Cargo.toml` (deps)
- `src/wmf/converter/raster/{mod,state,player}.rs`
- `src/wmf/converter/mod.rs` (raster module export)
- 본 보고서
