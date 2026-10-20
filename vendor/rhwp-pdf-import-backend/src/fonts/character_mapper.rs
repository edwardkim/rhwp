//! Character-to-Unicode mapping with priority-based fallback chain.
//!
//! Implements ISO 32000-1:2008 Section 9.10.2 Character-to-Unicode Mapping Priorities:
//! 1. ToUnicode CMap (highest priority)
//! 2. Adobe Glyph List (fallback 1)
//! 3. Predefined CMaps (fallback 2) -- CID-to-Unicode for CJK character collections
//! 4. ActualText attribute (fallback 3)
//! 5. Font encoding (lowest priority)
//!
//! This module provides a unified interface for character mapping that respects
//! the spec-defined priority order.

use super::adobe_glyph_list::ADOBE_GLYPH_LIST;
use super::cmap::CMap;
use std::collections::HashMap;

/// Configuration for predefined CMap lookup (Priority 3).
///
/// Per PDF Spec ISO 32000-1:2008 Section 9.7.5.2, predefined CMaps provide
/// CID-to-Unicode mappings for standard Adobe CJK character collections.
///
/// This stores the character collection ordering (from CIDSystemInfo) so the
/// mapper can look up CIDs in the appropriate predefined mapping table.
///
/// # Supported Character Collections
///
/// - `"GB1"` - Adobe-GB1 (Simplified Chinese)
/// - `"Japan1"` - Adobe-Japan1 (Japanese)
/// - `"CNS1"` - Adobe-CNS1 (Traditional Chinese)
/// - `"Korea1"` - Adobe-Korea1 (Korean)
/// - `"Identity"` - Identity mapping (CID == Unicode code point)
#[derive(Clone, Debug)]
pub struct PredefinedCMapConfig {
    /// The character collection ordering from CIDSystemInfo (e.g., "GB1", "Japan1").
    pub ordering: String,
}

/// Character-to-Unicode mapper with priority-based fallback chain.
///
/// Implements the PDF spec's 5-level priority order for character-to-Unicode mapping.
/// This ensures characters are mapped correctly even in PDFs with custom encodings,
/// symbol fonts, or missing ToUnicode CMaps.
///
/// # Example
///
/// ```
/// use pdf_oxide::fonts::character_mapper::CharacterMapper;
///
/// let mapper = CharacterMapper::new();
///
/// // Character 0x41 (ASCII 'A') maps via Adobe Glyph List to "A"
/// assert_eq!(mapper.map_character(0x41), Some("A".to_string()));
///
/// // Character 0x42 (ASCII 'B') maps via Adobe Glyph List to "B"
/// assert_eq!(mapper.map_character(0x42), Some("B".to_string()));
///
/// // Character 0x20 (space) maps via Adobe Glyph List to " "
/// assert_eq!(mapper.map_character(0x20), Some(" ".to_string()));
/// ```
#[derive(Clone)]
pub struct CharacterMapper {
    /// Priority 1: ToUnicode CMap (explicit character code to Unicode mapping)
    tounicode_cmap: Option<CMap>,

    /// Priority 3: Predefined CMap config for CID-to-Unicode lookup
    predefined_cmap: Option<PredefinedCMapConfig>,

    /// Priority 5: Font encoding (character code to glyph name or character)
    font_encoding: Option<HashMap<u32, char>>,
}

impl CharacterMapper {
    /// Create a new character mapper with no mappings set.
    pub fn new() -> Self {
        Self {
            tounicode_cmap: None,
            predefined_cmap: None,
            font_encoding: None,
        }
    }

    /// Set the ToUnicode CMap (Priority 1 - highest).
    ///
    /// The ToUnicode CMap provides explicit character code to Unicode mappings
    /// from the PDF file. This has the highest priority in the mapping chain.
    ///
    /// # Arguments
    /// * `cmap` - The ToUnicode CMap, or None to remove it
    pub fn set_tounicode_cmap(&mut self, cmap: Option<CMap>) {
        self.tounicode_cmap = cmap;
    }

    /// Set the predefined CMap configuration (Priority 3).
    ///
    /// Configures CID-to-Unicode lookup using predefined Adobe character collection
    /// mappings. This is used for CJK fonts without ToUnicode CMaps.
    ///
    /// Per PDF Spec ISO 32000-1:2008 Section 9.7.5.2, predefined CMaps provide
    /// standard CID-to-Unicode mappings for Adobe character collections.
    ///
    /// # Arguments
    /// * `config` - The predefined CMap configuration, or None to remove it
    ///
    /// # Example
    ///
    /// ```
    /// use pdf_oxide::fonts::character_mapper::{CharacterMapper, PredefinedCMapConfig};
    ///
    /// let mut mapper = CharacterMapper::new();
    /// mapper.set_predefined_cmap(Some(PredefinedCMapConfig {
    ///     ordering: "Japan1".to_string(),
    /// }));
    /// ```
    pub fn set_predefined_cmap(&mut self, config: Option<PredefinedCMapConfig>) {
        self.predefined_cmap = config;
    }

    /// Set the font encoding (Priority 5 - lowest).
    ///
    /// Font encoding provides a fallback mapping from character codes to characters.
    /// This is only used if higher-priority mappings are not available.
    ///
    /// # Arguments
    /// * `encoding` - HashMap mapping character codes to characters, or None to remove it
    pub fn set_font_encoding(&mut self, encoding: Option<HashMap<u32, char>>) {
        self.font_encoding = encoding;
    }

