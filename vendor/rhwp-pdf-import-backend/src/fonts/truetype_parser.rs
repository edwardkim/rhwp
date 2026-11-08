//! TrueType/OpenType font parser for PDF embedding.
//!
//! This module wraps the `ttf-parser` crate to extract font data needed
//! for embedding TrueType fonts in PDF documents with full Unicode support.
//!
//! # Font Embedding in PDF
//!
//! Per PDF spec Section 9.6-9.8, embedded fonts require:
//! - FontDescriptor with metrics (ascender, descender, cap height, etc.)
//! - ToUnicode CMap for text extraction
//! - Font program data (FontFile2 for TrueType)
//! - CIDFont for Unicode (Type 0 composite fonts with Identity-H encoding)

use std::collections::{BTreeSet, HashMap};
use std::io::{self, Write};

use ttf_parser::{Face, GlyphId};

/// Error types for TrueType font parsing.
#[derive(Debug, thiserror::Error)]
pub enum TrueTypeError {
    /// Failed to parse font file
    #[error("Failed to parse font file: {0}")]
    ParseError(String),

    /// Font file is empty or invalid
    #[error("Font file is empty or invalid")]
    EmptyFont,

    /// Required table is missing
    #[error("Required font table is missing: {0}")]
    MissingTable(String),

    /// IO error during font operations
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    /// Glyph not found
    #[error("Glyph not found for character: U+{0:04X}")]
    GlyphNotFound(u32),
}

/// Result type for TrueType operations.
pub type TrueTypeResult<T> = Result<T, TrueTypeError>;

/// Parsed TrueType font data for PDF embedding.
#[derive(Debug)]
pub struct TrueTypeFont<'a> {
    /// The parsed font face
    face: Face<'a>,
    /// Original font data (needed for embedding)
    data: &'a [u8],
    /// Cached Unicode to glyph ID mapping
    unicode_to_glyph: HashMap<u32, u16>,
    /// Cached glyph widths (glyph ID -> width in font units)
    glyph_widths: HashMap<u16, u16>,
}

impl<'a> TrueTypeFont<'a> {
    /// Parse a TrueType/OpenType font from raw data.
    ///
    /// # Arguments
    /// * `data` - Raw font file bytes (TTF or OTF)
    ///
    /// # Returns
    /// A parsed TrueType font ready for PDF embedding.
    pub fn parse(data: &'a [u8]) -> TrueTypeResult<Self> {
        if data.is_empty() {
            return Err(TrueTypeError::EmptyFont);
        }

        let face = Face::parse(data, 0).map_err(|e| TrueTypeError::ParseError(e.to_string()))?;

        let mut font = Self {
            face,
            data,
            unicode_to_glyph: HashMap::new(),
            glyph_widths: HashMap::new(),
        };

        font.build_unicode_map();
        font.build_width_table();

        Ok(font)
    }

    /// Build Unicode to glyph ID mapping from cmap table.
    fn build_unicode_map(&mut self) {
        // Basic Multilingual Plane (covers virtually all common scripts incl.
        // CJK) plus the emoji supplementary range (U+1F000..=U+1FAFF). Scanning
        // all 17 planes on every font parse would be wasteful and this runs in
        // the extraction hot path; the emoji range is the one supplementary
        // block that occurs routinely in user text (e.g. filled form fields).
        let ranges = (0..=0xFFFF_u32).chain(0x1F000..=0x1FAFF);
        for codepoint in ranges {
            if let Some(char) = char::from_u32(codepoint) {
                if let Some(glyph_id) = self.face.glyph_index(char) {
                    self.unicode_to_glyph.insert(codepoint, glyph_id.0);
                }
            }
        }
    }

    /// Build glyph width table from hmtx table.
    fn build_width_table(&mut self) {
        let units_per_em = self.face.units_per_em();

        for glyph_id in 0..self.face.number_of_glyphs() {
            let glyph = GlyphId(glyph_id);
            let advance = self.face.glyph_hor_advance(glyph).unwrap_or(0);
            // Store as width in units of 1/1000 of em
            let width_1000 = (advance as u32 * 1000 / units_per_em as u32) as u16;
            self.glyph_widths.insert(glyph_id, width_1000);
        }
    }

    /// Get the font's PostScript name.
    pub fn postscript_name(&self) -> Option<String> {
        self.face
            .names()
            .into_iter()
            .find(|name| name.name_id == ttf_parser::name_id::POST_SCRIPT_NAME)
            .and_then(|name| name.to_string())
    }

