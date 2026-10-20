//! Font management for PDF generation.
//!
//! This module provides font metrics and management for accurate
//! text positioning in generated PDFs.
//!
//! # Embedded Fonts (v0.3.0)
//!
//! Supports embedding TrueType/OpenType fonts for full Unicode support.
//! Per PDF spec Section 9.6-9.9, embedded fonts use:
//! - CIDFont (Type 2) for TrueType fonts
//! - Identity-H encoding for Unicode
//! - ToUnicode CMap for text extraction
//! - Font subsetting for reduced file size

use std::collections::HashMap;
use std::sync::Arc;

/// Font manager for PDF generation.
///
/// Manages fonts and provides metrics for accurate text layout.
/// Currently supports PDF Base-14 fonts with standard metrics.
#[derive(Debug, Clone)]
pub struct FontManager {
    /// Registered fonts (name -> font info)
    fonts: HashMap<String, FontInfo>,
    /// Font ID counter for resource naming
    next_font_id: u32,
}

impl FontManager {
    /// Create a new font manager with Base-14 fonts.
    pub fn new() -> Self {
        let mut manager = Self {
            fonts: HashMap::new(),
            next_font_id: 1,
        };

        // Register PDF Base-14 fonts
        manager.register_base14_fonts();
        manager
    }

    /// Register the PDF Base-14 standard fonts.
    fn register_base14_fonts(&mut self) {
        // Helvetica family
        self.register_font(FontInfo::base14(
            "Helvetica",
            FontFamily::Helvetica,
            FontWeight::Normal,
            false,
        ));
        self.register_font(FontInfo::base14(
            "Helvetica-Bold",
            FontFamily::Helvetica,
            FontWeight::Bold,
            false,
        ));
        self.register_font(FontInfo::base14(
            "Helvetica-Oblique",
            FontFamily::Helvetica,
            FontWeight::Normal,
            true,
        ));
        self.register_font(FontInfo::base14(
            "Helvetica-BoldOblique",
            FontFamily::Helvetica,
            FontWeight::Bold,
            true,
        ));

        // Times family
        self.register_font(FontInfo::base14(
            "Times-Roman",
            FontFamily::Times,
            FontWeight::Normal,
            false,
        ));
        self.register_font(FontInfo::base14(
            "Times-Bold",
            FontFamily::Times,
            FontWeight::Bold,
            false,
        ));
        self.register_font(FontInfo::base14(
            "Times-Italic",
            FontFamily::Times,
            FontWeight::Normal,
            true,
        ));
        self.register_font(FontInfo::base14(
            "Times-BoldItalic",
            FontFamily::Times,
            FontWeight::Bold,
            true,
        ));

        // Courier family
        self.register_font(FontInfo::base14(
            "Courier",
            FontFamily::Courier,
            FontWeight::Normal,
            false,
        ));
        self.register_font(FontInfo::base14(
            "Courier-Bold",
            FontFamily::Courier,
            FontWeight::Bold,
            false,
        ));
        self.register_font(FontInfo::base14(
            "Courier-Oblique",
            FontFamily::Courier,
            FontWeight::Normal,
            true,
        ));
        self.register_font(FontInfo::base14(
            "Courier-BoldOblique",
            FontFamily::Courier,
            FontWeight::Bold,
            true,
        ));

        // Symbol and ZapfDingbats
        self.register_font(FontInfo::base14_symbol("Symbol"));
        self.register_font(FontInfo::base14_symbol("ZapfDingbats"));
    }

    /// Register a font.
    fn register_font(&mut self, font: FontInfo) {
        self.fonts.insert(font.name.clone(), font);
    }

    /// Get font info by name.
    pub fn get_font(&self, name: &str) -> Option<&FontInfo> {
        self.fonts.get(name)
    }

    /// Get font info, falling back to Helvetica if not found.
    pub fn get_font_or_default(&self, name: &str) -> &FontInfo {
        self.fonts.get(name).unwrap_or_else(|| {
            self.fonts
                .get("Helvetica")
                .expect("Helvetica must be registered")
        })
    }

    /// Calculate the width of a string in the given font at the given size.
    ///
    /// Returns width in points.
    pub fn text_width(&self, text: &str, font_name: &str, font_size: f32) -> f32 {
        let font = self.get_font_or_default(font_name);
        font.text_width(text, font_size)
    }

    /// Calculate the width of a single character.
    pub fn char_width(&self, ch: char, font_name: &str, font_size: f32) -> f32 {
        let font = self.get_font_or_default(font_name);
        font.char_width(ch) * font_size / 1000.0
    }

    /// Get the next available font resource ID.
    pub fn next_font_resource_id(&mut self) -> String {
        let id = format!("F{}", self.next_font_id);
        self.next_font_id += 1;
        id
    }

    /// Check if a font name corresponds to a Base-14 font.
    pub fn is_base14(&self, name: &str) -> bool {
        self.fonts.get(name).map(|f| f.is_base14).unwrap_or(false)
    }

    /// Get all registered font names.
    pub fn font_names(&self) -> Vec<&str> {
        self.fonts.keys().map(|s| s.as_str()).collect()
    }

    /// Select the best matching font for the given criteria.
    pub fn select_font(&self, family: FontFamily, weight: FontWeight, italic: bool) -> &str {
        (match (family, weight, italic) {
            (FontFamily::Helvetica, FontWeight::Normal, false) => "Helvetica",
            (FontFamily::Helvetica, FontWeight::Bold, false) => "Helvetica-Bold",
            (FontFamily::Helvetica, FontWeight::Normal, true) => "Helvetica-Oblique",
            (FontFamily::Helvetica, FontWeight::Bold, true) => "Helvetica-BoldOblique",
            (FontFamily::Times, FontWeight::Normal, false) => "Times-Roman",
            (FontFamily::Times, FontWeight::Bold, false) => "Times-Bold",
            (FontFamily::Times, FontWeight::Normal, true) => "Times-Italic",
            (FontFamily::Times, FontWeight::Bold, true) => "Times-BoldItalic",
            (FontFamily::Courier, FontWeight::Normal, false) => "Courier",
            (FontFamily::Courier, FontWeight::Bold, false) => "Courier-Bold",
            (FontFamily::Courier, FontWeight::Normal, true) => "Courier-Oblique",
            (FontFamily::Courier, FontWeight::Bold, true) => "Courier-BoldOblique",
            (FontFamily::Symbol, _, _) => "Symbol",
            (FontFamily::ZapfDingbats, _, _) => "ZapfDingbats",
        }) as _
    }
}

impl Default for FontManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Font family classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontFamily {
    /// Helvetica (sans-serif)
    Helvetica,
    /// Times (serif)
    Times,
    /// Courier (monospace)
    Courier,
    /// Symbol
    Symbol,
    /// ZapfDingbats
    ZapfDingbats,
}

/// Font weight classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FontWeight {
    /// Normal weight
    #[default]
    Normal,
    /// Bold weight
    Bold,
}

/// Information about a font.
#[derive(Debug, Clone)]
pub struct FontInfo {
    /// Font name (e.g., "Helvetica-Bold")
    pub name: String,
    /// Font family
    pub family: FontFamily,
    /// Font weight
    pub weight: FontWeight,
    /// Whether the font is italic/oblique
    pub italic: bool,
    /// Whether this is a Base-14 font
    pub is_base14: bool,
    /// Character widths (glyph index -> width in 1/1000 of font size)
    widths: FontWidths,
    /// Ascender height (above baseline)
    pub ascender: f32,
    /// Descender depth (below baseline, negative)
    pub descender: f32,
    /// Line gap (extra space between lines)
    pub line_gap: f32,
    /// Cap height (height of capital letters)
    pub cap_height: f32,
    /// x-height (height of lowercase x)
    pub x_height: f32,
}

impl FontInfo {
    /// Create a Base-14 font info.
    fn base14(name: &str, family: FontFamily, weight: FontWeight, italic: bool) -> Self {
        let widths = FontWidths::for_base14(name);
        let metrics = base14_metrics(name);

        Self {
            name: name.to_string(),
            family,
            weight,
            italic,
            is_base14: true,
            widths,
            ascender: metrics.0,
            descender: metrics.1,
            line_gap: metrics.2,
            cap_height: metrics.3,
            x_height: metrics.4,
        }
    }

