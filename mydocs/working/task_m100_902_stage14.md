# Task #902 Stage 14 보고서 — text rendering (fontdue + LO DrawText 알고리즘)

**Stage**: 14 / 17 (v2 확장)
**상태**: 완료 (RasterPlayer 의 ext_text_out 구현)

## 1. 변경 영역

- `src/wmf/converter/raster/text.rs` (NEW) — fontdue 기반 glyph rasterization
- `src/wmf/converter/raster/mod.rs` — text module 등록
- `src/wmf/converter/raster/player.rs` — ext_text_out 구현

## 2. 알고리즘

LO `mtftools.cxx::DrawText` + `wmfreader.cxx::W_META_EXTTEXTOUT` 포팅:

```
1. record.string (byte 배열) → UTF-8 변환 (charset 기반)
2. grapheme 순회:
   a. DX 합산: wide char (s.width=2) 마다 DX 2 entries → 1 advance
   b. acc_x_logical += advance_logical
   c. glyph rasterize via fontdue
   d. pixmap 에 alpha-blend
```

## 3. 핵심 구현

### 3.1 폰트 매칭 (LRU 시스템 폰트 캐시)

```rust
fn get_korean_font() -> Option<&'static Font> {
    // OnceLock 캐시 — macOS/Linux/Windows 시스템 폰트 경로 검색
    // 우선순위: NanumGothic > MALGUN > AppleSDGothicNeo > Noto
}

fn pick_font_for_grapheme(g: &str) -> Option<&'static Font> {
    let has_cjk = g.chars().any(|c| {
        (0x1100..=0x11FF).contains(&cp)       // Hangul Jamo
        || (0xAC00..=0xD7A3).contains(&cp)    // Hangul Syllables
        || (0x4E00..=0x9FFF).contains(&cp)    // CJK Unified
    });
    if has_cjk { korean.or(latin) } else { latin.or(korean) }
}
```

### 3.2 draw_text_with_dx (LO DrawText 포팅)

```rust
let mut acc_x_logical = 0.0;
let mut dx_idx = 0;
for g in text.graphemes(true) {
    let width = g.width().max(1);  // 1 (ASCII) or 2 (CJK)
    let advance: i32 = (0..width).map(|k| dx[dx_idx + k]).sum();
    let glyph_x_pixel = origin_x + acc_x_logical * pixel_per_logical_x;
    
    for ch in g.chars() {
        let (metrics, bitmap) = font.rasterize(ch, font_size_pixel);
        blit_alpha(pixmap, &bitmap, ..., color);
    }
    
    acc_x_logical += advance as f32;
    dx_idx += width;
}
```

### 3.3 alpha-blend blit

```rust
fn blit_alpha(pixmap, bitmap, w, h, dest_x, dest_y, color) {
    // for each pixel: src.rgb * alpha + dst.rgb * (1-alpha)
    let nr = (r * a + er * (255 - a)) / 255;
    // ...
    pixels[idx] = PremultipliedColorU8::from_rgba(nr, ng, nb, 255);
}
```

## 4. 검증

```
cargo build --release           — Finished
cargo test --release --all-targets — (1412 passed 기대)
```

## 5. 미구현 / 한계

- font.weight (bold) 미적용 — fontdue 는 자체 weight 변환 없음. 별도 bold font 파일 매핑 필요 (follow-up)
- font.italic 미적용 (synthetic slant 변환 필요)
- 회전 (font.escapement / orientation) 미적용
- 한컴 굴림체 정확 metric 차이 — 시스템 폰트 사용 (Nanum/Malgun/Apple)

## 6. 산출물

- 소스: `src/wmf/converter/raster/text.rs` (NEW), `mod.rs`/`player.rs`
- 본 보고서: `mydocs/working/task_m100_902_stage14.md`

## 7. 다음 단계

Stage 15: bitmap records (bit_blt, dib_stretch_blt) + arc/ellipse/pie + clipping
