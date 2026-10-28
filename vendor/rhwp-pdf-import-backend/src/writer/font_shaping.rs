//! Text shaping via harfrust (the HarfBuzz project's own Rust port, MIT-licensed).
//!
//! `EmbeddedFont::encode_string` produces a naive char→glyph hex string
//! that works correctly for ASCII and most Latin Extended content but
//! falls down on:
//!
//! - **Ligatures** — e.g. `fi` should fuse to a single glyph in fonts
//!   that have one (DejaVu Sans Bold has `f_i`, Latin Modern has many).
//! - **Kerning** — `Av`, `To` etc. need glyph-pair x-advance adjustment.
//! - **Complex scripts** — Arabic glyph shaping is contextual; Devanagari
//!   reorders glyphs around base consonants; Indic clusters need rules
//!   from the font's GSUB/GPOS tables. None of this works without a
//!   shaping engine.
//!
//! This module wraps harfrust's shaper to produce a `ShapedRun` of
//! positioned glyphs that the v0.3.35 inline-formatting layer (Phase
//! LAYOUT) consumes when computing line widths and emitting Tj
//! operators with correct widths.
//!
//! Feature-gated on `system-fonts` (same as `font_discovery`) — the two
//! always travel together: discover a font with fontdb, shape with
//! harfrust against the same face bytes.

use harfrust::{FontRef, ShapeOptions, ShaperData, UnicodeBuffer};

/// One positioned glyph from text shaping. Coordinates are in font
/// design units (typically 1/units_per_em); convert to PDF points by
/// multiplying by `font_size / units_per_em`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapedGlyph {
    /// Glyph index in the original (pre-subset) font face.
    pub glyph_id: u16,
    /// Horizontal advance in font design units.
    pub x_advance: i32,
    /// Vertical advance in font design units (zero for horizontal text).
    pub y_advance: i32,
    /// Horizontal offset from the pen origin in font design units.
    pub x_offset: i32,
    /// Vertical offset from the baseline in font design units.
    pub y_offset: i32,
    /// Source-byte cluster index. HarfBuzz preserves clusters across
    /// reorderings so the inline-formatting layer can map a glyph back
    /// to the source character (needed for click-targeting, justify
    /// space distribution, hyphenation breakpoints, …).
    pub cluster: u32,
}

/// Result of shaping a string. Glyphs are in *visual* order
/// (left-to-right after BiDi reordering for the script's reading
/// direction), not source order.
#[derive(Debug, Clone, PartialEq)]
pub struct ShapedRun {
    /// Positioned glyphs.
    pub glyphs: Vec<ShapedGlyph>,
    /// Total horizontal advance in font design units. Sum of
    /// `glyph.x_advance` — convenience for line-breaking math.
    pub total_x_advance: i32,
}

/// Direction of the text run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Left-to-right (Latin, Cyrillic, CJK, …).
    Ltr,
    /// Right-to-left (Arabic, Hebrew).
    Rtl,
    /// Top-to-bottom — vertical scripts. Out of scope for v0.3.35
    /// (HTML→PDF doesn't ship CSS `writing-mode` until v0.3.36); we
    /// accept the variant so the API doesn't break when we add it.
    Ttb,
}

impl From<Direction> for harfrust::Direction {
    fn from(d: Direction) -> Self {
        match d {
            Direction::Ltr => harfrust::Direction::LeftToRight,
            Direction::Rtl => harfrust::Direction::RightToLeft,
            Direction::Ttb => harfrust::Direction::TopToBottom,
        }
    }
}

