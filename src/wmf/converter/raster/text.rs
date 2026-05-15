//! [Task #902 v2 Stage 14] WMF text rendering via fontdue.
//!
//! 알고리즘 출처: LibreOffice emfio (MPL 2.0)
//! - `mtftools.cxx::DrawText` — DX 합산 + glyph 위치 계산
//! - `wmfreader.cxx::W_META_EXTTEXTOUT` — MBCS byte → Unicode glyph index 변환
//!
//! 본 구현은 LO 의 DX 합산 로직과 정합:
//!   wide char (2 byte CJK) 마다 DX 2 entries 차지, 각 Unicode grapheme 의
//!   advance = sum(DX[byte..byte+width]).

use super::state::*;
use crate::wmf::parser::ColorRef;
use std::sync::OnceLock;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[cfg(not(target_arch = "wasm32"))]
use fontdue::{Font, FontSettings};
#[cfg(not(target_arch = "wasm32"))]
use tiny_skia::{Color, Paint, Pixmap, PixmapPaint, PremultipliedColorU8, Transform};

/// 시스템 폰트 캐시 — Korean 폰트 우선 검색.
#[cfg(not(target_arch = "wasm32"))]
static KOREAN_FONT: OnceLock<Option<Font>> = OnceLock::new();
#[cfg(not(target_arch = "wasm32"))]
static LATIN_FONT: OnceLock<Option<Font>> = OnceLock::new();

#[cfg(not(target_arch = "wasm32"))]
fn load_font_from_paths(paths: &[&str]) -> Option<Font> {
    for path in paths {
        let Ok(bytes) = std::fs::read(path) else { continue };
        if let Ok(font) = Font::from_bytes(bytes, FontSettings::default()) {
            return Some(font);
        }
    }
    None
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_korean_font() -> Option<&'static Font> {
    KOREAN_FONT
        .get_or_init(|| {
            // 시스템 별 Korean 폰트 경로 후보
            let home = std::env::var("HOME").unwrap_or_default();
            let user_nanum = format!("{}/Library/Fonts/NanumGothic.ttf", home);
            let user_malgun = format!("{}/Library/Fonts/MALGUN.TTF", home);
            load_font_from_paths(&[
                // macOS user
                user_nanum.as_str(),
                user_malgun.as_str(),
                "/Library/Fonts/AppleSDGothicNeo.ttc",
                "/System/Library/Fonts/AppleSDGothicNeo.ttc",
                // Linux 표준 경로
                "/usr/share/fonts/truetype/nanum/NanumGothic.ttf",
                "/usr/share/fonts/nanum/NanumGothic.ttf",
                "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
                "/usr/share/fonts/opentype/noto/NotoSansCJKkr-Regular.otf",
                // Windows
                "C:/Windows/Fonts/malgun.ttf",
            ])
        })
        .as_ref()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_latin_font() -> Option<&'static Font> {
    LATIN_FONT
        .get_or_init(|| {
            load_font_from_paths(&[
                // 기본 fallback — 한국어 폰트 우선 (Korean 폰트는 Latin glyph 도 포함)
                "/System/Library/Fonts/Helvetica.ttc",
                "/Library/Fonts/Arial.ttf",
                "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
                "C:/Windows/Fonts/arial.ttf",
            ])
        })
        .as_ref()
}

#[cfg(not(target_arch = "wasm32"))]
fn pick_font_for_grapheme(g: &str) -> Option<&'static Font> {
    // CJK 영역: Hangul (U+AC00~U+D7A3), CJK Unified Ideographs, etc.
    let has_cjk = g.chars().any(|c| {
        let cp = c as u32;
        (0x1100..=0x11FF).contains(&cp)       // Hangul Jamo
            || (0x3000..=0x303F).contains(&cp) // CJK 기호 영역
            || (0x3130..=0x318F).contains(&cp) // Hangul Compatibility Jamo
            || (0x4E00..=0x9FFF).contains(&cp) // CJK Unified
            || (0xAC00..=0xD7A3).contains(&cp) // Hangul Syllables
    });
    if has_cjk {
        get_korean_font().or_else(get_latin_font)
    } else {
        get_latin_font().or_else(get_korean_font)
    }
}