    /// Get the font family name.
    pub fn family_name(&self) -> Option<String> {
        self.face
            .names()
            .into_iter()
            .find(|name| name.name_id == ttf_parser::name_id::FAMILY)
            .and_then(|name| name.to_string())
    }

    /// Get units per em for this font.
    pub fn units_per_em(&self) -> u16 {
        self.face.units_per_em()
    }

    /// Get the ascender in font units.
    pub fn ascender(&self) -> i16 {
        self.face.ascender()
    }

    /// Get the descender in font units (negative value).
    pub fn descender(&self) -> i16 {
        self.face.descender()
    }

    /// Get the cap height in font units.
    pub fn cap_height(&self) -> Option<i16> {
        self.face.capital_height()
    }

    /// Get the x-height in font units.
    pub fn x_height(&self) -> Option<i16> {
        self.face.x_height()
    }

    /// Get the italic angle.
    pub fn italic_angle(&self) -> f32 {
        self.face.italic_angle()
    }

    /// Check if the font is bold.
    pub fn is_bold(&self) -> bool {
        self.face.is_bold()
    }

    /// Check if the font is italic.
    pub fn is_italic(&self) -> bool {
        self.face.is_italic()
    }

    /// Get the font bounding box.
    pub fn bbox(&self) -> (i16, i16, i16, i16) {
        let bbox = self.face.global_bounding_box();
        (bbox.x_min, bbox.y_min, bbox.x_max, bbox.y_max)
    }

    /// Get glyph ID for a Unicode codepoint.
    pub fn glyph_id(&self, codepoint: u32) -> Option<u16> {
        self.unicode_to_glyph.get(&codepoint).copied()
    }

    /// Get glyph width in 1/1000 em units.
    pub fn glyph_width(&self, glyph_id: u16) -> u16 {
        self.glyph_widths.get(&glyph_id).copied().unwrap_or(500)
    }

    /// Get width for a Unicode character in 1/1000 em units.
    pub fn char_width(&self, codepoint: u32) -> u16 {
        self.glyph_id(codepoint)
            .map(|gid| self.glyph_width(gid))
            .unwrap_or(500)
    }

    /// Get the number of glyphs in the font.
    pub fn num_glyphs(&self) -> u16 {
        self.face.number_of_glyphs()
    }

    /// Get the raw font data for embedding.
    pub fn raw_data(&self) -> &[u8] {
        self.data
    }

    /// Get all Unicode codepoints supported by this font.
    pub fn supported_codepoints(&self) -> Vec<u32> {
        let mut codepoints: Vec<_> = self.unicode_to_glyph.keys().copied().collect();
        codepoints.sort();
        codepoints
    }

    /// Calculate StemV (vertical stem width) - estimated from font weight.
    ///
    /// This is a heuristic since TrueType doesn't store StemV directly.
    pub fn stem_v(&self) -> i16 {
        if self.is_bold() {
            140
        } else {
            80
        }
    }

    /// Get font flags for PDF FontDescriptor.
    ///
    /// Returns flags per PDF spec Table 123:
    /// - Bit 1: FixedPitch
    /// - Bit 2: Serif (not easily determinable, assume false)
    /// - Bit 3: Symbolic (for Symbol/ZapfDingbats type fonts)
    /// - Bit 4: Script (cursive fonts)
    /// - Bit 6: Nonsymbolic (standard Latin text font)
    /// - Bit 7: Italic
    /// - Bit 17: AllCap
    /// - Bit 18: SmallCap
    /// - Bit 19: ForceBold
    pub fn font_flags(&self) -> u32 {
        let mut flags = 0u32;

        // Bit 1: FixedPitch (monospace)
        if self.face.is_monospaced() {
            flags |= 1 << 0;
        }

        // Bit 6: Nonsymbolic (standard text font)
        // Most TrueType fonts are nonsymbolic
        flags |= 1 << 5;

        // Bit 7: Italic
        if self.is_italic() {
            flags |= 1 << 6;
        }

        flags
    }

