//! PDF content stream builder.
//!
//! Builds PDF content streams containing graphics and text operators
//! according to PDF specification ISO 32000-1:2008 Section 8-9.

use crate::elements::{
    ContentElement, ImageContent, PathContent, PathOperation, StructureElement, TableCellAlign,
    TableContent, TextContent,
};
use crate::error::Result;
use crate::fonts::GlyphRemapper;
use crate::layout::Color;
use std::collections::HashMap;
use std::io::Write;

/// Map an arbitrary requested font name (+ bold flag) to the Standard-14
/// PostScript base-font name actually emitted by the `Tf` operator.
///
/// This is the single source of truth for the name that ends up in a
/// content stream, so any code that must *register* that font in a page's
/// `/Resources/Font` (see `DocumentEditor`'s overlay-additions path) keys
/// off this same function — otherwise the registered name and the emitted
/// `Tf` name diverge for styled / generic / Symbol requests, leaving a
/// dangling font tag.
///
/// Symbol and ZapfDingbats are deliberately NOT routed to their own faces:
/// they use built-in encodings (not WinAnsi) and are not pre-registered in
/// the page `/Font` dict, so emitting their names would produce a dangling
/// `Tf` and the wrong encoding. They fall through to the Helvetica
/// fallback — the markdown / text / HTML renderers never request them, and
/// any caller who genuinely needs them should use the embedded-font path.
pub(crate) fn map_base14_font_name(name: &str, bold: bool) -> String {
    let lower = name.to_lowercase();

    // Resolve the family. `sans` must be tested before `serif`
    // because "sans-serif" contains "serif". Unknown names keep the
    // historical Helvetica fallback — embedded fonts never reach this
    // path (they are emitted as ShowEmbeddedText), so this only
    // governs Base-14 substitution for generic family names.
    enum Family {
        Helvetica,
        Times,
        Courier,
    }
    let family = if lower.contains("courier") || lower.contains("mono") {
        Family::Courier
    } else if lower.contains("sans") || lower.contains("helvetica") || lower.contains("arial") {
        Family::Helvetica
    } else if lower.contains("times") || lower.contains("serif") {
        Family::Times
    } else {
        Family::Helvetica
    };

    // Weight/slant come from the caller's flag *or* an explicit
    // Standard-14 PostScript name (e.g. "Helvetica-Bold",
    // "Times-Italic"), so callers can request a styled face by name
    // without also threading a style struct through every layer.
    let want_bold = bold || lower.contains("bold");
    let want_italic = lower.contains("italic") || lower.contains("oblique");

    match family {
        Family::Helvetica => match (want_bold, want_italic) {
            (false, false) => "Helvetica",
            (true, false) => "Helvetica-Bold",
            (false, true) => "Helvetica-Oblique",
            (true, true) => "Helvetica-BoldOblique",
        },
        Family::Times => match (want_bold, want_italic) {
            (false, false) => "Times-Roman",
            (true, false) => "Times-Bold",
            (false, true) => "Times-Italic",
            (true, true) => "Times-BoldItalic",
        },
        Family::Courier => match (want_bold, want_italic) {
            (false, false) => "Courier",
            (true, false) => "Courier-Bold",
            (false, true) => "Courier-Oblique",
            (true, true) => "Courier-BoldOblique",
        },
    }
    .to_string()
}

/// Operations that can be added to a content stream.
#[derive(Debug, Clone)]
pub enum ContentStreamOp {
    /// Save graphics state (q)
    SaveState,
    /// Restore graphics state (Q)
    RestoreState,
    /// Set transformation matrix (cm)
    Transform(f32, f32, f32, f32, f32, f32),
    /// Begin text object (BT)
    BeginText,
    /// End text object (ET)
    EndText,
    /// Set font and size (Tf)
    SetFont(String, f32),
    /// Move text position (Td)
    MoveText(f32, f32),
    /// Set text matrix (Tm)
    SetTextMatrix(f32, f32, f32, f32, f32, f32),
    /// Show text (Tj) - literal string
    ShowText(String),
    /// Show hex-encoded text (Tj) - for CIDFonts/Unicode
    ShowHexText(String),
    /// Show text from a registered embedded font, carrying *original-face*
    /// glyph IDs together with the font's PDF resource name (e.g. `"EF1"`).
    ///
    /// The concrete hex bytes emitted into the content stream are computed
    /// at serialization time ([`ContentStreamBuilder::build_with_remappers`])
    /// so that every GID can be remapped through the font's subset
    /// [`GlyphRemapper`]. This is what makes FONT-3b — real font subsetting
    /// with GID remapping in already-emitted content streams — possible.
    ShowEmbeddedText {
        /// PDF resource name of the embedded font (e.g. `"EF1"`).
        font_name: String,
        /// Original-face glyph IDs in logical text order.
        glyph_ids: Vec<u16>,
    },
    /// Show text with positioning (TJ)
    ShowTextArray(Vec<TextArrayItem>),
    /// Set character spacing (Tc)
    SetCharacterSpacing(f32),
    /// Set word spacing (Tw)
    SetWordSpacing(f32),
    /// Set text leading (TL)
    SetTextLeading(f32),
    /// Move to next line (T*)
    NextLine,
    /// Set fill color RGB (rg)
    SetFillColorRGB(f32, f32, f32),
    /// Set stroke color RGB (RG)
    SetStrokeColorRGB(f32, f32, f32),
    /// Set fill color gray (g)
    SetFillColorGray(f32),
    /// Set stroke color gray (G)
    SetStrokeColorGray(f32),
    /// Set line width (w)
    SetLineWidth(f32),
    /// Move to (m)
    MoveTo(f32, f32),
    /// Line to (l)
    LineTo(f32, f32),
    /// Curve to (c)
    CurveTo(f32, f32, f32, f32, f32, f32),
    /// Rectangle (re)
    Rectangle(f32, f32, f32, f32),
    /// Close path (h)
    ClosePath,
    /// Stroke (S)
    Stroke,
    /// Fill (f)
    Fill,
    /// Fill and stroke (B)
    FillStroke,
    /// Close and stroke (s)
    CloseStroke,
    /// End path without filling/stroking (n)
    EndPath,
    /// Paint XObject (Do)
    PaintXObject(String),

    // === Marked Content Operations ===
    /// Begin marked content with dictionary (BDC) - for tagged PDF structure
    BeginMarkedContentDict {
        /// The tag/structure type (e.g., "P" for paragraph, "H1" for heading)
        tag: String,
        /// Marked Content ID for linking to structure tree
        mcid: u32,
    },
    /// End marked content (EMC)
    EndMarkedContent,

    /// Begin an Artifact marked-content section (BDC /Artifact).
    /// Used for pagination artifacts (headers, footers, page numbers) that
    /// should be ignored by AT (Assistive Technology). F-3.
    BeginArtifact {
        /// Artifact type, e.g. "Pagination", "Layout", "Page".
        artifact_type: String,
        /// Optional subtype, e.g. "Header", "Footer".
        subtype: Option<String>,
    },
    /// End an Artifact marked-content section (EMC).
    EndArtifact,

    // === Clipping Operations ===
    /// Clip using non-zero winding rule (W)
    Clip,
    /// Clip using even-odd rule (W*)
    ClipEvenOdd,

    // === Extended Graphics State ===
    /// Set graphics state from ExtGState dictionary (gs)
    SetExtGState(String),

    // === Color Space Operations ===
    /// Set fill color space (cs)
    SetFillColorSpace(String),
    /// Set stroke color space (CS)
    SetStrokeColorSpace(String),
    /// Set fill color in current color space (sc/scn)
    SetFillColorN(Vec<f32>),
    /// Set stroke color in current color space (SC/SCN)
    SetStrokeColorN(Vec<f32>),
    /// Set fill color with pattern (scn with pattern name)
    SetFillPattern(String, Vec<f32>),
    /// Set stroke color with pattern (SCN with pattern name)
    SetStrokePattern(String, Vec<f32>),

    // === Shading Operations ===
    /// Paint shading (sh)
    PaintShading(String),

    // === Additional Path Operations ===
    /// Curve with first control point on current point (v)
    CurveToV(f32, f32, f32, f32),
    /// Curve with second control point on end point (y)
    CurveToY(f32, f32, f32, f32),
    /// Fill using even-odd rule (f*)
    FillEvenOdd,
    /// Fill and stroke using even-odd rule (B*)
    FillStrokeEvenOdd,
    /// Close, fill and stroke (b)
    CloseFillStroke,
    /// Close, fill and stroke using even-odd rule (b*)
    CloseFillStrokeEvenOdd,

    // === Line Style Operations ===
    /// Set line cap style (J)
    SetLineCap(LineCap),
    /// Set line join style (j)
    SetLineJoin(LineJoin),
    /// Set miter limit (M)
    SetMiterLimit(f32),
    /// Set dash pattern (d)
    SetDashPattern(Vec<f32>, f32),

    // === CMYK Color Operations ===
    /// Set fill color CMYK (k)
    SetFillColorCMYK(f32, f32, f32, f32),
    /// Set stroke color CMYK (K)
    SetStrokeColorCMYK(f32, f32, f32, f32),

    /// Raw operator (for extensibility)
    Raw(String),
}

/// Line cap styles for path stroking.
#[derive(Debug, Clone, Copy, Default)]
pub enum LineCap {
    /// Square butt cap (default)
    #[default]
    Butt = 0,
    /// Round cap
    Round = 1,
    /// Projecting square cap
    Square = 2,
}

/// Line join styles for path stroking.
#[derive(Debug, Clone, Copy, Default)]
pub enum LineJoin {
    /// Miter join (default)
    #[default]
    Miter = 0,
    /// Round join
    Round = 1,
    /// Bevel join
    Bevel = 2,
}

/// Blend modes for transparency.
#[derive(Debug, Clone, Copy, Default)]
pub enum BlendMode {
    /// Normal blend (default)
    #[default]
    Normal,
    /// Multiply
    Multiply,
    /// Screen
    Screen,
    /// Overlay
    Overlay,
    /// Darken
    Darken,
    /// Lighten
    Lighten,
    /// Color dodge
    ColorDodge,
    /// Color burn
    ColorBurn,
    /// Hard light
    HardLight,
    /// Soft light
    SoftLight,
    /// Difference
    Difference,
    /// Exclusion
    Exclusion,
}

impl BlendMode {
    /// Get the PDF name for this blend mode.
    pub fn as_pdf_name(&self) -> &'static str {
        match self {
            BlendMode::Normal => "Normal",
            BlendMode::Multiply => "Multiply",
            BlendMode::Screen => "Screen",
            BlendMode::Overlay => "Overlay",
            BlendMode::Darken => "Darken",
            BlendMode::Lighten => "Lighten",
            BlendMode::ColorDodge => "ColorDodge",
            BlendMode::ColorBurn => "ColorBurn",
            BlendMode::HardLight => "HardLight",
            BlendMode::SoftLight => "SoftLight",
            BlendMode::Difference => "Difference",
            BlendMode::Exclusion => "Exclusion",
        }
    }
}

/// Item in a TJ array (text or positioning adjustment).
#[derive(Debug, Clone)]
pub enum TextArrayItem {
    /// Text string (literal)
    Text(String),
    /// Hex-encoded text string (for CIDFonts/Unicode)
    HexText(String),
    /// Positioning adjustment (negative = move right, positive = move left)
    Adjustment(f32),
}

/// A record of a structure element and its marked-content IDs, collected
/// during content-stream construction for StructTreeRoot emission.
///
/// Each `StructElemRecord` corresponds to one `StructureElement` that was
/// added via `add_element` / `add_structure_element`. The `mcid` field
/// is the Marked Content ID emitted for this element's BDC bracket;
/// `children` holds nested records from child `StructureElement`s.
#[derive(Debug, Clone)]
pub struct StructElemRecord {
    /// The PDF structure type tag (e.g. "P", "H1", "Figure").
    pub structure_type: String,
    /// Marked Content ID emitted for this element's BDC operator.
    pub mcid: u32,
    /// Alternate text for accessibility (/Alt in StructElem dict).
    pub alt_text: Option<String>,
    /// Language override for this element (/Lang in StructElem dict).
    pub language: Option<String>,
    /// Nested structure records from child StructureElements.
    pub children: Vec<StructElemRecord>,
}