/// EXTTEXTOUT 의 텍스트를 pixmap 에 렌더링한다.
///
/// LO `DrawText` 알고리즘 포팅:
///   1. DX 배열은 MBCS byte index — wide char (s.width=2) 마다 DX 2 entries 차지
///   2. 각 grapheme i 의 advance = sum(DX[byte..byte+width])
///   3. 각 glyph 의 origin = position.x + sum(advances[0..i])
///   4. fontdue 로 glyph rasterize 후 pixmap 에 alpha-blend
///
/// [Task #902 v2 Stage 19] synthetic bold (weight ≥ 700) + italic (slant).
/// 실제 bold/italic 글꼴 변형체가 없는 경우에도 시각 효과 제공.
#[cfg(not(target_arch = "wasm32"))]
pub fn draw_text_with_dx(
    pixmap: &mut Pixmap,
    text: &str,
    dx: &[i16],
    origin_x: f32,
    origin_y: f32,
    font_size_logical: f32,
    pixel_per_logical_x: f32,
    pixel_per_logical_y: f32,
    color: &ColorRef,
    weight: i16,
    italic: bool,
) {
    let font_size_pixel = font_size_logical * pixel_per_logical_y;
    if font_size_pixel < 1.0 {
        return;
    }
    // synthetic bold: pixel level 의 weight 효과
    //   weight 400 (normal): no extra
    //   weight 700+ (bold): glyph 을 1 pixel 두께 확장 (오프셋 +1px 재렌더)
    let bold_extra = if weight >= 700 { 1 } else { 0 };
    // synthetic italic: 글자 위 부분이 오른쪽으로 기울어진 효과
    //   italic_slant = pixel shift per pixel of glyph height
    let italic_slant: f32 = if italic { 0.25 } else { 0.0 };

    let mut acc_x_logical: f32 = 0.0;
    let mut dx_idx: usize = 0;

    for (i, g) in text.graphemes(true).enumerate() {
        let Some(font) = pick_font_for_grapheme(g) else { return };
        let width = g.width().max(1);

        // logical advance (WMF DX 합산)
        let advance_logical: i32 = (0..width)
            .map(|k| i32::from(*dx.get(dx_idx + k).unwrap_or(&0)))
            .sum();

        let glyph_x_pixel = origin_x + acc_x_logical * pixel_per_logical_x;
        let glyph_y_pixel = origin_y;

        // 각 char 렌더
        for ch in g.chars() {
            let (metrics, bitmap) = font.rasterize(ch, font_size_pixel);
            if metrics.width == 0 || metrics.height == 0 { continue; }

            // baseline-relative origin
            let bx = (glyph_x_pixel + metrics.xmin as f32).round() as i32;
            let by = (glyph_y_pixel - metrics.ymin as f32 - metrics.height as f32).round() as i32;

            blit_alpha(
                pixmap,
                &bitmap,
                metrics.width as i32,
                metrics.height as i32,
                bx,
                by,
                color,
                italic_slant,
            );
            if bold_extra > 0 {
                // 1px 오프셋 재렌더로 두께 확장 (synthetic bold)
                blit_alpha(
                    pixmap,
                    &bitmap,
                    metrics.width as i32,
                    metrics.height as i32,
                    bx + bold_extra,
                    by,
                    color,
                    italic_slant,
                );
            }
        }

        if i > 0 || width > 0 {
            acc_x_logical += advance_logical as f32;
        }
        dx_idx += width;
    }
}

/// Alpha glyph bitmap 을 pixmap 에 alpha-blend.
/// [Stage 19] italic_slant > 0 시 글자 위쪽이 오른쪽으로 기울어짐 (synthetic italic).
#[cfg(not(target_arch = "wasm32"))]
fn blit_alpha(
    pixmap: &mut Pixmap,
    bitmap: &[u8],
    w: i32,
    h: i32,
    dest_x: i32,
    dest_y: i32,
    color: &ColorRef,
    italic_slant: f32,
) {
    let pw = pixmap.width() as i32;
    let ph = pixmap.height() as i32;
    let pixels = pixmap.pixels_mut();
    let r = color.red;
    let g = color.green;
    let b = color.blue;

    for sy in 0..h {
        let py = dest_y + sy;
        if py < 0 || py >= ph { continue; }
        // italic shear: y 가 작을수록 (글자 위쪽) 오른쪽 shift
        let shear_offset = if italic_slant > 0.0 {
            ((h - sy) as f32 * italic_slant).round() as i32
        } else {
            0
        };
        for sx in 0..w {
            let px = dest_x + sx + shear_offset;
            if px < 0 || px >= pw { continue; }
            let alpha = bitmap[(sy * w + sx) as usize];
            if alpha == 0 { continue; }
            let idx = (py * pw + px) as usize;
            let existing = pixels[idx];
            // alpha blend: out = src * alpha + dst * (1 - alpha)
            let a = alpha as u32;
            let inv_a = 255 - a;
            let er = existing.red() as u32;
            let eg = existing.green() as u32;
            let eb = existing.blue() as u32;
            let nr = ((r as u32 * a + er * inv_a) / 255).min(255) as u8;
            let ng = ((g as u32 * a + eg * inv_a) / 255).min(255) as u8;
            let nb = ((b as u32 * a + eb * inv_a) / 255).min(255) as u8;
            if let Some(blended) = PremultipliedColorU8::from_rgba(nr, ng, nb, 255) {
                pixels[idx] = blended;
            }
        }
    }
}