/// Shape a UTF-8 string into a positioned glyph run.
///
/// `face_bytes` is the raw TrueType/OpenType bytes — typically what
/// [`super::font_discovery::ResolvedFont::bytes`] handed back, or the
/// bytes that went into `EmbeddedFont::from_data`.
///
/// `direction` controls visual reordering. For mixed-direction runs
/// (English text inside an Arabic paragraph, etc.) the caller must
/// segment first via the `unicode-bidi` crate and call this once per
/// run; harfrust itself does no BiDi.
///
/// Returns `None` if the face bytes can't be parsed.
pub fn shape(text: &str, face_bytes: &[u8], direction: Direction) -> Option<ShapedRun> {
    let font = FontRef::from_index(face_bytes, 0).ok()?;
    let shaper_data = ShaperData::new(&font);
    let shaper = shaper_data.shaper(&font).instance(None).build();

    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(text);
    buffer.set_direction(direction.into());
    // Fill in script + language from the text. `guess_segment_properties`
    // only sets properties still unset, so the explicit direction above is
    // preserved. Complex scripts (Arabic, Indic) need the script set for
    // the shaper to select the right GSUB/GPOS rules.
    buffer.guess_segment_properties();
    let glyphs_buffer = shaper.shape(buffer, ShapeOptions::new());

    let infos = glyphs_buffer.glyph_infos();
    let positions = glyphs_buffer.glyph_positions();
    debug_assert_eq!(
        infos.len(),
        positions.len(),
        "harfrust invariant: infos and positions match in length"
    );

    let mut glyphs = Vec::with_capacity(infos.len());
    let mut total_x_advance = 0i32;
    for (info, pos) in infos.iter().zip(positions.iter()) {
        total_x_advance = total_x_advance.saturating_add(pos.x_advance);
        // codepoint after shaping is the glyph_id in the font (per
        // HarfBuzz's documented behaviour for the output buffer).
        glyphs.push(ShapedGlyph {
            glyph_id: info.glyph_id as u16,
            x_advance: pos.x_advance,
            y_advance: pos.y_advance,
            x_offset: pos.x_offset,
            y_offset: pos.y_offset,
            cluster: info.cluster,
        });
    }

    Some(ShapedRun {
        glyphs,
        total_x_advance,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEJAVU: &[u8] = include_bytes!("../../tests/fixtures/fonts/DejaVuSans.ttf");

    #[test]
    fn shape_ltr_ascii() {
        let run = shape("ABC", DEJAVU, Direction::Ltr).expect("shape must succeed");
        assert_eq!(run.glyphs.len(), 3);
        // Each glyph should advance to the right (positive x_advance).
        for g in &run.glyphs {
            assert!(g.x_advance > 0, "ASCII glyph must advance forward");
        }
        // Cluster indices map back to source byte offsets — for ASCII
        // those are 0, 1, 2.
        let clusters: Vec<u32> = run.glyphs.iter().map(|g| g.cluster).collect();
        assert_eq!(clusters, vec![0, 1, 2]);
    }

    #[test]
    fn shape_empty_string_yields_no_glyphs() {
        let run = shape("", DEJAVU, Direction::Ltr).expect("shape must succeed");
        assert!(run.glyphs.is_empty());
        assert_eq!(run.total_x_advance, 0);
    }

    #[test]
    fn shape_garbage_face_returns_none() {
        assert!(shape("abc", b"not a font", Direction::Ltr).is_none());
    }

    #[test]
    fn shape_rtl_arabic() {
        // DejaVuSans covers basic Arabic. Even if the host doesn't have
        // a perfect Arabic font we just verify shaping produces glyphs
        // in visual order with non-zero advances.
        let run = shape("مرحبا", DEJAVU, Direction::Rtl).expect("shape must succeed");
        // Arabic shaping output may have different glyph count from
        // input character count due to ligation. Just verify we got
        // *some* glyphs.
        assert!(!run.glyphs.is_empty());
        assert!(run.total_x_advance > 0);
    }

    #[test]
    fn total_advance_equals_sum_of_glyph_advances() {
        let run = shape("Hello", DEJAVU, Direction::Ltr).expect("shape must succeed");
        let summed: i32 = run.glyphs.iter().map(|g| g.x_advance).sum();
        assert_eq!(summed, run.total_x_advance);
    }
}