    /// Map a character code to a Unicode string using the priority chain.
    ///
    /// Implements the PDF spec's priority order:
    /// 1. ToUnicode CMap - if present and has mapping
    /// 2. Adobe Glyph List - fallback to standard glyph names
    /// 3. Predefined CMaps - CID-to-Unicode for CJK character collections
    /// 4. ActualText - (handled externally in BDC operator processing)
    /// 5. Font encoding - lowest priority
    ///
    /// # Arguments
    /// * `code` - The character code to map (typically 0-255 for simple fonts, up to 0xFFFF for CID)
    ///
    /// # Returns
    /// * `Some(string)` - The mapped Unicode character(s)
    /// * `None` - No mapping found in any priority level
    ///
    /// # Spec Reference
    /// ISO 32000-1:2008, Section 9.10.2 - Character-to-Unicode Mapping Priorities
    pub fn map_character(&self, code: u32) -> Option<String> {
        // Priority 1: ToUnicode CMap
        //
        // Per ISO 32000-1 §9.10.2, when a ToUnicode CMap is attached to a font
        // it is the authoritative character→Unicode mapping. A miss against a
        // present CMap must produce U+FFFD — falling back to AGL or an Identity
        // CID-as-Unicode heuristic produces "ciphertext" on subset Type0 fonts
        // whose CIDs are insertion-ordered (not codepoint-aligned). See #363.
        if let Some(ref cmap) = self.tounicode_cmap {
            return match cmap.get(&code) {
                Some(mapped) => Some(mapped.into_owned()),
                None => Some("\u{FFFD}".to_string()),
            };
        }

        // Priority 2: Adobe Glyph List (standard glyph for code)
        if let Some(glyph_name) = self.code_to_glyph_name(code) {
            if let Some(unicode_str) = self.map_glyph_name_internal(&glyph_name) {
                return Some(unicode_str);
            }
        }

        // Priority 3: Predefined CMaps (CID-to-Unicode for CJK character collections)
        // Per PDF Spec Section 9.7.5.2, use the character collection ordering to
        // look up the Unicode code point for this CID.
        if let Some(ref config) = self.predefined_cmap {
            if let Some(unicode_str) = self.lookup_predefined_cmap(config, code) {
                return Some(unicode_str);
            }
        }

        // Priority 4: ActualText (handled externally in BDC operator / structure tree)

        // Priority 5: Font encoding
        if let Some(ref encoding) = self.font_encoding {
            if let Some(&ch) = encoding.get(&code) {
                return Some(ch.to_string());
            }
        }

        // No mapping found - return U+FFFD replacement character per PDF Spec 9.10.2
        Some("\u{FFFD}".to_string())
    }

    /// Look up a CID in a predefined CMap using the character collection ordering.
    ///
    /// Routes to the appropriate CID-to-Unicode mapping table based on the
    /// character collection ordering from CIDSystemInfo.
    ///
    /// For "Identity" ordering, the CID is treated as a direct Unicode code point
    /// (Identity-H/Identity-V mapping).
    fn lookup_predefined_cmap(&self, config: &PredefinedCMapConfig, code: u32) -> Option<String> {
        // Truncate to u16 for CID lookup (CIDs are 16-bit values)
        let cid = code as u16;

        let unicode_codepoint = match config.ordering.as_str() {
            "GB1" => super::cid_mappings::lookup_adobe_gb1(cid),
            "Japan1" => super::cid_mappings::lookup_adobe_japan1(cid),
            "CNS1" => super::cid_mappings::lookup_adobe_cns1(cid),
            "Korea1" => super::cid_mappings::lookup_adobe_korea1(cid),
            // Adobe-Arabic-1 / Adobe-Persian-1 (Nazanin, Yagut, Mitra, Lotus)
            // without /ToUnicode — §9.10.3 step-3 identity fallback over the
            // Arabic block; without it these decode as Latin-Extended-B garbage.
            "Arabic" | "Persian" => super::cid_mappings::lookup_adobe_arabic(cid),
            "Identity" => {
                // Identity mapping: CID == Unicode code point
                // Valid for BMP range (0x0000-0xFFFF)
                if code <= 0xFFFF {
                    Some(code)
                } else {
                    None
                }
            },
            _ => None,
        };

        unicode_codepoint.and_then(|cp| char::from_u32(cp).map(|ch| ch.to_string()))
    }

    /// Map a glyph name to its Unicode representation.
    ///
    /// Uses the Adobe Glyph List to find the Unicode character(s) for a named glyph.
    /// This is the public interface for glyph name mapping.
    ///
    /// # Arguments
    /// * `glyph_name` - Name of the glyph (e.g., "A", "ampersand", "fi")
    ///
    /// # Returns
    /// * `Some(string)` - The Unicode character(s) for this glyph
    /// * `None` - Glyph name not found
    pub fn map_glyph_name(&self, glyph_name: &str) -> Option<String> {
        self.map_glyph_name_internal(glyph_name)
    }

    /// Internal helper for glyph name mapping.
    fn map_glyph_name_internal(&self, glyph_name: &str) -> Option<String> {
        // Look up in Adobe Glyph List
        ADOBE_GLYPH_LIST.get(glyph_name).map(|&ch| ch.to_string())
    }
}

