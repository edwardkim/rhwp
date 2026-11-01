//! The glyph decoder shared by the extraction and rendering text paths.
//! (`TjBuffer`'s simple-font fast path and a few extraction loops still
//! decode inline.)
//!
//! `decode_text_to_unicode` and its helpers (`fallback_char_to_unicode`,
//! `get_byte_mode`, `TextCharIter`) used to exist twice — an extraction copy
//! in `extractors/text.rs` and a rendering copy in
//! `rendering/text_rasterizer.rs` — and the copies drifted in both
//! directions: UTF-8 codespace CMaps and `preserve_unmapped_glyphs` existed
//! only in extraction; ligature decomposition and dropped-glyph accounting
//! only in rendering. A Type0 font with a UTF-8 CMap extracted right and
//! rendered garbage. (Extraction ligatures were never lossy — they decompose
//! downstream in `ligature_processor`.) This module is the union; the
//! genuine policy differences are expressed in [`DecodePolicy`], not by
//! forking the decoder.

use crate::fonts::FontInfo;

/// Per-surface decoding policy — the only intended differences between the
/// extraction and rendering paths.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct DecodePolicy {
    /// Keep U+FFFD for unmapped codes instead of dropping them.
    /// Extraction honours the `preserve_unmapped_glyphs()` toggle; rendering
    /// always drops (a U+FFFD has no glyph to paint).
    pub preserve_unmapped: bool,
    /// Expand presentation-form ligature code points (fi, fl, ffi, …) into
    /// component letters. The rasterizer needs this so the shaper doesn't
    /// drop the cluster; extraction decomposes downstream in
    /// `ligature_processor`, so it must stay off here or chars would change.
    pub decompose_ligatures: bool,
    /// Emit '?' for codes that are no valid Unicode scalar value
    /// (surrogate-range CIDs, multi-byte codes past U+10FFFF) instead of
    /// dropping them. Extraction has always printed '?' here and its output
    /// bytes are pinned by corpus diffs; rendering keeps this off — there is
    /// no glyph for such a code, so it is dropped and tallied like any other
    /// unmappable code.
    pub question_mark_for_invalid: bool,
}