    /// Create a Base-14 symbol font.
    fn base14_symbol(name: &str) -> Self {
        Self {
            name: name.to_string(),
            family: if name == "Symbol" {
                FontFamily::Symbol
            } else {
                FontFamily::ZapfDingbats
            },
            weight: FontWeight::Normal,
            italic: false,
            is_base14: true,
            widths: FontWidths::Symbol,
            ascender: 800.0,
            descender: -200.0,
            line_gap: 0.0,
            cap_height: 700.0,
            x_height: 500.0,
        }
    }

    /// Calculate the width of text in this font.
    ///
    /// Returns width in points for the given font size.
    pub fn text_width(&self, text: &str, font_size: f32) -> f32 {
        let width_units: f32 = text.chars().map(|c| self.char_width(c)).sum();
        width_units * font_size / 1000.0
    }

    /// Get the width of a single character in font units (1/1000 of em).
    pub fn char_width(&self, ch: char) -> f32 {
        self.widths.width_for_char(ch)
    }

    /// Get the line height for this font at the given size.
    pub fn line_height(&self, font_size: f32) -> f32 {
        (self.ascender - self.descender + self.line_gap) * font_size / 1000.0
    }

    /// Get recommended line spacing multiplier.
    pub fn line_spacing_factor(&self) -> f32 {
        1.2 // Standard 120% line height
    }
}

/// Font width data.
#[derive(Debug, Clone)]
enum FontWidths {
    /// Proportional font with per-character widths
    Proportional(HashMap<char, f32>),
    /// Monospace font with fixed width
    Monospace(f32),
    /// Symbol font (use default width)
    Symbol,
}

impl FontWidths {
    /// Get widths for a Base-14 font.
    fn for_base14(name: &str) -> Self {
        match name {
            "Courier" | "Courier-Bold" | "Courier-Oblique" | "Courier-BoldOblique" => {
                FontWidths::Monospace(600.0)
            },
            "Symbol" | "ZapfDingbats" => FontWidths::Symbol,
            _ => FontWidths::Proportional(get_base14_widths(name)),
        }
    }

    /// Get width for a character.
    fn width_for_char(&self, ch: char) -> f32 {
        match self {
            FontWidths::Proportional(widths) => {
                *widths.get(&ch).unwrap_or(&500.0) // Default to 500 for unknown chars
            },
            FontWidths::Monospace(width) => *width,
            FontWidths::Symbol => 500.0,
        }
    }
}

/// Get metrics for a Base-14 font: (ascender, descender, line_gap, cap_height, x_height)
fn base14_metrics(name: &str) -> (f32, f32, f32, f32, f32) {
    match name {
        "Helvetica" | "Helvetica-Oblique" => (718.0, -207.0, 0.0, 718.0, 523.0),
        "Helvetica-Bold" | "Helvetica-BoldOblique" => (718.0, -207.0, 0.0, 718.0, 532.0),
        "Times-Roman" | "Times-Italic" => (683.0, -217.0, 0.0, 662.0, 450.0),
        "Times-Bold" | "Times-BoldItalic" => (676.0, -205.0, 0.0, 676.0, 461.0),
        "Courier" | "Courier-Oblique" => (629.0, -157.0, 0.0, 562.0, 426.0),
        "Courier-Bold" | "Courier-BoldOblique" => (626.0, -142.0, 0.0, 562.0, 439.0),
        _ => (750.0, -250.0, 0.0, 700.0, 500.0), // Default metrics
    }
}