/// Look up a PostScript glyph name and return its Unicode codepoint string.
///
/// Per [Adobe Glyph List Specification §6](https://github.com/adobe-type-tools/agl-specification),
/// the conversion proceeds in three steps:
///
/// 1. **AGL exact match** — `"bullet"` → U+2022, `"fi"` → U+FB01, `"space"` → U+0020, etc.
/// 2. **`uniXXXX` synthetic pattern** — exactly four ASCII hex digits (any case) map to U+XXXX.
/// 3. **`uXXXXX` synthetic pattern** — four to six ASCII hex digits (any case, BMP + SMP) map
///    to U+XXXXX (0x0000..=0x10FFFF, excluding surrogates).
///
/// Used as the §9.10.2 Priority 3c fallback when:
/// - the PDF's `ToUnicode` CMap doesn't cover a code (Priority 1 miss),
/// - the font's TrueType `cmap` table doesn't cover the glyph (Priority 4 miss),
/// - and we have the embedded font program's `post` table glyph name for the GID.
///
/// This is what pdf.js, MuPDF, and PDFBox 3.x use as the last name-based step
/// before falling back to CID-as-Unicode. Documented in
/// `docs/releases/plans/v0.3.54/research-tounicode-fallback-chain.md`.
pub(crate) fn glyph_name_to_unicode(glyph_name: &str) -> Option<String> {
    // 1) AGL exact match — covers "bullet", "fi", "fl", "ffi", "ffl", "endash", etc.
    if let Some(&ch) = ADOBE_GLYPH_LIST.get(glyph_name) {
        return Some(ch.to_string());
    }

    // Some font producers append `.altN` / `.sc` / `.001` style variants to the
    // canonical glyph name to disambiguate stylistic alternates (small caps,
    // alternative shapes, OpenType feature variants). Strip the suffix and try
    // again — the base codepoint is what we want for extraction.
    if let Some(dot_idx) = glyph_name.find('.') {
        let base = &glyph_name[..dot_idx];
        if !base.is_empty() {
            if let Some(&ch) = ADOBE_GLYPH_LIST.get(base) {
                return Some(ch.to_string());
            }
        }
    }

    // 2) `uniXXXX` synthetic — exactly four hex digits, BMP only.
    if let Some(hex) = glyph_name.strip_prefix("uni") {
        if hex.len() == 4 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
            if let Ok(cp) = u32::from_str_radix(hex, 16) {
                if let Some(c) = char::from_u32(cp) {
                    // Reject surrogates and control chars (unprintable as text).
                    if !c.is_control() {
                        return Some(c.to_string());
                    }
                }
            }
        }
    }

    // 3) `uXXXXX` synthetic — 4 to 6 hex digits, BMP + SMP + SIP.
    if let Some(hex) = glyph_name.strip_prefix('u') {
        if (4..=6).contains(&hex.len()) && hex.chars().all(|c| c.is_ascii_hexdigit()) {
            if let Ok(cp) = u32::from_str_radix(hex, 16) {
                // 0x0000..=0x10FFFF and not in the surrogate range.
                if (cp <= 0x10FFFF) && !(0xD800..=0xDFFF).contains(&cp) {
                    if let Some(c) = char::from_u32(cp) {
                        if !c.is_control() {
                            return Some(c.to_string());
                        }
                    }
                }
            }
        }
    }

    None
}

impl CharacterMapper {
    /// Convert a character code to a glyph name using standard mappings.
    ///
    /// For ASCII range (0x20-0x7E), this maps directly to character names.
    /// For extended ASCII (0x80-0xFF), uses WinAnsiEncoding fallback.
    /// For other ranges, returns None.
    fn code_to_glyph_name(&self, code: u32) -> Option<String> {
        // Try standard ASCII first
        if code <= 0x7E {
            return self.code_to_glyph_name_ascii(code);
        }

        // Try extended ASCII (WinAnsiEncoding) fallback
        self.code_to_glyph_name_extended(code)
    }

    /// Map ASCII range (0x20-0x7E) to standard glyph names.
    fn code_to_glyph_name_ascii(&self, code: u32) -> Option<String> {
        match code {
            // ASCII printable range
            0x20 => Some("space".to_string()),
            0x21 => Some("exclam".to_string()),
            0x22 => Some("quotedbl".to_string()),
            0x23 => Some("numbersign".to_string()),
            0x24 => Some("dollar".to_string()),
            0x25 => Some("percent".to_string()),
            0x26 => Some("ampersand".to_string()),
            0x27 => Some("quoteright".to_string()),
            0x28 => Some("parenleft".to_string()),
            0x29 => Some("parenright".to_string()),
            0x2A => Some("asterisk".to_string()),
            0x2B => Some("plus".to_string()),
            0x2C => Some("comma".to_string()),
            0x2D => Some("hyphen".to_string()),
            0x2E => Some("period".to_string()),
            0x2F => Some("slash".to_string()),

            // Digits 0-9 use glyph names "zero" through "nine"
            0x30 => Some("zero".to_string()),
            0x31 => Some("one".to_string()),
            0x32 => Some("two".to_string()),
            0x33 => Some("three".to_string()),
            0x34 => Some("four".to_string()),
            0x35 => Some("five".to_string()),
            0x36 => Some("six".to_string()),
            0x37 => Some("seven".to_string()),
            0x38 => Some("eight".to_string()),
            0x39 => Some("nine".to_string()),

            0x3A => Some("colon".to_string()),
            0x3B => Some("semicolon".to_string()),
            0x3C => Some("less".to_string()),
            0x3D => Some("equal".to_string()),
            0x3E => Some("greater".to_string()),
            0x3F => Some("question".to_string()),
            0x40 => Some("at".to_string()),

            // Uppercase A-Z
            0x41..=0x5A => {
                let ch = (code - 0x41) as u8 + b'A';
                Some((ch as char).to_string())
            },

            0x5B => Some("bracketleft".to_string()),
            0x5C => Some("backslash".to_string()),
            0x5D => Some("bracketright".to_string()),
            0x5E => Some("asciicircum".to_string()),
            0x5F => Some("underscore".to_string()),
            0x60 => Some("grave".to_string()),

            // Lowercase a-z
            0x61..=0x7A => {
                let ch = (code - 0x61) as u8 + b'a';
                Some((ch as char).to_string())
            },

            0x7B => Some("braceleft".to_string()),
            0x7C => Some("bar".to_string()),
            0x7D => Some("braceright".to_string()),
            0x7E => Some("asciitilde".to_string()),

            _ => None,
        }
    }

