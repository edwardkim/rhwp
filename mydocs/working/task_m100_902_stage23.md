# Task #902 Stage 23 보고서 — bitmap records 확장 (DIBBITBLT + STRETCHDIB)

**Stage**: 23 / 25+ (v2 점진 포팅)
**상태**: 완료

## 1. 변경 영역

`src/wmf/converter/raster/player.rs`:
- `blit_dib()` helper 함수 (Stage 18 의 DIBSTRETCHBLT 로직 추출)
- `device_independent_bitmap_bit_blt()` 구현
- `stretch_device_independent_bitmap()` 구현

## 2. 알고리즘

Stage 18 의 DIBSTRETCHBLT 와 동일 패턴:
- DIB → BMP bytes (Bitmap::from)
- image crate decode + Lanczos3 resize
- pixmap.pixels_mut() blit

```rust
fn blit_dib(&mut self, dib, x_dest, y_dest, dest_w, dest_h) {
    let bmp = Bitmap::from(dib).to_vec();
    let img = image::load_from_memory_with_format(&bmp, ImageFormat::Bmp)?;
    let resized = img.resize_exact(target_w, target_h, FilterType::Lanczos3);
    // pixmap blit
}
```

3 records 가 동일 helper 사용 → DRY.

## 3. 미구현 변형

- `bit_blt` (META_BITBLT — non-DIB) — drawing primitives 만 사용 시 미사용
- `stretch_blt` (META_STRETCHBLT — non-DIB) — 동일

대부분 WMF 는 DIB 변형 사용. non-DIB 는 sample16/18 에 없음.

## 4. 검증

```
cargo build --release           — Finished
cargo test --release --all-targets — (1412 passed)
```

## 5. 다음 단계

Stage 24: 추가 records (region ops, palette ops, set_layout) — 빈도 낮음
또는 Stage 25 wrap-up (PR 직전)