/// Get character widths for Base-14 proportional fonts.
///
/// These are standard PostScript/PDF metrics in units of 1/1000 em.
fn get_base14_widths(name: &str) -> HashMap<char, f32> {
    let mut widths = HashMap::new();

    // Common ASCII characters with approximate standard widths
    // These are based on standard PostScript font metrics

    let (space_w, period_w, comma_w, hyphen_w, colon_w) = match name {
        "Helvetica" | "Helvetica-Oblique" => (278.0, 278.0, 278.0, 333.0, 278.0),
        "Helvetica-Bold" | "Helvetica-BoldOblique" => (278.0, 278.0, 278.0, 333.0, 333.0),
        "Times-Roman" | "Times-Italic" => (250.0, 250.0, 250.0, 333.0, 278.0),
        "Times-Bold" | "Times-BoldItalic" => (250.0, 250.0, 250.0, 333.0, 333.0),
        _ => (250.0, 250.0, 250.0, 333.0, 278.0),
    };

    // Whitespace and punctuation
    widths.insert(' ', space_w);
    widths.insert('.', period_w);
    widths.insert(',', comma_w);
    widths.insert('-', hyphen_w);
    widths.insert(':', colon_w);
    widths.insert(';', 278.0);
    widths.insert('!', 333.0);
    widths.insert('?', 500.0);
    widths.insert('\'', 222.0);
    widths.insert('"', 400.0);
    widths.insert('(', 333.0);
    widths.insert(')', 333.0);
    widths.insert('[', 333.0);
    widths.insert(']', 333.0);
    widths.insert('{', 333.0);
    widths.insert('}', 333.0);
    widths.insert('/', 278.0);
    widths.insert('\\', 278.0);
    widths.insert('@', 800.0);
    widths.insert('#', 556.0);
    widths.insert('$', 556.0);
    widths.insert('%', 889.0);
    widths.insert('^', 500.0);
    widths.insert('&', 722.0);
    widths.insert('*', 389.0);
    widths.insert('+', 584.0);
    widths.insert('=', 584.0);
    widths.insert('<', 584.0);
    widths.insert('>', 584.0);
    widths.insert('|', 280.0);
    widths.insert('`', 333.0);
    widths.insert('~', 584.0);
    widths.insert('_', 556.0);

    // Numbers (fairly consistent across fonts)
    for digit in '0'..='9' {
        widths.insert(digit, 556.0);
    }

    // Uppercase letters - Helvetica-style widths
    let uppercase_widths = match name {
        "Helvetica" | "Helvetica-Oblique" => [
            ('A', 722.0),
            ('B', 722.0),
            ('C', 722.0),
            ('D', 722.0),
            ('E', 667.0),
            ('F', 611.0),
            ('G', 778.0),
            ('H', 722.0),
            ('I', 278.0),
            ('J', 556.0),
            ('K', 722.0),
            ('L', 611.0),
            ('M', 833.0),
            ('N', 722.0),
            ('O', 778.0),
            ('P', 667.0),
            ('Q', 778.0),
            ('R', 722.0),
            ('S', 667.0),
            ('T', 611.0),
            ('U', 722.0),
            ('V', 667.0),
            ('W', 944.0),
            ('X', 667.0),
            ('Y', 667.0),
            ('Z', 611.0),
        ],
        "Helvetica-Bold" | "Helvetica-BoldOblique" => [
            ('A', 722.0),
            ('B', 722.0),
            ('C', 722.0),
            ('D', 722.0),
            ('E', 667.0),
            ('F', 611.0),
            ('G', 778.0),
            ('H', 722.0),
            ('I', 278.0),
            ('J', 556.0),
            ('K', 722.0),
            ('L', 611.0),
            ('M', 833.0),
            ('N', 722.0),
            ('O', 778.0),
            ('P', 667.0),
            ('Q', 778.0),
            ('R', 722.0),
            ('S', 667.0),
            ('T', 611.0),
            ('U', 722.0),
            ('V', 667.0),
            ('W', 944.0),
            ('X', 667.0),
            ('Y', 667.0),
            ('Z', 611.0),
        ],
        "Times-Roman" | "Times-Italic" => [
            ('A', 722.0),
            ('B', 667.0),
            ('C', 667.0),
            ('D', 722.0),
            ('E', 611.0),
            ('F', 556.0),
            ('G', 722.0),
            ('H', 722.0),
            ('I', 333.0),
            ('J', 389.0),
            ('K', 722.0),
            ('L', 611.0),
            ('M', 889.0),
            ('N', 722.0),
            ('O', 722.0),
            ('P', 556.0),
            ('Q', 722.0),
            ('R', 667.0),
            ('S', 556.0),
            ('T', 611.0),
            ('U', 722.0),
            ('V', 722.0),
            ('W', 944.0),
            ('X', 722.0),
            ('Y', 722.0),
            ('Z', 611.0),
        ],
        "Times-Bold" | "Times-BoldItalic" => [
            ('A', 722.0),
            ('B', 667.0),
            ('C', 722.0),
            ('D', 722.0),
            ('E', 667.0),
            ('F', 611.0),
            ('G', 778.0),
            ('H', 778.0),
            ('I', 389.0),
            ('J', 500.0),
            ('K', 778.0),
            ('L', 667.0),
            ('M', 944.0),
            ('N', 722.0),
            ('O', 778.0),
            ('P', 611.0),
            ('Q', 778.0),
            ('R', 722.0),
            ('S', 556.0),
            ('T', 667.0),
            ('U', 722.0),
            ('V', 722.0),
            ('W', 1000.0),
            ('X', 722.0),
            ('Y', 722.0),
            ('Z', 667.0),
        ],
        _ => [
            ('A', 722.0),
            ('B', 667.0),
            ('C', 667.0),
            ('D', 722.0),
            ('E', 611.0),
            ('F', 556.0),
            ('G', 722.0),
            ('H', 722.0),
            ('I', 333.0),
            ('J', 389.0),
            ('K', 722.0),
            ('L', 611.0),
            ('M', 889.0),
            ('N', 722.0),
            ('O', 722.0),
            ('P', 556.0),
            ('Q', 722.0),
            ('R', 667.0),
            ('S', 556.0),
            ('T', 611.0),
            ('U', 722.0),
            ('V', 722.0),
            ('W', 944.0),
            ('X', 722.0),
            ('Y', 722.0),
            ('Z', 611.0),
        ],
    };

    for (ch, w) in uppercase_widths {
        widths.insert(ch, w);
    }

    // Lowercase letters - Helvetica-style widths
    let lowercase_widths = match name {
        "Helvetica" | "Helvetica-Oblique" => [
            ('a', 556.0),
            ('b', 611.0),
            ('c', 556.0),
            ('d', 611.0),
            ('e', 556.0),
            ('f', 278.0),
            ('g', 611.0),
            ('h', 611.0),
            ('i', 222.0),
            ('j', 222.0),
            ('k', 556.0),
            ('l', 222.0),
            ('m', 833.0),
            ('n', 611.0),
            ('o', 611.0),
            ('p', 611.0),
            ('q', 611.0),
            ('r', 389.0),
            ('s', 556.0),
            ('t', 333.0),
            ('u', 611.0),
            ('v', 556.0),
            ('w', 778.0),
            ('x', 556.0),
            ('y', 556.0),
            ('z', 500.0),
        ],
        "Helvetica-Bold" | "Helvetica-BoldOblique" => [
            ('a', 556.0),
            ('b', 611.0),
            ('c', 556.0),
            ('d', 611.0),
            ('e', 556.0),
            ('f', 333.0),
            ('g', 611.0),
            ('h', 611.0),
            ('i', 278.0),
            ('j', 278.0),
            ('k', 556.0),
            ('l', 278.0),
            ('m', 889.0),
            ('n', 611.0),
            ('o', 611.0),
            ('p', 611.0),
            ('q', 611.0),
            ('r', 389.0),
            ('s', 556.0),
            ('t', 333.0),
            ('u', 611.0),
            ('v', 556.0),
            ('w', 778.0),
            ('x', 556.0),
            ('y', 556.0),
            ('z', 500.0),
        ],
        "Times-Roman" | "Times-Italic" => [
            ('a', 444.0),
            ('b', 500.0),
            ('c', 444.0),
            ('d', 500.0),
            ('e', 444.0),
            ('f', 333.0),
            ('g', 500.0),
            ('h', 500.0),
            ('i', 278.0),
            ('j', 278.0),
            ('k', 500.0),
            ('l', 278.0),
            ('m', 778.0),
            ('n', 500.0),
            ('o', 500.0),
            ('p', 500.0),
            ('q', 500.0),
            ('r', 333.0),
            ('s', 389.0),
            ('t', 278.0),
            ('u', 500.0),
            ('v', 500.0),
            ('w', 722.0),
            ('x', 500.0),
            ('y', 500.0),
            ('z', 444.0),
        ],
        "Times-Bold" | "Times-BoldItalic" => [
            ('a', 500.0),
            ('b', 556.0),
            ('c', 444.0),
            ('d', 556.0),
            ('e', 444.0),
            ('f', 333.0),
            ('g', 500.0),
            ('h', 556.0),
            ('i', 278.0),
            ('j', 333.0),
            ('k', 556.0),
            ('l', 278.0),
            ('m', 833.0),
            ('n', 556.0),
            ('o', 500.0),
            ('p', 556.0),
            ('q', 556.0),
            ('r', 444.0),
            ('s', 389.0),
            ('t', 333.0),
            ('u', 556.0),
            ('v', 500.0),
            ('w', 722.0),
            ('x', 500.0),
            ('y', 500.0),
            ('z', 444.0),
        ],
        _ => [
            ('a', 444.0),
            ('b', 500.0),
            ('c', 444.0),
            ('d', 500.0),
            ('e', 444.0),
            ('f', 333.0),
            ('g', 500.0),
            ('h', 500.0),
            ('i', 278.0),
            ('j', 278.0),
            ('k', 500.0),
            ('l', 278.0),
            ('m', 778.0),
            ('n', 500.0),
            ('o', 500.0),
            ('p', 500.0),
            ('q', 500.0),
            ('r', 333.0),
            ('s', 389.0),
            ('t', 278.0),
            ('u', 500.0),
            ('v', 500.0),
            ('w', 722.0),
            ('x', 500.0),
            ('y', 500.0),
            ('z', 444.0),
        ],
    };

    for (ch, w) in lowercase_widths {
        widths.insert(ch, w);
    }

    widths
}

/// Text layout helper for calculating text positioning.
#[derive(Debug)]
pub struct TextLayout {
    /// Font manager reference
    font_manager: FontManager,
}

impl TextLayout {
    /// Create a new text layout helper.
    pub fn new() -> Self {
        Self {
            font_manager: FontManager::new(),
        }
    }

    /// Create with a specific font manager.
    pub fn with_font_manager(font_manager: FontManager) -> Self {
        Self { font_manager }
    }

    /// Calculate wrapped lines for text within a given width.
    ///
    /// Returns a vector of (line_text, line_width) pairs.
    pub fn wrap_text(
        &self,
        text: &str,
        font_name: &str,
        font_size: f32,
        max_width: f32,
    ) -> Vec<(String, f32)> {
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0.0;
        let space_width = self.font_manager.char_width(' ', font_name, font_size);

        for word in text.split_whitespace() {
            let word_width = self.font_manager.text_width(word, font_name, font_size);

            if current_line.is_empty() {
                // First word on line
                current_line = word.to_string();
                current_width = word_width;
            } else if current_width + space_width + word_width <= max_width {
                // Word fits on current line
                current_line.push(' ');
                current_line.push_str(word);
                current_width += space_width + word_width;
            } else {
                // Word doesn't fit, start new line
                lines.push((current_line, current_width));
                current_line = word.to_string();
                current_width = word_width;
            }
        }

        // Don't forget the last line
        if !current_line.is_empty() {
            // Preserve a trailing space if the source ended with one. Callers
            // (e.g. StreamingTable cell text) rely on a trailing space so
            // adjacent cells don't merge in the extracted text. `split_whitespace`
            // discards that information, so we restore it here.
            if text.ends_with(char::is_whitespace) && !current_line.ends_with(' ') {
                current_line.push(' ');
                current_width += space_width;
            }
            lines.push((current_line, current_width));
        }

        if lines.is_empty() {
            lines.push((String::new(), 0.0));
        }

        lines
    }

    /// Calculate the bounding box dimensions for wrapped text.
    pub fn text_bounds(
        &self,
        text: &str,
        font_name: &str,
        font_size: f32,
        max_width: f32,
    ) -> (f32, f32) {
        let lines = self.wrap_text(text, font_name, font_size, max_width);
        let font = self.font_manager.get_font_or_default(font_name);
        let line_height = font.line_height(font_size) * font.line_spacing_factor();

        let max_line_width = lines.iter().map(|(_, w)| *w).fold(0.0_f32, f32::max);
        let total_height = lines.len() as f32 * line_height;

        (max_line_width, total_height)
    }