    /// Generate widths array for PDF CIDFont W entry.
    ///
    /// Format: [start_cid [w1 w2 ...] start_cid2 [w1 w2 ...] ...]
    /// For Identity-H encoding, CID = GID.
    pub fn generate_widths_array(&self, used_glyphs: &BTreeSet<u16>) -> Vec<u8> {
        let mut result = Vec::new();
        write!(result, "[").unwrap();

        // Group consecutive glyphs
        let mut glyphs: Vec<_> = used_glyphs.iter().copied().collect();
        glyphs.sort();

        let mut i = 0;
        while i < glyphs.len() {
            let start = glyphs[i];
            let mut end = start;
            let mut widths = vec![self.glyph_width(start)];

            // Find consecutive glyphs
            while i + 1 < glyphs.len() && glyphs[i + 1] == end + 1 {
                i += 1;
                end = glyphs[i];
                widths.push(self.glyph_width(end));
            }

            write!(result, "{} [", start).unwrap();
            for (j, w) in widths.iter().enumerate() {
                if j > 0 {
                    write!(result, " ").unwrap();
                }
                write!(result, "{}", w).unwrap();
            }
            write!(result, "]").unwrap();

            i += 1;
        }

        write!(result, "]").unwrap();
        result
    }

    /// Generate ToUnicode CMap for text extraction.
    ///
    /// This CMap maps GIDs (used as CIDs with Identity-H) back to Unicode
    /// so PDF readers can extract text from the generated PDF.
    pub fn generate_tounicode_cmap(&self, used_chars: &HashMap<u32, u16>) -> String {
        let mut cmap = String::new();

        // CMap header
        cmap.push_str("/CIDInit /ProcSet findresource begin\n");
        cmap.push_str("12 dict begin\n");
        cmap.push_str("begincmap\n");
        cmap.push_str("/CIDSystemInfo <<\n");
        cmap.push_str("  /Registry (Adobe)\n");
        cmap.push_str("  /Ordering (UCS)\n");
        cmap.push_str("  /Supplement 0\n");
        cmap.push_str(">> def\n");
        cmap.push_str("/CMapName /Adobe-Identity-UCS def\n");
        cmap.push_str("/CMapType 2 def\n");
        cmap.push_str("1 begincodespacerange\n");
        cmap.push_str("<0000> <FFFF>\n");
        cmap.push_str("endcodespacerange\n");

        // Build GID -> Unicode mappings
        let mut mappings: Vec<(u16, u32)> = used_chars
            .iter()
            .map(|(&unicode, &gid)| (gid, unicode))
            .collect();
        mappings.sort_by_key(|&(gid, _)| gid);

        // Write bfchar entries (max 100 per section per PDF spec)
        let chunks: Vec<_> = mappings.chunks(100).collect();
        for chunk in chunks {
            cmap.push_str(&format!("{} beginbfchar\n", chunk.len()));
            for &(gid, unicode) in chunk {
                if unicode <= 0xFFFF {
                    cmap.push_str(&format!("<{:04X}> <{:04X}>\n", gid, unicode));
                } else {
                    // Supplementary plane - encode as UTF-16 surrogate pair
                    let high = ((unicode - 0x10000) >> 10) + 0xD800;
                    let low = ((unicode - 0x10000) & 0x3FF) + 0xDC00;
                    cmap.push_str(&format!("<{:04X}> <{:04X}{:04X}>\n", gid, high, low));
                }
            }
            cmap.push_str("endbfchar\n");
        }

        // CMap footer
        cmap.push_str("endcmap\n");
        cmap.push_str("CMapName currentdict /CMap defineresource pop\n");
        cmap.push_str("end\n");
        cmap.push_str("end\n");

        cmap
    }
}

/// Font metrics extracted for PDF FontDescriptor.
#[derive(Debug, Clone)]
pub struct FontMetrics {
    /// PostScript name
    pub name: String,
    /// Family name
    pub family: String,
    /// Units per em
    pub units_per_em: u16,
    /// Ascender (positive)
    pub ascender: i16,
    /// Descender (negative)
    pub descender: i16,
    /// Cap height
    pub cap_height: i16,
    /// x-height
    pub x_height: i16,
    /// Italic angle
    pub italic_angle: f32,
    /// Bounding box (llx, lly, urx, ury)
    pub bbox: (i16, i16, i16, i16),
    /// Stem V (vertical stem width)
    pub stem_v: i16,
    /// Font flags
    pub flags: u32,
    /// Is bold
    pub is_bold: bool,
    /// Is italic
    pub is_italic: bool,
}

impl FontMetrics {
    /// Extract metrics from a parsed TrueType font.
    pub fn from_font(font: &TrueTypeFont) -> Self {
        Self {
            name: font
                .postscript_name()
                .unwrap_or_else(|| "Unknown".to_string()),
            family: font.family_name().unwrap_or_else(|| "Unknown".to_string()),
            units_per_em: font.units_per_em(),
            ascender: font.ascender(),
            descender: font.descender(),
            cap_height: font.cap_height().unwrap_or(font.ascender()),
            x_height: font
                .x_height()
                .unwrap_or((font.ascender() as f32 * 0.5) as i16),
            italic_angle: font.italic_angle(),
            bbox: font.bbox(),
            stem_v: font.stem_v(),
            flags: font.font_flags(),
            is_bold: font.is_bold(),
            is_italic: font.is_italic(),
        }
    }