/// An image that needs to be registered as an XObject.
///
/// When ContentStreamBuilder encounters an ImageContent, it generates
/// the content stream operators but also tracks the image data so it
/// can be registered as an XObject when the PDF is saved.
#[derive(Debug, Clone)]
pub struct PendingImage {
    /// The image content
    pub image: ImageContent,
    /// The resource ID assigned to this image (e.g., "Im1")
    pub resource_id: String,
}

/// Builder for PDF content streams.
///
/// Creates the byte sequence for a PDF content stream from operations
/// or ContentElements.
#[derive(Debug, Default)]
pub struct ContentStreamBuilder {
    /// Operations in the stream
    operations: Vec<ContentStreamOp>,
    /// Current font name
    current_font: Option<String>,
    /// Current font size
    current_font_size: f32,
    /// Whether we're in a text object
    in_text_object: bool,
    /// MCID (Marked Content ID) counter for tagged PDF structure
    mcid_counter: u32,
    /// Images that need to be registered as XObjects
    pending_images: Vec<PendingImage>,
    /// Next image resource ID counter
    next_image_id: u32,
    /// Structure element records accumulated from add_element(Structure(...))
    /// calls. Used by PdfWriter::finish to build the StructTreeRoot when
    /// tagged PDF mode is enabled.
    struct_records: Vec<StructElemRecord>,
}

impl ContentStreamBuilder {
    /// Create a new content stream builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an operation to the stream.
    pub fn op(&mut self, op: ContentStreamOp) -> &mut Self {
        self.operations.push(op);
        self
    }

    /// Add multiple operations.
    pub fn ops(&mut self, ops: impl IntoIterator<Item = ContentStreamOp>) -> &mut Self {
        self.operations.extend(ops);
        self
    }

    /// Begin a text object.
    pub fn begin_text(&mut self) -> &mut Self {
        if !self.in_text_object {
            self.op(ContentStreamOp::BeginText);
            self.in_text_object = true;
        }
        self
    }

    /// End a text object.
    pub fn end_text(&mut self) -> &mut Self {
        if self.in_text_object {
            self.op(ContentStreamOp::EndText);
            self.in_text_object = false;
        }
        self
    }

    /// Set font for text operations.
    pub fn set_font(&mut self, font_name: &str, size: f32) -> &mut Self {
        if self.current_font.as_deref() != Some(font_name) || self.current_font_size != size {
            self.op(ContentStreamOp::SetFont(font_name.to_string(), size));
            self.current_font = Some(font_name.to_string());
            self.current_font_size = size;
        }
        self
    }

    /// Add text at a position (literal string for Base-14 fonts).
    pub fn text(&mut self, text: &str, x: f32, y: f32) -> &mut Self {
        self.begin_text();
        self.op(ContentStreamOp::SetTextMatrix(1.0, 0.0, 0.0, 1.0, x, y));
        self.op(ContentStreamOp::ShowText(text.to_string()));
        self
    }

    /// Add hex-encoded text at a position (for CIDFonts/Unicode).
    ///
    /// The hex_string should already be formatted as "<XXXX...>" where each
    /// 4-digit hex value is a glyph ID.
    pub fn hex_text(&mut self, hex_string: &str, x: f32, y: f32) -> &mut Self {
        self.begin_text();
        self.op(ContentStreamOp::SetTextMatrix(1.0, 0.0, 0.0, 1.0, x, y));
        self.op(ContentStreamOp::ShowHexText(hex_string.to_string()));
        self
    }

    /// Add text from a registered embedded font at a position, deferring
    /// hex encoding to serialization time so that subsetting can remap
    /// GIDs into subset-local indices.
    ///
    /// `font_name` is the PDF resource name of the font (e.g. `"EF1"`).
    /// `glyph_ids` are *original-face* glyph IDs; the remapper paired
    /// with the same resource name in
    /// [`ContentStreamBuilder::build_with_remappers`] maps them to the
    /// subset-local IDs actually emitted as hex.
    pub fn embedded_text(
        &mut self,
        font_name: &str,
        glyph_ids: Vec<u16>,
        x: f32,
        y: f32,
    ) -> &mut Self {
        self.begin_text();
        self.op(ContentStreamOp::SetTextMatrix(1.0, 0.0, 0.0, 1.0, x, y));
        self.op(ContentStreamOp::ShowEmbeddedText {
            font_name: font_name.to_string(),
            glyph_ids,
        });
        self
    }

    /// Set fill color.
    pub fn fill_color(&mut self, color: Color) -> &mut Self {
        self.op(ContentStreamOp::SetFillColorRGB(color.r, color.g, color.b))
    }

