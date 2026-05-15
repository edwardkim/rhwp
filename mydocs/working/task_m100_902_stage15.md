# Task #902 Stage 15 보고서 — ellipse / round_rect + dispatcher 통합 helper

**Stage**: 15 / 17 (v2 확장)
**상태**: 완료 (geometry 추가 + 공개 API)

## 1. 추가 구현

### 1.1 raster Player records

| Record | 처리 |
|--------|------|
| `ellipse` | KAPPA (0.5522847498) 4-cubic-bezier 근사 ellipse |
| `round_rect` | 단순화 (rectangle 로 처리, 라운드 코너 follow-up) |

`arc` / `pie` / `chord` 는 cubic bezier sweep 변환 복잡 — follow-up.

### 1.2 dispatcher 공개 API

`src/renderer/svg.rs`:
```rust
pub fn rasterize_wmf_direct_pub(
    wmf_data: &[u8],
    target_width_px: f32,
    target_height_px: f32,
) -> Option<Vec<u8>>
```

WMF binary → RasterPlayer 직접 렌더링 → PNG. 2x supersampling.

### 1.3 검증 example

`examples/wmf_raster_test.rs` — CLI 도구:
```
cargo run --release --example wmf_raster_test -- input.wmf out.png [w] [h]
```

## 2. sample16 시험

```
Input WMF: 4773574 bytes (sample16 bin_id=3)
Output PNG: 92845 bytes, 2434×1648
```

RasterPlayer 가 4.7MB WMF 의 ~20K records 를 처리하여 PNG 출력 성공.

**한계**:
- Bitmap records (DIBSTRETCHBLT 177개) 미구현 → embedded bitmap 영역 누락
- arc/pie/chord 미구현
- 폰트 weight (bold) / italic 미구현

## 3. 검증

```
cargo build --release           — Finished
cargo build --release --example wmf_raster_test — Finished
```

## 4. 산출물

- 소스: `src/wmf/converter/raster/player.rs` (ellipse/round_rect), `src/renderer/svg.rs` (rasterize_wmf_direct + 공개 wrapper)
- example: `examples/wmf_raster_test.rs`
- 본 보고서: `mydocs/working/task_m100_902_stage15.md`
- 시험 출력: `/tmp/task902_raster_s16.png` (sample16 WMF → RasterPlayer)

## 5. dispatcher 결정 (Stage 16~17 영향)

**현재 default**: WMF → SVG → resvg PNG (기존 Stage 10 경로, 안정)
**RasterPlayer**: opt-in API (`rasterize_wmf_direct_pub`) 로만 제공

RasterPlayer 를 default 로 전환 시 bitmap 누락 = visual 회귀 위험. Stage 17 까지 SVG 경로 유지, RasterPlayer 는 별도 모듈로 공존. bitmap 구현 후속 task 에서 default 전환 검토.

## 6. 다음 단계

Stage 16: 광범위 회귀 검증 + 시각 비교 (LibreOffice 출력 대비)
Stage 17: 최종 보고서 + PR
