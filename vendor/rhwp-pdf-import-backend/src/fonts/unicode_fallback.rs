//! Unicode-capable system-font fallback for office round-trip.
//!
//! Base-14 PDF fonts cover Latin-1 only. Source PDFs containing
//! Hebrew, Arabic, Devanagari, CJK, or even most Latin Extended
//! characters embed their own fonts that the writer typically can't
//! re-embed (CID-only subsets, Type 1 programs, etc.). Without a
//! Unicode-capable fallback the renderer emits `.notdef` for every
//! such glyph, which surfaces as `?` or missing-glyph boxes on the
//! round-trip PDF.
//!
//! This module locates a system font that covers a broad Unicode
//! range — DejaVu Sans on Linux, falling back to FreeSans or Noto
//! Sans — loads its raw bytes, and exposes them to the office
//! writer so they can be registered alongside the source's embedded
//! fonts. The caller decides per-span whether to route to the
//! fallback via `needs_unicode_fallback`.
//!
//! The font is loaded at most once per process (cached via
//! `OnceLock`). The bytes are cloned on each retrieval — a few
//! hundred KB, ~once per round-trip, not a hot path.

use std::sync::OnceLock;

/// Resource-name we register the Unicode fallback under. Stable
/// across docx / pptx / xlsx so the back-to-PDF code path can find
/// the font by name regardless of source format.
pub const UNICODE_FALLBACK_NAME: &str = "Pdfox-UnicodeFallback";

/// Resource-name for the CJK-capable fallback. Distinct from the
/// general fallback because the CJK font program is much larger
/// (4-19 MB) — we only register it when CJK text is actually
/// present in the document, to keep small-doc round-trip output
/// slim.
pub const UNICODE_FALLBACK_CJK_NAME: &str = "Pdfox-UnicodeFallback-CJK";

static CACHED_BYTES: OnceLock<Option<Vec<u8>>> = OnceLock::new();
static CACHED_CJK_BYTES: OnceLock<Option<Vec<u8>>> = OnceLock::new();

