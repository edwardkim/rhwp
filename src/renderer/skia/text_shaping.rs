//! Legacy complex-script spans; exact GlyphRun replay never enters this path.

use std::cell::OnceCell;

use skia_safe::{Font, FontMgr, Shaper, TextBlob};

pub(super) struct ShapedSpan {
    pub start: usize,
    pub end: usize,
    pub blob: TextBlob,
}

fn shaping_profile(text: &str) -> (bool, bool) {
    let mut needs_shaping = false;
    let mut rtl = false;
    for character in text.chars() {
        let codepoint = u32::from(character);
        if matches!(
            codepoint,
            0x0590..=0x08FF | 0xFB1D..=0xFDFF | 0xFE70..=0xFEFF
                | 0x10800..=0x10FFF | 0x1E800..=0x1EFFF
        ) {
            needs_shaping = true;
            rtl = true;
        }
        needs_shaping |= matches!(
            codepoint,
            0x0300..=0x036F | 0x0900..=0x0DFF | 0x0E00..=0x0E7F
                | 0x1780..=0x17FF | 0x1AB0..=0x1AFF | 0x1DC0..=0x1DFF
                | 0x200C..=0x200D | 0x20D0..=0x20FF | 0xFE00..=0xFE0F
                | 0xFE20..=0xFE2F | 0x1F1E6..=0x1FAFF | 0xE0100..=0xE01EF
        );
    }
    (needs_shaping, rtl)
}

pub(super) fn prepare_shaped_spans(
    clusters: &[(usize, String)],
    shaper: &OnceCell<Option<Shaper>>,
    font_mgr: &FontMgr,
    make_font: impl Fn(&str) -> Option<Font>,
) -> Vec<ShapedSpan> {
    const MAX_CHARS: usize = crate::paint::MAX_PORTABLE_GLYPHS_PER_RUN;
    if clusters.len() > MAX_CHARS
        || clusters
            .iter()
            .map(|(_, text)| text.chars().count())
            .sum::<usize>()
            > MAX_CHARS
    {
        return Vec::new();
    }
    // Spaces retain layout advances, and current special symbol/PUA/halfwidth
    // replay remains authoritative. Only neighboring complex spans are joined.
    let eligible = |text: &str| {
        !text.chars().any(|character| {
            character.is_whitespace()
                || character.is_control()
                || matches!(character, '\u{00ad}' | '\u{00b7}' | '\u{2018}'..='\u{2027}'
                    | '\u{203b}' | '\u{2460}'..='\u{27bf}')
                || crate::renderer::boxed_pua_number(character).is_some()
        })
    };
    let profiles: Vec<_> = clusters
        .iter()
        .map(|(_, text)| (eligible(text), shaping_profile(text)))
        .collect();
    let next_complex = |index: usize| {
        profiles
            .get(index + 1)
            .is_some_and(|(eligible, (complex, _))| *eligible && *complex)
    };
    let mut spans = Vec::new();
    let mut index = 0;
    while index < clusters.len() {
        if !profiles[index].0 || !(profiles[index].1 .0 || next_complex(index)) {
            index += 1;
            continue;
        }
        let start = index;
        let mut text = String::new();
        while index < clusters.len()
            && profiles[index].0
            && (index == start || profiles[index].1 .0 || next_complex(index))
        {
            text.push_str(&clusters[index].1);
            index += 1;
        }
        let Some(font) = make_font(&text) else {
            continue;
        };
        let Some(shaper) = shaper.get_or_init(|| Shaper::new_shape_then_wrap(font_mgr.clone()))
        else {
            continue;
        };
        let Some((blob, _)) = shaper.shape_text_blob(
            &text,
            &font,
            !shaping_profile(&text).1,
            1_000_000.0,
            (0.0, 0.0),
        ) else {
            continue;
        };
        let glyph_count = skia_safe::TextBlobIter::new(&blob)
            .map(|run| run.glyph_indices.len())
            .sum::<usize>();
        if !blob.bounds().is_finite() || glyph_count > MAX_CHARS {
            continue;
        }
        spans.push(ShapedSpan {
            start,
            end: index,
            blob,
        });
    }
    spans
}