    /// Map extended ASCII range (0x80-0xFF) to glyph names using WinAnsiEncoding.
    ///
    /// This implements the WinAnsiEncoding (Windows-1252) character mappings
    /// which are commonly used as a fallback in PDF documents.
    /// Per PDF Spec ISO 32000-1:2008 Section 9.6.6.1.
    pub fn code_to_glyph_name_extended(&self, code: u32) -> Option<String> {
        // WinAnsiEncoding (Windows-1252) mappings for 0x80-0xFF range
        // Maps character codes to standard glyph names per Adobe Glyph List
        match code {
            // 0x80-0x8F: Special WinAnsiEncoding characters
            0x80 => Some("Euro".to_string()), // € (U+20AC)
            0x81 => None,                     // Undefined in WinAnsiEncoding
            0x82 => Some("quotesinglbase".to_string()), // ‚ (U+201A)
            0x83 => Some("florin".to_string()), // ƒ (U+0192)
            0x84 => Some("quotedblbase".to_string()), // „ (U+201E)
            0x85 => Some("ellipsis".to_string()), // … (U+2026)
            0x86 => Some("dagger".to_string()), // † (U+2020)
            0x87 => Some("daggerdbl".to_string()), // ‡ (U+2021)
            0x88 => Some("circumflex".to_string()), // ˆ (U+02C6)
            0x89 => Some("perthousand".to_string()), // ‰ (U+2030)
            0x8A => Some("Scaron".to_string()), // Š (U+0160)
            0x8B => Some("guilsinglleft".to_string()), // ‹ (U+2039)
            0x8C => Some("OEligature".to_string()), // Œ (U+0152)
            0x8D => None,                     // Undefined
            0x8E => Some("Zcaron".to_string()), // Ž (U+017D)
            0x8F => None,                     // Undefined

            // 0x90-0x9F: More WinAnsiEncoding specials
            0x90 => None,                               // Undefined
            0x91 => Some("quoteleft".to_string()),      // ' (U+2018)
            0x92 => Some("quoteright".to_string()),     // ' (U+2019)
            0x93 => Some("quotedblleft".to_string()),   // " (U+201C)
            0x94 => Some("quotedblright".to_string()),  // " (U+201D)
            0x95 => Some("bullet".to_string()),         // • (U+2022)
            0x96 => Some("endash".to_string()),         // – (U+2013) - COMMON: en-dash
            0x97 => Some("emdash".to_string()),         // — (U+2014) - COMMON: em-dash
            0x98 => Some("tilde".to_string()),          // ˜ (U+02DC)
            0x99 => Some("trademark".to_string()),      // ™ (U+2122) - COMMON: trademark
            0x9A => Some("scaron".to_string()),         // š (U+0161)
            0x9B => Some("guilsinglright".to_string()), // › (U+203A)
            0x9C => Some("oeligature".to_string()),     // œ (U+0153)
            0x9D => None,                               // Undefined
            0x9E => Some("zcaron".to_string()),         // ž (U+017E)
            0x9F => Some("ydieresis".to_string()),      // Ÿ (U+0178)

            // 0xA0-0xFF: Latin-1 Supplement (ISO-8859-1 compatible)
            0xA0 => Some("space".to_string()), // Non-breaking space (U+00A0)
            0xA1 => Some("exclamdown".to_string()), // ¡ (U+00A1)
            0xA2 => Some("cent".to_string()),  // ¢ (U+00A2)
            0xA3 => Some("sterling".to_string()), // £ (U+00A3) - COMMON: pound sign
            0xA4 => Some("currency".to_string()), // ¤ (U+00A4)
            0xA5 => Some("yen".to_string()),   // ¥ (U+00A5)
            0xA6 => Some("brokenbar".to_string()), // ¦ (U+00A6)
            0xA7 => Some("section".to_string()), // § (U+00A7)
            0xA8 => Some("dieresis".to_string()), // ¨ (U+00A8)
            0xA9 => Some("copyright".to_string()), // © (U+00A9) - COMMON: copyright
            0xAA => Some("ordfeminine".to_string()), // ª (U+00AA)
            0xAB => Some("guillemotleft".to_string()), // « (U+00AB)
            0xAC => Some("logicalnot".to_string()), // ¬ (U+00AC)
            0xAD => Some("hyphen".to_string()), // Soft hyphen (U+00AD)
            0xAE => Some("registered".to_string()), // ® (U+00AE) - COMMON: registered
            0xAF => Some("macron".to_string()), // ¯ (U+00AF)
            0xB0 => Some("degree".to_string()), // ° (U+00B0) - COMMON: degree
            0xB1 => Some("plusminus".to_string()), // ± (U+00B1)
            0xB2 => Some("twosuperior".to_string()), // ² (U+00B2)
            0xB3 => Some("threesuperior".to_string()), // ³ (U+00B3)
            0xB4 => Some("acute".to_string()), // ´ (U+00B4)
            0xB5 => Some("mu".to_string()),    // µ (U+00B5)
            0xB6 => Some("paragraph".to_string()), // ¶ (U+00B6)
            0xB7 => Some("periodcentered".to_string()), // · (U+00B7)
            0xB8 => Some("cedilla".to_string()), // ¸ (U+00B8)
            0xB9 => Some("onesuperior".to_string()), // ¹ (U+00B9)
            0xBA => Some("ordmasculine".to_string()), // º (U+00BA)
            0xBB => Some("guillemotright".to_string()), // » (U+00BB)
            0xBC => Some("onequarter".to_string()), // ¼ (U+00BC)
            0xBD => Some("onehalf".to_string()), // ½ (U+00BD)
            0xBE => Some("threequarters".to_string()), // ¾ (U+00BE)
            0xBF => Some("questiondown".to_string()), // ¿ (U+00BF)

            // 0xC0-0xFF: Accented uppercase and lowercase letters
            0xC0 => Some("Agrave".to_string()),      // À (U+00C0)
            0xC1 => Some("Aacute".to_string()),      // Á (U+00C1)
            0xC2 => Some("Acircumflex".to_string()), // Â (U+00C2)
            0xC3 => Some("Atilde".to_string()),      // Ã (U+00C3)
            0xC4 => Some("Adieresis".to_string()),   // Ä (U+00C4)
            0xC5 => Some("Aring".to_string()),       // Å (U+00C5)
            0xC6 => Some("AEligature".to_string()),  // Æ (U+00C6)
            0xC7 => Some("Ccedilla".to_string()),    // Ç (U+00C7)
            0xC8 => Some("Egrave".to_string()),      // È (U+00C8)
            0xC9 => Some("Eacute".to_string()),      // É (U+00C9)
            0xCA => Some("Ecircumflex".to_string()), // Ê (U+00CA)
            0xCB => Some("Edieresis".to_string()),   // Ë (U+00CB)
            0xCC => Some("Igrave".to_string()),      // Ì (U+00CC)
            0xCD => Some("Iacute".to_string()),      // Í (U+00CD)
            0xCE => Some("Icircumflex".to_string()), // Î (U+00CE)
            0xCF => Some("Idieresis".to_string()),   // Ï (U+00CF)
            0xD0 => Some("Eth".to_string()),         // Ð (U+00D0)
            0xD1 => Some("Ntilde".to_string()),      // Ñ (U+00D1)
            0xD2 => Some("Ograve".to_string()),      // Ò (U+00D2)
            0xD3 => Some("Oacute".to_string()),      // Ó (U+00D3)
            0xD4 => Some("Ocircumflex".to_string()), // Ô (U+00D4)
            0xD5 => Some("Otilde".to_string()),      // Õ (U+00D5)
            0xD6 => Some("Odieresis".to_string()),   // Ö (U+00D6)
            0xD7 => Some("multiply".to_string()),    // × (U+00D7)
            0xD8 => Some("Oslash".to_string()),      // Ø (U+00D8)
            0xD9 => Some("Ugrave".to_string()),      // Ù (U+00D9)
            0xDA => Some("Uacute".to_string()),      // Ú (U+00DA)
            0xDB => Some("Ucircumflex".to_string()), // Û (U+00DB)
            0xDC => Some("Udieresis".to_string()),   // Ü (U+00DC)
            0xDD => Some("Yacute".to_string()),      // Ý (U+00DD)
            0xDE => Some("Thorn".to_string()),       // Þ (U+00DE)
            0xDF => Some("germandbls".to_string()),  // ß (U+00DF)
            0xE0 => Some("agrave".to_string()),      // à (U+00E0)
            0xE1 => Some("aacute".to_string()),      // á (U+00E1)
            0xE2 => Some("acircumflex".to_string()), // â (U+00E2)
            0xE3 => Some("atilde".to_string()),      // ã (U+00E3)
            0xE4 => Some("adieresis".to_string()),   // ä (U+00E4) - COMMON: German a-umlaut
            0xE5 => Some("aring".to_string()),       // å (U+00E5)
            0xE6 => Some("aeligature".to_string()),  // æ (U+00E6)
            0xE7 => Some("ccedilla".to_string()),    // ç (U+00E7) - COMMON: French c-cedilla
            0xE8 => Some("egrave".to_string()),      // è (U+00E8)
            0xE9 => Some("eacute".to_string()),      // é (U+00E9) - COMMON: French e-acute
            0xEA => Some("ecircumflex".to_string()), // ê (U+00EA)
            0xEB => Some("edieresis".to_string()),   // ë (U+00EB)
            0xEC => Some("igrave".to_string()),      // ì (U+00EC)
            0xED => Some("iacute".to_string()),      // í (U+00ED)
            0xEE => Some("icircumflex".to_string()), // î (U+00EE)
            0xEF => Some("idieresis".to_string()),   // ï (U+00EF)
            0xF0 => Some("eth".to_string()),         // ð (U+00F0)
            0xF1 => Some("ntilde".to_string()),      // ñ (U+00F1)
            0xF2 => Some("ograve".to_string()),      // ò (U+00F2)
            0xF3 => Some("oacute".to_string()),      // ó (U+00F3)
            0xF4 => Some("ocircumflex".to_string()), // ô (U+00F4)
            0xF5 => Some("otilde".to_string()),      // õ (U+00F5)
            0xF6 => Some("odieresis".to_string()),   // ö (U+00F6)
            0xF7 => Some("divide".to_string()),      // ÷ (U+00F7)
            0xF8 => Some("oslash".to_string()),      // ø (U+00F8)
            0xF9 => Some("ugrave".to_string()),      // ù (U+00F9)
            0xFA => Some("uacute".to_string()),      // ú (U+00FA)
            0xFB => Some("ucircumflex".to_string()), // û (U+00FB)
            0xFC => Some("udieresis".to_string()),   // ü (U+00FC) - COMMON: German u-umlaut
            0xFD => Some("yacute".to_string()),      // ý (U+00FD)
            0xFE => Some("thorn".to_string()),       // þ (U+00FE)
            0xFF => Some("ydieresis".to_string()),   // ÿ (U+00FF)

            _ => None,
        }
    }
}

