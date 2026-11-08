//! Office style to PDF style mapping.
//!
//! Maps formatting from Office documents (DOCX, XLSX, PPTX) to PDF equivalents.

/// Text formatting style from Office documents.
#[derive(Debug, Clone, Default)]
pub struct TextStyle {
    /// Font name
    pub font_name: Option<String>,
    /// Font size in points
    pub font_size: Option<f32>,
    /// Bold
    pub bold: bool,
    /// Italic
    pub italic: bool,
    /// Underline
    pub underline: bool,
    /// Strikethrough
    pub strikethrough: bool,
    /// Text color (RGB)
    pub color: Option<(f32, f32, f32)>,
}

impl TextStyle {
    /// Get the PDF font name based on style.
    pub fn pdf_font_name(&self) -> &str {
        let base = self.font_name.as_deref().unwrap_or("Helvetica");

        // Map common Office fonts to PDF base fonts
        let mapped = match base.to_lowercase().as_str() {
            "times new roman" | "times" => "Times-Roman",
            "arial" | "helvetica" => "Helvetica",
            "courier new" | "courier" => "Courier",
            "symbol" => "Symbol",
            "zapfdingbats" | "wingdings" => "ZapfDingbats",
            _ => "Helvetica",
        };

        // Apply bold/italic variants
        match (self.bold, self.italic) {
            (true, true) => match mapped {
                "Helvetica" => "Helvetica-BoldOblique",
                "Times-Roman" => "Times-BoldItalic",
                "Courier" => "Courier-BoldOblique",
                _ => mapped,
            },
            (true, false) => match mapped {
                "Helvetica" => "Helvetica-Bold",
                "Times-Roman" => "Times-Bold",
                "Courier" => "Courier-Bold",
                _ => mapped,
            },
            (false, true) => match mapped {
                "Helvetica" => "Helvetica-Oblique",
                "Times-Roman" => "Times-Italic",
                "Courier" => "Courier-Oblique",
                _ => mapped,
            },
            (false, false) => mapped,
        }
    }
}

/// Paragraph formatting style from Office documents.
#[derive(Debug, Clone, Default)]
pub struct ParagraphStyle {
    /// Text alignment
    pub alignment: ParagraphAlignment,
    /// Space before paragraph in points
    pub space_before: f32,
    /// Space after paragraph in points
    pub space_after: f32,
    /// Whether this is a heading
    pub heading_level: Option<u8>,
    /// Whether this is a list item
    pub list_level: Option<u8>,
}

/// Paragraph alignment options.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ParagraphAlignment {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

/// Convert Office color values to RGB.
///
/// Office uses various color formats (theme colors, RGB, etc.).
/// This function normalizes them to RGB (0.0-1.0 range).
pub fn parse_color(color_str: &str) -> Option<(f32, f32, f32)> {
    // Handle hex colors (RRGGBB format)
    if color_str.len() == 6 {
        let r = u8::from_str_radix(&color_str[0..2], 16).ok()?;
        let g = u8::from_str_radix(&color_str[2..4], 16).ok()?;
        let b = u8::from_str_radix(&color_str[4..6], 16).ok()?;
        return Some((r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0));
    }

    // Handle named colors
    match color_str.to_lowercase().as_str() {
        "black" => Some((0.0, 0.0, 0.0)),
        "white" => Some((1.0, 1.0, 1.0)),
        "red" => Some((1.0, 0.0, 0.0)),
        "green" => Some((0.0, 1.0, 0.0)),
        "blue" => Some((0.0, 0.0, 1.0)),
        "yellow" => Some((1.0, 1.0, 0.0)),
        "cyan" => Some((0.0, 1.0, 1.0)),
        "magenta" => Some((1.0, 0.0, 1.0)),
        "gray" | "grey" => Some((0.5, 0.5, 0.5)),
        _ => None,
    }
}

/// Convert Office half-points to points.
///
/// Font sizes in DOCX are often in half-points.
pub fn half_points_to_points(half_points: i32) -> f32 {
    half_points as f32 / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_style_font_mapping() {
        let mut style = TextStyle::default();
        assert_eq!(style.pdf_font_name(), "Helvetica");

        style.bold = true;
        assert_eq!(style.pdf_font_name(), "Helvetica-Bold");

        style.italic = true;
        assert_eq!(style.pdf_font_name(), "Helvetica-BoldOblique");

        style.bold = false;
        assert_eq!(style.pdf_font_name(), "Helvetica-Oblique");
    }

    #[test]
    fn test_text_style_times_font() {
        let mut style = TextStyle {
            font_name: Some("Times New Roman".to_string()),
            ..Default::default()
        };
        assert_eq!(style.pdf_font_name(), "Times-Roman");

        style.bold = true;
        assert_eq!(style.pdf_font_name(), "Times-Bold");

        style.italic = true;
        assert_eq!(style.pdf_font_name(), "Times-BoldItalic");
    }

    #[test]
    fn test_parse_color_hex() {
        let color = parse_color("FF0000").unwrap();
        assert!((color.0 - 1.0).abs() < 0.01);
        assert!(color.1 < 0.01);
        assert!(color.2 < 0.01);
    }

    #[test]
    fn test_parse_color_named() {
        assert_eq!(parse_color("black"), Some((0.0, 0.0, 0.0)));
        assert_eq!(parse_color("white"), Some((1.0, 1.0, 1.0)));
        assert_eq!(parse_color("red"), Some((1.0, 0.0, 0.0)));
    }

    #[test]
    fn test_half_points_to_points() {
        assert_eq!(half_points_to_points(24), 12.0);
        assert_eq!(half_points_to_points(22), 11.0);
    }
}