/// Load (and cache) a system Unicode-capable font. Returns the raw
/// TTF bytes the office writer can hand to
/// `register_embedded_font` / `embed_font`.
///
/// First match wins from a fixed candidate list — DejaVu Sans (very
/// broad coverage, ships with most Linux distros), then GNU FreeSans
/// (BSD-compatible), then Chrome OS / Noto Sans, then Tinos /
/// Arimo. On systems with none of these the helper returns `None`
/// and the round-trip silently degrades to the existing
/// `?`-glyph behaviour rather than panicking.
pub fn load_unicode_fallback_bytes() -> Option<Vec<u8>> {
    CACHED_BYTES
        .get_or_init(|| {
            const CANDIDATES: &[&str] = &[
                // Broad Unicode coverage (Latin Extended, Greek,
                // Cyrillic, Hebrew, Arabic, Vietnamese, etc.).
                "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
                "/usr/share/fonts/dejavu-sans-fonts/DejaVuSans.ttf",
                "/usr/share/fonts/TTF/DejaVuSans.ttf",
                "/Library/Fonts/DejaVuSans.ttf",
                // Backup: GNU FreeFont — covers most modern scripts.
                "/usr/share/fonts/truetype/freefont/FreeSans.ttf",
                "/usr/share/fonts/gnu-free/FreeSans.ttf",
                // Chrome OS / Linux Noto package.
                "/usr/share/fonts/chromeos/noto/NotoSans-Regular.ttf",
                "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
                // Last-resort: croscore (Liberation-equivalent) —
                // Latin Extended + Greek + Cyrillic but not Hebrew /
                // Arabic. Better than `?` for Latin Extended cases.
                "/usr/share/fonts/chromeos/croscore/Arimo-Regular.ttf",
                "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            ];
            for path in CANDIDATES {
                if let Ok(bytes) = std::fs::read(path) {
                    if !bytes.is_empty() {
                        return Some(bytes);
                    }
                }
            }
            None
        })
        .clone()
}

/// Load (and cache) a system CJK-capable font. Used as a secondary
/// fallback when text contains Han / Hiragana / Katakana / Hangul
/// characters that the general Unicode fallback (DejaVu Sans /
/// FreeSans) doesn't cover.
///
/// Prefers a standalone TrueType file over a TTC since the office
/// writer's font pipeline doesn't yet handle TrueType Collections.
/// Returns `None` when no CJK font is found on the system — the
/// renderer then falls back to `.notdef` and CJK glyphs render
/// as the missing-glyph box (same behaviour as before this helper
/// existed).
pub fn load_cjk_fallback_bytes() -> Option<Vec<u8>> {
    CACHED_CJK_BYTES
        .get_or_init(|| {
            const CANDIDATES: &[&str] = &[
                // Broadest CJK coverage in a standalone TTF: Droid
                // Sans Fallback Full covers Simplified Chinese,
                // Traditional Chinese, Japanese (kana + most
                // kanji), and basic Hangul. 4 MB.
                "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
                "/usr/share/fonts/android/DroidSansFallbackFull.ttf",
                // IPA Gothic (Japanese only, but a complete kanji
                // set). Covers issue-71 Chinese partially since
                // many Han codepoints overlap.
                "/usr/share/fonts/opentype/ipafont-gothic/ipag.ttf",
                // Chrome OS Nanum Gothic (Korean Hangul + ASCII).
                "/usr/share/fonts/chromeos/ko-nanum/NanumGothic.ttf",
                // Last resort: Unifont (covers ALL of Unicode but
                // glyphs are bitmap-style 16×16 hex outlines).
                "/usr/share/fonts/opentype/unifont/unifont.otf",
            ];
            for path in CANDIDATES {
                if let Ok(bytes) = std::fs::read(path) {
                    if !bytes.is_empty() {
                        return Some(bytes);
                    }
                }
            }
            None
        })
        .clone()
}

/// Returns `true` when the text contains at least one character in
/// the CJK Unicode ranges. CJK script needs a different fallback
/// font from Latin / Hebrew / Arabic because the general Unicode
/// fallback (DejaVu Sans) has zero Han / Hiragana / Hangul
/// coverage — routing CJK text to it would still emit `.notdef`.
///
/// Ranges covered (PDF/spec terminology):
/// - U+3000..U+303F  CJK Symbols and Punctuation
/// - U+3040..U+30FF  Hiragana + Katakana
/// - U+31F0..U+31FF  Katakana Phonetic Extensions
/// - U+3400..U+4DBF  CJK Unified Ideographs Extension A
/// - U+4E00..U+9FFF  CJK Unified Ideographs
/// - U+A000..U+A4CF  Yi Syllables + Radicals (treated as CJK-region)
/// - U+AC00..U+D7AF  Hangul Syllables
/// - U+F900..U+FAFF  CJK Compatibility Ideographs
/// - U+FE30..U+FE4F  CJK Compatibility Forms
/// - U+FF00..U+FFEF  Halfwidth and Fullwidth Forms
pub fn needs_cjk_fallback(text: &str) -> bool {
    text.chars().any(|c| {
        let code = c as u32;
        matches!(
            code,
            0x3000..=0x303F
            | 0x3040..=0x30FF
            | 0x31F0..=0x31FF
            | 0x3400..=0x4DBF
            | 0x4E00..=0x9FFF
            | 0xA000..=0xA4CF
            | 0xAC00..=0xD7AF
            | 0xF900..=0xFAFF
            | 0xFE30..=0xFE4F
            | 0xFF00..=0xFFEF
        )
    })
}

/// Returns `true` when the supplied text contains at least one
/// character that base-14 PDF fonts (Helvetica / Times / Courier
/// + Symbol + ZapfDingbats) cannot render via WinAnsi encoding.
///
/// Base-14 fonts cover Latin-1 (U+0000..U+00FF) **plus** a handful
/// of typographic Unicode codepoints commonly mapped in WinAnsi:
/// curly quotes, em / en dash, bullet, ellipsis, trademark, etc.
/// Routing those to a Unicode-capable face would needlessly switch
/// font family on regular Western text (curly quotes appear in
/// almost every form / policy document), which produces a visible
/// regression even when the source text is otherwise pure Latin.
///
/// Empty strings, ASCII-only strings, and strings whose only
/// non-Latin-1 codepoints are in the WinAnsi-extra set return
/// `false`. Hebrew, Arabic, CJK, Devanagari, Greek, Cyrillic, and
/// Latin Extended-A/B return `true` and route to the Unicode
/// fallback.
pub fn needs_unicode_fallback(text: &str) -> bool {
    text.chars().any(|c| {
        let code = c as u32;
        if code <= 0x00FF {
            return false;
        }
        // WinAnsi-encoded typographic extras: these are the common
        // Microsoft Word / typographically-aware codepoints the
        // base-14 fonts can resolve via their WinAnsi mapping.
        // Conservative list — only the symbols that show up in
        // ordinary Western text. Anything else routes to the
        // Unicode fallback.
        !matches!(
            code,
            0x0152 | 0x0153  // OE / oe ligature
            | 0x0160 | 0x0161  // S / s caron
            | 0x0178           // Y diaeresis
            | 0x017D | 0x017E  // Z / z caron
            | 0x0192           // f hook
            | 0x02C6 | 0x02DC  // circumflex / small tilde
            | 0x2013 | 0x2014  // en / em dash
            | 0x2018 | 0x2019  // left / right single quote
            | 0x201A           // single low-9 quote
            | 0x201C | 0x201D  // left / right double quote
            | 0x201E           // double low-9 quote
            | 0x2020 | 0x2021  // dagger / double dagger
            | 0x2022           // bullet
            | 0x2026           // ellipsis
            | 0x2030           // per mille
            | 0x2039 | 0x203A  // single guillemets
            | 0x20AC           // euro
            | 0x2122 // trademark
        )
    })
}