impl Default for CharacterMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod internal_tests {
    use super::*;

    #[test]
    fn test_ascii_glyph_names() {
        let mapper = CharacterMapper::new();

        // Test ASCII character to glyph name conversion
        assert_eq!(mapper.code_to_glyph_name(0x20), Some("space".to_string()));
        assert_eq!(mapper.code_to_glyph_name(0x41), Some("A".to_string()));
        assert_eq!(mapper.code_to_glyph_name(0x61), Some("a".to_string()));
    }

    // ==========================================================================
    // Adobe-Arabic-1 / Adobe-Persian-1 /Ordering routing
    // ==========================================================================

    #[test]
    fn test_arabic_persian_ordering_maps_to_arabic_block() {
        let mapper = CharacterMapper::new();
        let arabic = PredefinedCMapConfig {
            ordering: "Arabic".to_string(),
        };
        let persian = PredefinedCMapConfig {
            ordering: "Persian".to_string(),
        };

        // CID 0x0627 = ا (ARABIC LETTER ALEF), 0x0641 = ف (ARABIC LETTER FEH).
        assert_eq!(
            mapper.lookup_predefined_cmap(&arabic, 0x0627),
            Some("\u{0627}".to_string()),
            "Arabic ordering must map CID 0x0627 → U+0627"
        );
        assert_eq!(
            mapper.lookup_predefined_cmap(&persian, 0x0641),
            Some("\u{0641}".to_string()),
            "Persian ordering must map CID 0x0641 → U+0641"
        );
    }

