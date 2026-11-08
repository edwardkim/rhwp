//! Bundled fallback fonts.
//!
//! These TTFs ship inside the pdf_oxide binary so that markdown / DOCX /
//! PPTX → PDF rendering paths can fall back to a Unicode-capable face when
//! the requested text contains characters absent from the standard 14 PDF
//! fonts (Greek, Latin Extended, Cyrillic, etc.). The fonts are subset on
//! emit, so the cost in the output PDF is just the glyphs actually used.
//!
//! License: DejaVu Sans is distributed under a BSD-style license that
//! permits redistribution; see `src/fonts/assets/LICENSE-DejaVu`.
//!
//! Binary size impact when this module is included:
//!   DejaVu Sans Regular  → 760 KB
//!   DejaVu Sans Bold     → 709 KB

/// DejaVu Sans Regular — covers Latin (incl. Extended-A/B), Greek, Cyrillic,
/// Vietnamese, IPA, common math symbols, and many punctuation/typographic
/// characters that the standard 14 PDF fonts (WinAnsiEncoding) lack.
pub const DEJAVU_SANS: &[u8] = include_bytes!("assets/DejaVuSans.ttf");

/// DejaVu Sans Bold — same Unicode coverage as `DEJAVU_SANS`, bold weight.
pub const DEJAVU_SANS_BOLD: &[u8] = include_bytes!("assets/DejaVuSans-Bold.ttf");

/// Quick check: does the string contain any character that the standard
/// 14 PDF fonts (WinAnsiEncoding) cannot render? If yes, the rendering
/// path should embed [`DEJAVU_SANS`] for those runs.
///
/// We treat math-alphanumeric symbols as renderable because the writer
/// normalizes them to plain Latin/Greek before encoding.
pub fn needs_unicode_font(s: &str) -> bool {
    s.chars().any(|c| {
        let cp = c as u32;
        let normalized = super::encoding::math_alphanumeric_base(cp).unwrap_or(cp);
        super::encoding::unicode_to_winansi(normalized).is_none()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_bytes_load() {
        assert!(DEJAVU_SANS.len() > 100_000, "DejaVu Sans should be ~760KB");
        assert!(DEJAVU_SANS_BOLD.len() > 100_000);
        // TTF files start with the 'true' or 0x00010000 sfnt version.
        assert_eq!(&DEJAVU_SANS[..4], &[0x00, 0x01, 0x00, 0x00]);
    }

    #[test]
    fn ascii_does_not_need_unicode() {
        assert!(!needs_unicode_font("Hello, World!"));
    }

    #[test]
    fn smart_quote_in_winansi() {
        // U+2019 IS in WinAnsi at 0x92 — no fallback needed.
        assert!(!needs_unicode_font("Pearson\u{2019}s"));
    }

    #[test]
    fn greek_needs_unicode() {
        assert!(needs_unicode_font("β")); // U+03B2
        assert!(needs_unicode_font("σ")); // U+03C3
    }

    #[test]
    fn math_italic_does_not_need_unicode() {
        // Math italic chars get normalized to plain Latin/Greek; the Latin
        // ones land in WinAnsi, the Greek ones DO need the Unicode font.
        assert!(!needs_unicode_font("\u{1D465}")); // 𝑥 → x (Latin → WinAnsi)
        assert!(needs_unicode_font("\u{1D6FD}")); // 𝛽 → β (Greek → needs font)
    }
}