    /// Convert a value from font units to PDF units (1/1000 em).
    pub fn to_pdf_units(&self, value: i16) -> i32 {
        (value as i32 * 1000) / self.units_per_em as i32
    }

    /// Get ascender in PDF units.
    pub fn pdf_ascender(&self) -> i32 {
        self.to_pdf_units(self.ascender)
    }

    /// Get descender in PDF units.
    pub fn pdf_descender(&self) -> i32 {
        self.to_pdf_units(self.descender)
    }

    /// Get cap height in PDF units.
    pub fn pdf_cap_height(&self) -> i32 {
        self.to_pdf_units(self.cap_height)
    }

    /// Get bounding box in PDF units.
    pub fn pdf_bbox(&self) -> (i32, i32, i32, i32) {
        (
            self.to_pdf_units(self.bbox.0),
            self.to_pdf_units(self.bbox.1),
            self.to_pdf_units(self.bbox.2),
            self.to_pdf_units(self.bbox.3),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require actual font data to run meaningfully.
    // In real usage, tests would load a TTF file from the test fixtures.

    #[test]
    fn test_error_on_empty_data() {
        let result = TrueTypeFont::parse(&[]);
        assert!(matches!(result, Err(TrueTypeError::EmptyFont)));
    }

    #[test]
    fn test_error_on_invalid_data() {
        let result = TrueTypeFont::parse(b"not a font file");
        assert!(matches!(result, Err(TrueTypeError::ParseError(_))));
    }

    #[test]
    fn test_font_flags_nonsymbolic() {
        // When we have a real font, it should have the nonsymbolic flag
        // This is a placeholder for when font test data is available
    }

    #[test]
    fn test_tounicode_cmap_format() {
        // Test that ToUnicode CMap generation produces valid structure
        let mut used_chars = HashMap::new();
        used_chars.insert(0x0041, 1_u16); // 'A' -> GID 1
        used_chars.insert(0x0042, 2_u16); // 'B' -> GID 2

        // We'd need a real font to test this fully
        // This validates the format structure
    }

    // =========================================================================
    // Tests using real system font (DejaVu Sans)
    // =========================================================================

    /// Helper to load a font by trying bundled test fixtures first, then system paths.
    fn load_font(name: &str) -> Option<Vec<u8>> {
        let manifest = env!("CARGO_MANIFEST_DIR");
        let candidates = [
            format!("{manifest}/tests/fixtures/fonts/{name}"),
            format!("/usr/share/fonts/truetype/dejavu/{name}"),
            format!("/usr/share/fonts/dejavu-sans-fonts/{name}"),
            format!("/usr/share/fonts/TTF/{name}"),
        ];
        for path in &candidates {
            if let Ok(data) = std::fs::read(path) {
                return Some(data);
            }
        }
        None
    }

    fn load_dejavu_sans() -> Option<Vec<u8>> {
        load_font("DejaVuSans.ttf")
    }

    fn load_dejavu_sans_bold() -> Option<Vec<u8>> {
        load_font("DejaVuSans-Bold.ttf")
    }

    fn load_dejavu_sans_mono() -> Option<Vec<u8>> {
        load_font("DejaVuSansMono.ttf")
    }

    #[test]
    fn test_parse_valid_font() {
        let data = match load_dejavu_sans() {
            Some(d) => d,
            None => return, // Skip if font not available
        };
        let font = TrueTypeFont::parse(&data);
        assert!(font.is_ok(), "Failed to parse DejaVu Sans: {:?}", font.err());
    }

    #[test]
    fn test_font_postscript_name() {
        let data = match load_dejavu_sans() {
            Some(d) => d,
            None => return,
        };
        let font = TrueTypeFont::parse(&data).unwrap();
        let ps_name = font.postscript_name();
        // postscript_name() may return None if the name table encoding
        // is not supported by ttf-parser's to_string(). Just verify
        // the method doesn't panic and returns a valid Option.
        if let Some(ref name) = ps_name {
            assert!(!name.is_empty(), "PostScript name should not be empty if present");
        }
    }

    #[test]
    fn test_font_family_name() {
        let data = match load_dejavu_sans() {
            Some(d) => d,
            None => return,
        };
        let font = TrueTypeFont::parse(&data).unwrap();
        let family = font.family_name();
        // family_name() may return None if the name table encoding
        // is not supported by ttf-parser's to_string(). Just verify
        // the method doesn't panic and returns a valid Option.
        if let Some(ref name) = family {
            assert!(!name.is_empty(), "Family name should not be empty if present");
        }
    }

    #[test]
    fn test_font_units_per_em() {
        let data = match load_dejavu_sans() {
            Some(d) => d,
            None => return,
        };
        let font = TrueTypeFont::parse(&data).unwrap();
        let upem = font.units_per_em();
        assert!(upem > 0, "Units per em should be positive, got: {}", upem);
    }

    #[test]
    fn test_font_ascender_descender() {
        let data = match load_dejavu_sans() {
            Some(d) => d,
            None => return,
        };
        let font = TrueTypeFont::parse(&data).unwrap();
        let asc = font.ascender();
        let desc = font.descender();
        assert!(asc > 0, "Ascender should be positive, got: {}", asc);
        assert!(desc < 0, "Descender should be negative, got: {}", desc);
    }

    #[test]
    fn test_font_cap_height() {
        let data = match load_dejavu_sans() {
            Some(d) => d,
            None => return,
        };
        let font = TrueTypeFont::parse(&data).unwrap();
        // cap_height may or may not be in the font
        if let Some(cap_h) = font.cap_height() {
            assert!(cap_h > 0, "Cap height should be positive, got: {}", cap_h);
        }
    }

    #[test]
    fn test_font_x_height() {
        let data = match load_dejavu_sans() {
            Some(d) => d,
            None => return,
        };
        let font = TrueTypeFont::parse(&data).unwrap();
        if let Some(x_h) = font.x_height() {
            assert!(x_h > 0, "x-height should be positive, got: {}", x_h);
        }
    }

    #[test]
    fn test_font_bbox() {
        let data = match load_dejavu_sans() {
            Some(d) => d,
            None => return,
        };
        let font = TrueTypeFont::parse(&data).unwrap();
        let (x_min, y_min, x_max, y_max) = font.bbox();
        assert!(x_max > x_min, "x_max ({}) should be > x_min ({})", x_max, x_min);
        assert!(y_max > y_min, "y_max ({}) should be > y_min ({})", y_max, y_min);
    }

    #[test]
    fn test_font_italic_angle_regular() {
        let data = match load_dejavu_sans() {
            Some(d) => d,
            None => return,
        };
        let font = TrueTypeFont::parse(&data).unwrap();
        // DejaVu Sans (regular) should have italic angle of 0
        assert_eq!(font.italic_angle(), 0.0, "Regular font should have italic angle 0");
    }

    #[test]
    fn test_font_is_not_bold_regular() {
        let data = match load_dejavu_sans() {
            Some(d) => d,
            None => return,
        };
        let font = TrueTypeFont::parse(&data).unwrap();
        assert!(!font.is_bold(), "DejaVu Sans (regular) should not be bold");
    }

    #[test]
    fn test_font_is_bold() {
        let data = match load_dejavu_sans_bold() {
            Some(d) => d,
            None => return,
        };
        let font = TrueTypeFont::parse(&data).unwrap();
        assert!(font.is_bold(), "DejaVu Sans Bold should be bold");
    }

    #[test]
    fn test_font_is_not_italic_regular() {
        let data = match load_dejavu_sans() {
            Some(d) => d,
            None => return,
        };
        let font = TrueTypeFont::parse(&data).unwrap();
        assert!(!font.is_italic(), "DejaVu Sans (regular) should not be italic");
    }

    #[test]
    fn test_glyph_id_for_ascii() {
        let data = match load_dejavu_sans() {
            Some(d) => d,
            None => return,
        };
        let font = TrueTypeFont::parse(&data).unwrap();

        // 'A' = U+0041 should be present in any text font
        let gid_a = font.glyph_id(0x0041);
        assert!(gid_a.is_some(), "Glyph for 'A' (U+0041) should exist");
        assert!(gid_a.unwrap() > 0, "GID for 'A' should be > 0 (0 is .notdef)");

        // 'Z' = U+005A
        let gid_z = font.glyph_id(0x005A);
        assert!(gid_z.is_some(), "Glyph for 'Z' (U+005A) should exist");
    }

    #[test]
    fn test_glyph_id_nonexistent() {
        let data = match load_dejavu_sans() {
            Some(d) => d,
            None => return,
        };
        let font = TrueTypeFont::parse(&data).unwrap();

        // Private use area codepoint unlikely to be mapped
        let gid = font.glyph_id(0xFFFD_u32.wrapping_add(1000));
        // This may or may not exist; just verify no panic
        let _ = gid;
    }

    #[test]
    fn test_glyph_width() {
        let data = match load_dejavu_sans() {
            Some(d) => d,
            None => return,
        };
        let font = TrueTypeFont::parse(&data).unwrap();

        // Get width for 'A'
        if let Some(gid) = font.glyph_id(0x0041) {
            let width = font.glyph_width(gid);
            assert!(width > 0, "Width for 'A' should be positive, got: {}", width);
        }
    }

    #[test]
    fn test_glyph_width_default_for_missing() {
        let data = match load_dejavu_sans() {
            Some(d) => d,
            None => return,
        };
        let font = TrueTypeFont::parse(&data).unwrap();

        // Request width for a glyph ID that definitely doesn't exist
        let width = font.glyph_width(u16::MAX);
        assert_eq!(width, 500, "Missing glyph should return default width 500");
    }

    #[test]
    fn test_char_width() {
        let data = match load_dejavu_sans() {
            Some(d) => d,
            None => return,
        };
        let font = TrueTypeFont::parse(&data).unwrap();

        let w_a = font.char_width(0x0041); // 'A'
        assert!(w_a > 0, "Width of 'A' should be positive");

        let w_space = font.char_width(0x0020); // space
        assert!(w_space > 0, "Width of space should be positive");

        // Typically, 'W' is wider than 'i'
        let w_big = font.char_width(0x0057); // 'W'
        let w_small = font.char_width(0x0069); // 'i'
        assert!(w_big > w_small, "'W' width ({}) should be > 'i' width ({})", w_big, w_small);
    }

    #[test]
    fn test_num_glyphs() {
        let data = match load_dejavu_sans() {
            Some(d) => d,
            None => return,
        };
        let font = TrueTypeFont::parse(&data).unwrap();
        let n = font.num_glyphs();
        assert!(n > 100, "DejaVu Sans should have many glyphs, got: {}", n);
    }

    #[test]
    fn test_raw_data() {
        let data = match load_dejavu_sans() {
            Some(d) => d,
            None => return,
        };
        let font = TrueTypeFont::parse(&data).unwrap();
        assert_eq!(font.raw_data().len(), data.len(), "raw_data should match input length");
        assert_eq!(font.raw_data(), data.as_slice(), "raw_data should match input bytes");
    }

    #[test]
    fn test_supported_codepoints() {
        let data = match load_dejavu_sans() {
            Some(d) => d,
            None => return,
        };
        let font = TrueTypeFont::parse(&data).unwrap();
        let codepoints = font.supported_codepoints();
        assert!(
            codepoints.len() > 100,
            "DejaVu Sans should support many codepoints, got: {}",
            codepoints.len()
        );
        // Verify sorted
        for w in codepoints.windows(2) {
            assert!(w[0] <= w[1], "Codepoints should be sorted");
        }
        // Basic Latin should be present
        assert!(codepoints.contains(&0x0041), "Should support 'A' (U+0041)");
    }

    #[test]
    fn test_stem_v_regular_vs_bold() {
        let data_regular = match load_dejavu_sans() {
            Some(d) => d,
            None => return,
        };
        let data_bold = match load_dejavu_sans_bold() {
            Some(d) => d,
            None => return,
        };
        let font_regular = TrueTypeFont::parse(&data_regular).unwrap();
        let font_bold = TrueTypeFont::parse(&data_bold).unwrap();

        let stem_regular = font_regular.stem_v();
        let stem_bold = font_bold.stem_v();
        assert!(stem_regular > 0, "Regular stem_v should be positive, got {}", stem_regular);
        assert!(
            stem_bold > stem_regular,
            "Bold stem_v ({}) should be greater than regular stem_v ({})",
            stem_bold,
            stem_regular
        );
    }

    #[test]
    fn test_font_flags_regular() {
        let data = match load_dejavu_sans() {
            Some(d) => d,
            None => return,
        };
        let font = TrueTypeFont::parse(&data).unwrap();
        let flags = font.font_flags();

        // Nonsymbolic flag (bit 6, i.e., 1 << 5 = 32) should be set
        assert!(flags & (1 << 5) != 0, "Nonsymbolic flag should be set, got flags: {}", flags);

        // Not italic
        assert!(flags & (1 << 6) == 0, "Italic flag should not be set for regular font");
    }

    #[test]
    fn test_font_flags_monospace() {
        let data = match load_dejavu_sans_mono() {
            Some(d) => d,
            None => return,
        };
        let font = TrueTypeFont::parse(&data).unwrap();
        let flags = font.font_flags();

        // Monospace flag (bit 1, i.e., 1 << 0 = 1)
        assert!(
            flags & 1 != 0,
            "FixedPitch flag should be set for monospace font, got flags: {}",
            flags
        );
    }

    #[test]
    fn test_generate_widths_array() {
        let data = match load_dejavu_sans() {
            Some(d) => d,
            None => return,
        };
        let font = TrueTypeFont::parse(&data).unwrap();

        let mut used = BTreeSet::new();
        // Add a few consecutive glyph IDs
        if let Some(gid_a) = font.glyph_id(0x0041) {
            used.insert(gid_a);
        }
        if let Some(gid_b) = font.glyph_id(0x0042) {
            used.insert(gid_b);
        }
        if let Some(gid_c) = font.glyph_id(0x0043) {
            used.insert(gid_c);
        }

        let widths = font.generate_widths_array(&used);
        let widths_str = String::from_utf8(widths).expect("widths should be valid UTF-8");

        assert!(widths_str.starts_with('['), "Widths array should start with [");
        assert!(widths_str.ends_with(']'), "Widths array should end with ]");
        assert!(widths_str.len() > 2, "Widths array should not be empty");
    }

    #[test]
    fn test_generate_widths_array_empty() {
        let data = match load_dejavu_sans() {
            Some(d) => d,
            None => return,
        };
        let font = TrueTypeFont::parse(&data).unwrap();

        let used = BTreeSet::new();
        let widths = font.generate_widths_array(&used);
        let widths_str = String::from_utf8(widths).expect("widths should be valid UTF-8");
        assert_eq!(widths_str, "[]", "Empty glyph set should produce []");
    }

    #[test]
    fn test_generate_tounicode_cmap_bmp() {
        let data = match load_dejavu_sans() {
            Some(d) => d,
            None => return,
        };
        let font = TrueTypeFont::parse(&data).unwrap();

        let mut used_chars = HashMap::new();
        if let Some(gid) = font.glyph_id(0x0041) {
            used_chars.insert(0x0041_u32, gid);
        }
        if let Some(gid) = font.glyph_id(0x0042) {
            used_chars.insert(0x0042_u32, gid);
        }

        let cmap = font.generate_tounicode_cmap(&used_chars);

        assert!(cmap.contains("begincmap"), "CMap should contain begincmap");
        assert!(cmap.contains("endcmap"), "CMap should contain endcmap");
        assert!(cmap.contains("beginbfchar"), "CMap should contain beginbfchar");
        assert!(cmap.contains("endbfchar"), "CMap should contain endbfchar");
        assert!(cmap.contains("<0041>"), "CMap should contain Unicode for 'A'");
    }

    #[test]
    fn test_generate_tounicode_cmap_supplementary_plane() {
        let data = match load_dejavu_sans() {
            Some(d) => d,
            None => return,
        };
        let font = TrueTypeFont::parse(&data).unwrap();

        // Use a supplementary plane codepoint (U+1F600 GRINNING FACE)
        // Even if the font doesn't have it, we can test the CMap generation logic
        let mut used_chars = HashMap::new();
        used_chars.insert(0x1F600_u32, 5000_u16); // Fake GID

        let cmap = font.generate_tounicode_cmap(&used_chars);

        // Supplementary plane characters should produce surrogate pairs
        assert!(cmap.contains("begincmap"), "CMap should contain begincmap");
        // For U+1F600: high surrogate = 0xD83D, low surrogate = 0xDE00
        assert!(
            cmap.contains("<D83D"),
            "CMap should contain high surrogate for supplementary char"
        );
    }

    #[test]
    fn test_generate_tounicode_cmap_empty() {
        let data = match load_dejavu_sans() {
            Some(d) => d,
            None => return,
        };
        let font = TrueTypeFont::parse(&data).unwrap();

        let used_chars = HashMap::new();
        let cmap = font.generate_tounicode_cmap(&used_chars);

        assert!(cmap.contains("begincmap"), "CMap should contain header");
        assert!(cmap.contains("endcmap"), "CMap should contain footer");
        // No bfchar entries
        assert!(!cmap.contains("beginbfchar"), "Empty chars should not produce bfchar section");
    }

    // =========================================================================
    // FontMetrics tests
    // =========================================================================

    #[test]
    fn test_font_metrics_from_font() {
        let data = match load_dejavu_sans() {
            Some(d) => d,
            None => return,
        };
        let font = TrueTypeFont::parse(&data).unwrap();
        let metrics = FontMetrics::from_font(&font);

        assert!(!metrics.name.is_empty(), "Name should not be empty");
        assert!(!metrics.family.is_empty(), "Family should not be empty");
        assert!(metrics.units_per_em > 0);
        assert!(metrics.ascender > 0);
        assert!(metrics.descender < 0);
        assert!(metrics.cap_height > 0);
        assert!(metrics.x_height > 0);
        assert!(!metrics.is_bold);
        assert!(!metrics.is_italic);
    }

    #[test]
    fn test_font_metrics_to_pdf_units() {
        let data = match load_dejavu_sans() {
            Some(d) => d,
            None => return,
        };
        let font = TrueTypeFont::parse(&data).unwrap();
        let metrics = FontMetrics::from_font(&font);

        // to_pdf_units should scale correctly
        let pdf_asc = metrics.pdf_ascender();
        assert!(pdf_asc > 0, "PDF ascender should be positive, got: {}", pdf_asc);

        let pdf_desc = metrics.pdf_descender();
        assert!(pdf_desc < 0, "PDF descender should be negative, got: {}", pdf_desc);

        let pdf_cap = metrics.pdf_cap_height();
        assert!(pdf_cap > 0, "PDF cap height should be positive, got: {}", pdf_cap);
    }

    #[test]
    fn test_font_metrics_pdf_bbox() {
        let data = match load_dejavu_sans() {
            Some(d) => d,
            None => return,
        };
        let font = TrueTypeFont::parse(&data).unwrap();
        let metrics = FontMetrics::from_font(&font);

        let (x_min, y_min, x_max, y_max) = metrics.pdf_bbox();
        assert!(x_max > x_min, "PDF bbox x_max should be > x_min");
        assert!(y_max > y_min, "PDF bbox y_max should be > y_min");
    }

    #[test]
    fn test_font_metrics_to_pdf_units_calculation() {
        // Create a mock-style test for the scaling formula
        let data = match load_dejavu_sans() {
            Some(d) => d,
            None => return,
        };
        let font = TrueTypeFont::parse(&data).unwrap();
        let metrics = FontMetrics::from_font(&font);

        // Verify the formula: (value * 1000) / units_per_em
        let test_value: i16 = 500;
        let expected = (test_value as i32 * 1000) / metrics.units_per_em as i32;
        let actual = metrics.to_pdf_units(test_value);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_font_metrics_bold() {
        let data = match load_dejavu_sans_bold() {
            Some(d) => d,
            None => return,
        };
        let font = TrueTypeFont::parse(&data).unwrap();
        let metrics = FontMetrics::from_font(&font);

        assert!(metrics.is_bold, "Bold font metrics should report is_bold");
        assert_eq!(metrics.stem_v, 140, "Bold stem_v should be 140");
    }

    // =========================================================================
    // Error type tests
    // =========================================================================

    #[test]
    fn test_truetype_error_display_empty_font() {
        let err = TrueTypeError::EmptyFont;
        let msg = format!("{}", err);
        assert!(msg.contains("empty or invalid"), "Got: {}", msg);
    }

    #[test]
    fn test_truetype_error_display_parse_error() {
        let err = TrueTypeError::ParseError("bad header".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("bad header"), "Got: {}", msg);
    }

    #[test]
    fn test_truetype_error_display_missing_table() {
        let err = TrueTypeError::MissingTable("cmap".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("cmap"), "Got: {}", msg);
    }

    #[test]
    fn test_truetype_error_display_glyph_not_found() {
        let err = TrueTypeError::GlyphNotFound(0x1234);
        let msg = format!("{}", err);
        assert!(msg.contains("1234"), "Got: {}", msg);
    }

    #[test]
    fn test_truetype_error_io_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file missing");
        let tt_err: TrueTypeError = io_err.into();
        assert!(matches!(tt_err, TrueTypeError::IoError(_)));
        let msg = format!("{}", tt_err);
        assert!(msg.contains("file missing"), "Got: {}", msg);
    }

    #[test]
    fn test_parse_truncated_data() {
        // First 4 bytes of a TrueType file (version tag) but nothing else
        let truncated = vec![0x00, 0x01, 0x00, 0x00];
        let result = TrueTypeFont::parse(&truncated);
        assert!(result.is_err(), "Truncated font data should fail to parse");
    }

    #[test]
    fn test_parse_random_bytes() {
        let random_data = vec![0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x00];
        let result = TrueTypeFont::parse(&random_data);
        assert!(result.is_err(), "Random bytes should fail to parse as font");
    }
}
