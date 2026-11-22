//! 보이는 기호의 cmap 항목이 빈 윤곽을 가리키면 다음 글꼴에 맡긴다.
//!
//! 원본 프로그램은 그대로 두고 실제 fallback의 문자 공급 범위만 제한한다.
//! 공백·문자·결합/서식 문자와 별도 color/bitmap/SVG 표현은 이 정책의 대상이 아니다.

mod bitmap;

use std::collections::BTreeSet;
use ttf_parser::{CFFError, Face, GlyphId, OutlineBuilder, Tag};
use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};

const HYPHENATION_POINT: char = '\u{2027}';

/// 실제 프로그램의 cmap로 요청 글리프 공급을 확인한다. 파일·별칭·캐시를 바꾸지 않는다.
pub fn font_supports_text(data: &[u8], face_index: u32, text: &str) -> bool {
    font_missing_text(data, face_index, text).is_some_and(|missing| missing.is_empty())
}

/// 다음 실제 공급자가 맡아야 할 글리프다. 해석 실패는 빈 집합(완전 공급)이 아니다.
pub fn font_missing_text(data: &[u8], face_index: u32, text: &str) -> Option<String> {
    font_missing_text_impl(data, face_index, text, false)
}

/// 메타파일의 실제 공백 글리프도 확인한다. 기존 일반 문자 계약은 유지한다.
pub fn font_missing_text_with_spaces(data: &[u8], face_index: u32, text: &str) -> Option<String> {
    font_missing_text_impl(data, face_index, text, true)
}

fn font_missing_text_impl(
    data: &[u8],
    face_index: u32,
    text: &str,
    include_spaces: bool,
) -> Option<String> {
    if data.len() > crate::paint::MAX_PORTABLE_FONT_BLOB_BYTES
        || text.chars().count() > crate::paint::MAX_PORTABLE_GLYPHS_PER_RUN
    {
        return None;
    }
    let face = Face::parse(data, face_index).ok()?;
    Some(
        text.chars()
            .filter(|character| {
                (if include_spaces {
                    !character.is_control()
                } else {
                    !character.is_whitespace()
                }) && !face
                    .glyph_index(*character)
                    .is_some_and(|glyph| glyph.0 != 0)
            })
            .collect(),
    )
}

pub fn is_visible_symbol(character: char) -> bool {
    // 가운뎃점은 문장부호이며 일부 HFT 변환 글꼴에는 폭만 남아 있다.
    if character == HYPHENATION_POINT {
        return true;
    }
    // Unicode BRAILLE PATTERN BLANK는 OtherSymbol이지만 본래 빈 칸이다.
    character != '\u{2800}'
        && matches!(
            character.general_category(),
            GeneralCategory::MathSymbol
                | GeneralCategory::CurrencySymbol
                | GeneralCategory::ModifierSymbol
                | GeneralCategory::OtherSymbol
        )
}

fn declared_empty_glyf(face: &Face<'_>, glyph: GlyphId) -> Option<bool> {
    use ttf_parser::head::IndexToLocationFormat;
    let raw = face.raw_face();
    let loca = raw.table(Tag::from_bytes(b"loca"))?;
    let glyf = raw.table(Tag::from_bytes(b"glyf"))?;
    let offset = |id: usize| -> Option<usize> {
        match face.tables().head.index_to_location_format {
            IndexToLocationFormat::Short => Some(
                usize::from(u16::from_be_bytes(
                    loca.get(id * 2..id * 2 + 2)?.try_into().ok()?,
                )) * 2,
            ),
            IndexToLocationFormat::Long => usize::try_from(u32::from_be_bytes(
                loca.get(id * 4..id * 4 + 4)?.try_into().ok()?,
            ))
            .ok(),
        }
    };
    let start = offset(usize::from(glyph.0))?;
    let end = offset(usize::from(glyph.0) + 1)?;
    let data = glyf.get(start..end)?;
    if data.is_empty() {
        return Some(true);
    }
    // 불완전한 헤더·해석 실패를 빈 glyph라는 판정으로 바꾸지 않는다.
    (data.len() >= 10).then(|| i16::from_be_bytes([data[0], data[1]]) == 0)
}

struct DiscardOutline;

impl OutlineBuilder for DiscardOutline {
    fn move_to(&mut self, _: f32, _: f32) {}
    fn line_to(&mut self, _: f32, _: f32) {}
    fn quad_to(&mut self, _: f32, _: f32, _: f32, _: f32) {}
    fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {}
    fn close(&mut self) {}
}

/// 빈 glyf 또는 비어 있음이 확인된 CFF 가운뎃점에서 다른 잉크 표현도 없는 경우를 반환한다.
pub fn empty_symbol_codepoints(data: &[u8], face_index: u32) -> BTreeSet<u32> {
    let mut empty = BTreeSet::new();
    let Ok(face) = Face::parse(data, face_index) else {
        return empty;
    };
    if face.is_variable() {
        return empty;
    }
    // 외부 cmap의 범위를 순회하지 않아 조작된 format 12의 거대한 group에도
    // 실행량이 늘어나지 않는다. Unicode의 유한한 기호 집합만 한 번 마련한다.
    static SYMBOLS: std::sync::OnceLock<Vec<char>> = std::sync::OnceLock::new();
    let cff = face.tables().cff;
    let characters = if face.tables().glyf.is_some() {
        SYMBOLS
            .get_or_init(|| {
                (0..=0x10FFFF)
                    .filter_map(char::from_u32)
                    .filter(|ch| is_visible_symbol(*ch))
                    .collect()
            })
            .as_slice()
    } else if cff.is_some() {
        &[HYPHENATION_POINT]
    } else {
        return empty;
    };
    for &character in characters {
        let Some(glyph) = face.glyph_index(character) else {
            continue;
        };
        if glyph.0 == 0 {
            continue;
        }
        let has_empty_outline = if face.tables().glyf.is_some() {
            declared_empty_glyf(&face, glyph) == Some(true)
        } else {
            // ZeroBBox는 해석이 정상 종료됐지만 윤곽을 만들지 않았다는 뜻이다.
            cff.is_some_and(|table| {
                matches!(
                    table.outline(glyph, &mut DiscardOutline),
                    Err(CFFError::ZeroBBox)
                )
            })
        };
        if has_empty_outline
            && !face.is_color_glyph(glyph)
            && face.glyph_svg_image(glyph).is_none()
            && !bitmap::may_supply_ink(&face, glyph)
        {
            empty.insert(character as u32);
        }
    }
    empty
}

/// FontFace unicodeRange가 Native와 같은 빈 기호 제외 결과를 소비한다.
/// 빈 문자열은 제한 없음이다. 원본 bytes/cmap/glyph/advance를 수정하지 않는다.
#[cfg_attr(feature = "wasm-api", wasm_bindgen::prelude::wasm_bindgen(js_name = fontSymbolUnicodeRange))]
pub fn font_symbol_unicode_range(data: &[u8], face_index: u32) -> String {
    let empty = empty_symbol_codepoints(data, face_index);
    if empty.is_empty() {
        return String::new();
    }
    let mut ranges = Vec::new();
    let mut start = 0;
    for codepoint in empty {
        if start < codepoint {
            ranges.push(format!("U+{start:X}-{:X}", codepoint - 1));
        }
        start = codepoint + 1;
    }
    if start <= 0x10FFFF {
        ranges.push(format!("U+{start:X}-10FFFF"));
    }
    ranges.join(",")
}
