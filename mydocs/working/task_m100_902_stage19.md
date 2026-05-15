# Task #902 Stage 19 보고서 — font weight + italic (synthetic)

**Stage**: 19 / 24+ (v2 점진 포팅)
**상태**: 완료

## 1. 변경 영역

- `src/wmf/converter/raster/text.rs` — synthetic bold + italic 처리
- `src/wmf/converter/raster/player.rs` — font.weight + font.italic 전달

## 2. 알고리즘

### 2.1 Synthetic bold (weight ≥ 700)

```rust
let bold_extra = if weight >= 700 { 1 } else { 0 };
// ...
blit_alpha(pixmap, &bitmap, w, h, bx, by, color, slant);
if bold_extra > 0 {
    // 1px 오른쪽 오프셋 재렌더 → 두께 확장
    blit_alpha(pixmap, &bitmap, w, h, bx + bold_extra, by, color, slant);
}
```

실제 Bold 글꼴 변형체 부재 시도 시각 효과 제공. LO 의 synthetic bold 와 유사.

### 2.2 Synthetic italic (slant)

```rust
let italic_slant: f32 = if italic { 0.25 } else { 0.0 };
// blit_alpha 안에서:
let shear_offset = if italic_slant > 0.0 {
    ((h - sy) as f32 * italic_slant).round() as i32
} else { 0 };
let px = dest_x + sx + shear_offset;  // glyph 위쪽이 오른쪽으로 shift
```

각 픽셀 행을 y 에 비례하여 오른쪽 shift — synthetic italic.

## 3. 검증

```
cargo build --release           — Finished
cargo test --release --all-targets — 1412 passed / 0 failed
```

## 4. 산출물

- 소스: `src/wmf/converter/raster/text.rs`, `player.rs`
- 본 보고서: `mydocs/working/task_m100_902_stage19.md`
- 검증: `/tmp/task902_s19.png` (sample16)

## 5. 다음 단계

Stage 20: arc / pie / chord (cubic bezier sweep 변환)