    /// Get the font manager.
    pub fn font_manager(&self) -> &FontManager {
        &self.font_manager
    }
}

impl Default for TextLayout {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Embedded Font Support (v0.3.0)
// =============================================================================

use crate::fonts::{FontSubsetter, TrueTypeFont, UnicodeEncoder};

/// Embedded TrueType font for PDF generation.
///
/// Contains parsed font data, subsetter, and encoder for generating
/// CIDFont dictionaries and ToUnicode CMaps.
#[derive(Debug)]
pub struct EmbeddedFont {
    /// Font name (PostScript name or user-provided)
    pub name: String,
    /// Subset name with tag (e.g., "ABCDEF+Arial")
    subset_name: Option<String>,
    /// Raw font data (for embedding)
    font_data: Arc<Vec<u8>>,
    /// Subsetter tracking used glyphs
    subsetter: FontSubsetter,
    /// Unicode encoder for text
    #[allow(dead_code)]
    encoder: UnicodeEncoder,
    /// Cached glyph lookup (Unicode -> GID)
    glyph_lookup: HashMap<u32, u16>,
    /// Cached glyph widths (GID -> width in 1/1000 em)
    glyph_widths: HashMap<u16, u16>,
    /// Font ascender value
    pub ascender: i32,
    /// Font descender value
    pub descender: i32,
    /// Cap height (height of capital letters)
    pub cap_height: i32,
    /// x-height (height of lowercase x)
    pub x_height: i32,
    /// Font bounding box (llx, lly, urx, ury)
    pub bbox: (i32, i32, i32, i32),
    /// Font flags for PDF
    pub flags: u32,
    /// Stem vertical width
    pub stem_v: i16,
    /// Italic angle in degrees
    pub italic_angle: f32,
    /// Units per em
    #[allow(dead_code)]
    units_per_em: u16,
}

impl EmbeddedFont {
    /// Create an embedded font from raw TTF/OTF data.
    ///
    /// # Arguments
    /// * `name` - Font name to use (if None, uses PostScript name from font)
    /// * `data` - Raw TTF/OTF file data
    pub fn from_data(name: Option<String>, data: Vec<u8>) -> Result<Self, String> {
        let font =
            TrueTypeFont::parse(&data).map_err(|e| format!("Failed to parse font: {}", e))?;

        let font_name = name.unwrap_or_else(|| {
            font.postscript_name()
                .unwrap_or_else(|| "Unknown".to_string())
        });

        // Extract metrics
        let metrics = crate::fonts::FontMetrics::from_font(&font);

        // Build glyph lookup
        let mut glyph_lookup = HashMap::new();
        let mut glyph_widths = HashMap::new();

        for codepoint in font.supported_codepoints() {
            if let Some(gid) = font.glyph_id(codepoint) {
                glyph_lookup.insert(codepoint, gid);
                glyph_widths.insert(gid, font.glyph_width(gid));
            }
        }

        Ok(Self {
            name: font_name,
            subset_name: None,
            font_data: Arc::new(data),
            subsetter: FontSubsetter::new(),
            encoder: UnicodeEncoder::new(),
            glyph_lookup,
            glyph_widths,
            ascender: metrics.pdf_ascender(),
            descender: metrics.pdf_descender(),
            cap_height: metrics.pdf_cap_height(),
            x_height: metrics.to_pdf_units(metrics.x_height),
            bbox: metrics.pdf_bbox(),
            flags: metrics.flags,
            stem_v: metrics.stem_v,
            italic_angle: metrics.italic_angle,
            units_per_em: metrics.units_per_em,
        })
    }

    /// Load an embedded font from a file.
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self, String> {
        let data =
            std::fs::read(path.as_ref()).map_err(|e| format!("Failed to read font file: {}", e))?;
        Self::from_data(None, data)
    }

    /// Override the Unicode→GID lookup with externally-supplied
    /// mappings.
    ///
    /// PDF→office round-trips often re-embed CFF font subsets that
    /// have no Unicode cmap (CID-only encoding). `from_data`
    /// successfully parses the OTF wrapper but `glyph_lookup`
    /// remains empty, so `has_usable_unicode_cmap()` returns false
    /// and the font is skipped — falling back to base-14 Helvetica
    /// across the whole document.
    ///
    /// Caller can recover the missing mapping by inverting the
    /// source PDF's `/ToUnicode` CMap (byte-code → Unicode) against
    /// the font's CFF byte→GID table (`cff_gid_map` in pdf_oxide's
    /// FontInfo) to produce Unicode → GID, then pass it here. The
    /// font then registers and renders correctly with the source
    /// typeface program rather than Helvetica fallback.
    ///
    /// Existing entries in `glyph_lookup` are kept; the override
    /// extends them. Glyph widths are filled in for newly-mapped
    /// GIDs from the underlying font's hmtx table on the fly.
    pub fn extend_glyph_lookup(&mut self, mapping: HashMap<u32, u16>) {
        // Re-parse once so we can pull widths only for newly-introduced GIDs.
        let face = TrueTypeFont::parse(self.font_data.as_ref()).ok();
        for (codepoint, gid) in mapping {
            self.glyph_lookup.insert(codepoint, gid);
            self.glyph_widths.entry(gid).or_insert_with(|| {
                face.as_ref()
                    .map(|font| font.glyph_width(gid))
                    .unwrap_or(500) // sensible default for missing widths (1000ths em)
            });
        }
    }

    /// True when the font carries enough Unicode→GID coverage to be
    /// useful for plain Latin / Latin-1 text rendering. PDF→office
    /// round-trips that re-embed source-PDF font subsets often hit a
    /// pathological case: the source font is a CID-encoded subset
    /// without a Unicode cmap, so `glyph_lookup` is empty even though
    /// the font program parsed successfully. Registering such a font
    /// makes downstream text emission route every glyph through GID 0
    /// (.notdef) — visible as missing-glyph squares, or invisible
    /// when the font program has no .notdef outline at all.
    ///
    /// Non-Latin fonts qualify as long as they expose at least one
    /// non-zero Unicode→GID mapping; ASCII coverage is not required.
    pub fn has_usable_unicode_cmap(&self) -> bool {
        self.glyph_lookup.values().any(|&gid| gid != 0)
    }

    /// Get the glyph ID for a Unicode codepoint, with cascading fallbacks:
    ///
    /// 1. Direct lookup in the font's cmap.
    /// 2. Mathematical Alphanumeric Symbols (U+1D400-1D7FF) — italic / bold
    ///    / script / fraktur / etc. styled letters used in formulae —
    ///    collapse to their plain Latin/Greek base via
    ///    `math_alphanumeric_base`. Lossless for word-level text recovery.
    /// 3. Common typographic chars (smart quotes, dashes, bullets, NBSP)
    ///    fall back to ASCII equivalents the standard fonts always have.
    ///
    /// Without these fallbacks, missing chars get GID 0 (.notdef) which
    /// renders as a missing-glyph box and disappears from extracted text.
    pub fn glyph_id(&self, codepoint: u32) -> Option<u16> {
        if let Some(gid) = self.glyph_lookup.get(&codepoint).copied() {
            return Some(gid);
        }
        // Math-alphanumeric → plain Latin/Greek.
        if let Some(base) = crate::fonts::encoding::math_alphanumeric_base(codepoint) {
            if let Some(gid) = self.glyph_lookup.get(&base).copied() {
                return Some(gid);
            }
            // Fall through with base for the typographic-fallback table below
            // (e.g. bold dash maps to base then through the dash fallback).
            return self.glyph_id_fallback(base);
        }
        self.glyph_id_fallback(codepoint)
    }

    fn glyph_id_fallback(&self, codepoint: u32) -> Option<u16> {
        let fallback = match codepoint {
            // Smart quotes → straight
            0x2018..=0x201B => Some(0x27u32), // '
            0x201C..=0x201F => Some(0x22),    // "
            // Dashes
            0x2013 | 0x2014 | 0x2212 => Some(0x2D), // -
            // Non-breaking spaces
            0x00A0 | 0x2007 | 0x202F | 0x2009 | 0x200A => Some(0x20), // space
            // Bullets
            0x2022 | 0x2023 | 0x25E6 => Some(0x2A), // *
            _ => None,
        };
        fallback.and_then(|cp| self.glyph_lookup.get(&cp).copied())
    }

