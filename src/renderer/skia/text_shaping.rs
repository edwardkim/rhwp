//! Legacy complex-script spans; exact GlyphRun replay never enters this path.

use std::cell::OnceCell;

use skia_safe::{Font, FontMgr, Shaper, TextBlob};

pub(super) struct ShapedSpan {
    pub start: usize,
    pub end: usize,
    pub blob: TextBlob,
    /// SkShaper의 위쪽 원점을 공통 글줄의 기준선으로 되돌리는 실제 run ascent.
    pub baseline_offset: f32,
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
                | 0x1100..=0x11FF | 0xA960..=0xA97F | 0xD7B0..=0xD7FF
                | 0x1780..=0x17FF | 0x1AB0..=0x1AFF | 0x1DC0..=0x1DFF
                | 0x200C..=0x200D | 0x20D0..=0x20FF | 0xFE00..=0xFE0F
                | 0xFE20..=0xFE2F | 0x1F1E6..=0x1FAFF | 0xE0100..=0xE01EF
        );
    }
    (needs_shaping, rtl)
}

pub(super) fn prepare_shaped_spans(
    clusters: &[(usize, String)],
    visual: Option<&crate::renderer::text_direction::VisualTextPositions<'_>>,
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
                // 개체 마커를 이웃 결합 문자와 같은 blob에 넣으면 별도 cluster의
                // 비표시 계약을 우회한다. 앞뒤 span은 기존 배치 원점을 유지한다.
                || character == '\u{FFFC}'
                || matches!(character, '\u{00ad}' | '\u{00b7}' | '\u{2018}'..='\u{2027}'
                    | '\u{203b}' | '\u{2460}'..='\u{27bf}')
                || crate::renderer::boxed_pua_number(character).is_some()
        })
    };
    let fonts: Vec<_> = clusters.iter().map(|(_, text)| make_font(text)).collect();
    let profiles: Vec<_> = clusters
        .iter()
        .enumerate()
        .map(|(index, (_, text))| {
            // 가시 호환 글리프가 있는 source face는 nominal 경로에서 한 번 그린다.
            let compatibility = fonts[index]
                .as_ref()
                .and_then(|font| super::text_replay::fonts::compatibility_glyph(font, text))
                .is_some();
            (eligible(text) && !compatibility, shaping_profile(text))
        })
        .collect();
    let scopes: Vec<_> = clusters
        .iter()
        .map(|(start, text)| {
            visual.and_then(|v| v.span_scope(*start, *start + text.chars().count()))
        })
        .collect();
    // 이웃의 기본 글꼴로 묶은 뒤 OS가 별도 폴백을 고르게 하지 않는다.
    // 각 cluster에 대해 본문과 같은 후보 체인이 선택한 실제 face를 보존한다.
    let next_complex = |index: usize| {
        profiles
            .get(index + 1)
            .is_some_and(|(eligible, (complex, _))| *eligible && *complex)
            && fonts[index]
                .as_ref()
                .zip(fonts.get(index + 1).and_then(Option::as_ref))
                .is_some_and(|(a, b)| a.typeface().unique_id() == b.typeface().unique_id())
    };
    let mut spans = Vec::new();
    let mut index = 0;
    while index < clusters.len() {
        if !profiles[index].0 || !(profiles[index].1 .0 || next_complex(index)) {
            index += 1;
            continue;
        }
        let start = index;
        let Some(font) = fonts[start].as_ref() else {
            index += 1;
            continue;
        };
        let typeface = font.typeface().unique_id();
        let mut text = String::new();
        while index < clusters.len()
            && profiles[index].0
            && scopes[index] == scopes[start]
            && (index == start || profiles[index].1 .0 || next_complex(index))
            && fonts[index]
                .as_ref()
                .is_some_and(|f| f.typeface().unique_id() == typeface)
        {
            text.push_str(&clusters[index].1);
            index += 1;
        }
        let Some(shaper) = shaper.get_or_init(|| Shaper::new_shape_then_wrap(font_mgr.clone()))
        else {
            continue;
        };
        let Some((blob, _)) = shaper.shape_text_blob(
            &text,
            font,
            !scopes[start].map_or_else(|| shaping_profile(&text).1, |(_, _, rtl)| rtl),
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
        // SkTextBlobBuilderRunHandler는 commitRunInfo에서 -max(ascent)를 더한다.
        // 잉크 bounds가 아니라 blob에 실제 사용된 face의 줄 메트릭으로 상쇄한다.
        let baseline_offset = skia_safe::TextBlobIter::new(&blob)
            .filter_map(|run| run.typeface().as_ref().cloned())
            .map(|face| Font::new(face, font.size()).metrics().1.ascent)
            .fold(0.0_f32, f32::min);
        spans.push(ShapedSpan {
            start,
            end: index,
            blob,
            baseline_offset,
        });
    }
    spans
}