/// Fallback function to map common character codes to Unicode when ToUnicode CMap fails.
///
/// PDF Spec Compliance: ISO 32000-1:2008 Section 9.10.2
/// This function implements Priority 6 (enhanced fallback) after the standard 5-tier
/// encoding system (ToUnicode CMap, predefined encodings, Adobe Glyph List, etc.) fails.
///
/// Multi-tier fallback strategy:
/// 1. Common punctuation and symbols (em dash, en dash, quotes, bullets)
/// 2. Mathematical operators (∂, ∇, ∑, ∏, ∫, √, ∞, ≤, ≥, ≠)
/// 3. Greek letters (α, β, γ, δ, θ, λ, μ, π, σ, ω - both cases)
/// 4. Currency symbols (€, £, ¥, ¢)
/// 5. Direct Unicode (if char_code is in valid Unicode range)
/// 6. Private Use Area visual description (U+E000-U+F8FF)
/// 7. Replacement character "?" as last resort
///
/// # Arguments
/// * `char_code` - 16-bit character code that failed to decode via standard system
///
/// # Returns
/// Best-effort Unicode string representation, or "?" if no mapping possible
pub(crate) fn fallback_char_to_unicode(char_code: u32) -> String {
    match char_code {
        // ==================================================================================
        // PRIORITY 1: Common Punctuation (most frequently failing)
        // ==================================================================================
        0x2014 => "—".to_string(),        // Em dash
        0x2013 => "–".to_string(),        // En dash
        0x2018 => "\u{2018}".to_string(), // Left single quotation mark (')
        0x2019 => "\u{2019}".to_string(), // Right single quotation mark (')
        0x201C => "\u{201C}".to_string(), // Left double quotation mark (")
        0x201D => "\u{201D}".to_string(), // Right double quotation mark (")
        0x2022 => "•".to_string(),        // Bullet
        0x2026 => "…".to_string(),        // Horizontal ellipsis
        0x00B0 => "°".to_string(),        // Degree sign

        // ==================================================================================
        // PRIORITY 2: Mathematical Operators (common in academic papers)
        // ==================================================================================
        0x00B1 => "±".to_string(), // Plus-minus sign
        0x00D7 => "×".to_string(), // Multiplication sign
        0x00F7 => "÷".to_string(), // Division sign
        0x2202 => "∂".to_string(), // Partial differential
        0x2207 => "∇".to_string(), // Nabla (del operator)
        0x220F => "∏".to_string(), // N-ary product
        0x2211 => "∑".to_string(), // N-ary summation
        0x221A => "√".to_string(), // Square root
        0x221E => "∞".to_string(), // Infinity
        0x2260 => "≠".to_string(), // Not equal to
        0x2261 => "≡".to_string(), // Identical to
        0x2264 => "≤".to_string(), // Less-than or equal to
        0x2265 => "≥".to_string(), // Greater-than or equal to
        0x222B => "∫".to_string(), // Integral
        0x2248 => "≈".to_string(), // Almost equal to
        0x2282 => "⊂".to_string(), // Subset of
        0x2283 => "⊃".to_string(), // Superset of
        0x2286 => "⊆".to_string(), // Subset of or equal to
        0x2287 => "⊇".to_string(), // Superset of or equal to
        0x2208 => "∈".to_string(), // Element of
        0x2209 => "∉".to_string(), // Not an element of
        0x2200 => "∀".to_string(), // For all
        0x2203 => "∃".to_string(), // There exists
        0x2205 => "∅".to_string(), // Empty set
        0x2227 => "∧".to_string(), // Logical and
        0x2228 => "∨".to_string(), // Logical or
        0x00AC => "¬".to_string(), // Not sign
        0x2192 => "→".to_string(), // Rightwards arrow
        0x2190 => "←".to_string(), // Leftwards arrow
        0x2194 => "↔".to_string(), // Left right arrow
        0x21D2 => "⇒".to_string(), // Rightwards double arrow
        0x21D4 => "⇔".to_string(), // Left right double arrow

        // ==================================================================================
        // PRIORITY 3: Greek Letters (common in scientific/mathematical texts)
        // ==================================================================================
        // Lowercase Greek
        0x03B1 => "α".to_string(), // Alpha
        0x03B2 => "β".to_string(), // Beta
        0x03B3 => "γ".to_string(), // Gamma
        0x03B4 => "δ".to_string(), // Delta
        0x03B5 => "ε".to_string(), // Epsilon
        0x03B6 => "ζ".to_string(), // Zeta
        0x03B7 => "η".to_string(), // Eta
        0x03B8 => "θ".to_string(), // Theta
        0x03B9 => "ι".to_string(), // Iota
        0x03BA => "κ".to_string(), // Kappa
        0x03BB => "λ".to_string(), // Lambda
        0x03BC => "μ".to_string(), // Mu
        0x03BD => "ν".to_string(), // Nu
        0x03BE => "ξ".to_string(), // Xi
        0x03BF => "ο".to_string(), // Omicron
        0x03C0 => "π".to_string(), // Pi
        0x03C1 => "ρ".to_string(), // Rho
        0x03C2 => "ς".to_string(), // Final sigma
        0x03C3 => "σ".to_string(), // Sigma
        0x03C4 => "τ".to_string(), // Tau
        0x03C5 => "υ".to_string(), // Upsilon
        0x03C6 => "φ".to_string(), // Phi
        0x03C7 => "χ".to_string(), // Chi
        0x03C8 => "ψ".to_string(), // Psi
        0x03C9 => "ω".to_string(), // Omega

        // Uppercase Greek
        0x0391 => "Α".to_string(), // Alpha
        0x0392 => "Β".to_string(), // Beta
        0x0393 => "Γ".to_string(), // Gamma
        0x0394 => "Δ".to_string(), // Delta
        0x0395 => "Ε".to_string(), // Epsilon
        0x0396 => "Ζ".to_string(), // Zeta
        0x0397 => "Η".to_string(), // Eta
        0x0398 => "Θ".to_string(), // Theta
        0x0399 => "Ι".to_string(), // Iota
        0x039A => "Κ".to_string(), // Kappa
        0x039B => "Λ".to_string(), // Lambda
        0x039C => "Μ".to_string(), // Mu
        0x039D => "Ν".to_string(), // Nu
        0x039E => "Ξ".to_string(), // Xi
        0x039F => "Ο".to_string(), // Omicron
        0x03A0 => "Π".to_string(), // Pi
        0x03A1 => "Ρ".to_string(), // Rho
        0x03A3 => "Σ".to_string(), // Sigma
        0x03A4 => "Τ".to_string(), // Tau
        0x03A5 => "Υ".to_string(), // Upsilon
        0x03A6 => "Φ".to_string(), // Phi
        0x03A7 => "Χ".to_string(), // Chi
        0x03A8 => "Ψ".to_string(), // Psi
        0x03A9 => "Ω".to_string(), // Omega

        // ==================================================================================
        // PRIORITY 4: Currency Symbols
        // ==================================================================================
        0x20AC => "€".to_string(), // Euro
        0x00A3 => "£".to_string(), // Pound sterling
        0x00A5 => "¥".to_string(), // Yen
        0x00A2 => "¢".to_string(), // Cent
        0x20A3 => "₣".to_string(), // French franc
        0x20A4 => "₤".to_string(), // Lira
        0x20A9 => "₩".to_string(), // Won
        0x20AA => "₪".to_string(), // New shekel
        0x20AB => "₫".to_string(), // Dong
        0x20B9 => "₹".to_string(), // Indian rupee

        // ==================================================================================
        // PRIORITY 5: Direct Unicode (for valid ranges)
        // ==================================================================================
        // Valid Unicode: BMP (0x0000-0xD7FF, 0xE000-0xFFFF) and supplementary planes
        // Excludes surrogate pairs (0xD800-0xDFFF)
        code => {
            if let Some(ch) = char::from_u32(code) {
                if (0xE000..=0xF8FF).contains(&code) {
                    log::debug!("Private Use Area character: U+{:04X}", code);
                }
                ch.to_string()
            } else {
                log::warn!("Character code 0x{:04X} is not a valid Unicode code point", code);
                "?".to_string()
            }
        },
    }
}