    /// Get the width of a glyph in 1/1000 em units.
    pub fn glyph_width(&self, gid: u16) -> u16 {
        self.glyph_widths.get(&gid).copied().unwrap_or(500)
    }

    /// Get the width of a character in 1/1000 em units.
    pub fn char_width(&self, codepoint: u32) -> u16 {
        self.glyph_id(codepoint)
            .map(|gid| self.glyph_width(gid))
            .unwrap_or(500)
    }

    /// Calculate text width in points at the given font size.
    pub fn text_width(&self, text: &str, font_size: f32) -> f32 {
        let width_units: f32 = text.chars().map(|c| self.char_width(c as u32) as f32).sum();
        width_units * font_size / 1000.0
    }

    /// Record that a string is being used (for subsetting).
    pub fn use_string(&mut self, text: &str) {
        for ch in text.chars() {
            let codepoint = ch as u32;
            if let Some(gid) = self.glyph_id(codepoint) {
                self.subsetter.use_char(codepoint, gid);
            }
        }
    }

    /// Encode a string into a sequence of original-face glyph IDs for
    /// later emission into a PDF content stream.
    ///
    /// Each char is looked up in the font's cmap; unknown chars fall
    /// back to GID 0 (`.notdef`). The returned `Vec<u16>` is in logical
    /// text order and is passed verbatim to
    /// [`crate::writer::content_stream::ContentStreamBuilder::embedded_text`],
    /// which defers hex stringification until serialization time so
    /// the subset [`GlyphRemapper`](crate::fonts::GlyphRemapper) can
    /// renumber the GIDs into their subset-local indices.
    pub fn encode_string(&mut self, text: &str) -> Vec<u16> {
        self.use_string(text);
        text.chars()
            .map(|ch| self.glyph_id(ch as u32).unwrap_or(0))
            .collect()
    }

    /// Encode a [`crate::writer::font_shaping::ShapedRun`] as a sequence
    /// of original-face glyph IDs AND register each glyph with the
    /// subsetter under its source codepoint so the resulting PDF's
    /// ToUnicode CMap round-trips the text via `extract_text`.
    ///
    /// The shaper preserves `cluster` (a byte index into the source
    /// string) on every glyph. For simple scripts (CJK, Cyrillic, Greek,
    /// Latin) cluster is 1:1 with a single codepoint and ToUnicode is
    /// exact. For ligatures (one glyph covers multiple codepoints) we
    /// record the *first* codepoint of the cluster — close enough to
    /// recover the leading char on extract; exact multi-char ToUnicode
    /// is a later refinement.
    ///
    /// Returned GIDs are original-face IDs; see [`Self::encode_string`]
    /// for why serialization is deferred.
    #[cfg(feature = "system-fonts")]
    pub fn encode_shaped_run(
        &mut self,
        shaped: &crate::writer::font_shaping::ShapedRun,
        source_text: &str,
    ) -> Vec<u16> {
        let mut glyph_ids = Vec::with_capacity(shaped.glyphs.len());
        for g in &shaped.glyphs {
            // Some glyphs (e.g. notdef fallbacks) may map to an empty
            // cluster or off the end of the string — fall back to
            // `use_glyph` (no ToUnicode) rather than panic on bad bounds.
            let codepoint = source_text
                .get(g.cluster as usize..)
                .and_then(|tail| tail.chars().next())
                .map(|ch| ch as u32);
            if let Some(cp) = codepoint {
                self.subsetter.use_char(cp, g.glyph_id);
            } else {
                self.subsetter.use_glyph(g.glyph_id);
            }
            glyph_ids.push(g.glyph_id);
        }
        glyph_ids
    }

    /// Get the subset font name (generates tag if needed).
    pub fn subset_name(&mut self) -> &str {
        if self.subset_name.is_none() {
            self.subset_name = Some(self.subsetter.subset_font_name(&self.name));
        }
        self.subset_name.as_ref().unwrap()
    }

    /// Get the raw font data for embedding.
    pub fn font_data(&self) -> &[u8] {
        &self.font_data
    }

    /// Get the set of original-face glyph IDs this font has recorded as
    /// used during content-stream emission. The set is fed to
    /// [`crate::fonts::subset_font_bytes`] at finalize time to build the
    /// subset FontFile2 stream and the paired [`crate::fonts::GlyphRemapper`].
    pub fn used_glyphs(&self) -> &std::collections::BTreeSet<u16> {
        self.subsetter.used_glyphs()
    }

    /// Get subset statistics.
    pub fn subset_stats(&self) -> (usize, usize) {
        (self.subsetter.char_count(), self.subsetter.glyph_count())
    }

    /// Check if any text has been used with this font.
    pub fn is_used(&self) -> bool {
        !self.subsetter.is_empty()
    }

    /// Generate the CID widths array for the `/W` entry, keyed by
    /// *subset-local* GIDs produced by `remapper`.
    ///
    /// Widths themselves are fetched via [`Self::glyph_width`] which
    /// still reads from the original face's `glyph_widths` HashMap
    /// (original-GID keyed — unchanged by subsetting). The emission
    /// key, however, is the remapper-translated subset GID so that
    /// `/W` aligns with the `CIDFontType2` glyph indices in the
    /// subset font face and the GIDs emitted in the content stream.
    ///
    /// Because the remapper compacts kept glyphs to `0..N` roughly in
    /// original-GID order, consecutive original GIDs almost always
    /// produce consecutive subset GIDs — but not always, so we re-sort
    /// by subset GID before run-length grouping.
    pub fn generate_widths_array(&self, remapper: &crate::fonts::GlyphRemapper) -> String {
        // Collect (subset_gid, width) pairs, then group by consecutive
        // subset_gid for compact `/W` emission. When the remapper has
        // no mapping for a GID, the writer is emitting the original
        // GID unchanged (full-font fallback path) — so the /W key
        // must also use the original GID to keep the widths aligned
        // with what the content stream actually uses.
        let mut pairs: Vec<(u16, u16)> = self
            .subsetter
            .used_glyphs()
            .iter()
            .map(|&orig| {
                let subset_gid = remapper.get(orig).unwrap_or(orig);
                (subset_gid, self.glyph_width(orig))
            })
            .collect();
        pairs.sort_by_key(|&(gid, _)| gid);
        pairs.dedup_by_key(|&mut (gid, _)| gid);

        let mut result = String::from("[");
        let mut i = 0;
        while i < pairs.len() {
            let start = pairs[i].0;
            let mut widths = vec![pairs[i].1];

            // Find consecutive subset GIDs
            while i + 1 < pairs.len() && pairs[i + 1].0 == pairs[i].0 + 1 {
                i += 1;
                widths.push(pairs[i].1);
            }

            result.push_str(&format!("{} [", start));
            for (j, w) in widths.iter().enumerate() {
                if j > 0 {
                    result.push(' ');
                }
                result.push_str(&w.to_string());
            }
            result.push(']');

            i += 1;
        }

        result.push(']');
        result
    }

    /// Generate the ToUnicode CMap for text extraction, keyed by
    /// *subset-local* GIDs produced by `remapper`.
    ///
    /// `used_chars` tracks (codepoint, original-GID) pairs. We remap
    /// the GID key at emission time so that CMap `bfchar` entries and
    /// the content stream hex both reference the same subset GID
    /// space — a precondition for `extract_text` round-tripping.
    pub fn generate_tounicode_cmap(&self, remapper: &crate::fonts::GlyphRemapper) -> String {
        let used_chars = self.subsetter.used_chars();

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

        // Build subset-GID -> Unicode mappings. When the remapper
        // produces a mapping we use it (subset emission). When it
        // doesn't — the case our patched-CFF fallback hits with an
        // empty `GlyphRemapper::new()` so the content stream emits
        // ORIGINAL GIDs unchanged via `.unwrap_or(orig)` — fall back
        // to the original GID here too so the CMap key matches the
        // emitted CID. Without this, every CMap entry collapses to
        // CID 0 (.notdef) and `extract_text` returns 0 chars on the
        // round-trip PDF.
        let mut mappings: Vec<(u16, u32)> = used_chars
            .iter()
            .map(|(&unicode, &orig_gid)| {
                let key = remapper.get(orig_gid).unwrap_or(orig_gid);
                (key, unicode)
            })
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

/// Extended font manager with embedded font support.
#[derive(Debug, Default)]
pub struct EmbeddedFontManager {
    /// Embedded fonts by name
    fonts: HashMap<String, EmbeddedFont>,
    /// Font resource IDs (name -> resource ID like "F1")
    resource_ids: HashMap<String, String>,
    /// Next font resource ID number
    next_id: u32,
}

impl EmbeddedFontManager {
    /// Create a new embedded font manager.
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
            resource_ids: HashMap::new(),
            next_id: 1,
        }
    }

