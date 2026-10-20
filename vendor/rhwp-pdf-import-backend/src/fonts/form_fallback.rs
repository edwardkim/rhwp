//! Bundled fallback fonts for baking non-Latin / emoji form-field values when
//! a filled interactive form is flattened (ISO 32000-1 §12.7.3.3).
//!
//! A field's `/DA` font (typically Helvetica/WinAnsi) cannot render CJK or
//! emoji, and the document usually carries no font that can. Flattening bakes
//! pixels, so — like conforming readers and tools such as PyMuPDF — we
//! substitute and embed a covering font. The font data is compiled in only
//! with the `cjk-form-fonts` feature; the text-classification helpers are
//! always available so callers can detect when a fallback would be needed.

/// Which bundled fallback font covers a run of text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Fallback {
    /// CJK (and other non-Latin BMP) via Droid Sans Fallback.
    Cjk,
    /// Emoji via Noto Emoji (monochrome).
    Emoji,
}

/// Classify a character: `None` if a Latin-1 glyph from the field's own `/DA`
/// font suffices, otherwise the bundled fallback that should render it.
pub fn classify(c: char) -> Option<Fallback> {
    let cp = c as u32;
    if cp <= 0x00FF {
        return None; // Latin-1 — representable by the field's /DA font.
    }
    if is_emoji(cp) {
        return Some(Fallback::Emoji);
    }
    Some(Fallback::Cjk)
}

fn is_emoji(cp: u32) -> bool {
    matches!(
        cp,
        // Regional indicators (0x1F1E6..=0x1F1FF) already fall inside the
        // 0x1F000..=0x1FAFF block, so they are not listed separately.
        0x1F000..=0x1FAFF | 0x2600..=0x26FF | 0x2700..=0x27BF
    )
}

/// A maximal run of consecutive characters sharing a fallback class.
pub struct Run {
    /// `None` = render with the field's `/DA` font; otherwise the fallback.
    pub fallback: Option<Fallback>,
    /// The run's text in logical order.
    pub text: String,
}

/// Split `text` into runs by fallback class, preserving order. Variation
/// selectors (U+FE0E/FE0F) and ZWJ attach to the preceding run so an emoji
/// sequence stays on one font.
pub fn split_runs(text: &str) -> Vec<Run> {
    let mut runs: Vec<Run> = Vec::new();
    for c in text.chars() {
        let cp = c as u32;
        let attach = matches!(cp, 0xFE0E | 0xFE0F | 0x200D);
        let class = if attach {
            runs.last().map(|r| r.fallback).unwrap_or(None)
        } else {
            classify(c)
        };
        match runs.last_mut() {
            Some(r) if r.fallback == class => r.text.push(c),
            _ => runs.push(Run {
                fallback: class,
                text: c.to_string(),
            }),
        }
    }
    runs
}

/// Raw TrueType bytes for a bundled fallback font.
///
/// Available to the form-flatten path (`cjk-form-fonts`) and to the page
/// renderer's CJK substitution path (`cjk-render-fallback`); both bundle the
/// same Droid Sans Fallback asset.
#[cfg(any(feature = "cjk-form-fonts", feature = "cjk-render-fallback"))]
pub fn font_bytes(kind: Fallback) -> &'static [u8] {
    match kind {
        Fallback::Cjk => include_bytes!("assets/DroidSansFallbackFull.ttf"),
        Fallback::Emoji => include_bytes!("assets/NotoEmoji-Regular.ttf"),
    }
}

/// Raw TrueType bytes for the bundled CJK fallback font, exposed for the
/// page-render substitution path (ISO 32000-2 §9.7.5.2 — predefined CIDFont
/// glyph supply). Gated on `cjk-render-fallback`; the form-flatten path uses
/// the sibling `font_bytes(Fallback::Cjk)` accessor under `cjk-form-fonts` and
/// the two paths are intentionally independent so enabling one does not
/// activate the other.
#[cfg(feature = "cjk-render-fallback")]
pub fn render_cjk_fallback_bytes() -> &'static [u8] {
    include_bytes!("assets/DroidSansFallbackFull.ttf")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latin_needs_no_fallback() {
        assert_eq!(classify('A'), None);
        assert_eq!(classify('ñ'), None); // U+00F1, within Latin-1
    }

    #[test]
    fn cjk_and_emoji_classified() {
        assert_eq!(classify('や'), Some(Fallback::Cjk));
        assert_eq!(classify('東'), Some(Fallback::Cjk));
        assert_eq!(classify('🍺'), Some(Fallback::Emoji));
    }

    #[test]
    fn runs_split_mixed_cjk_emoji() {
        let runs = split_runs("やまだ🍺A");
        let classes: Vec<_> = runs.iter().map(|r| r.fallback).collect();
        assert_eq!(classes, vec![Some(Fallback::Cjk), Some(Fallback::Emoji), None]);
        assert_eq!(runs[0].text, "やまだ");
        assert_eq!(runs[1].text, "🍺");
        assert_eq!(runs[2].text, "A");
    }
}