/// Byte grouping mode for CID font character code decoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ByteMode {
    /// Single-byte codes (simple fonts, some predefined CMaps)
    OneByte,
    /// Always 2-byte codes (Identity-H/V, UCS2)
    TwoByte,
    /// Shift-JIS variable-width (1 or 2 bytes depending on lead byte)
    ShiftJIS,
}

/// True when a Type0 font's `/Encoding` is a UTF-8 (variable-width) CMap —
/// `Uni-Utf8-H` (embedded, pdf.js issue18117) or the Adobe predefined
/// `UniGB-UTF8-H` / `UniCNS-UTF8-H` / `UniJIS-UTF8-H` / `UniKS-UTF8-H` family.
/// Such codes are 1–4 bytes and must be segmented by UTF-8 lead-byte rules
/// (see `decode_text_to_unicode`), not the fixed 1/2-byte `ByteMode`. Matching
/// on the CMap name keeps the behaviour isolated to these fonts.
pub(crate) fn font_has_utf8_cmap(font: &FontInfo) -> bool {
    if font.subtype != "Type0" {
        return false;
    }
    if let crate::fonts::Encoding::Standard(name) = &font.encoding {
        let lower = name.to_ascii_lowercase();
        lower.contains("utf8") || lower.contains("utf-8")
    } else {
        false
    }
}