    /// Draw an image XObject at the specified position and size.
    ///
    /// # Arguments
    /// * `resource_id` - The XObject resource ID (e.g., "Im1")
    /// * `x` - X position (left edge)
    /// * `y` - Y position (bottom edge)
    /// * `width` - Display width
    /// * `height` - Display height
    pub fn draw_image(
        &mut self,
        resource_id: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> &mut Self {
        // End any open text object
        self.end_text();

        // Save graphics state, apply transform, draw image, restore state
        self.op(ContentStreamOp::SaveState);
        self.op(ContentStreamOp::Transform(width, 0.0, 0.0, height, x, y));
        self.op(ContentStreamOp::PaintXObject(resource_id.to_string()));
        self.op(ContentStreamOp::RestoreState);
        self
    }

    /// Draw an image using an ImagePlacement specification.
    pub fn draw_image_at(
        &mut self,
        resource_id: &str,
        placement: &super::image_handler::ImagePlacement,
    ) -> &mut Self {
        self.draw_image(resource_id, placement.x, placement.y, placement.width, placement.height)
    }

    /// Set stroke color.
    pub fn stroke_color(&mut self, color: Color) -> &mut Self {
        self.op(ContentStreamOp::SetStrokeColorRGB(color.r, color.g, color.b))
    }

    /// Set fill color with RGB values.
    pub fn set_fill_color(&mut self, r: f32, g: f32, b: f32) -> &mut Self {
        self.op(ContentStreamOp::SetFillColorRGB(r, g, b))
    }

    /// Set stroke color with RGB values.
    pub fn set_stroke_color(&mut self, r: f32, g: f32, b: f32) -> &mut Self {
        self.op(ContentStreamOp::SetStrokeColorRGB(r, g, b))
    }

    /// Set line width.
    pub fn set_line_width(&mut self, width: f32) -> &mut Self {
        self.op(ContentStreamOp::SetLineWidth(width))
    }

    /// Move to a point (start a new subpath).
    pub fn move_to(&mut self, x: f32, y: f32) -> &mut Self {
        self.op(ContentStreamOp::MoveTo(x, y))
    }

    /// Draw a line to a point.
    pub fn line_to(&mut self, x: f32, y: f32) -> &mut Self {
        self.op(ContentStreamOp::LineTo(x, y))
    }

    /// Draw a rectangle.
    pub fn rect(&mut self, x: f32, y: f32, width: f32, height: f32) -> &mut Self {
        self.op(ContentStreamOp::Rectangle(x, y, width, height))
    }

    /// Stroke the current path.
    pub fn stroke(&mut self) -> &mut Self {
        self.op(ContentStreamOp::Stroke)
    }

    /// Fill the current path.
    pub fn fill(&mut self) -> &mut Self {
        self.op(ContentStreamOp::Fill)
    }

    /// Fill using even-odd rule.
    pub fn fill_even_odd(&mut self) -> &mut Self {
        self.op(ContentStreamOp::FillEvenOdd)
    }

    /// Fill and stroke the current path.
    pub fn fill_stroke(&mut self) -> &mut Self {
        self.op(ContentStreamOp::FillStroke)
    }

    /// Fill and stroke using even-odd rule.
    pub fn fill_stroke_even_odd(&mut self) -> &mut Self {
        self.op(ContentStreamOp::FillStrokeEvenOdd)
    }

    /// Close, fill, and stroke the path.
    pub fn close_fill_stroke(&mut self) -> &mut Self {
        self.op(ContentStreamOp::CloseFillStroke)
    }

    /// Close path.
    pub fn close_path(&mut self) -> &mut Self {
        self.op(ContentStreamOp::ClosePath)
    }

    // === Clipping Path Methods ===

    /// Clip to the current path using non-zero winding rule.
    ///
    /// After calling this, use `end_path()` to consume the path without painting,
    /// or combine with stroke/fill operations.
    pub fn clip(&mut self) -> &mut Self {
        self.op(ContentStreamOp::Clip)
    }

    /// Clip to the current path using even-odd rule.
    pub fn clip_even_odd(&mut self) -> &mut Self {
        self.op(ContentStreamOp::ClipEvenOdd)
    }

    /// End path without painting (use after clip).
    pub fn end_path(&mut self) -> &mut Self {
        self.op(ContentStreamOp::EndPath)
    }

    /// Create a rectangular clipping region.
    ///
    /// This is a convenience method that creates a rectangle path and clips to it.
    pub fn clip_rect(&mut self, x: f32, y: f32, width: f32, height: f32) -> &mut Self {
        self.rect(x, y, width, height).clip().end_path()
    }

    // === Graphics State Methods ===

    /// Save the current graphics state.
    pub fn save_state(&mut self) -> &mut Self {
        self.op(ContentStreamOp::SaveState)
    }

    /// Restore the previous graphics state.
    pub fn restore_state(&mut self) -> &mut Self {
        self.op(ContentStreamOp::RestoreState)
    }

    /// Set extended graphics state (for transparency, blend modes, etc.).
    ///
    /// The `gs_name` should reference an ExtGState resource defined in the page.
    pub fn set_ext_gstate(&mut self, gs_name: &str) -> &mut Self {
        self.op(ContentStreamOp::SetExtGState(gs_name.to_string()))
    }

    // === Transform Methods ===

    /// Apply a transformation matrix.
    ///
    /// Matrix is specified as [a b c d e f] where:
    /// - a, d: scaling
    /// - b, c: rotation/skewing
    /// - e, f: translation
    pub fn transform(&mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> &mut Self {
        self.op(ContentStreamOp::Transform(a, b, c, d, e, f))
    }

    /// Translate (move) the coordinate system.
    pub fn translate(&mut self, tx: f32, ty: f32) -> &mut Self {
        self.transform(1.0, 0.0, 0.0, 1.0, tx, ty)
    }

    /// Scale the coordinate system.
    pub fn scale(&mut self, sx: f32, sy: f32) -> &mut Self {
        self.transform(sx, 0.0, 0.0, sy, 0.0, 0.0)
    }

    /// Rotate the coordinate system by angle in radians.
    pub fn rotate(&mut self, angle: f32) -> &mut Self {
        let cos = angle.cos();
        let sin = angle.sin();
        self.transform(cos, sin, -sin, cos, 0.0, 0.0)
    }

    /// Rotate the coordinate system by angle in degrees.
    pub fn rotate_degrees(&mut self, degrees: f32) -> &mut Self {
        self.rotate(degrees * std::f32::consts::PI / 180.0)
    }

    // === Line Style Methods ===

    /// Set line cap style.
    pub fn set_line_cap(&mut self, cap: LineCap) -> &mut Self {
        self.op(ContentStreamOp::SetLineCap(cap))
    }

    /// Set line join style.
    pub fn set_line_join(&mut self, join: LineJoin) -> &mut Self {
        self.op(ContentStreamOp::SetLineJoin(join))
    }

    /// Set miter limit.
    pub fn set_miter_limit(&mut self, limit: f32) -> &mut Self {
        self.op(ContentStreamOp::SetMiterLimit(limit))
    }

    /// Set dash pattern.
    ///
    /// # Arguments
    /// * `pattern` - Array of dash lengths (e.g., [3.0, 2.0] for 3pt dash, 2pt gap)
    /// * `phase` - Starting offset into the pattern
    pub fn set_dash_pattern(&mut self, pattern: Vec<f32>, phase: f32) -> &mut Self {
        self.op(ContentStreamOp::SetDashPattern(pattern, phase))
    }

    /// Set solid line (no dashing).
    pub fn set_solid_line(&mut self) -> &mut Self {
        self.set_dash_pattern(vec![], 0.0)
    }

    // === Color Space Methods ===

    /// Set fill color space.
    pub fn set_fill_color_space(&mut self, name: &str) -> &mut Self {
        self.op(ContentStreamOp::SetFillColorSpace(name.to_string()))
    }

    /// Set stroke color space.
    pub fn set_stroke_color_space(&mut self, name: &str) -> &mut Self {
        self.op(ContentStreamOp::SetStrokeColorSpace(name.to_string()))
    }

    /// Set fill color in current color space.
    pub fn set_fill_color_n(&mut self, components: Vec<f32>) -> &mut Self {
        self.op(ContentStreamOp::SetFillColorN(components))
    }

    /// Set stroke color in current color space.
    pub fn set_stroke_color_n(&mut self, components: Vec<f32>) -> &mut Self {
        self.op(ContentStreamOp::SetStrokeColorN(components))
    }

    /// Set fill color with CMYK values.
    pub fn set_fill_color_cmyk(&mut self, c: f32, m: f32, y: f32, k: f32) -> &mut Self {
        self.op(ContentStreamOp::SetFillColorCMYK(c, m, y, k))
    }

    /// Set stroke color with CMYK values.
    pub fn set_stroke_color_cmyk(&mut self, c: f32, m: f32, y: f32, k: f32) -> &mut Self {
        self.op(ContentStreamOp::SetStrokeColorCMYK(c, m, y, k))
    }

    // === Pattern Methods ===

    /// Set fill pattern.
    ///
    /// # Arguments
    /// * `pattern_name` - Name of the pattern resource
    /// * `components` - Additional color components (empty for colored patterns)
    pub fn set_fill_pattern(&mut self, pattern_name: &str, components: Vec<f32>) -> &mut Self {
        self.op(ContentStreamOp::SetFillPattern(pattern_name.to_string(), components))
    }

    /// Set stroke pattern.
    pub fn set_stroke_pattern(&mut self, pattern_name: &str, components: Vec<f32>) -> &mut Self {
        self.op(ContentStreamOp::SetStrokePattern(pattern_name.to_string(), components))
    }

    // === Shading Methods ===

    /// Paint a shading (gradient).
    ///
    /// The shading fills the current clipping path. Use with `save_state()`,
    /// `clip_rect()`, and `restore_state()` to control the painted area.
    pub fn paint_shading(&mut self, shading_name: &str) -> &mut Self {
        self.op(ContentStreamOp::PaintShading(shading_name.to_string()))
    }

    /// Draw a linear gradient within a rectangle.
    ///
    /// This is a convenience method that clips to the rectangle and paints the shading.
    /// The shading resource must be defined separately.
    pub fn draw_gradient_rect(
        &mut self,
        shading_name: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> &mut Self {
        self.save_state()
            .rect(x, y, width, height)
            .clip()
            .end_path()
            .paint_shading(shading_name)
            .restore_state()
    }

    // === Additional Path Methods ===

    /// Draw a Bézier curve (full control).
    pub fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) -> &mut Self {
        self.op(ContentStreamOp::CurveTo(x1, y1, x2, y2, x3, y3))
    }

    /// Draw a Bézier curve with first control point at current position.
    pub fn curve_to_v(&mut self, x2: f32, y2: f32, x3: f32, y3: f32) -> &mut Self {
        self.op(ContentStreamOp::CurveToV(x2, y2, x3, y3))
    }

    /// Draw a Bézier curve with second control point at end point.
    pub fn curve_to_y(&mut self, x1: f32, y1: f32, x3: f32, y3: f32) -> &mut Self {
        self.op(ContentStreamOp::CurveToY(x1, y1, x3, y3))
    }

    /// Draw a circle.
    ///
    /// Uses Bézier curves to approximate a circle.
    pub fn circle(&mut self, cx: f32, cy: f32, radius: f32) -> &mut Self {
        // Bézier approximation constant for circles
        let k = 0.552_284_8; // 4/3 * (sqrt(2) - 1)
        let c = radius * k;

        self.move_to(cx + radius, cy)
            .curve_to(cx + radius, cy + c, cx + c, cy + radius, cx, cy + radius)
            .curve_to(cx - c, cy + radius, cx - radius, cy + c, cx - radius, cy)
            .curve_to(cx - radius, cy - c, cx - c, cy - radius, cx, cy - radius)
            .curve_to(cx + c, cy - radius, cx + radius, cy - c, cx + radius, cy)
            .close_path()
    }

    /// Draw an ellipse.
    pub fn ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32) -> &mut Self {
        let kx = rx * 0.552_284_8;
        let ky = ry * 0.552_284_8;

        self.move_to(cx + rx, cy)
            .curve_to(cx + rx, cy + ky, cx + kx, cy + ry, cx, cy + ry)
            .curve_to(cx - kx, cy + ry, cx - rx, cy + ky, cx - rx, cy)
            .curve_to(cx - rx, cy - ky, cx - kx, cy - ry, cx, cy - ry)
            .curve_to(cx + kx, cy - ry, cx + rx, cy - ky, cx + rx, cy)
            .close_path()
    }

    /// Draw a rounded rectangle.
    pub fn rounded_rect(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        radius: f32,
    ) -> &mut Self {
        let r = radius.min(width / 2.0).min(height / 2.0);
        let k = r * 0.552_284_8;

        // Start at top-left corner (after radius)
        self.move_to(x + r, y)
            // Top edge
            .line_to(x + width - r, y)
            // Top-right corner
            .curve_to(x + width - r + k, y, x + width, y + k, x + width, y + r)
            // Right edge
            .line_to(x + width, y + height - r)
            // Bottom-right corner
            .curve_to(
                x + width,
                y + height - r + k,
                x + width - k,
                y + height,
                x + width - r,
                y + height,
            )
            // Bottom edge
            .line_to(x + r, y + height)
            // Bottom-left corner
            .curve_to(x + r - k, y + height, x, y + height - k, x, y + height - r)
            // Left edge
            .line_to(x, y + r)
            // Top-left corner
            .curve_to(x, y + r - k, x + r - k, y, x + r, y)
            .close_path()
    }

    /// Add a ContentElement to the stream.
    pub fn add_element(&mut self, element: &ContentElement) -> &mut Self {
        match element {
            ContentElement::Text(text) => self.add_text_content(text),
            ContentElement::Path(path) => self.add_path_content(path),
            ContentElement::Image(image) => self.add_image_content(image),
            ContentElement::Structure(s) => {
                // Build the BDC/EMC brackets and collect the StructElemRecord
                // so PdfWriter::finish can build the StructTreeRoot.
                let record = self.add_structure_element_impl(s);
                self.struct_records.push(record);
                self
            },
            ContentElement::Table(table) => self.add_table_content(table),
        }
    }

    /// Add text content element.
    fn add_text_content(&mut self, text: &TextContent) -> &mut Self {
        // F-3: If this text has an artifact type, wrap it in /Artifact BDC/EMC
        // so Assistive Technology skips it.
        let is_artifact = text.artifact_type.is_some();
        if is_artifact {
            use crate::extractors::text::ArtifactType;
            // End any open text object before BDC (BDC must be outside BT/ET).
            self.end_text();
            let (artifact_type, subtype) = match &text.artifact_type {
                Some(ArtifactType::Pagination(sub)) => {
                    use crate::extractors::text::PaginationSubtype;
                    let sub_str = match sub {
                        PaginationSubtype::Header => Some("Header".to_string()),
                        PaginationSubtype::Footer => Some("Footer".to_string()),
                        PaginationSubtype::PageNumber => Some("PageNum".to_string()),
                        PaginationSubtype::Watermark => Some("Watermark".to_string()),
                        PaginationSubtype::Other => None,
                    };
                    ("Pagination".to_string(), sub_str)
                },
                Some(ArtifactType::Layout) => ("Layout".to_string(), None),
                Some(ArtifactType::Page) => ("Page".to_string(), None),
                Some(ArtifactType::Background) => ("Background".to_string(), None),
                None => unreachable!(),
            };
            self.op(ContentStreamOp::BeginArtifact {
                artifact_type,
                subtype,
            });
        }

        self.begin_text();

        // Always set the fill colour explicitly. Without this, a previous
        // `rg` operation in the content stream (e.g. a table cell's gray
        // background fill before the text is drawn) bleeds into the text
        // and renders body content in pale gray instead of black —
        // exactly the "no text at all" symptom we hit on XLSX→PDF tables.
        self.fill_color(text.style.color);

        // Set font
        let font_name = self.map_font_name(&text.font.name, text.style.weight.is_bold());
        self.set_font(&font_name, text.font.size);

        // Position and show text
        self.op(ContentStreamOp::SetTextMatrix(1.0, 0.0, 0.0, 1.0, text.bbox.x, text.bbox.y));
        self.op(ContentStreamOp::ShowText(text.text.clone()));

        if is_artifact {
            self.end_text();
            self.op(ContentStreamOp::EndArtifact);
        }

        self
    }

    /// Map a font name to a PDF base font name.
    fn map_font_name(&self, name: &str, bold: bool) -> String {
        map_base14_font_name(name, bold)
    }

    /// Add path content element.
    fn add_path_content(&mut self, path: &PathContent) -> &mut Self {
        // End any text object first
        self.end_text();

        // Artifact wrapping (e.g. footnote separator line).
        let is_artifact = path.artifact_type.is_some();
        if is_artifact {
            use crate::extractors::text::ArtifactType;
            let (artifact_type, subtype) = match &path.artifact_type {
                Some(ArtifactType::Pagination(sub)) => {
                    use crate::extractors::text::PaginationSubtype;
                    let sub_str = match sub {
                        PaginationSubtype::Header => Some("Header".to_string()),
                        PaginationSubtype::Footer => Some("Footer".to_string()),
                        PaginationSubtype::PageNumber => Some("PageNum".to_string()),
                        PaginationSubtype::Watermark => Some("Watermark".to_string()),
                        PaginationSubtype::Other => None,
                    };
                    ("Pagination".to_string(), sub_str)
                },
                Some(ArtifactType::Layout) => ("Layout".to_string(), None),
                Some(ArtifactType::Page) => ("Page".to_string(), None),
                Some(ArtifactType::Background) => ("Background".to_string(), None),
                None => unreachable!(),
            };
            self.op(ContentStreamOp::BeginArtifact {
                artifact_type,
                subtype,
            });
        }

        // If the path carries a 2D affine transform, bracket it in
        // `q cm ... Q` so graphics state stays scoped to this path
        // (#393 Bundle A-2 follow-up). The `had_matrix` flag drives
        // the matching `Q` after the stroke/fill op below.
        let had_matrix = if let Some(m) = path.matrix {
            self.op(ContentStreamOp::SaveState);
            self.op(ContentStreamOp::Transform(m[0], m[1], m[2], m[3], m[4], m[5]));
            true
        } else {
            false
        };

        // Set stroke properties
        if let Some(color) = path.stroke_color {
            self.stroke_color(color);
        }
        if let Some(color) = path.fill_color {
            self.fill_color(color);
        }
        self.op(ContentStreamOp::SetLineWidth(path.stroke_width));

        // Dash pattern (if any) must come before stroke ops. Reset to
        // solid afterwards so subsequent paths don't inherit a stale
        // pattern. (PDF graphics state bleeds across uncontained
        // operations; this is safer than assuming a surrounding q/Q.)
        let had_dash = if let Some((dashes, phase)) = path.dash_pattern.as_ref() {
            self.set_dash_pattern(dashes.clone(), *phase);
            true
        } else {
            false
        };

        // Add path operations
        for op in &path.operations {
            match op {
                PathOperation::MoveTo(x, y) => {
                    self.op(ContentStreamOp::MoveTo(*x, *y));
                },
                PathOperation::LineTo(x, y) => {
                    self.op(ContentStreamOp::LineTo(*x, *y));
                },
                PathOperation::CurveTo(x1, y1, x2, y2, x3, y3) => {
                    self.op(ContentStreamOp::CurveTo(*x1, *y1, *x2, *y2, *x3, *y3));
                },
                PathOperation::Rectangle(x, y, w, h) => {
                    self.op(ContentStreamOp::Rectangle(*x, *y, *w, *h));
                },
                PathOperation::ClosePath => {
                    self.op(ContentStreamOp::ClosePath);
                },
            }
        }

        // Apply stroke/fill
        match (path.stroke_color.is_some(), path.fill_color.is_some()) {
            (true, true) => self.op(ContentStreamOp::FillStroke),
            (true, false) => self.op(ContentStreamOp::Stroke),
            (false, true) => self.op(ContentStreamOp::Fill),
            (false, false) => self.op(ContentStreamOp::EndPath),
        };

        // Restore solid strokes for subsequent paths.
        if had_dash {
            self.set_dash_pattern(Vec::new(), 0.0);
        }

        // Close the `q cm` bracket if we opened one above. RestoreState
        // also rolls back the line-width + colours + dash pattern, so
        // the explicit `set_dash_pattern([], 0)` above is redundant in
        // the had_matrix case — harmless, but documented here so a
        // reader doesn't think we're leaking dash state on transforms.
        if had_matrix {
            self.op(ContentStreamOp::RestoreState);
        }

        if is_artifact {
            self.op(ContentStreamOp::EndArtifact);
        }

        self
    }

    /// Add table content element.
    ///
    /// Renders the table directly to the content stream using:
    /// - Rectangle operations for cell backgrounds
    /// - Line operations for borders
    /// - Text operations for cell content
    fn add_table_content(&mut self, table: &TableContent) -> &mut Self {
        // End any text object first
        self.end_text();

        let style = &table.style;
        let padding = style.cell_padding;

        // Save graphics state for table rendering
        self.op(ContentStreamOp::SaveState);

        // Calculate row positions based on bounding boxes
        let mut current_y = table.bbox.y + table.bbox.height;

        for (row_idx, row) in table.rows.iter().enumerate() {
            let row_height = row
                .height
                .unwrap_or_else(|| table.bbox.height / table.rows.len() as f32);
            current_y -= row_height;

            let mut current_x = table.bbox.x;

            // Draw row background if specified
            if let Some((r, g, b)) = row.background {
                self.op(ContentStreamOp::SetFillColorRGB(r, g, b));
                self.op(ContentStreamOp::Rectangle(
                    table.bbox.x,
                    current_y,
                    table.bbox.width,
                    row_height,
                ));
                self.op(ContentStreamOp::Fill);
            }

            // Draw stripe background for alternating rows
            if row_idx % 2 == 1 {
                if let Some((r, g, b)) = style.stripe_background {
                    self.op(ContentStreamOp::SetFillColorRGB(r, g, b));
                    self.op(ContentStreamOp::Rectangle(
                        table.bbox.x,
                        current_y,
                        table.bbox.width,
                        row_height,
                    ));
                    self.op(ContentStreamOp::Fill);
                }
            }

            // Draw header background if this is a header row
            if row.is_header {
                if let Some((r, g, b)) = style.header_background {
                    self.op(ContentStreamOp::SetFillColorRGB(r, g, b));
                    self.op(ContentStreamOp::Rectangle(
                        table.bbox.x,
                        current_y,
                        table.bbox.width,
                        row_height,
                    ));
                    self.op(ContentStreamOp::Fill);
                }
            }

            for (col_idx, cell) in row.cells.iter().enumerate() {
                // Calculate cell width
                let cell_width = if col_idx < table.column_widths.len() {
                    table.column_widths[col_idx] * cell.colspan as f32
                } else if !table.column_widths.is_empty() {
                    table.column_widths[0]
                } else {
                    table.bbox.width / row.cells.len() as f32
                };

                // Draw cell background if specified
                if let Some((r, g, b)) = cell.background {
                    self.op(ContentStreamOp::SetFillColorRGB(r, g, b));
                    self.op(ContentStreamOp::Rectangle(
                        current_x, current_y, cell_width, row_height,
                    ));
                    self.op(ContentStreamOp::Fill);
                }

                // Draw cell text
                if !cell.text.is_empty() {
                    let font_size = cell.font_size.unwrap_or(10.0);
                    let font_name = if cell.bold {
                        "Helvetica-Bold"
                    } else {
                        "Helvetica"
                    };

                    // Calculate text position based on alignment
                    let text_x = match cell.align {
                        TableCellAlign::Left => current_x + padding,
                        TableCellAlign::Center => current_x + cell_width / 2.0,
                        TableCellAlign::Right => current_x + cell_width - padding,
                    };

                    // Position text at top of cell with padding
                    let text_y = current_y + row_height - padding - font_size;

                    self.begin_text();
                    self.op(ContentStreamOp::SetFillColorRGB(0.0, 0.0, 0.0)); // Black text
                    self.set_font(font_name, font_size);
                    self.op(ContentStreamOp::SetTextMatrix(1.0, 0.0, 0.0, 1.0, text_x, text_y));
                    self.op(ContentStreamOp::ShowText(cell.text.clone()));
                    self.end_text();
                }

                current_x += cell_width;
            }
        }

        // Draw borders
        if style.border_width > 0.0 {
            let (r, g, b) = style.border_color;
            self.op(ContentStreamOp::SetStrokeColorRGB(r, g, b));
            self.op(ContentStreamOp::SetLineWidth(style.border_width));

            // Outer border
            if style.outer_border {
                self.op(ContentStreamOp::Rectangle(
                    table.bbox.x,
                    table.bbox.y,
                    table.bbox.width,
                    table.bbox.height,
                ));
                self.op(ContentStreamOp::Stroke);
            }

            // Horizontal borders
            if style.horizontal_borders {
                let mut y = table.bbox.y + table.bbox.height;
                for row in &table.rows {
                    let row_height = row
                        .height
                        .unwrap_or_else(|| table.bbox.height / table.rows.len() as f32);
                    y -= row_height;
                    if y > table.bbox.y {
                        self.op(ContentStreamOp::MoveTo(table.bbox.x, y));
                        self.op(ContentStreamOp::LineTo(table.bbox.x + table.bbox.width, y));
                        self.op(ContentStreamOp::Stroke);
                    }
                }
            }

            // Vertical borders
            if style.vertical_borders && !table.column_widths.is_empty() {
                let mut x = table.bbox.x;
                for (i, &width) in table.column_widths.iter().enumerate() {
                    x += width;
                    if i < table.column_widths.len() - 1 {
                        self.op(ContentStreamOp::MoveTo(x, table.bbox.y));
                        self.op(ContentStreamOp::LineTo(x, table.bbox.y + table.bbox.height));
                        self.op(ContentStreamOp::Stroke);
                    }
                }
            }
        }

        // Restore graphics state
        self.op(ContentStreamOp::RestoreState);

        self
    }

    /// Add image content element.
    ///
    /// Registers the image for XObject creation and emits a Do operator
    /// to paint the image at its specified position.
    ///
    /// After calling `build()`, use `take_pending_images()` to retrieve
    /// the images that need to be registered as XObjects.
    fn add_image_content(&mut self, image: &ImageContent) -> &mut Self {
        // End any text object first
        self.end_text();

        // PDF/UA-1 F-3: decorative images → /Artifact BDC/EMC.
        // PDF/UA-1 F-1: images with alt text → /Figure BDC/EMC + StructElemRecord.
        let is_artifact = image.is_artifact;
        let has_alt = image.alt_text.is_some() && !is_artifact;

        let mcid = if has_alt {
            let mcid = self.next_mcid();
            self.op(ContentStreamOp::BeginMarkedContentDict {
                tag: "Figure".to_string(),
                mcid,
            });
            Some(mcid)
        } else if is_artifact {
            self.op(ContentStreamOp::BeginArtifact {
                artifact_type: "Layout".to_string(),
                subtype: None,
            });
            None
        } else {
            None
        };

        // If the image carries a 2D affine transform, bracket it in
        // `q cm ... Q`. #393 Bundle A-2 follow-up.
        let had_matrix = if let Some(m) = image.matrix {
            self.op(ContentStreamOp::SaveState);
            self.op(ContentStreamOp::Transform(m[0], m[1], m[2], m[3], m[4], m[5]));
            true
        } else {
            false
        };

        // Allocate resource ID for this image
        self.next_image_id += 1;
        let resource_id = format!("Im{}", self.next_image_id);

        // Track the image for XObject registration
        self.pending_images.push(PendingImage {
            image: image.clone(),
            resource_id: resource_id.clone(),
        });

        // Draw the image using the transformation matrix
        self.draw_image(
            &resource_id,
            image.bbox.x,
            image.bbox.y,
            image.bbox.width,
            image.bbox.height,
        );

        if had_matrix {
            self.op(ContentStreamOp::RestoreState);
        }

        if has_alt {
            self.op(ContentStreamOp::EndMarkedContent);
            // Push a StructElemRecord so pdf_writer.rs builds the /Figure
            // StructElem with /Alt when assembling the StructTreeRoot.
            self.struct_records.push(StructElemRecord {
                structure_type: "Figure".to_string(),
                mcid: mcid.unwrap(),
                alt_text: image.alt_text.clone(),
                language: None,
                children: Vec::new(),
            });
        } else if is_artifact {
            self.op(ContentStreamOp::EndArtifact);
        }

        self
    }

    /// Take the pending images that need to be registered as XObjects.
    ///
    /// This should be called after `build()` to retrieve images that
    /// need to be added to the page's Resources dictionary.
    pub fn take_pending_images(&mut self) -> Vec<PendingImage> {
        std::mem::take(&mut self.pending_images)
    }

    /// Get a reference to pending images without removing them.
    pub fn pending_images(&self) -> &[PendingImage] {
        &self.pending_images
    }

    /// Build multiple elements into the stream.
    pub fn add_elements(&mut self, elements: &[ContentElement]) -> &mut Self {
        for element in elements {
            self.add_element(element);
        }
        // Make sure to end any open text object
        self.end_text();
        self
    }

    /// Get the next MCID value and increment the counter.
    pub fn next_mcid(&mut self) -> u32 {
        let mcid = self.mcid_counter;
        self.mcid_counter += 1;
        mcid
    }

    /// Add a StructureElement with marked content wrapping.
    ///
    /// This wraps the structure element's children in BDC/EMC (Begin/End Marked Content)
    /// operators to enable tagged PDF support. Each content element gets a unique MCID.
    ///
    /// # Arguments
    ///
    /// * `elem` - The structure element to add, containing the hierarchy and content
    ///
    /// # PDF Spec Compliance
    ///
    /// - ISO 32000-1:2008, Section 14.7.4 - Marked Content Sequences
    /// - BDC operator with tag and MCID property dictionary
    /// - EMC operator for proper nesting
    pub fn add_structure_element(&mut self, elem: &StructureElement) -> &mut Self {
        let record = self.add_structure_element_impl(elem);
        self.struct_records.push(record);
        self
    }

    /// Internal recursive implementation for adding structure elements.
    ///
    /// Returns a [`StructElemRecord`] capturing the allocated MCID and any
    /// nested records from child `StructureElement`s. The caller is
    /// responsible for storing or discarding the record.
    fn add_structure_element_impl(&mut self, elem: &StructureElement) -> StructElemRecord {
        // Allocate MCID for this structure element
        let mcid = self.next_mcid();

        // Begin marked content with structure type as tag and MCID property
        self.op(ContentStreamOp::BeginMarkedContentDict {
            tag: elem.structure_type.clone(),
            mcid,
        });

        // Add children (recursively for nested structures), accumulating records
        let mut child_records: Vec<StructElemRecord> = Vec::new();
        for child in &elem.children {
            match child {
                ContentElement::Structure(nested_elem) => {
                    // Recursively add nested structure element and collect record
                    let child_record = self.add_structure_element_impl(nested_elem);
                    child_records.push(child_record);
                },
                _ => {
                    // Add regular content element (no MCID record for leaf content)
                    self.add_element(child);
                },
            }
        }

        // End marked content
        self.op(ContentStreamOp::EndMarkedContent);

        StructElemRecord {
            structure_type: elem.structure_type.clone(),
            mcid,
            alt_text: elem.alt_text.clone(),
            language: elem.language.clone(),
            children: child_records,
        }
    }

    /// Take the accumulated structure element records from this page's content stream.
    ///
    /// Called by `PdfWriter::finish` after processing each page to collect
    /// the structure records needed to build the StructTreeRoot dict.
    pub fn take_struct_records(&mut self) -> Vec<StructElemRecord> {
        std::mem::take(&mut self.struct_records)
    }

    /// Build the content stream to bytes.
    ///
    /// Any [`ContentStreamOp::ShowEmbeddedText`] ops are serialized as-is
    /// with *original-face* GIDs. For correct subset-indexed output,
    /// use [`ContentStreamBuilder::build_with_remappers`] instead — the
    /// production writer pipeline ([`crate::writer::PdfWriter::finish`])
    /// always goes through the remapper-aware path.
    pub fn build(&self) -> Result<Vec<u8>> {
        self.build_with_remappers(&HashMap::new())
    }

    /// Build the content stream to bytes, remapping every embedded-font
    /// glyph ID through its per-font [`GlyphRemapper`].
    ///
    /// `remappers` is keyed by the PDF resource name (e.g. `"EF1"`) used
    /// in the matching [`ContentStreamOp::ShowEmbeddedText::font_name`].
    /// Missing remappers fall back to emitting the original GID unchanged
    /// — a defensive path; in practice `PdfWriter::finish` always
    /// supplies a remapper for every embedded font it has registered.
    pub fn build_with_remappers(
        &self,
        remappers: &HashMap<String, GlyphRemapper>,
    ) -> Result<Vec<u8>> {
        let mut buf = Vec::new();

        for op in &self.operations {
            self.write_op(&mut buf, op, remappers)?;
            writeln!(buf)?;
        }

        Ok(buf)
    }

    /// Write a single operation to the buffer.
    fn write_op<W: Write>(
        &self,
        w: &mut W,
        op: &ContentStreamOp,
        remappers: &HashMap<String, GlyphRemapper>,
    ) -> std::io::Result<()> {
        match op {
            ContentStreamOp::SaveState => write!(w, "q"),
            ContentStreamOp::RestoreState => write!(w, "Q"),
            ContentStreamOp::Transform(a, b, c, d, e, f) => {
                write!(w, "{} {} {} {} {} {} cm", a, b, c, d, e, f)
            },
            ContentStreamOp::BeginText => write!(w, "BT"),
            ContentStreamOp::EndText => write!(w, "ET"),
            ContentStreamOp::SetFont(name, size) => write!(w, "/{} {} Tf", name, size),
            ContentStreamOp::MoveText(tx, ty) => write!(w, "{} {} Td", tx, ty),
            ContentStreamOp::SetTextMatrix(a, b, c, d, e, f) => {
                write!(w, "{} {} {} {} {} {} Tm", a, b, c, d, e, f)
            },
            ContentStreamOp::ShowText(text) => {
                write!(w, "(")?;
                self.write_escaped_string(w, text)?;
                write!(w, ") Tj")
            },
            ContentStreamOp::ShowHexText(hex) => {
                // Hex string already formatted as <XXXX...>
                write!(w, "{} Tj", hex)
            },
            ContentStreamOp::ShowEmbeddedText {
                font_name,
                glyph_ids,
            } => {
                // Resolve original GIDs through the font's subset remapper.
                // Missing remapper is a defensive fallback — production
                // writer always supplies one.
                let remapper = remappers.get(font_name);
                write!(w, "<")?;
                for &orig in glyph_ids {
                    let emitted = remapper.and_then(|r| r.get(orig)).unwrap_or(orig);
                    write!(w, "{:04X}", emitted)?;
                }
                write!(w, "> Tj")
            },
            ContentStreamOp::ShowTextArray(items) => {
                write!(w, "[")?;
                for item in items {
                    match item {
                        TextArrayItem::Text(t) => {
                            write!(w, "(")?;
                            self.write_escaped_string(w, t)?;
                            write!(w, ")")?;
                        },
                        TextArrayItem::HexText(hex) => {
                            // Hex string already formatted as <XXXX...>
                            write!(w, "{}", hex)?;
                        },
                        TextArrayItem::Adjustment(adj) => {
                            write!(w, "{}", adj)?;
                        },
                    }
                    write!(w, " ")?;
                }
                write!(w, "] TJ")
            },
            ContentStreamOp::SetCharacterSpacing(spacing) => write!(w, "{} Tc", spacing),
            ContentStreamOp::SetWordSpacing(spacing) => write!(w, "{} Tw", spacing),
            ContentStreamOp::SetTextLeading(leading) => write!(w, "{} TL", leading),
            ContentStreamOp::NextLine => write!(w, "T*"),
            ContentStreamOp::SetFillColorRGB(r, g, b) => write!(w, "{} {} {} rg", r, g, b),
            ContentStreamOp::SetStrokeColorRGB(r, g, b) => write!(w, "{} {} {} RG", r, g, b),
            ContentStreamOp::SetFillColorGray(g) => write!(w, "{} g", g),
            ContentStreamOp::SetStrokeColorGray(g) => write!(w, "{} G", g),
            ContentStreamOp::SetLineWidth(width) => write!(w, "{} w", width),
            ContentStreamOp::MoveTo(x, y) => write!(w, "{} {} m", x, y),
            ContentStreamOp::LineTo(x, y) => write!(w, "{} {} l", x, y),
            ContentStreamOp::CurveTo(x1, y1, x2, y2, x3, y3) => {
                write!(w, "{} {} {} {} {} {} c", x1, y1, x2, y2, x3, y3)
            },
            ContentStreamOp::Rectangle(x, y, w_val, h) => {
                write!(w, "{} {} {} {} re", x, y, w_val, h)
            },
            ContentStreamOp::ClosePath => write!(w, "h"),
            ContentStreamOp::Stroke => write!(w, "S"),
            ContentStreamOp::Fill => write!(w, "f"),
            ContentStreamOp::FillStroke => write!(w, "B"),
            ContentStreamOp::CloseStroke => write!(w, "s"),
            ContentStreamOp::EndPath => write!(w, "n"),
            ContentStreamOp::PaintXObject(name) => write!(w, "/{} Do", name),

            // Marked content operations
            ContentStreamOp::BeginMarkedContentDict { tag, mcid } => {
                write!(w, "/{} <</MCID {}>> BDC", tag, mcid)
            },
            ContentStreamOp::EndMarkedContent => write!(w, "EMC"),

            // Artifact marked content (F-3)
            ContentStreamOp::BeginArtifact {
                artifact_type,
                subtype,
            } => {
                write!(w, "/Artifact <<")?;
                write!(w, "/Type /{}", artifact_type)?;
                if let Some(sub) = subtype {
                    write!(w, " /Subtype /{}", sub)?;
                }
                write!(w, ">> BDC")
            },
            ContentStreamOp::EndArtifact => write!(w, "EMC"),

            // Clipping operations
            ContentStreamOp::Clip => write!(w, "W"),
            ContentStreamOp::ClipEvenOdd => write!(w, "W*"),

            // Extended graphics state
            ContentStreamOp::SetExtGState(name) => write!(w, "/{} gs", name),

            // Color space operations
            ContentStreamOp::SetFillColorSpace(name) => write!(w, "/{} cs", name),
            ContentStreamOp::SetStrokeColorSpace(name) => write!(w, "/{} CS", name),
            ContentStreamOp::SetFillColorN(components) => {
                for c in components {
                    write!(w, "{} ", c)?;
                }
                write!(w, "scn")
            },
            ContentStreamOp::SetStrokeColorN(components) => {
                for c in components {
                    write!(w, "{} ", c)?;
                }
                write!(w, "SCN")
            },
            ContentStreamOp::SetFillPattern(name, components) => {
                for c in components {
                    write!(w, "{} ", c)?;
                }
                write!(w, "/{} scn", name)
            },
            ContentStreamOp::SetStrokePattern(name, components) => {
                for c in components {
                    write!(w, "{} ", c)?;
                }
                write!(w, "/{} SCN", name)
            },

            // Shading
            ContentStreamOp::PaintShading(name) => write!(w, "/{} sh", name),

            // Additional path operations
            ContentStreamOp::CurveToV(x2, y2, x3, y3) => {
                write!(w, "{} {} {} {} v", x2, y2, x3, y3)
            },
            ContentStreamOp::CurveToY(x1, y1, x3, y3) => {
                write!(w, "{} {} {} {} y", x1, y1, x3, y3)
            },
            ContentStreamOp::FillEvenOdd => write!(w, "f*"),
            ContentStreamOp::FillStrokeEvenOdd => write!(w, "B*"),
            ContentStreamOp::CloseFillStroke => write!(w, "b"),
            ContentStreamOp::CloseFillStrokeEvenOdd => write!(w, "b*"),

            // Line style operations
            ContentStreamOp::SetLineCap(cap) => write!(w, "{} J", *cap as u8),
            ContentStreamOp::SetLineJoin(join) => write!(w, "{} j", *join as u8),
            ContentStreamOp::SetMiterLimit(limit) => write!(w, "{} M", limit),
            ContentStreamOp::SetDashPattern(pattern, phase) => {
                write!(w, "[")?;
                for (i, p) in pattern.iter().enumerate() {
                    if i > 0 {
                        write!(w, " ")?;
                    }
                    write!(w, "{}", p)?;
                }
                write!(w, "] {} d", phase)
            },

            // CMYK colors
            ContentStreamOp::SetFillColorCMYK(c, m, y, k) => {
                write!(w, "{} {} {} {} k", c, m, y, k)
            },
            ContentStreamOp::SetStrokeColorCMYK(c, m, y, k) => {
                write!(w, "{} {} {} {} K", c, m, y, k)
            },

            ContentStreamOp::Raw(raw) => write!(w, "{}", raw),
        }
    }

    /// Write an escaped PDF string for Base-14 font content streams (WinAnsiEncoding).
    ///
    /// Iterates Unicode scalar values and maps each to its WinAnsi/Latin-1 byte
    /// (code-point value for U+0000–U+00FF).  Characters above U+00FF cannot be
    /// represented in WinAnsiEncoding and are replaced with '?'; those require an
    /// embedded font with Identity-H encoding.
    fn write_escaped_string<W: Write>(&self, w: &mut W, text: &str) -> std::io::Result<()> {
        for ch in text.chars() {
            let cp = ch as u32;
            // First, collapse Mathematical Alphanumeric Symbols (U+1D400-1D7FF)
            // — italic/bold/script/etc. styled letters used in formulae — to
            // their plain Latin/Greek base. None of these have glyphs in the
            // standard 14 fonts, but `𝑥`→`x`, `𝛽`→`β`, `𝟗`→`9` is lossless
            // for word-level text recovery and only loses the styling.
            let cp = crate::fonts::encoding::math_alphanumeric_base(cp).unwrap_or(cp);
            // Then map to WinAnsi. Most chars below 0xFF map directly; chars
            // in 0x80-0x9F (smart quotes, em-dash, ellipsis, bullet, …) and
            // a handful above 0xFF (Euro, OE-ligature, …) go through
            // `unicode_to_winansi`. Genuine non-WinAnsi (Greek, CJK, …)
            // degrades to `?` — those need an embedded Unicode font.
            let b = match crate::fonts::encoding::unicode_to_winansi(cp) {
                Some(b) => b,
                None => {
                    w.write_all(b"?")?;
                    continue;
                },
            };
            match b {
                b'(' => write!(w, "\\(")?,
                b')' => write!(w, "\\)")?,
                b'\\' => write!(w, "\\\\")?,
                b'\n' => write!(w, "\\n")?,
                b'\r' => write!(w, "\\r")?,
                b'\t' => write!(w, "\\t")?,
                _ => w.write_all(&[b])?,
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::{FontSpec, TextStyle};
    use crate::geometry::Rect;

    #[test]
    fn test_simple_text() {
        let mut builder = ContentStreamBuilder::new();
        builder
            .begin_text()
            .set_font("Helvetica", 12.0)
            .text("Hello, World!", 72.0, 720.0)
            .end_text();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("BT"));
        assert!(content.contains("/Helvetica 12 Tf"));
        assert!(content.contains("(Hello, World!) Tj"));
        assert!(content.contains("ET"));
    }

    #[test]
    fn test_text_content_element() {
        let text_content = TextContent {
            artifact_type: None,
            text: "Test".to_string(),
            bbox: Rect::new(100.0, 700.0, 50.0, 12.0),
            font: FontSpec::new("Helvetica", 12.0),
            style: TextStyle::default(),
            reading_order: Some(0),
            origin: None,
            rotation_degrees: None,
            matrix: None,
        };

        let mut builder = ContentStreamBuilder::new();
        builder.add_element(&ContentElement::Text(text_content));
        builder.end_text();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("BT"));
        assert!(content.contains("100 700"));
        assert!(content.contains("(Test) Tj"));
        assert!(content.contains("ET"));
    }

    #[test]
    fn test_path_operations() {
        let mut builder = ContentStreamBuilder::new();
        builder
            .stroke_color(Color::black())
            .op(ContentStreamOp::SetLineWidth(1.0))
            .op(ContentStreamOp::MoveTo(0.0, 0.0))
            .op(ContentStreamOp::LineTo(100.0, 100.0))
            .stroke();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("0 0 0 RG"));
        assert!(content.contains("1 w"));
        assert!(content.contains("0 0 m"));
        assert!(content.contains("100 100 l"));
        assert!(content.contains("S"));
    }

    #[test]
    fn test_marked_content_operators() {
        let mut builder = ContentStreamBuilder::new();

        builder
            .op(ContentStreamOp::BeginMarkedContentDict {
                tag: "P".to_string(),
                mcid: 0,
            })
            .op(ContentStreamOp::EndMarkedContent);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/P <</MCID 0>> BDC"));
        assert!(content.contains("EMC"));
    }

    #[test]
    fn test_mcid_allocation() {
        let mut builder = ContentStreamBuilder::new();
        assert_eq!(builder.next_mcid(), 0);
        assert_eq!(builder.next_mcid(), 1);
        assert_eq!(builder.next_mcid(), 2);
    }

    #[test]
    fn test_structure_element_with_text() {
        use crate::elements::FontSpec;
        use crate::geometry::Rect;

        let text_content = TextContent {
            artifact_type: None,
            text: "Hello".to_string(),
            bbox: Rect::new(100.0, 700.0, 50.0, 12.0),
            font: FontSpec::new("Helvetica", 12.0),
            style: TextStyle::default(),
            reading_order: Some(0),
            origin: None,
            rotation_degrees: None,
            matrix: None,
        };

        let structure = StructureElement {
            structure_type: "P".to_string(),
            bbox: Rect::new(100.0, 700.0, 200.0, 50.0),
            children: vec![ContentElement::Text(text_content)],
            reading_order: Some(0),
            alt_text: None,
            language: None,
        };

        let mut builder = ContentStreamBuilder::new();
        builder.add_structure_element(&structure);
        builder.end_text();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/P <</MCID 0>> BDC"));
        assert!(content.contains("EMC"));
        assert!(content.contains("(Hello) Tj"));
    }

    #[test]
    fn test_nested_structure_elements() {
        use crate::geometry::Rect;

        let inner_structure = StructureElement {
            structure_type: "Span".to_string(),
            bbox: Rect::new(100.0, 700.0, 50.0, 12.0),
            children: vec![],
            reading_order: None,
            alt_text: None,
            language: None,
        };

        let outer_structure = StructureElement {
            structure_type: "P".to_string(),
            bbox: Rect::new(100.0, 700.0, 200.0, 50.0),
            children: vec![ContentElement::Structure(inner_structure)],
            reading_order: Some(0),
            alt_text: None,
            language: None,
        };

        let mut builder = ContentStreamBuilder::new();
        builder.add_structure_element(&outer_structure);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        // Should have BDC/EMC pairs for both outer and inner structures
        assert!(content.contains("/P <</MCID 0>> BDC"));
        assert!(content.contains("/Span <</MCID 1>> BDC"));

        // Count EMC to ensure proper nesting
        let emc_count = content.matches("EMC").count();
        assert_eq!(emc_count, 2);
    }

    #[test]
    fn test_rectangle() {
        let mut builder = ContentStreamBuilder::new();
        builder.rect(72.0, 72.0, 468.0, 648.0).stroke();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("72 72 468 648 re"));
        assert!(content.contains("S"));
    }