    /// Register an embedded font.
    ///
    /// # Arguments
    /// * `name` - Name to register the font under
    /// * `font` - The embedded font
    ///
    /// # Returns
    /// The font resource ID (e.g., "F1")
    pub fn register(&mut self, name: impl Into<String>, font: EmbeddedFont) -> String {
        let name = name.into();
        let resource_id = format!("F{}", self.next_id);
        self.next_id += 1;

        self.resource_ids.insert(name.clone(), resource_id.clone());
        self.fonts.insert(name, font);

        resource_id
    }

    /// Load and register a font from file.
    pub fn register_from_file(
        &mut self,
        name: impl Into<String>,
        path: impl AsRef<std::path::Path>,
    ) -> Result<String, String> {
        let font = EmbeddedFont::from_file(path)?;
        Ok(self.register(name, font))
    }

    /// Get a font by name.
    pub fn get(&self, name: &str) -> Option<&EmbeddedFont> {
        self.fonts.get(name)
    }

    /// Get a mutable font by name.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut EmbeddedFont> {
        self.fonts.get_mut(name)
    }

    /// Get the resource ID for a font.
    pub fn resource_id(&self, name: &str) -> Option<&str> {
        self.resource_ids.get(name).map(|s| s.as_str())
    }

    /// Iterate over all registered fonts.
    pub fn fonts(&self) -> impl Iterator<Item = (&str, &EmbeddedFont)> {
        self.fonts.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Iterate over all fonts with resource IDs.
    pub fn fonts_with_ids(&self) -> impl Iterator<Item = (&str, &str, &EmbeddedFont)> {
        self.fonts.iter().filter_map(|(name, font)| {
            self.resource_ids
                .get(name)
                .map(|id| (name.as_str(), id.as_str(), font))
        })
    }

    /// Get the number of registered fonts.
    pub fn len(&self) -> usize {
        self.fonts.len()
    }

    /// Check if any fonts are registered.
    pub fn is_empty(&self) -> bool {
        self.fonts.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_manager_creation() {
        let manager = FontManager::new();
        assert!(manager.get_font("Helvetica").is_some());
        assert!(manager.get_font("Times-Roman").is_some());
        assert!(manager.get_font("Courier").is_some());
    }

    #[test]
    fn test_base14_fonts() {
        let manager = FontManager::new();
        assert!(manager.is_base14("Helvetica"));
        assert!(manager.is_base14("Helvetica-Bold"));
        assert!(manager.is_base14("Times-Roman"));
        assert!(manager.is_base14("Courier"));
        assert!(!manager.is_base14("Arial")); // Not a Base-14 font
    }

    #[test]
    fn test_text_width_calculation() {
        let manager = FontManager::new();

        // "Hello" in Helvetica 12pt
        let width = manager.text_width("Hello", "Helvetica", 12.0);
        assert!(width > 0.0);
        assert!(width < 100.0); // Reasonable range for 5 chars at 12pt
    }

    #[test]
    fn test_monospace_consistency() {
        let manager = FontManager::new();

        // All characters should have the same width in Courier
        let w1 = manager.char_width('i', "Courier", 12.0);
        let w2 = manager.char_width('m', "Courier", 12.0);
        let w3 = manager.char_width('W', "Courier", 12.0);

        assert!((w1 - w2).abs() < 0.001);
        assert!((w2 - w3).abs() < 0.001);
    }

    #[test]
    fn test_proportional_variance() {
        let manager = FontManager::new();

        // 'i' should be narrower than 'W' in Helvetica
        let w_i = manager.char_width('i', "Helvetica", 12.0);
        let w_w = manager.char_width('W', "Helvetica", 12.0);

        assert!(w_i < w_w);
    }

    #[test]
    fn test_font_selection() {
        let manager = FontManager::new();

        assert_eq!(
            manager.select_font(FontFamily::Helvetica, FontWeight::Normal, false),
            "Helvetica"
        );
        assert_eq!(
            manager.select_font(FontFamily::Helvetica, FontWeight::Bold, false),
            "Helvetica-Bold"
        );
        assert_eq!(
            manager.select_font(FontFamily::Times, FontWeight::Normal, true),
            "Times-Italic"
        );
        assert_eq!(
            manager.select_font(FontFamily::Courier, FontWeight::Bold, true),
            "Courier-BoldOblique"
        );
    }

    #[test]
    fn test_font_metrics() {
        let manager = FontManager::new();
        let font = manager.get_font("Helvetica").unwrap();

        assert!(font.ascender > 0.0);
        assert!(font.descender < 0.0);
        assert!(font.cap_height > 0.0);
        assert!(font.x_height > 0.0);
        assert!(font.x_height < font.cap_height);
    }

    #[test]
    fn test_line_height() {
        let manager = FontManager::new();
        let font = manager.get_font("Helvetica").unwrap();

        let line_height = font.line_height(12.0);
        // Raw line height is the em-square height (ascender - descender)
        // For Helvetica: (718 - (-207)) * 12 / 1000 = 11.1 points
        assert!(line_height > 10.0);
        assert!(line_height < 15.0);

        // With spacing factor (1.2), we get a comfortable reading line height
        let visual_line_height = line_height * font.line_spacing_factor();
        assert!(visual_line_height > 12.0); // Should be > font size with spacing
    }

    #[test]
    fn test_text_layout_wrap() {
        let layout = TextLayout::new();

        let text = "The quick brown fox jumps over the lazy dog";
        let lines = layout.wrap_text(text, "Helvetica", 12.0, 100.0);

        assert!(lines.len() > 1); // Should wrap into multiple lines
        for (line, width) in &lines {
            assert!(!line.is_empty() || lines.len() == 1);
            assert!(*width <= 100.0 || line.split_whitespace().count() == 1);
        }
    }

    #[test]
    fn test_text_bounds() {
        let layout = TextLayout::new();

        let text = "Hello World";
        let (width, height) = layout.text_bounds(text, "Helvetica", 12.0, 1000.0);

        assert!(width > 0.0);
        assert!(height > 0.0);
    }

    #[test]
    fn test_empty_text() {
        let layout = TextLayout::new();
        let lines = layout.wrap_text("", "Helvetica", 12.0, 100.0);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].0.is_empty());
    }

    // ========== Additional Coverage Tests ==========

    #[test]
    fn test_font_manager_default() {
        let manager = FontManager::default();
        // Default should be same as new()
        assert!(manager.get_font("Helvetica").is_some());
        assert!(manager.get_font("Symbol").is_some());
        assert!(manager.get_font("ZapfDingbats").is_some());
    }

    #[test]
    fn test_all_base14_fonts_registered() {
        let manager = FontManager::new();
        let names = manager.font_names();
        assert!(names.len() >= 14);
        // Check all 14 base fonts
        for name in &[
            "Helvetica",
            "Helvetica-Bold",
            "Helvetica-Oblique",
            "Helvetica-BoldOblique",
            "Times-Roman",
            "Times-Bold",
            "Times-Italic",
            "Times-BoldItalic",
            "Courier",
            "Courier-Bold",
            "Courier-Oblique",
            "Courier-BoldOblique",
            "Symbol",
            "ZapfDingbats",
        ] {
            assert!(manager.get_font(name).is_some(), "Missing font: {}", name);
            assert!(manager.is_base14(name), "Not base14: {}", name);
        }
    }

    #[test]
    fn test_get_font_or_default_existing() {
        let manager = FontManager::new();
        let font = manager.get_font_or_default("Times-Roman");
        assert_eq!(font.name, "Times-Roman");
    }

    #[test]
    fn test_get_font_or_default_fallback() {
        let manager = FontManager::new();
        let font = manager.get_font_or_default("NonExistentFont");
        assert_eq!(font.name, "Helvetica");
    }

    #[test]
    fn test_get_font_none() {
        let manager = FontManager::new();
        assert!(manager.get_font("FakeFont").is_none());
    }