/// Get byte grouping mode for a font (v0.3.14).
pub(crate) fn get_byte_mode(font: Option<&FontInfo>) -> ByteMode {
    if let Some(font) = font {
        if font.subtype == "Type0" {
            // If the ToUnicode CMap declares a 2-byte codespace range, always use
            // TwoByte mode regardless of the encoding name. This handles CJK fonts
            // whose /Encoding name is a custom CMap stream that doesn't match the
            // well-known keyword patterns below (e.g. "H", "V", "UniCNS-H", …).
            // See PDF Spec §9.7.5 — `begincodespacerange` is authoritative.
            if let Some(ref lazy_cmap) = font.to_unicode {
                if lazy_cmap.code_width() == 2 {
                    return ByteMode::TwoByte;
                }
            }

            match &font.encoding {
                crate::fonts::Encoding::Identity => ByteMode::TwoByte,
                crate::fonts::Encoding::Standard(name) => {
                    if (name.contains("Identity") && !name.contains("OneByteIdentity"))
                        || name.contains("UCS2")
                        || name.contains("UTF16")
                        // CORPUS-3: bare Adobe predefined horizontal/vertical CMaps
                        // ("H"/"V", e.g. Adobe-Japan1-H) are 2-byte by definition;
                        // without this they were read single-byte → CJK garbage
                        // ("あいうえお" → "CACCCECGCI" on noembed-jis7).
                        || name == "H"
                        || name == "V"
                    {
                        ByteMode::TwoByte
                    } else if name.contains("RKSJ") {
                        ByteMode::ShiftJIS
                    } else if name.contains("EUC")
                        || name.contains("GBK")
                        || name.contains("GBpc")
                        || name.contains("GB-")
                        || name.contains("CNS")
                        || name.contains("B5")
                        || name.contains("KSC")
                        || name.contains("KSCms")
                    {
                        ByteMode::TwoByte
                    } else {
                        ByteMode::OneByte
                    }
                },
                _ => ByteMode::OneByte,
            }
        } else {
            ByteMode::OneByte
        }
    } else {
        ByteMode::OneByte
    }
}

/// Iterator over characters in a PDF string based on font encoding (v0.3.14).
pub(crate) struct TextCharIter<'a> {
    bytes: &'a [u8],
    byte_mode: ByteMode,
    index: usize,
}

impl<'a> TextCharIter<'a> {
    pub(crate) fn new(bytes: &'a [u8], font: Option<&FontInfo>) -> Self {
        Self {
            bytes,
            byte_mode: get_byte_mode(font),
            index: 0,
        }
    }
}

impl<'a> Iterator for TextCharIter<'a> {
    type Item = (u16, usize); // (char_code, bytes_consumed)

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.bytes.len() {
            return None;
        }

        let (char_code, bytes_consumed) = match self.byte_mode {
            ByteMode::TwoByte if self.index + 1 < self.bytes.len() => {
                (((self.bytes[self.index] as u16) << 8) | (self.bytes[self.index + 1] as u16), 2)
            },
            ByteMode::ShiftJIS => {
                let b = self.bytes[self.index];
                let is_lead = (0x81..=0x9F).contains(&b) || (0xE0..=0xFC).contains(&b);
                if is_lead && self.index + 1 < self.bytes.len() {
                    (((b as u16) << 8) | (self.bytes[self.index + 1] as u16), 2)
                } else {
                    (b as u16, 1)
                }
            },
            _ => (self.bytes[self.index] as u16, 1),
        };

        self.index += bytes_consumed;
        Some((char_code, bytes_consumed))
    }
}