    #[test]
    fn test_cjk_orderings_unaffected_by_arabic_arm() {
        // Regression: the new arm must not perturb the Identity path.
        let mapper = CharacterMapper::new();
        let identity = PredefinedCMapConfig {
            ordering: "Identity".to_string(),
        };
        assert_eq!(mapper.lookup_predefined_cmap(&identity, 0x0041), Some("A".to_string()));
    }

    // ==========================================================================
    // Adobe Glyph List §6 + synthetic-name patterns (v0.3.54 #535)
    // ==========================================================================
    // Covers Priority 3c of the §9.10.2 fallback chain: PostScript glyph names
    // coming from the embedded font program's `post` (TrueType) or `charset`
    // (CFF) table get mapped to Unicode via AGL exact match, then `uniXXXX`,
    // then `uXXXXX`. Used by the new `embedded_glyph_name` lookup in
    // `font_dict.rs` to recover bullets and `fi`/`fl` ligatures on Identity-H
    // subset fonts without a `CIDToGIDMap`.

    #[test]
    fn glyph_name_to_unicode_agl_exact_match() {
        // Canonical AGL entries — the bullet/ligature cases that motivate #535.
        assert_eq!(glyph_name_to_unicode("bullet"), Some("\u{2022}".to_string()));
        assert_eq!(glyph_name_to_unicode("fi"), Some("\u{FB01}".to_string()));
        assert_eq!(glyph_name_to_unicode("fl"), Some("\u{FB02}".to_string()));
        // Sanity: ASCII names.
        assert_eq!(glyph_name_to_unicode("A"), Some("A".to_string()));
        assert_eq!(glyph_name_to_unicode("space"), Some(" ".to_string()));
    }

    #[test]
    #[allow(non_snake_case)]
    fn glyph_name_to_unicode_uniXXXX_synth() {
        // Per AGL §6, `uni` + 4 hex digits = U+XXXX.
        assert_eq!(glyph_name_to_unicode("uni2022"), Some("\u{2022}".to_string()));
        assert_eq!(glyph_name_to_unicode("uniFB01"), Some("\u{FB01}".to_string()));
        assert_eq!(glyph_name_to_unicode("uni00E9"), Some("é".to_string()));
        // 3 or 5 hex digits → not the uniXXXX pattern, fall through.
        assert_eq!(glyph_name_to_unicode("uni202"), None);
        assert_eq!(glyph_name_to_unicode("uni20221"), None);
        // Non-hex chars → reject.
        assert_eq!(glyph_name_to_unicode("uniGGGG"), None);
    }