    #[test]
    fn test_next_font_resource_id() {
        let mut manager = FontManager::new();
        assert_eq!(manager.next_font_resource_id(), "F1");
        assert_eq!(manager.next_font_resource_id(), "F2");
        assert_eq!(manager.next_font_resource_id(), "F3");
    }

    #[test]
    fn test_text_width_nonexistent_font_fallback() {
        let manager = FontManager::new();
        // Should fall back to Helvetica
        let width = manager.text_width("Hello", "FakeFont", 12.0);
        let helvetica_width = manager.text_width("Hello", "Helvetica", 12.0);
        assert!((width - helvetica_width).abs() < 0.001);
    }

    #[test]
    fn test_char_width_nonexistent_font_fallback() {
        let manager = FontManager::new();
        let width = manager.char_width('A', "NonExistent", 12.0);
        let helvetica_width = manager.char_width('A', "Helvetica", 12.0);
        assert!((width - helvetica_width).abs() < 0.001);
    }

    #[test]
    fn test_select_font_all_combinations() {
        let manager = FontManager::new();

        // All Helvetica
        assert_eq!(
            manager.select_font(FontFamily::Helvetica, FontWeight::Normal, false),
            "Helvetica"
        );
        assert_eq!(
            manager.select_font(FontFamily::Helvetica, FontWeight::Bold, false),
            "Helvetica-Bold"
        );
        assert_eq!(
            manager.select_font(FontFamily::Helvetica, FontWeight::Normal, true),
            "Helvetica-Oblique"
        );
        assert_eq!(
            manager.select_font(FontFamily::Helvetica, FontWeight::Bold, true),
            "Helvetica-BoldOblique"
        );

        // All Times
        assert_eq!(
            manager.select_font(FontFamily::Times, FontWeight::Normal, false),
            "Times-Roman"
        );
        assert_eq!(manager.select_font(FontFamily::Times, FontWeight::Bold, false), "Times-Bold");
        assert_eq!(
            manager.select_font(FontFamily::Times, FontWeight::Normal, true),
            "Times-Italic"
        );
        assert_eq!(
            manager.select_font(FontFamily::Times, FontWeight::Bold, true),
            "Times-BoldItalic"
        );

        // All Courier
        assert_eq!(manager.select_font(FontFamily::Courier, FontWeight::Normal, false), "Courier");
        assert_eq!(
            manager.select_font(FontFamily::Courier, FontWeight::Bold, false),
            "Courier-Bold"
        );
        assert_eq!(
            manager.select_font(FontFamily::Courier, FontWeight::Normal, true),
            "Courier-Oblique"
        );
        assert_eq!(
            manager.select_font(FontFamily::Courier, FontWeight::Bold, true),
            "Courier-BoldOblique"
        );

        // Symbol and ZapfDingbats (weight/italic ignored)
        assert_eq!(manager.select_font(FontFamily::Symbol, FontWeight::Normal, false), "Symbol");
        assert_eq!(manager.select_font(FontFamily::Symbol, FontWeight::Bold, true), "Symbol");
        assert_eq!(
            manager.select_font(FontFamily::ZapfDingbats, FontWeight::Normal, false),
            "ZapfDingbats"
        );
        assert_eq!(
            manager.select_font(FontFamily::ZapfDingbats, FontWeight::Bold, true),
            "ZapfDingbats"
        );
    }

    #[test]
    fn test_font_info_family_properties() {
        let manager = FontManager::new();

        let helv = manager.get_font("Helvetica").unwrap();
        assert_eq!(helv.family, FontFamily::Helvetica);
        assert_eq!(helv.weight, FontWeight::Normal);
        assert!(!helv.italic);
        assert!(helv.is_base14);

        let helv_bold = manager.get_font("Helvetica-Bold").unwrap();
        assert_eq!(helv_bold.weight, FontWeight::Bold);
        assert!(!helv_bold.italic);

        let helv_obl = manager.get_font("Helvetica-Oblique").unwrap();
        assert_eq!(helv_obl.weight, FontWeight::Normal);
        assert!(helv_obl.italic);

        let helv_bo = manager.get_font("Helvetica-BoldOblique").unwrap();
        assert_eq!(helv_bo.weight, FontWeight::Bold);
        assert!(helv_bo.italic);
    }

    #[test]
    fn test_times_font_properties() {
        let manager = FontManager::new();

        let tr = manager.get_font("Times-Roman").unwrap();
        assert_eq!(tr.family, FontFamily::Times);
        assert_eq!(tr.weight, FontWeight::Normal);
        assert!(!tr.italic);

        let tb = manager.get_font("Times-Bold").unwrap();
        assert_eq!(tb.weight, FontWeight::Bold);

        let ti = manager.get_font("Times-Italic").unwrap();
        assert!(ti.italic);

        let tbi = manager.get_font("Times-BoldItalic").unwrap();
        assert_eq!(tbi.weight, FontWeight::Bold);
        assert!(tbi.italic);
    }

    #[test]
    fn test_courier_font_properties() {
        let manager = FontManager::new();

        let c = manager.get_font("Courier").unwrap();
        assert_eq!(c.family, FontFamily::Courier);

        let cb = manager.get_font("Courier-Bold").unwrap();
        assert_eq!(cb.weight, FontWeight::Bold);

        let co = manager.get_font("Courier-Oblique").unwrap();
        assert!(co.italic);

        let cbo = manager.get_font("Courier-BoldOblique").unwrap();
        assert_eq!(cbo.weight, FontWeight::Bold);
        assert!(cbo.italic);
    }

    #[test]
    fn test_symbol_font_properties() {
        let manager = FontManager::new();

        let sym = manager.get_font("Symbol").unwrap();
        assert_eq!(sym.family, FontFamily::Symbol);
        assert!(sym.is_base14);

        let zd = manager.get_font("ZapfDingbats").unwrap();
        assert_eq!(zd.family, FontFamily::ZapfDingbats);
        assert!(zd.is_base14);
    }

    #[test]
    fn test_symbol_font_char_width() {
        let manager = FontManager::new();
        let sym = manager.get_font("Symbol").unwrap();
        // Symbol uses fixed 500 width for all chars
        assert!((sym.char_width('A') - 500.0).abs() < 0.001);
        assert!((sym.char_width('z') - 500.0).abs() < 0.001);
    }

    #[test]
    fn test_courier_font_metrics() {
        let manager = FontManager::new();
        let courier = manager.get_font("Courier").unwrap();
        assert_eq!(courier.ascender, 629.0);
        assert_eq!(courier.descender, -157.0);

        let courier_bold = manager.get_font("Courier-Bold").unwrap();
        assert_eq!(courier_bold.ascender, 626.0);
    }

    #[test]
    fn test_helvetica_bold_metrics() {
        let manager = FontManager::new();
        let font = manager.get_font("Helvetica-Bold").unwrap();
        assert_eq!(font.ascender, 718.0);
        assert_eq!(font.descender, -207.0);
        assert_eq!(font.x_height, 532.0);
    }

    #[test]
    fn test_times_metrics() {
        let manager = FontManager::new();
        let font = manager.get_font("Times-Roman").unwrap();
        assert_eq!(font.ascender, 683.0);
        assert_eq!(font.descender, -217.0);

        let bold = manager.get_font("Times-Bold").unwrap();
        assert_eq!(bold.ascender, 676.0);
    }

    #[test]
    fn test_font_info_line_spacing_factor() {
        let manager = FontManager::new();
        let font = manager.get_font("Helvetica").unwrap();
        assert!((font.line_spacing_factor() - 1.2).abs() < 0.001);
    }

    #[test]
    fn test_font_info_text_width_empty() {
        let manager = FontManager::new();
        let font = manager.get_font("Helvetica").unwrap();
        assert!((font.text_width("", 12.0)).abs() < 0.001);
    }

    #[test]
    fn test_font_info_char_width_unknown_char() {
        let manager = FontManager::new();
        let font = manager.get_font("Helvetica").unwrap();
        // Unknown character should return 500 (default)
        let width = font.char_width('\u{FFFF}');
        assert!((width - 500.0).abs() < 0.001);
    }