/// Segment `bytes` into UTF-8-CMap character codes (1–4 bytes each, by
/// lead-byte width; invalid lead bytes consume one byte so the scan can't
/// stall). Both `decode_text_to_unicode` and `char_codes` read this same
/// segmentation, so decoded text and per-code lookups stay aligned.
fn utf8_codes(bytes: &[u8]) -> impl Iterator<Item = u32> + '_ {
    let mut i = 0;
    std::iter::from_fn(move || {
        if i >= bytes.len() {
            return None;
        }
        let width = match bytes[i] {
            0x00..=0x7F => 1,
            0xC0..=0xDF => 2,
            0xE0..=0xEF => 3,
            0xF0..=0xF7 => 4,
            _ => 1,
        }
        .min(bytes.len() - i);
        let mut code: u32 = 0;
        for &b in &bytes[i..i + width] {
            code = (code << 8) | b as u32;
        }
        i += width;
        Some(code)
    })
}

/// `char_to_unicode` then the shared fallback, except that a code outside
/// the Unicode scalar range only becomes '?' under
/// `DecodePolicy::question_mark_for_invalid` — otherwise it is returned as
/// U+FFFD so the caller's drop/tally branch sees it.
/// [`char_codes`], paired with each code's byte width in `bytes`. Callers
/// that need the decode's own segmentation *and* a per-code Tw-eligibility
/// gate (ISO 32000-1 §9.3.3: word spacing applies only to the single-byte
/// code 32, never a byte value 32 inside a multi-byte code) should use this
/// rather than `char_codes` — width is exactly the piece plain `char_codes`
/// throws away.
#[cfg_attr(not(feature = "rendering"), allow(dead_code))]
pub(crate) fn char_codes_with_widths(bytes: &[u8], font: &FontInfo) -> Vec<(u32, usize)> {
    if font_has_utf8_cmap(font) {
        let mut i = 0;
        utf8_codes(bytes)
            .map(|code| {
                let width = match bytes[i] {
                    0x00..=0x7F => 1,
                    0xC0..=0xDF => 2,
                    0xE0..=0xEF => 3,
                    0xF0..=0xF7 => 4,
                    _ => 1,
                }
                .min(bytes.len() - i);
                i += width;
                (code, width)
            })
            .collect()
    } else {
        TextCharIter::new(bytes, Some(font))
            .map(|(code, width)| (code as u32, width))
            .collect()
    }
}

fn resolve_char(font: &FontInfo, code: u32, policy: DecodePolicy) -> String {
    if let Some(char_str) = font.char_to_unicode(code) {
        return char_str;
    }
    if char::from_u32(code).is_none() && !policy.question_mark_for_invalid {
        return "\u{FFFD}".to_string();
    }
    fallback_char_to_unicode(code)
}