    #[test]
    #[allow(non_snake_case)]
    fn glyph_name_to_unicode_uXXXXX_synth() {
        // `u` + 4..6 hex digits = U+XXXXX (BMP + SMP + SIP, no surrogates).
        assert_eq!(glyph_name_to_unicode("u2022"), Some("\u{2022}".to_string()));
        assert_eq!(glyph_name_to_unicode("u1F600"), Some("\u{1F600}".to_string())); // 😀 (SMP)
                                                                                    // Surrogate codepoint → reject.
        assert_eq!(glyph_name_to_unicode("uD800"), None);
        // Beyond 0x10FFFF → reject (7 hex digits also out of allowed length).
        assert_eq!(glyph_name_to_unicode("u110000"), None);
        // 3 hex digits or 7 hex digits → fall through.
        assert_eq!(glyph_name_to_unicode("u20"), None);
        assert_eq!(glyph_name_to_unicode("u1F60000"), None);
    }

    #[test]
    fn glyph_name_to_unicode_variant_suffix_stripped() {
        // Subset fonts often use `.alt`, `.sc`, `.001`, etc. as stylistic variant
        // markers. The base name before the dot is the canonical glyph; the
        // codepoint is what we want for extraction.
        assert_eq!(glyph_name_to_unicode("A.sc"), Some("A".to_string()));
        assert_eq!(glyph_name_to_unicode("bullet.alt"), Some("\u{2022}".to_string()));
        assert_eq!(glyph_name_to_unicode("fi.001"), Some("\u{FB01}".to_string()));
        // Unknown base name → still None.
        assert_eq!(glyph_name_to_unicode("xyzzy.sc"), None);
    }

    #[test]
    fn glyph_name_to_unicode_unknown_or_control() {
        // Not in AGL, not a synth pattern.
        assert_eq!(glyph_name_to_unicode(""), None);
        assert_eq!(glyph_name_to_unicode(".notdef"), None);
        assert_eq!(glyph_name_to_unicode("xyzzy"), None);
        // uniXXXX for a control char → reject (not useful as text).
        assert_eq!(glyph_name_to_unicode("uni0007"), None); // BEL
    }

    #[test]
    fn test_glyph_name_lookup() {
        let mapper = CharacterMapper::new();

        // Test that Adobe Glyph List lookups work
        assert!(mapper.map_glyph_name("A").is_some());
        assert!(mapper.map_glyph_name("space").is_some());
    }

    // ===== Tests for Predefined CMap Support (Issue #104 Sub-category 3) =====

    #[test]
    fn test_predefined_cmap_japan1_ascii() {
        let mut mapper = CharacterMapper::new();
        mapper.set_predefined_cmap(Some(PredefinedCMapConfig {
            ordering: "Japan1".to_string(),
        }));
        // CID 34 -> 'A' (U+0041) in Adobe-Japan1
        // Note: This goes through Priority 2 (glyph list) first for ASCII range.
        // For CIDs outside ASCII range, it falls through to Priority 3.
        let result = mapper.map_character(34);
        assert!(result.is_some());
    }

    #[test]
    fn test_predefined_cmap_japan1_hiragana() {
        let mut mapper = CharacterMapper::new();
        // Clear tounicode_cmap so Priority 3 is reached
        mapper.set_tounicode_cmap(None);
        mapper.set_predefined_cmap(Some(PredefinedCMapConfig {
            ordering: "Japan1".to_string(),
        }));
        // CID 843 -> U+3042 (hiragana 'a') in Adobe-Japan1
        let result = mapper.map_character(843);
        assert_eq!(result, Some("\u{3042}".to_string())); // あ
    }

    #[test]
    fn test_predefined_cmap_gb1_chinese() {
        let mut mapper = CharacterMapper::new();
        mapper.set_predefined_cmap(Some(PredefinedCMapConfig {
            ordering: "GB1".to_string(),
        }));
        // CID 4559 -> U+4E2D (中) in Adobe-GB1
        let result = mapper.map_character(4559);
        assert_eq!(result, Some("\u{4E2D}".to_string())); // 中
    }

    #[test]
    fn test_predefined_cmap_korea1_hangul() {
        let mut mapper = CharacterMapper::new();
        mapper.set_predefined_cmap(Some(PredefinedCMapConfig {
            ordering: "Korea1".to_string(),
        }));
        // CID 1086 -> U+AC00 (가) in Adobe-Korea1
        let result = mapper.map_character(1086);
        assert_eq!(result, Some("\u{AC00}".to_string())); // 가
    }

    #[test]
    fn test_predefined_cmap_cns1() {
        let mut mapper = CharacterMapper::new();
        mapper.set_predefined_cmap(Some(PredefinedCMapConfig {
            ordering: "CNS1".to_string(),
        }));
        // CID 34 -> 'A' (U+0041) in Adobe-CNS1
        let result = mapper.map_character(34);
        assert!(result.is_some());
    }