    #[test]
    fn test_font_widths_proportional_known_chars() {
        let manager = FontManager::new();
        let font = manager.get_font("Helvetica").unwrap();

        // Space should have a specific width
        let space_w = font.char_width(' ');
        assert!((space_w - 278.0).abs() < 0.001);

        // 'A' for Helvetica = 722
        let a_w = font.char_width('A');
        assert!((a_w - 722.0).abs() < 0.001);

        // 'a' for Helvetica = 556
        let a_lower_w = font.char_width('a');
        assert!((a_lower_w - 556.0).abs() < 0.001);
    }

    #[test]
    fn test_font_widths_times_chars() {
        let manager = FontManager::new();
        let font = manager.get_font("Times-Roman").unwrap();

        // Space for Times = 250
        let space_w = font.char_width(' ');
        assert!((space_w - 250.0).abs() < 0.001);

        // 'I' for Times = 333
        let i_w = font.char_width('I');
        assert!((i_w - 333.0).abs() < 0.001);
    }

    #[test]
    fn test_font_widths_times_bold_chars() {
        let manager = FontManager::new();
        let font = manager.get_font("Times-Bold").unwrap();

        // 'W' for Times-Bold = 1000
        let w_w = font.char_width('W');
        assert!((w_w - 1000.0).abs() < 0.001);
    }

    #[test]
    fn test_helvetica_bold_widths() {
        let manager = FontManager::new();
        let font = manager.get_font("Helvetica-Bold").unwrap();

        // 'f' for Helvetica-Bold = 333 (different from Helvetica=278)
        let f_w = font.char_width('f');
        assert!((f_w - 333.0).abs() < 0.001);
    }

    #[test]
    fn test_digit_widths() {
        let manager = FontManager::new();
        let font = manager.get_font("Helvetica").unwrap();

        // All digits should be 556 for Helvetica
        for digit in '0'..='9' {
            let w = font.char_width(digit);
            assert!((w - 556.0).abs() < 0.001, "Digit {} has width {}", digit, w);
        }
    }

    #[test]
    fn test_punctuation_widths() {
        let manager = FontManager::new();
        let font = manager.get_font("Helvetica").unwrap();

        // Test various punctuation
        assert!(font.char_width('@') > 700.0); // @ is wide
        assert!(font.char_width('!') > 200.0);
        assert!(font.char_width('.') > 200.0);
    }

    #[test]
    fn test_text_width_scaling() {
        let manager = FontManager::new();
        let w12 = manager.text_width("Hello", "Helvetica", 12.0);
        let w24 = manager.text_width("Hello", "Helvetica", 24.0);
        // Width at 24pt should be exactly 2x width at 12pt
        assert!((w24 - 2.0 * w12).abs() < 0.01);
    }

    #[test]
    fn test_line_height_scaling() {
        let manager = FontManager::new();
        let font = manager.get_font("Helvetica").unwrap();
        let lh12 = font.line_height(12.0);
        let lh24 = font.line_height(24.0);
        assert!((lh24 - 2.0 * lh12).abs() < 0.01);
    }

    #[test]
    fn test_text_layout_default() {
        let layout = TextLayout::default();
        let lines = layout.wrap_text("Hello", "Helvetica", 12.0, 1000.0);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].0, "Hello");
    }

    #[test]
    fn test_text_layout_with_font_manager() {
        let fm = FontManager::new();
        let layout = TextLayout::with_font_manager(fm);
        let lines = layout.wrap_text("Test", "Helvetica", 12.0, 1000.0);
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_text_layout_font_manager_accessor() {
        let layout = TextLayout::new();
        let fm = layout.font_manager();
        assert!(fm.get_font("Helvetica").is_some());
    }

    #[test]
    fn test_text_layout_single_long_word() {
        let layout = TextLayout::new();
        // A very long word should stay on one line even if wider than max_width
        let lines = layout.wrap_text("Superlongword", "Helvetica", 12.0, 10.0);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].0, "Superlongword");
    }

    #[test]
    fn test_text_layout_wrap_exact_fit() {
        let layout = TextLayout::new();
        // Single word on its own line
        let lines = layout.wrap_text("A B", "Helvetica", 12.0, 1000.0);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].0, "A B");
    }

    #[test]
    fn test_text_bounds_multiline() {
        let layout = TextLayout::new();
        // Force wrapping by using very narrow width
        let (width, height) = layout.text_bounds("Hello World", "Helvetica", 12.0, 30.0);
        assert!(width > 0.0);
        assert!(height > 0.0);
    }

    #[test]
    fn test_text_bounds_empty() {
        let layout = TextLayout::new();
        let (_width, height) = layout.text_bounds("", "Helvetica", 12.0, 100.0);
        // Empty text should still have one line height
        assert!(height > 0.0);
    }

    #[test]
    fn test_font_weight_default() {
        let weight = FontWeight::default();
        assert_eq!(weight, FontWeight::Normal);
    }

    #[test]
    fn test_embedded_font_manager_new() {
        let mgr = EmbeddedFontManager::new();
        assert!(mgr.is_empty());
        assert_eq!(mgr.len(), 0);
    }

    #[test]
    fn test_embedded_font_manager_default() {
        let mgr = EmbeddedFontManager::default();
        assert!(mgr.is_empty());
        assert_eq!(mgr.len(), 0);
    }

    #[test]
    fn test_base14_metrics_default_branch() {
        // Test the default branch of base14_metrics
        let metrics = base14_metrics("Unknown");
        assert_eq!(metrics.0, 750.0);
        assert_eq!(metrics.1, -250.0);
    }

    #[test]
    fn test_font_widths_for_base14_symbol() {
        let widths = FontWidths::for_base14("Symbol");
        assert!((widths.width_for_char('A') - 500.0).abs() < 0.001);
    }

    #[test]
    fn test_font_widths_for_base14_unknown() {
        // Falls through to default proportional widths
        let widths = FontWidths::for_base14("UnknownFont");
        // Should still have proportional widths with defaults
        let w = widths.width_for_char('A');
        assert!(w > 0.0);
    }

    #[test]
    fn test_courier_monospace_all_chars_same() {
        let manager = FontManager::new();
        let font = manager.get_font("Courier").unwrap();
        let width_a = font.char_width('A');
        let width_z = font.char_width('z');
        let width_at = font.char_width('@');
        assert!((width_a - 600.0).abs() < 0.001);
        assert!((width_z - 600.0).abs() < 0.001);
        assert!((width_at - 600.0).abs() < 0.001);
    }

    #[test]
    fn test_courier_bold_monospace() {
        let manager = FontManager::new();
        let font = manager.get_font("Courier-Bold").unwrap();
        assert!((font.char_width('A') - 600.0).abs() < 0.001);
    }

    #[test]
    fn test_courier_oblique_monospace() {
        let manager = FontManager::new();
        let font = manager.get_font("Courier-Oblique").unwrap();
        assert!((font.char_width('X') - 600.0).abs() < 0.001);
    }

    #[test]
    fn test_courier_boldoblique_monospace() {
        let manager = FontManager::new();
        let font = manager.get_font("Courier-BoldOblique").unwrap();
        assert!((font.char_width('M') - 600.0).abs() < 0.001);
    }

    #[test]
    fn test_helvetica_oblique_widths() {
        let manager = FontManager::new();
        let font = manager.get_font("Helvetica-Oblique").unwrap();
        // Same widths as Helvetica
        let a_w = font.char_width('A');
        assert!((a_w - 722.0).abs() < 0.001);
    }

    #[test]
    fn test_times_italic_widths() {
        let manager = FontManager::new();
        let font = manager.get_font("Times-Italic").unwrap();
        // Same widths as Times-Roman
        let space_w = font.char_width(' ');
        assert!((space_w - 250.0).abs() < 0.001);
    }

    #[test]
    fn test_times_bolditalic_widths() {
        let manager = FontManager::new();
        let font = manager.get_font("Times-BoldItalic").unwrap();
        // 'a' for Times-BoldItalic = 500
        let a_w = font.char_width('a');
        assert!((a_w - 500.0).abs() < 0.001);
    }

    #[test]
    fn test_helvetica_boldoblique_widths() {
        let manager = FontManager::new();
        let font = manager.get_font("Helvetica-BoldOblique").unwrap();
        // Colon for Helvetica-Bold = 333
        let w = font.char_width(':');
        assert!((w - 333.0).abs() < 0.001);
    }
}