    #[test]
    fn test_escaped_text() {
        let mut builder = ContentStreamBuilder::new();
        builder
            .begin_text()
            .set_font("Helvetica", 12.0)
            .text("Text with (parens) and \\backslash", 72.0, 720.0)
            .end_text();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("\\(parens\\)"));
        assert!(content.contains("\\\\backslash"));
    }

    #[test]
    fn test_font_mapping() {
        let builder = ContentStreamBuilder::new();

        assert_eq!(builder.map_font_name("Arial", false), "Helvetica");
        assert_eq!(builder.map_font_name("Arial", true), "Helvetica-Bold");
        assert_eq!(builder.map_font_name("Times New Roman", false), "Times-Roman");
        assert_eq!(builder.map_font_name("Courier", false), "Courier");
    }

    /// Issue #525 core: an explicit Standard-14 PostScript name (with or
    /// without a style flag) must resolve to *itself*, not collapse to
    /// the regular family face. Also covers the oblique path that did
    /// not exist before. Symbol/ZapfDingbats are intentionally NOT
    /// routed (they need built-in non-WinAnsi encodings and aren't in
    /// the page font set) — they fall through to Helvetica.
    #[test]
    fn test_font_mapping_explicit_standard14() {
        let b = ContentStreamBuilder::new();

        // Every Latin Standard-14 face round-trips by name.
        for f in [
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
        ] {
            assert_eq!(b.map_font_name(f, false), f, "{f} did not round-trip");
        }

        // The style flag composes with a name-derived style.
        assert_eq!(b.map_font_name("Helvetica", true), "Helvetica-Bold");
        assert_eq!(b.map_font_name("Helvetica-Oblique", true), "Helvetica-BoldOblique");

        // Case-insensitive, and generic styled aliases.
        assert_eq!(b.map_font_name("helvetica-bold", false), "Helvetica-Bold");
        assert_eq!(b.map_font_name("Arial Bold", false), "Helvetica-Bold");
        assert_eq!(b.map_font_name("Times New Roman Italic", false), "Times-Italic");

        // Symbol / ZapfDingbats are NOT routed by this function: they
        // use built-in (non-WinAnsi) encodings and aren't pre-registered
        // in the page /Font dict, so emitting their names would yield a
        // dangling Tf (Copilot review on PR #523 caught this). They
        // fall through to the Helvetica fallback; callers who actually
        // need Symbol/ZapfDingbats must use the embedded-font path.
        assert_eq!(b.map_font_name("Symbol", false), "Helvetica");
        assert_eq!(b.map_font_name("Symbol", true), "Helvetica-Bold");
        assert_eq!(b.map_font_name("ZapfDingbats", true), "Helvetica-Bold");
    }

    #[test]
    fn test_table_content_rendering() {
        use crate::elements::{TableCellContent, TableContent, TableContentStyle, TableRowContent};

        // Create a simple 2x2 table
        let mut table = TableContent::new(Rect::new(72.0, 600.0, 200.0, 100.0));
        table.column_widths = vec![100.0, 100.0];
        table.style = TableContentStyle::bordered();

        // Header row
        let header = TableRowContent::header(vec![
            TableCellContent::header("Name"),
            TableCellContent::header("Value"),
        ]);
        table.add_row(header);

        // Data row
        let row =
            TableRowContent::new(vec![TableCellContent::new("Item"), TableCellContent::new("100")]);
        table.add_row(row);

        let mut builder = ContentStreamBuilder::new();
        builder.add_element(&ContentElement::Table(table));

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        // Should contain graphics state operations
        assert!(content.contains("q")); // Save state
        assert!(content.contains("Q")); // Restore state

        // Should contain text for cells
        assert!(content.contains("(Name) Tj"));
        assert!(content.contains("(Value) Tj"));
        assert!(content.contains("(Item) Tj"));
        assert!(content.contains("(100) Tj"));

        // Should contain stroke operations for borders
        assert!(content.contains("re")); // Rectangle
        assert!(content.contains("S")); // Stroke

        // No pending images
        assert!(builder.pending_images().is_empty());
    }

    #[test]
    fn test_image_content_rendering() {
        use crate::elements::{ColorSpace, ImageContent, ImageFormat};

        // Create a test image
        let image = ImageContent {
            bbox: Rect::new(100.0, 500.0, 200.0, 150.0),
            format: ImageFormat::Jpeg,
            data: vec![0xFF, 0xD8, 0xFF, 0xE0], // JPEG magic bytes
            width: 800,
            height: 600,
            bits_per_component: 8,
            color_space: ColorSpace::RGB,
            reading_order: Some(0),
            alt_text: Some("Test image".to_string()),
            horizontal_dpi: None,
            vertical_dpi: None,
            soft_mask: None,
            matrix: None,
            is_artifact: false,
        };

        let mut builder = ContentStreamBuilder::new();
        builder.add_element(&ContentElement::Image(image));

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        // Should contain image drawing operations
        assert!(content.contains("q")); // Save state
        assert!(content.contains("Q")); // Restore state
        assert!(content.contains("cm")); // Transform matrix
        assert!(content.contains("Do")); // Paint XObject

        // Should have one pending image
        let pending = builder.pending_images();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].resource_id, "Im1");
        assert_eq!(pending[0].image.width, 800);
        assert_eq!(pending[0].image.height, 600);
    }

    #[test]
    fn test_mixed_content_elements() {
        use crate::elements::{
            ColorSpace, ImageContent, ImageFormat, TableCellContent, TableContent,
            TableContentStyle, TableRowContent,
        };

        let mut builder = ContentStreamBuilder::new();

        // Add text
        let text_content = TextContent {
            artifact_type: None,
            text: "Header".to_string(),
            bbox: Rect::new(72.0, 720.0, 100.0, 14.0),
            font: FontSpec::new("Helvetica", 14.0),
            style: TextStyle::default(),
            reading_order: Some(0),
            origin: None,
            rotation_degrees: None,
            matrix: None,
        };
        builder.add_element(&ContentElement::Text(text_content));

        // Add table
        let mut table = TableContent::new(Rect::new(72.0, 600.0, 200.0, 50.0));
        table.column_widths = vec![200.0];
        table.style = TableContentStyle::minimal();
        table.add_row(TableRowContent::new(vec![TableCellContent::new("Row 1")]));
        builder.add_element(&ContentElement::Table(table));

        // Add image
        let image = ImageContent {
            bbox: Rect::new(72.0, 400.0, 100.0, 100.0),
            format: ImageFormat::Png,
            data: vec![0x89, 0x50, 0x4E, 0x47], // PNG magic bytes
            width: 200,
            height: 200,
            bits_per_component: 8,
            color_space: ColorSpace::RGB,
            reading_order: Some(2),
            alt_text: None,
            horizontal_dpi: None,
            vertical_dpi: None,
            soft_mask: None,
            matrix: None,
            is_artifact: false,
        };
        builder.add_element(&ContentElement::Image(image));

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        // Verify all content types are present
        assert!(content.contains("(Header) Tj")); // Text
        assert!(content.contains("(Row 1) Tj")); // Table cell text
        assert!(content.contains("/Im1 Do")); // Image

        // Should have one pending image
        assert_eq!(builder.pending_images().len(), 1);
    }

    #[test]
    fn test_take_pending_images() {
        use crate::elements::{ColorSpace, ImageContent, ImageFormat};

        let image = ImageContent {
            bbox: Rect::new(0.0, 0.0, 100.0, 100.0),
            format: ImageFormat::Jpeg,
            data: vec![0xFF, 0xD8],
            width: 100,
            height: 100,
            bits_per_component: 8,
            color_space: ColorSpace::RGB,
            reading_order: None,
            alt_text: None,
            horizontal_dpi: None,
            vertical_dpi: None,
            soft_mask: None,
            matrix: None,
            is_artifact: false,
        };

        let mut builder = ContentStreamBuilder::new();
        builder.add_element(&ContentElement::Image(image));

        // Take pending images
        let pending = builder.take_pending_images();
        assert_eq!(pending.len(), 1);

        // After taking, should be empty
        assert!(builder.pending_images().is_empty());
        assert!(builder.take_pending_images().is_empty());
    }

    // ========== Additional Coverage Tests ==========

    #[test]
    fn test_save_restore_state() {
        let mut builder = ContentStreamBuilder::new();
        builder.save_state().restore_state();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("q\n"));
        assert!(content.contains("Q\n"));
    }

    #[test]
    fn test_transform_matrix() {
        let mut builder = ContentStreamBuilder::new();
        builder.transform(1.0, 0.0, 0.0, 1.0, 100.0, 200.0);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("1 0 0 1 100 200 cm"));
    }

    #[test]
    fn test_translate() {
        let mut builder = ContentStreamBuilder::new();
        builder.translate(50.0, 75.0);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("1 0 0 1 50 75 cm"));
    }

    #[test]
    fn test_scale() {
        let mut builder = ContentStreamBuilder::new();
        builder.scale(2.0, 3.0);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("2 0 0 3 0 0 cm"));
    }

    #[test]
    fn test_rotate() {
        let mut builder = ContentStreamBuilder::new();
        builder.rotate(std::f32::consts::PI / 2.0);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("cm"));
    }

    #[test]
    fn test_rotate_degrees() {
        let mut builder = ContentStreamBuilder::new();
        builder.rotate_degrees(90.0);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("cm"));
    }

    #[test]
    fn test_fill_color() {
        let mut builder = ContentStreamBuilder::new();
        builder.fill_color(Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
        });

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("1 0 0 rg"));
    }

    #[test]
    fn test_stroke_color() {
        let mut builder = ContentStreamBuilder::new();
        builder.stroke_color(Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
        });

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("0 1 0 RG"));
    }

    #[test]
    fn test_set_fill_color_rgb() {
        let mut builder = ContentStreamBuilder::new();
        builder.set_fill_color(0.5, 0.6, 0.7);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("0.5 0.6 0.7 rg"));
    }

    #[test]
    fn test_set_stroke_color_rgb() {
        let mut builder = ContentStreamBuilder::new();
        builder.set_stroke_color(0.1, 0.2, 0.3);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("0.1 0.2 0.3 RG"));
    }

    #[test]
    fn test_set_line_width() {
        let mut builder = ContentStreamBuilder::new();
        builder.set_line_width(2.5);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("2.5 w"));
    }

    #[test]
    fn test_move_to_and_line_to() {
        let mut builder = ContentStreamBuilder::new();
        builder.move_to(10.0, 20.0).line_to(30.0, 40.0);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("10 20 m"));
        assert!(content.contains("30 40 l"));
    }

    #[test]
    fn test_close_path() {
        let mut builder = ContentStreamBuilder::new();
        builder
            .move_to(0.0, 0.0)
            .line_to(100.0, 0.0)
            .line_to(100.0, 100.0)
            .close_path();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("h\n"));
    }

    #[test]
    fn test_fill() {
        let mut builder = ContentStreamBuilder::new();
        builder.rect(0.0, 0.0, 100.0, 100.0).fill();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("re\n"));
        assert!(content.contains("f\n"));
    }

    #[test]
    fn test_fill_stroke() {
        let mut builder = ContentStreamBuilder::new();
        builder.rect(0.0, 0.0, 100.0, 100.0).fill_stroke();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("B\n"));
    }

    #[test]
    fn test_fill_even_odd() {
        let mut builder = ContentStreamBuilder::new();
        builder.rect(0.0, 0.0, 100.0, 100.0).fill_even_odd();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("f*\n"));
    }

    #[test]
    fn test_fill_stroke_even_odd() {
        let mut builder = ContentStreamBuilder::new();
        builder.rect(0.0, 0.0, 100.0, 100.0).fill_stroke_even_odd();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("B*\n"));
    }

    #[test]
    fn test_close_fill_stroke() {
        let mut builder = ContentStreamBuilder::new();
        builder
            .move_to(0.0, 0.0)
            .line_to(100.0, 0.0)
            .close_fill_stroke();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("b\n"));
    }

    #[test]
    fn test_clip() {
        let mut builder = ContentStreamBuilder::new();
        builder.rect(10.0, 10.0, 200.0, 200.0).clip().end_path();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("W\n"));
        assert!(content.contains("n\n"));
    }

    #[test]
    fn test_clip_even_odd() {
        let mut builder = ContentStreamBuilder::new();
        builder
            .rect(10.0, 10.0, 200.0, 200.0)
            .clip_even_odd()
            .end_path();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("W*\n"));
    }

    #[test]
    fn test_clip_rect() {
        let mut builder = ContentStreamBuilder::new();
        builder.clip_rect(10.0, 10.0, 200.0, 200.0);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("10 10 200 200 re"));
        assert!(content.contains("W\n"));
        assert!(content.contains("n\n"));
    }

    #[test]
    fn test_end_path() {
        let mut builder = ContentStreamBuilder::new();
        builder.rect(0.0, 0.0, 100.0, 100.0).end_path();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("n\n"));
    }

    #[test]
    fn test_set_ext_gstate() {
        let mut builder = ContentStreamBuilder::new();
        builder.set_ext_gstate("GS0");

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("/GS0 gs"));
    }

    #[test]
    fn test_curve_to() {
        let mut builder = ContentStreamBuilder::new();
        builder
            .move_to(0.0, 0.0)
            .curve_to(10.0, 20.0, 30.0, 40.0, 50.0, 60.0);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("10 20 30 40 50 60 c"));
    }

    #[test]
    fn test_curve_to_v() {
        let mut builder = ContentStreamBuilder::new();
        builder.move_to(0.0, 0.0).curve_to_v(10.0, 20.0, 30.0, 40.0);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("10 20 30 40 v"));
    }

    #[test]
    fn test_curve_to_y() {
        let mut builder = ContentStreamBuilder::new();
        builder.move_to(0.0, 0.0).curve_to_y(10.0, 20.0, 30.0, 40.0);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("10 20 30 40 y"));
    }

    #[test]
    fn test_circle() {
        let mut builder = ContentStreamBuilder::new();
        builder.circle(100.0, 100.0, 50.0);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        // Circle uses move_to and curve_to
        assert!(content.contains("m\n"));
        assert!(content.contains("c\n"));
        assert!(content.contains("h\n")); // close_path
    }

    #[test]
    fn test_ellipse() {
        let mut builder = ContentStreamBuilder::new();
        builder.ellipse(200.0, 200.0, 80.0, 40.0);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("m\n"));
        assert!(content.contains("c\n"));
        assert!(content.contains("h\n"));
    }

    #[test]
    fn test_rounded_rect() {
        let mut builder = ContentStreamBuilder::new();
        builder.rounded_rect(50.0, 50.0, 200.0, 100.0, 10.0);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        // Should contain move, line, and curve operations
        assert!(content.contains("m\n"));
        assert!(content.contains("l\n"));
        assert!(content.contains("c\n"));
        assert!(content.contains("h\n"));
    }

    #[test]
    fn test_rounded_rect_large_radius() {
        let mut builder = ContentStreamBuilder::new();
        // Radius larger than half width -- should be clamped
        builder.rounded_rect(0.0, 0.0, 20.0, 40.0, 50.0);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("m\n"));
    }

    #[test]
    fn test_set_line_cap() {
        let mut builder = ContentStreamBuilder::new();
        builder.set_line_cap(LineCap::Round);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("1 J"));
    }

    #[test]
    fn test_set_line_cap_square() {
        let mut builder = ContentStreamBuilder::new();
        builder.set_line_cap(LineCap::Square);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("2 J"));
    }

    #[test]
    fn test_set_line_join() {
        let mut builder = ContentStreamBuilder::new();
        builder.set_line_join(LineJoin::Round);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("1 j"));
    }

    #[test]
    fn test_set_line_join_bevel() {
        let mut builder = ContentStreamBuilder::new();
        builder.set_line_join(LineJoin::Bevel);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("2 j"));
    }

    #[test]
    fn test_set_miter_limit() {
        let mut builder = ContentStreamBuilder::new();
        builder.set_miter_limit(10.0);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("10 M"));
    }

    #[test]
    fn test_set_dash_pattern() {
        let mut builder = ContentStreamBuilder::new();
        builder.set_dash_pattern(vec![3.0, 2.0], 0.0);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("[3 2] 0 d"));
    }

    #[test]
    fn test_set_solid_line() {
        let mut builder = ContentStreamBuilder::new();
        builder.set_solid_line();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("[] 0 d"));
    }

    #[test]
    fn test_set_fill_color_space() {
        let mut builder = ContentStreamBuilder::new();
        builder.set_fill_color_space("DeviceRGB");

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("/DeviceRGB cs"));
    }

    #[test]
    fn test_set_stroke_color_space() {
        let mut builder = ContentStreamBuilder::new();
        builder.set_stroke_color_space("DeviceCMYK");

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("/DeviceCMYK CS"));
    }

    #[test]
    fn test_set_fill_color_n() {
        let mut builder = ContentStreamBuilder::new();
        builder.set_fill_color_n(vec![0.1, 0.2, 0.3]);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("0.1 0.2 0.3 scn"));
    }

    #[test]
    fn test_set_stroke_color_n() {
        let mut builder = ContentStreamBuilder::new();
        builder.set_stroke_color_n(vec![0.4, 0.5]);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("0.4 0.5 SCN"));
    }

    #[test]
    fn test_set_fill_color_cmyk() {
        let mut builder = ContentStreamBuilder::new();
        builder.set_fill_color_cmyk(0.0, 1.0, 1.0, 0.0);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("0 1 1 0 k"));
    }

    #[test]
    fn test_set_stroke_color_cmyk() {
        let mut builder = ContentStreamBuilder::new();
        builder.set_stroke_color_cmyk(1.0, 0.0, 0.0, 0.0);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("1 0 0 0 K"));
    }

    #[test]
    fn test_set_fill_pattern() {
        let mut builder = ContentStreamBuilder::new();
        builder.set_fill_pattern("P1", vec![]);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("/P1 scn"));
    }

    #[test]
    fn test_set_stroke_pattern() {
        let mut builder = ContentStreamBuilder::new();
        builder.set_stroke_pattern("P2", vec![0.5]);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("0.5 /P2 SCN"));
    }

    #[test]
    fn test_paint_shading() {
        let mut builder = ContentStreamBuilder::new();
        builder.paint_shading("Sh1");

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("/Sh1 sh"));
    }

    #[test]
    fn test_draw_gradient_rect() {
        let mut builder = ContentStreamBuilder::new();
        builder.draw_gradient_rect("Sh0", 10.0, 20.0, 200.0, 100.0);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("q\n")); // save
        assert!(content.contains("10 20 200 100 re"));
        assert!(content.contains("W\n")); // clip
        assert!(content.contains("n\n")); // end path
        assert!(content.contains("/Sh0 sh"));
        assert!(content.contains("Q\n")); // restore
    }

    #[test]
    fn test_paint_xobject() {
        let mut builder = ContentStreamBuilder::new();
        builder.op(ContentStreamOp::PaintXObject("Img0".to_string()));

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("/Img0 Do"));
    }

    #[test]
    fn test_close_stroke() {
        let mut builder = ContentStreamBuilder::new();
        builder.op(ContentStreamOp::CloseStroke);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("s\n"));
    }

    #[test]
    fn test_close_fill_stroke_even_odd() {
        let mut builder = ContentStreamBuilder::new();
        builder.op(ContentStreamOp::CloseFillStrokeEvenOdd);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("b*\n"));
    }

    #[test]
    fn test_set_fill_color_gray() {
        let mut builder = ContentStreamBuilder::new();
        builder.op(ContentStreamOp::SetFillColorGray(0.5));

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("0.5 g"));
    }

    #[test]
    fn test_set_stroke_color_gray() {
        let mut builder = ContentStreamBuilder::new();
        builder.op(ContentStreamOp::SetStrokeColorGray(0.75));

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("0.75 G"));
    }

    #[test]
    fn test_set_character_spacing() {
        let mut builder = ContentStreamBuilder::new();
        builder.op(ContentStreamOp::SetCharacterSpacing(2.0));

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("2 Tc"));
    }

    #[test]
    fn test_set_word_spacing() {
        let mut builder = ContentStreamBuilder::new();
        builder.op(ContentStreamOp::SetWordSpacing(5.0));

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("5 Tw"));
    }

    #[test]
    fn test_set_text_leading() {
        let mut builder = ContentStreamBuilder::new();
        builder.op(ContentStreamOp::SetTextLeading(14.0));

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("14 TL"));
    }

    #[test]
    fn test_next_line() {
        let mut builder = ContentStreamBuilder::new();
        builder.op(ContentStreamOp::NextLine);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("T*"));
    }

    #[test]
    fn test_move_text() {
        let mut builder = ContentStreamBuilder::new();
        builder.op(ContentStreamOp::MoveText(10.0, -14.0));

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("10 -14 Td"));
    }

    #[test]
    fn test_set_text_matrix() {
        let mut builder = ContentStreamBuilder::new();
        builder.op(ContentStreamOp::SetTextMatrix(1.0, 0.0, 0.0, 1.0, 72.0, 720.0));

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("1 0 0 1 72 720 Tm"));
    }

    #[test]
    fn test_show_hex_text() {
        let mut builder = ContentStreamBuilder::new();
        builder.begin_text();
        builder.op(ContentStreamOp::ShowHexText("<0041004200>".to_string()));
        builder.end_text();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("<0041004200> Tj"));
    }

    #[test]
    fn test_show_text_array() {
        let mut builder = ContentStreamBuilder::new();
        builder.begin_text();
        builder.op(ContentStreamOp::ShowTextArray(vec![
            TextArrayItem::Text("Hello".to_string()),
            TextArrayItem::Adjustment(-10.0),
            TextArrayItem::Text("World".to_string()),
        ]));
        builder.end_text();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("[(Hello) -10 (World) ] TJ"));
    }

    #[test]
    fn test_show_text_array_with_hex() {
        let mut builder = ContentStreamBuilder::new();
        builder.begin_text();
        builder.op(ContentStreamOp::ShowTextArray(vec![
            TextArrayItem::HexText("<0041>".to_string()),
            TextArrayItem::Adjustment(-50.0),
            TextArrayItem::HexText("<0042>".to_string()),
        ]));
        builder.end_text();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("<0041>"));
        assert!(content.contains("<0042>"));
        assert!(content.contains("TJ"));
    }

    #[test]
    fn test_raw_operator() {
        let mut builder = ContentStreamBuilder::new();
        builder.op(ContentStreamOp::Raw("% custom comment".to_string()));

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("% custom comment"));
    }

    #[test]
    fn test_draw_image() {
        let mut builder = ContentStreamBuilder::new();
        builder.draw_image("Im1", 100.0, 200.0, 300.0, 400.0);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("q\n"));
        assert!(content.contains("300 0 0 400 100 200 cm"));
        assert!(content.contains("/Im1 Do"));
        assert!(content.contains("Q\n"));
    }

    #[test]
    fn test_hex_text_method() {
        let mut builder = ContentStreamBuilder::new();
        builder.begin_text();
        builder.set_font("F1", 12.0);
        builder.hex_text("<00410042>", 72.0, 720.0);
        builder.end_text();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("<00410042> Tj"));
    }

    #[test]
    fn test_begin_text_idempotent() {
        let mut builder = ContentStreamBuilder::new();
        builder.begin_text();
        builder.begin_text(); // Should not add another BT
        builder.end_text();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        let bt_count = content.matches("BT\n").count();
        assert_eq!(bt_count, 1);
    }

    #[test]
    fn test_end_text_idempotent() {
        let mut builder = ContentStreamBuilder::new();
        builder.end_text(); // Not in text -- should be no-op
        builder.begin_text();
        builder.end_text();
        builder.end_text(); // Should be no-op

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        let et_count = content.matches("ET\n").count();
        assert_eq!(et_count, 1);
    }

    #[test]
    fn test_set_font_caching() {
        let mut builder = ContentStreamBuilder::new();
        builder.begin_text();
        builder.set_font("Helvetica", 12.0);
        builder.set_font("Helvetica", 12.0); // Same font, should not emit again
        builder.set_font("Helvetica", 14.0); // Different size, should emit
        builder.end_text();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        // Should have 2 Tf operations (not 3)
        let tf_count = content.matches("Tf\n").count();
        assert_eq!(tf_count, 2);
    }

    #[test]
    fn test_ops_method() {
        let mut builder = ContentStreamBuilder::new();
        builder.ops(vec![
            ContentStreamOp::SaveState,
            ContentStreamOp::SetLineWidth(2.0),
            ContentStreamOp::RestoreState,
        ]);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("q\n"));
        assert!(content.contains("2 w\n"));
        assert!(content.contains("Q\n"));
    }

    #[test]
    fn test_add_elements() {
        let text1 = TextContent {
            artifact_type: None,
            text: "First".to_string(),
            bbox: Rect::new(72.0, 720.0, 50.0, 12.0),
            font: FontSpec::new("Helvetica", 12.0),
            style: TextStyle::default(),
            reading_order: Some(0),
            origin: None,
            rotation_degrees: None,
            matrix: None,
        };
        let text2 = TextContent {
            artifact_type: None,
            text: "Second".to_string(),
            bbox: Rect::new(72.0, 700.0, 50.0, 12.0),
            font: FontSpec::new("Helvetica", 12.0),
            style: TextStyle::default(),
            reading_order: Some(1),
            origin: None,
            rotation_degrees: None,
            matrix: None,
        };

        let mut builder = ContentStreamBuilder::new();
        builder.add_elements(&[ContentElement::Text(text1), ContentElement::Text(text2)]);

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("(First) Tj"));
        assert!(content.contains("(Second) Tj"));
    }

    #[test]
    fn test_escaped_special_chars() {
        let mut builder = ContentStreamBuilder::new();
        builder
            .begin_text()
            .set_font("Helvetica", 12.0)
            .text("line1\nline2\rtab\there", 72.0, 720.0)
            .end_text();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("\\n"));
        assert!(content.contains("\\r"));
        assert!(content.contains("\\t"));
    }

    #[test]
    fn test_font_mapping_sans_serif() {
        let builder = ContentStreamBuilder::new();
        assert_eq!(builder.map_font_name("sans-serif", false), "Helvetica");
    }

    #[test]
    fn test_font_mapping_serif() {
        let builder = ContentStreamBuilder::new();
        assert_eq!(builder.map_font_name("serif", false), "Times-Roman");
        // Standard-14 bold serif is "Times-Bold" (issue #525): the old
        // "Times-Roman-Bold" was not a real Base-14 name and selected no
        // embedded resource, so bold serif text rendered as regular.
        assert_eq!(builder.map_font_name("serif", true), "Times-Bold");
    }

    #[test]
    fn test_font_mapping_monospace() {
        let builder = ContentStreamBuilder::new();
        assert_eq!(builder.map_font_name("monospace", false), "Courier");
        assert_eq!(builder.map_font_name("monospace", true), "Courier-Bold");
    }

    #[test]
    fn test_font_mapping_unknown() {
        let builder = ContentStreamBuilder::new();
        assert_eq!(builder.map_font_name("Unknown Font", false), "Helvetica");
        assert_eq!(builder.map_font_name("Unknown Font", true), "Helvetica-Bold");
    }

    #[test]
    fn test_blend_mode_names() {
        assert_eq!(BlendMode::Normal.as_pdf_name(), "Normal");
        assert_eq!(BlendMode::Multiply.as_pdf_name(), "Multiply");
        assert_eq!(BlendMode::Screen.as_pdf_name(), "Screen");
        assert_eq!(BlendMode::Overlay.as_pdf_name(), "Overlay");
        assert_eq!(BlendMode::Darken.as_pdf_name(), "Darken");
        assert_eq!(BlendMode::Lighten.as_pdf_name(), "Lighten");
        assert_eq!(BlendMode::ColorDodge.as_pdf_name(), "ColorDodge");
        assert_eq!(BlendMode::ColorBurn.as_pdf_name(), "ColorBurn");
        assert_eq!(BlendMode::HardLight.as_pdf_name(), "HardLight");
        assert_eq!(BlendMode::SoftLight.as_pdf_name(), "SoftLight");
        assert_eq!(BlendMode::Difference.as_pdf_name(), "Difference");
        assert_eq!(BlendMode::Exclusion.as_pdf_name(), "Exclusion");
    }

    #[test]
    fn test_blend_mode_default() {
        let mode = BlendMode::default();
        assert_eq!(mode.as_pdf_name(), "Normal");
    }

    #[test]
    fn test_line_cap_default() {
        let cap = LineCap::default();
        assert_eq!(cap as u8, 0);
    }

    #[test]
    fn test_line_join_default() {
        let join = LineJoin::default();
        assert_eq!(join as u8, 0);
    }

    #[test]
    fn test_path_content_stroke_and_fill() {
        use crate::elements::PathContent;

        let path = PathContent {
            paint: Default::default(),
            operations: vec![
                PathOperation::MoveTo(0.0, 0.0),
                PathOperation::LineTo(100.0, 0.0),
                PathOperation::LineTo(100.0, 100.0),
                PathOperation::ClosePath,
            ],
            stroke_color: Some(Color::black()),
            fill_color: Some(Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
            }),
            stroke_width: 2.0,
            bbox: Rect::new(0.0, 0.0, 100.0, 100.0),
            line_cap: Default::default(),
            line_join: Default::default(),
            dash_pattern: None,
            matrix: None,
            reading_order: None,
            artifact_type: None,
            layer: None,
        };

        let mut builder = ContentStreamBuilder::new();
        builder.add_element(&ContentElement::Path(path));

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("B\n")); // FillStroke
    }

    #[test]
    fn test_path_content_stroke_only() {
        use crate::elements::PathContent;

        let path = PathContent {
            paint: Default::default(),
            operations: vec![
                PathOperation::MoveTo(0.0, 0.0),
                PathOperation::LineTo(100.0, 100.0),
            ],
            stroke_color: Some(Color::black()),
            fill_color: None,
            stroke_width: 1.0,
            bbox: Rect::new(0.0, 0.0, 100.0, 100.0),
            line_cap: Default::default(),
            line_join: Default::default(),
            dash_pattern: None,
            matrix: None,
            reading_order: None,
            artifact_type: None,
            layer: None,
        };

        let mut builder = ContentStreamBuilder::new();
        builder.add_element(&ContentElement::Path(path));

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("S\n")); // Stroke only
    }

    #[test]
    fn test_path_content_fill_only() {
        use crate::elements::PathContent;

        let path = PathContent {
            paint: Default::default(),
            operations: vec![PathOperation::Rectangle(0.0, 0.0, 100.0, 100.0)],
            stroke_color: None,
            fill_color: Some(Color {
                r: 0.0,
                g: 0.0,
                b: 1.0,
            }),
            stroke_width: 0.0,
            bbox: Rect::new(0.0, 0.0, 100.0, 100.0),
            line_cap: Default::default(),
            line_join: Default::default(),
            dash_pattern: None,
            matrix: None,
            reading_order: None,
            artifact_type: None,
            layer: None,
        };

        let mut builder = ContentStreamBuilder::new();
        builder.add_element(&ContentElement::Path(path));

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("f\n")); // Fill only
    }

    #[test]
    fn test_path_content_no_stroke_no_fill() {
        use crate::elements::PathContent;

        let path = PathContent {
            paint: Default::default(),
            operations: vec![
                PathOperation::MoveTo(0.0, 0.0),
                PathOperation::CurveTo(10.0, 20.0, 30.0, 40.0, 50.0, 60.0),
            ],
            stroke_color: None,
            fill_color: None,
            stroke_width: 0.0,
            bbox: Rect::new(0.0, 0.0, 50.0, 60.0),
            line_cap: Default::default(),
            line_join: Default::default(),
            dash_pattern: None,
            matrix: None,
            reading_order: None,
            artifact_type: None,
            layer: None,
        };

        let mut builder = ContentStreamBuilder::new();
        builder.add_element(&ContentElement::Path(path));

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("n\n")); // EndPath
    }

    #[test]
    fn test_empty_build() {
        let builder = ContentStreamBuilder::new();
        let bytes = builder.build().unwrap();
        assert!(bytes.is_empty());
    }
}