pub(crate) fn decode_text_to_unicode(
    bytes: &[u8],
    font: Option<&FontInfo>,
    policy: DecodePolicy,
) -> String {
    let raw_result = if let Some(font) = font {
        let mut result = String::new();
        // Use pre-computed lookup table for performance if it's a simple font
        if font.subtype != "Type0" {
            let table = font.get_byte_to_char_table();
            for &byte in bytes {
                let c = table[byte as usize];
                if c != '\0' {
                    result.push(c);
                } else {
                    // Fallback: multi-char mapping or unmapped byte
                    let char_str = resolve_char(font, byte as u32, policy);
                    if char_str != "\u{FFFD}" || policy.preserve_unmapped {
                        result.push_str(&char_str);
                    }
                }
            }
        } else if font_has_utf8_cmap(font) {
            // Type0 font whose /Encoding is an embedded CMap with a UTF-8
            // (variable-width) codespace — e.g. `Uni-Utf8-H` (pdf.js
            // issue18117) and the Adobe predefined `Uni*-UTF8-H` family.
            // Codes are 1–4 bytes segmented by UTF-8 lead-byte rules, which
            // exceed the u16 of `TextCharIter`. Segment into u32 codes and
            // resolve via the (present) ToUnicode CMap, which is keyed by
            // the same multi-byte codes. Isolated to UTF-8-CMap fonts: every
            // other font keeps the path below unchanged.
            for code in utf8_codes(bytes) {
                let char_str = resolve_char(font, code, policy);
                if char_str != "\u{FFFD}" || policy.preserve_unmapped {
                    result.push_str(&char_str);
                }
            }
        } else {
            // Complex font: use unified iterator for robust multi-byte decoding
            for (char_code, _) in TextCharIter::new(bytes, Some(font)) {
                let char_str = resolve_char(font, char_code as u32, policy);

                if char_str != "\u{FFFD}" || policy.preserve_unmapped {
                    result.push_str(&char_str);
                }
            }
        }
        result
    } else {
        // No font - fallback to Latin-1 (ISO 8859-1) encoding
        // Per PDF Spec ISO 32000-1:2008, Section 9.6.6, Latin-1 maps bytes 0x00-0xFF
        // directly to Unicode code points U+0000-U+00FF
        log::warn!(
            "⚠️  No font provided for {} bytes, using Latin-1 fallback (PDF spec compliant)",
            bytes.len()
        );
        bytes.iter().map(|&b| char::from(b)).collect()
    };

    // Filter control characters from failed encoding resolution
    // Keep: \t (0x09), \n (0x0A), \r (0x0D), and all printable chars (>= 0x20)
    let mut filtered = String::with_capacity(raw_result.len());
    for c in raw_result.chars() {
        if c < '\x20' && c != '\t' && c != '\n' && c != '\r' {
            continue;
        }
        if policy.decompose_ligatures {
            if let Some(components) = crate::text::ligature_processor::get_ligature_components(c) {
                filtered.push_str(components);
                continue;
            }
        }
        filtered.push(c);
    }
    filtered
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fonts::{CIDToGIDMap, Encoding, FontInfo, VerticalMetrics};
    use std::collections::HashMap;

    fn utf8_cmap_font() -> FontInfo {
        FontInfo {
            base_font: "TestUtf8CMap".to_string(),
            subtype: "Type0".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("UniFull-UTF8-H".to_string()),
            to_unicode: None,
            font_weight: None,
            flags: None,
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: Some(CIDToGIDMap::Identity),
            cid_system_info: None,
            cid_font_type: Some("CIDFontType2".to_string()),
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        }
    }

    /// UTF-8-CMap codes are lead-byte-width segmented; an invalid lead byte
    /// consumes exactly one byte, and a truncated tail clamps rather than
    /// stalling the scan.
    #[test]
    fn utf8_codes_segments_by_lead_byte_width() {
        let bytes = [
            0x41, // 1-byte
            0xC3, 0xA9, // 2-byte
            0xE4, 0xB8, 0xAD, // 3-byte
            0xF0, 0x9F, 0x98, 0x80, // 4-byte
            0x80, // invalid lead: one byte
            0xE4, 0xB8, // truncated 3-byte tail: clamps to the 2 bytes left
        ];
        let codes: Vec<u32> = utf8_codes(&bytes).collect();
        assert_eq!(codes, vec![0x41, 0xC3A9, 0xE4B8AD, 0xF09F_9880, 0x80, 0xE4B8]);
    }

    /// The rasterizer's parallel CID/width arrays must be built from the same
    /// segmentation the decode uses: for a UTF-8 codespace CMap that is
    /// variable-width, not `TextCharIter`'s fixed grouping.
    #[test]
    fn char_codes_with_widths_uses_decode_segmentation_for_utf8_cmaps() {
        let font = utf8_cmap_font();
        assert!(font_has_utf8_cmap(&font), "fixture font must select the UTF-8 route");
        let bytes = [0x41, 0xC3, 0xA9, 0xE4, 0xB8, 0xAD];
        assert_eq!(
            char_codes_with_widths(&bytes, &font),
            vec![(0x41, 1), (0xC3A9, 2), (0xE4B8AD, 3)]
        );
    }
}
