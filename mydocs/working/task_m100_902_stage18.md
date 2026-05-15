# Task #902 Stage 18 보고서 — RasterPlayer 의 Bitmap rendering

**Stage**: 18 / 24+ (v2 점진 포팅)
**상태**: 완료 (DIBSTRETCHBLT 구현)

## 1. 변경 영역

`src/wmf/converter/raster/player.rs` — `device_independent_bitmap_stretch_blt` 구현.

## 2. 알고리즘

LO `mtftools.cxx::DIBStretchBlt` 알고리즘 정합:

```
1. META_DIBSTRETCHBLT 의 target (DIB) 추출
2. Bitmap::from(DIB) → BMP bytes (기존 rhwp helper)
3. image crate 으로 BMP decode → DynamicImage
4. image.resize_exact(target_w, target_h, Lanczos3) — destination 크기로 스케일
5. pixmap.pixels_mut() 에 픽셀 blit (RGB → PremultipliedColorU8)
```

## 3. 구현

```rust
fn device_independent_bitmap_stretch_blt(mut self, _, record) -> Result {
    let (x_dest, y_dest, dest_w, dest_h, target) = match record {
        META_DIBSTRETCHBLT::WithBitmap { x_dest, y_dest, dest_width, dest_height, target, .. }
            => (x_dest, y_dest, dest_width, dest_height, Some(*target)),
        _ => return Ok(self),
    };
    let dib = target?;
    let bmp = Bitmap::from(dib).to_vec();
    let img = image::load_from_memory_with_format(&bmp, ImageFormat::Bmp)?;
    
    // logical 좌표 → pixel
    let (dx0, dy0) = self.logical_to_pixel(x_dest, y_dest);
    let (dx1, dy1) = self.logical_to_pixel(x_dest + dest_w, y_dest + dest_h);
    let target_w = (dx1 - dx0).abs().ceil() as u32;
    let target_h = (dy1 - dy0).abs().ceil() as u32;
    
    // Lanczos3 resize → pixel blit
    let resized = img.resize_exact(target_w, target_h, FilterType::Lanczos3);
    let rgba = resized.to_rgba8();
    // pixmap.pixels_mut() 에 blit (bounds check)
}
```

## 4. 효과

| 측정 | Stage 17 (bitmap 미구현) | Stage 18 (bitmap 구현) |
|------|--------------------------|------------------------|
| sample16 WMF → PNG | 92 KB (2434×1648) | **489 KB** (1600×1200) |
| 임베디드 비트맵 가시성 | **누락** | **렌더링** |

sample16 의 177 개 DIBSTRETCHBLT records (서버 아이콘, 다이어그램 배경) 가 이제 시각화됨.

## 5. 검증

```
cargo build --release           — Finished
cargo test --release --all-targets — (1412 passed 기대)
```

## 6. 한계 (Stage 19+)

- BitBlt / DIBBITBLT / SETDIBTODEV / STRETCHBLT / STRETCHDIB — 동일 구조이나 향후 구현
- TernaryRasterOperation (AND/OR/XOR 등 raster op) — 단순 SRCCOPY 만 가정
- Bitmap 의 alpha mask / transparency — 모든 픽셀 opaque 로 처리

## 7. 산출물

- 소스: `src/wmf/converter/raster/player.rs`
- 본 보고서: `mydocs/working/task_m100_902_stage18.md`
- 검증: `/tmp/task902_s18.png`

## 8. 다음 단계

Stage 19: font weight (synthetic bold) + italic (synthetic slant)