    #[test]
    fn test_predefined_cmap_identity() {
        let mut mapper = CharacterMapper::new();
        mapper.set_predefined_cmap(Some(PredefinedCMapConfig {
            ordering: "Identity".to_string(),
        }));
        // Identity: CID 0x4E2D == U+4E2D directly
        let result = mapper.map_character(0x4E2D);
        assert_eq!(result, Some("\u{4E2D}".to_string())); // 中
    }

    #[test]
    fn test_predefined_cmap_unknown_ordering() {
        let mut mapper = CharacterMapper::new();
        mapper.set_predefined_cmap(Some(PredefinedCMapConfig {
            ordering: "UnknownCollection".to_string(),
        }));
        // Unknown ordering should fall through to next priority
        // Code 0x4E2D is outside ASCII/WinAnsi range, so no glyph name match
        // With unknown ordering, predefined CMap returns None
        // Falls through to U+FFFD
        let result = mapper.map_character(0x4E2D);
        assert_eq!(result, Some("\u{FFFD}".to_string()));
    }

    #[test]
    fn test_predefined_cmap_not_set() {
        let mapper = CharacterMapper::new();
        // Without predefined CMap set, mapper should still work for ASCII
        assert_eq!(mapper.map_character(0x41), Some("A".to_string()));
    }

    #[test]
    fn test_tounicode_overrides_predefined_cmap() {
        use super::super::cmap::parse_tounicode_cmap;

        let mut mapper = CharacterMapper::new();
        mapper.set_predefined_cmap(Some(PredefinedCMapConfig {
            ordering: "Japan1".to_string(),
        }));

        // Create a simple ToUnicode CMap that maps CID 843 to 'X'
        let cmap_data = b"/CIDInit /ProcSet findresource begin\n\
            12 dict begin\n\
            begincmap\n\
            /CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def\n\
            /CMapName /Adobe-Identity-UCS def\n\
            1 beginbfchar\n\
            <034B> <0058>\n\
            endbfchar\n\
            endcmap\n\
            CMapName currentdict /CMap defineresource pop\n\
            end\n\
            end";

        if let Ok(cmap) = parse_tounicode_cmap(cmap_data) {
            mapper.set_tounicode_cmap(Some(cmap));
        }

        // ToUnicode (Priority 1) should override predefined CMap (Priority 3)
        let result = mapper.map_character(843); // 0x034B
        assert_eq!(result, Some("X".to_string()));
    }

    #[test]
    fn test_predefined_cmap_config_clone() {
        let config = PredefinedCMapConfig {
            ordering: "Japan1".to_string(),
        };
        let cloned = config.clone();
        assert_eq!(cloned.ordering, "Japan1");
    }

    // Regression test for #363: subset Type0 font with Identity-H and an
    // incomplete ToUnicode CMap. When the CMap is present but misses a CID,
    // the old fallback chain would either:
    //   - Priority 2 map the CID through the ASCII glyph list (treating cid
    //     0x25 as '%' even though the font's CID 0x25 is some other letter)
    //   - Priority 3 with Identity ordering return `cid as u32` directly
    //
    // Both produce ASCII-shifted "ciphertext" like `%B+$%8A//$2*%01*1%6APP` in
    // real PDFs (nougat_035.pdf page 13 was the original symptom).
    //
    // Per ISO 32000-1 §9.10.2: when a ToUnicode CMap is attached to a font,
    // it is authoritative. A miss must produce U+FFFD, not be papered over
    // by a code-as-ASCII or CID-as-Unicode heuristic.
    #[test]
    fn tounicode_miss_on_identity_h_font_returns_replacement() {
        use super::super::cmap::parse_tounicode_cmap;

        let mut mapper = CharacterMapper::new();
        mapper.set_predefined_cmap(Some(PredefinedCMapConfig {
            ordering: "Identity".to_string(),
        }));

        // Minimal ToUnicode: only CID 0x0001 → 'T'. 0x0025 is absent.
        let cmap_data = b"/CIDInit /ProcSet findresource begin\n\
            12 dict begin\n\
            begincmap\n\
            /CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def\n\
            /CMapName /Adobe-Identity-UCS def\n\
            1 beginbfchar\n\
            <0001> <0054>\n\
            endbfchar\n\
            endcmap\n\
            CMapName currentdict /CMap defineresource pop\n\
            end\n\
            end";
        mapper.set_tounicode_cmap(Some(parse_tounicode_cmap(cmap_data).unwrap()));

        // ToUnicode hit: baseline sanity.
        assert_eq!(mapper.map_character(0x0001), Some("T".to_string()));

        // ToUnicode miss: must return U+FFFD, not '%' (AGL) and not U+0025 (Identity).
        let result = mapper.map_character(0x0025);
        assert_eq!(
            result,
            Some("\u{FFFD}".to_string()),
            "ToUnicode-present-but-missed must produce U+FFFD, got {result:?}"
        );
    }

    // Guard: simple fonts (Type1 / TrueType with WinAnsiEncoding) that have no
    // ToUnicode CMap must still resolve codes via the Adobe Glyph List. The
    // #363 fix only activates when a ToUnicode CMap is attached.
    #[test]
    fn no_tounicode_still_uses_adobe_glyph_list() {
        let mapper = CharacterMapper::new();
        assert_eq!(mapper.map_character(0x41), Some("A".to_string()));
        assert_eq!(mapper.map_character(0x25), Some("%".to_string()));
    }
}
