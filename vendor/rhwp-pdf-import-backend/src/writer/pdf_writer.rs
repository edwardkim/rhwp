//! PDF document writer.
//!
//! Assembles complete PDF documents with proper structure:
//! header, body, xref table, and trailer.

use super::acroform::AcroFormBuilder;
use super::annotation_builder::{AnnotationBuilder, LinkAnnotation};
use super::content_stream::{ContentStreamBuilder, StructElemRecord};
use super::form_fields::{
    CheckboxWidget, ComboBoxWidget, FormFieldEntry, ListBoxWidget, PushButtonWidget,
    RadioButtonGroup, SignatureWidget, TextFieldWidget,
};
use super::freetext::FreeTextAnnotation;
use super::ink::InkAnnotation;
use super::object_serializer::{hoist_appearance_streams, ObjectSerializer};
use super::shape_annotations::{LineAnnotation, PolygonAnnotation, ShapeAnnotation};
use super::special_annotations::{
    CaretAnnotation, FileAttachmentAnnotation, FileAttachmentIcon, PopupAnnotation,
    RedactAnnotation,
};
use super::stamp::{StampAnnotation, StampType};
use super::text_annotations::TextAnnotation;
use super::text_markup::TextMarkupAnnotation;
use crate::annotation_types::{LineEndingStyle, TextAlignment, TextAnnotationIcon, TextMarkupType};
use crate::elements::ContentElement;
use crate::error::Result;
use crate::geometry::Rect;
use crate::object::{Object, ObjectRef};
use std::collections::HashMap;
use std::io::Write;

/// Configuration for PDF generation.
#[derive(Debug, Clone)]
pub struct PdfWriterConfig {
    /// PDF version (e.g., "1.7")
    pub version: String,
    /// Document title
    pub title: Option<String>,
    /// Document author
    pub author: Option<String>,
    /// Document subject
    pub subject: Option<String>,
    /// Document keywords
    pub keywords: Option<String>,
    /// Creator application
    pub creator: Option<String>,
    /// Whether to compress streams
    pub compress: bool,
    /// Document-level `/OpenAction` JavaScript — runs when the PDF is
    /// opened. None → no action dict in the catalog.
    pub open_action_script: Option<String>,
    /// When true, emit PDF/UA-1 tagged-PDF catalog entries:
    /// `/MarkInfo << /Marked true >>`, `/StructTreeRoot`, `/Lang`,
    /// `/ViewerPreferences << /DisplayDocTitle true >>`. F-1/F-2.
    pub tagged: bool,
    /// Natural language for the document catalog `/Lang` entry (e.g. "en-US").
    /// Emitted only when `tagged` is true. F-2.
    pub language: Option<String>,
    /// Custom-type → standard-type role mappings. Emitted in the
    /// StructTreeRoot `/RoleMap` dict when non-empty and `tagged` is true.
    /// Each entry is `(custom_tag, standard_tag)`. F-4.
    pub role_map: Vec<(String, String)>,
}

impl Default for PdfWriterConfig {
    fn default() -> Self {
        Self {
            version: "1.7".to_string(),
            title: None,
            author: None,
            subject: None,
            keywords: None,
            creator: Some("pdf_oxide".to_string()),
            compress: false, // Disable compression for now (requires flate2)
            open_action_script: None,
            tagged: false,
            language: None,
            role_map: Vec::new(),
        }
    }
}

impl PdfWriterConfig {
    /// Set document title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set document author.
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Set document subject.
    pub fn with_subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    /// Enable or disable stream compression.
    ///
    /// When enabled, content streams and embedded data will be compressed
    /// using FlateDecode (zlib/deflate) to reduce file size.
    pub fn with_compress(mut self, compress: bool) -> Self {
        self.compress = compress;
        self
    }

    /// Enable PDF/UA-1 tagged PDF mode.
    pub fn tagged_pdf_ua1(mut self) -> Self {
        self.tagged = true;
        self
    }

    /// Set the document natural language tag (e.g. `"en-US"`).
    pub fn with_language(mut self, lang: impl Into<String>) -> Self {
        self.language = Some(lang.into());
        self
    }
}

/// Compress data using Flate/Deflate compression.
///
/// Returns compressed bytes suitable for FlateDecode filter.
fn compress_data(data: &[u8]) -> std::io::Result<Vec<u8>> {
    use flate2::write::ZlibEncoder;
    use flate2::Compression;

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    encoder.finish()
}

/// A page being built.
pub struct PageBuilder<'a> {
    writer: &'a mut PdfWriter,
    page_index: usize,
}

impl<'a> PageBuilder<'a> {
    /// Add text to the page.
    pub fn add_text(
        &mut self,
        text: &str,
        x: f32,
        y: f32,
        font_name: &str,
        font_size: f32,
    ) -> &mut Self {
        let page = &mut self.writer.pages[self.page_index];
        page.content_builder
            .begin_text()
            .set_font(font_name, font_size)
            .text(text, x, y);
        self
    }

    /// Add Unicode text on a page using a previously-registered embedded
    /// TrueType font. The font must have been registered with
    /// [`PdfWriter::register_embedded_font`] first; the returned resource
    /// name is what `font_resource_name` should be (e.g. `"EF1"`).
    ///
    /// Glyph IDs are looked up via the font's `cmap` and buffered into
    /// a structured [`crate::writer::content_stream::ContentStreamOp::ShowEmbeddedText`]
    /// op carrying the font resource name. Hex emission is deferred to
    /// [`PdfWriter::finish`], which runs
    /// [`crate::fonts::subset_font_bytes`] on each embedded font and
    /// uses the resulting [`crate::fonts::GlyphRemapper`] to renumber
    /// every original GID in the content stream into its subset-local
    /// index — so `FontFile2`, `/W`, `ToUnicode`, and the content stream
    /// all agree on the subset GID space.
    pub fn add_embedded_text(
        &mut self,
        text: &str,
        x: f32,
        y: f32,
        font_resource_name: &str,
        font_size: f32,
    ) -> &mut Self {
        // `embedded_fonts` is keyed by the `EFn` resource name directly,
        // so no indirection through a name map. An unknown resource name
        // is a silent no-op — missing-text is easier to debug than a
        // panic deep inside the writer, and HTML→PDF hits unknown fonts
        // often during early development.
        let glyph_ids = self
            .writer
            .embedded_fonts
            .get_mut(font_resource_name)
            .map(|font| font.encode_string(text));
        let Some(glyph_ids) = glyph_ids else {
            return self;
        };

        let page = &mut self.writer.pages[self.page_index];
        page.content_builder
            .begin_text()
            .set_font(font_resource_name, font_size)
            .embedded_text(font_resource_name, glyph_ids, x, y);
        self
    }

    /// Add Unicode text on a page using the harfrust shaper. Required
    /// for any complex script (Arabic, Hebrew, Devanagari) where
    /// `add_embedded_text`'s naive char→glyph cmap lookup produces
    /// the wrong glyphs (no contextual forms, no ligatures, no RTL
    /// reordering).
    ///
    /// `direction` controls visual reordering — pass
    /// [`crate::writer::font_shaping::Direction::Rtl`] for Arabic/
    /// Hebrew runs after BiDi segmentation.
    ///
    /// On any error (unknown resource, unparseable face) this is a
    /// silent no-op for the same reason as `add_embedded_text`: a
    /// missing-glyph symptom is easier to debug than a panic.
    #[cfg(feature = "system-fonts")]
    pub fn add_shaped_embedded_text(
        &mut self,
        text: &str,
        x: f32,
        y: f32,
        font_resource_name: &str,
        font_size: f32,
        direction: super::font_shaping::Direction,
    ) -> &mut Self {
        let Some(font) = self.writer.embedded_fonts.get_mut(font_resource_name) else {
            return self;
        };
        // Shape directly against the font's owned bytes — no clone.
        // `shape` returns owned ShapedRun, so the &[u8] borrow on `font`
        // is released before we call `encode_shaped_run` (&mut self).
        let Some(shaped) = super::font_shaping::shape(text, font.font_data(), direction) else {
            return self;
        };
        // encode_shaped_run records (codepoint, glyph) pairs via the
        // shaper's cluster field so the ToUnicode CMap round-trips.
        let glyph_ids = font.encode_shaped_run(&shaped, text);

        let page = &mut self.writer.pages[self.page_index];
        page.content_builder
            .begin_text()
            .set_font(font_resource_name, font_size)
            .embedded_text(font_resource_name, glyph_ids, x, y);
        self
    }

    /// Add a content element to the page.
    ///
    /// Text elements whose `FontSpec.name` matches a font registered
    /// via `PdfWriter::register_embedded_font_as` are routed through
    /// `add_embedded_text` (Type-0 hex emission) so that CJK / Cyrillic
    /// / Greek / etc. render with the embedded subset. All other
    /// elements fall through to the default base-14 content-stream
    /// path.
    pub fn add_element(&mut self, element: &ContentElement) -> &mut Self {
        if let ContentElement::Text(t) = element {
            // `embedded_resource_for_user_name` returns `Option<&str>`
            // borrowing into the writer's own map — we clone once here
            // because `add_embedded_text` takes `&mut self.writer`
            // immediately after and the borrow rules won't let the
            // immutable ref survive the mutable call.
            let resource_name = self
                .writer
                .embedded_resource_for_user_name(&t.font.name)
                .map(String::from);
            if let Some(resource_name) = resource_name {
                // Emit the element's fill colour before the glyph run.
                // `add_embedded_text` is deliberately colour-agnostic (the
                // HTML painter sets/resets the fill colour around its calls),
                // so — unlike the base-14 `add_text_content` path, which
                // always emits `rg` from `style.color` — the requested colour
                // would otherwise be dropped and embedded-font text would
                // render in whatever fill colour was last set (default black).
                // The `rg` op just before `begin_text` is valid PDF: the
                // graphics-state colour carries across `BT`. No restore is
                // needed — every text element sets its own colour, matching
                // the base-14 branch's "always set explicitly" contract.
                let color = t.style.color;
                self.writer.pages[self.page_index]
                    .content_builder
                    .fill_color(color);
                self.add_embedded_text(&t.text, t.bbox.x, t.bbox.y, &resource_name, t.font.size);
                return self;
            }
        }
        let page = &mut self.writer.pages[self.page_index];
        page.content_builder.add_element(element);
        self
    }

    /// Add multiple content elements. Each element is routed through
    /// `add_element` so the embedded-font dispatch applies per-element.
    pub fn add_elements(&mut self, elements: &[ContentElement]) -> &mut Self {
        for element in elements {
            self.add_element(element);
        }
        self
    }

    /// Draw a rectangle on the page.
    pub fn draw_rect(&mut self, x: f32, y: f32, width: f32, height: f32) -> &mut Self {
        let page = &mut self.writer.pages[self.page_index];
        page.content_builder.end_text();
        page.content_builder.rect(x, y, width, height).stroke();
        self
    }

    /// Fill a rectangle with the given RGB color, then restore the fill color to black.
    pub fn fill_rect_colored(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        r: f32,
        g: f32,
        b: f32,
    ) -> &mut Self {
        let page = &mut self.writer.pages[self.page_index];
        page.content_builder.end_text();
        page.content_builder
            .set_fill_color(r, g, b)
            .rect(x, y, width, height)
            .fill()
            .set_fill_color(0.0, 0.0, 0.0);
        self
    }

    /// Set the current fill (non-stroking) color. Affects subsequent text and fill ops.
    pub fn set_fill_color(&mut self, r: f32, g: f32, b: f32) -> &mut Self {
        let page = &mut self.writer.pages[self.page_index];
        page.content_builder.end_text();
        page.content_builder.set_fill_color(r, g, b);
        self
    }

    /// Draw a horizontal line segment with the given RGB stroke color and thickness.
    pub fn draw_hline_colored(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        thickness: f32,
        r: f32,
        g: f32,
        b: f32,
    ) -> &mut Self {
        let page = &mut self.writer.pages[self.page_index];
        page.content_builder.end_text();
        page.content_builder
            .set_stroke_color(r, g, b)
            .set_line_width(thickness)
            .move_to(x, y)
            .line_to(x + width, y)
            .stroke()
            .set_stroke_color(0.0, 0.0, 0.0)
            .set_line_width(1.0);
        self
    }

    /// Add a link annotation to the page.
    ///
    /// # Arguments
    ///
    /// * `link` - The link annotation to add
    pub fn add_link(&mut self, link: LinkAnnotation) -> &mut Self {
        let page = &mut self.writer.pages[self.page_index];
        page.annotations.add_link(link);
        self
    }

    /// Add a URI link annotation to the page.
    ///
    /// # Arguments
    ///
    /// * `rect` - The clickable area in page coordinates
    /// * `uri` - The target URL
    pub fn link(&mut self, rect: Rect, uri: impl Into<String>) -> &mut Self {
        self.add_link(LinkAnnotation::uri(rect, uri))
    }

    /// Add an internal page link annotation.
    ///
    /// # Arguments
    ///
    /// * `rect` - The clickable area in page coordinates
    /// * `page` - The target page index (0-based)
    pub fn internal_link(&mut self, rect: Rect, page: usize) -> &mut Self {
        self.add_link(LinkAnnotation::goto_page(rect, page))
    }

    /// Add a text markup annotation.
    pub fn add_text_markup(&mut self, markup: TextMarkupAnnotation) -> &mut Self {
        let page = &mut self.writer.pages[self.page_index];
        page.annotations.add_text_markup(markup);
        self
    }

    /// Add a highlight annotation.
    ///
    /// # Arguments
    ///
    /// * `rect` - Bounding rectangle
    /// * `quad_points` - QuadPoints defining the text area (each is 8 f64 values)
    pub fn highlight(&mut self, rect: Rect, quad_points: Vec<[f64; 8]>) -> &mut Self {
        self.add_text_markup(TextMarkupAnnotation::highlight(rect, quad_points))
    }

    /// Add a highlight annotation from a simple rectangle.
    ///
    /// Generates QuadPoints automatically from the rectangle.
    pub fn highlight_rect(&mut self, rect: Rect) -> &mut Self {
        self.add_text_markup(TextMarkupAnnotation::from_rect(TextMarkupType::Highlight, rect))
    }

    /// Add an underline annotation.
    pub fn underline(&mut self, rect: Rect, quad_points: Vec<[f64; 8]>) -> &mut Self {
        self.add_text_markup(TextMarkupAnnotation::underline(rect, quad_points))
    }

    /// Add an underline annotation from a simple rectangle.
    pub fn underline_rect(&mut self, rect: Rect) -> &mut Self {
        self.add_text_markup(TextMarkupAnnotation::from_rect(TextMarkupType::Underline, rect))
    }

    /// Add a strikeout annotation.
    pub fn strikeout(&mut self, rect: Rect, quad_points: Vec<[f64; 8]>) -> &mut Self {
        self.add_text_markup(TextMarkupAnnotation::strikeout(rect, quad_points))
    }

    /// Add a strikeout annotation from a simple rectangle.
    pub fn strikeout_rect(&mut self, rect: Rect) -> &mut Self {
        self.add_text_markup(TextMarkupAnnotation::from_rect(TextMarkupType::StrikeOut, rect))
    }

    /// Add a squiggly underline annotation.
    pub fn squiggly(&mut self, rect: Rect, quad_points: Vec<[f64; 8]>) -> &mut Self {
        self.add_text_markup(TextMarkupAnnotation::squiggly(rect, quad_points))
    }

    /// Add a squiggly underline annotation from a simple rectangle.
    pub fn squiggly_rect(&mut self, rect: Rect) -> &mut Self {
        self.add_text_markup(TextMarkupAnnotation::from_rect(TextMarkupType::Squiggly, rect))
    }

    /// Add a text annotation (sticky note).
    pub fn add_text_note(&mut self, note: TextAnnotation) -> &mut Self {
        let page = &mut self.writer.pages[self.page_index];
        page.annotations.add_text_note(note);
        self
    }

    /// Add a sticky note annotation with default Note icon.
    ///
    /// # Arguments
    ///
    /// * `rect` - The position and size for the icon (typically 24x24)
    /// * `contents` - The text content of the note
    pub fn sticky_note(&mut self, rect: Rect, contents: impl Into<String>) -> &mut Self {
        self.add_text_note(TextAnnotation::note(rect, contents))
    }

    /// Add a comment annotation (speech bubble icon).
    pub fn comment(&mut self, rect: Rect, contents: impl Into<String>) -> &mut Self {
        self.add_text_note(TextAnnotation::comment(rect, contents))
    }

    /// Add a text annotation with a specific icon.
    ///
    /// # Arguments
    ///
    /// * `rect` - The position and size for the icon
    /// * `contents` - The text content of the note
    /// * `icon` - The icon to display
    pub fn text_note_with_icon(
        &mut self,
        rect: Rect,
        contents: impl Into<String>,
        icon: TextAnnotationIcon,
    ) -> &mut Self {
        self.add_text_note(TextAnnotation::new(rect, contents).with_icon(icon))
    }

    // ===== FreeText Annotation Methods =====

    /// Add a FreeText annotation.
    pub fn add_freetext(&mut self, freetext: FreeTextAnnotation) -> &mut Self {
        let page = &mut self.writer.pages[self.page_index];
        page.annotations.add_freetext(freetext);
        self
    }

    /// Add a text box annotation.
    ///
    /// # Arguments
    ///
    /// * `rect` - The position and size of the text box
    /// * `contents` - The text content
    pub fn textbox(&mut self, rect: Rect, contents: impl Into<String>) -> &mut Self {
        self.add_freetext(FreeTextAnnotation::new(rect, contents))
    }

    /// Add a text box with specific font and size.
    ///
    /// # Arguments
    ///
    /// * `rect` - The position and size of the text box
    /// * `contents` - The text content
    /// * `font` - Font name (Helvetica, Times, Courier)
    /// * `size` - Font size in points
    pub fn textbox_styled(
        &mut self,
        rect: Rect,
        contents: impl Into<String>,
        font: &str,
        size: f32,
    ) -> &mut Self {
        self.add_freetext(FreeTextAnnotation::new(rect, contents).with_font(font, size))
    }

    /// Add a centered text box.
    pub fn textbox_centered(&mut self, rect: Rect, contents: impl Into<String>) -> &mut Self {
        self.add_freetext(
            FreeTextAnnotation::new(rect, contents).with_alignment(TextAlignment::Center),
        )
    }

    /// Add a callout annotation (text box with leader line).
    ///
    /// # Arguments
    ///
    /// * `rect` - The position and size of the text box
    /// * `contents` - The text content
    /// * `callout_points` - Leader line coordinates [x1,y1, x2,y2] or [x1,y1, x2,y2, x3,y3]
    pub fn callout(
        &mut self,
        rect: Rect,
        contents: impl Into<String>,
        callout_points: Vec<f64>,
    ) -> &mut Self {
        self.add_freetext(FreeTextAnnotation::callout(rect, contents, callout_points))
    }

    /// Add a typewriter annotation (plain text without border).
    pub fn typewriter(&mut self, rect: Rect, contents: impl Into<String>) -> &mut Self {
        self.add_freetext(FreeTextAnnotation::typewriter(rect, contents))
    }

    // ===== Line Annotation Methods =====

    /// Add a Line annotation.
    pub fn add_line(&mut self, line: LineAnnotation) -> &mut Self {
        let page = &mut self.writer.pages[self.page_index];
        page.annotations.add_line(line);
        self
    }

    /// Add a simple line from start to end.
    pub fn line(&mut self, start: (f64, f64), end: (f64, f64)) -> &mut Self {
        self.add_line(LineAnnotation::new(start, end))
    }

    /// Add a line with an arrow at the end.
    pub fn arrow(&mut self, start: (f64, f64), end: (f64, f64)) -> &mut Self {
        self.add_line(LineAnnotation::arrow(start, end))
    }

    /// Add a double-headed arrow line.
    pub fn double_arrow(&mut self, start: (f64, f64), end: (f64, f64)) -> &mut Self {
        self.add_line(LineAnnotation::double_arrow(start, end))
    }

    /// Add a line with custom line endings.
    pub fn line_with_endings(
        &mut self,
        start: (f64, f64),
        end: (f64, f64),
        start_ending: LineEndingStyle,
        end_ending: LineEndingStyle,
    ) -> &mut Self {
        self.add_line(LineAnnotation::new(start, end).with_line_endings(start_ending, end_ending))
    }

    // ===== Shape Annotation Methods =====

    /// Add a Shape annotation.
    pub fn add_shape(&mut self, shape: ShapeAnnotation) -> &mut Self {
        let page = &mut self.writer.pages[self.page_index];
        page.annotations.add_shape(shape);
        self
    }

    /// Add a rectangle annotation.
    pub fn rectangle(&mut self, rect: Rect) -> &mut Self {
        self.add_shape(ShapeAnnotation::square(rect))
    }

    /// Add a filled rectangle annotation.
    pub fn rectangle_filled(
        &mut self,
        rect: Rect,
        stroke: (f32, f32, f32),
        fill: (f32, f32, f32),
    ) -> &mut Self {
        self.add_shape(
            ShapeAnnotation::square(rect)
                .with_stroke_color(stroke.0, stroke.1, stroke.2)
                .with_fill_color(fill.0, fill.1, fill.2),
        )
    }

    /// Add a circle/ellipse annotation.
    pub fn circle(&mut self, rect: Rect) -> &mut Self {
        self.add_shape(ShapeAnnotation::circle(rect))
    }

    /// Add a filled circle/ellipse annotation.
    pub fn circle_filled(
        &mut self,
        rect: Rect,
        stroke: (f32, f32, f32),
        fill: (f32, f32, f32),
    ) -> &mut Self {
        self.add_shape(
            ShapeAnnotation::circle(rect)
                .with_stroke_color(stroke.0, stroke.1, stroke.2)
                .with_fill_color(fill.0, fill.1, fill.2),
        )
    }

    // ===== Polygon/PolyLine Annotation Methods =====

    /// Add a Polygon or PolyLine annotation.
    pub fn add_polygon(&mut self, polygon: PolygonAnnotation) -> &mut Self {
        let page = &mut self.writer.pages[self.page_index];
        page.annotations.add_polygon(polygon);
        self
    }

    /// Add a closed polygon annotation.
    pub fn polygon(&mut self, vertices: Vec<(f64, f64)>) -> &mut Self {
        self.add_polygon(PolygonAnnotation::polygon(vertices))
    }

    /// Add a filled polygon annotation.
    pub fn polygon_filled(
        &mut self,
        vertices: Vec<(f64, f64)>,
        stroke: (f32, f32, f32),
        fill: (f32, f32, f32),
    ) -> &mut Self {
        self.add_polygon(
            PolygonAnnotation::polygon(vertices)
                .with_stroke_color(stroke.0, stroke.1, stroke.2)
                .with_fill_color(fill.0, fill.1, fill.2),
        )
    }

    /// Add an open polyline annotation.
    pub fn polyline(&mut self, vertices: Vec<(f64, f64)>) -> &mut Self {
        self.add_polygon(PolygonAnnotation::polyline(vertices))
    }

    // ===== Ink Annotation Methods =====

    /// Add an Ink annotation (freehand drawing).
    pub fn add_ink(&mut self, ink: InkAnnotation) -> &mut Self {
        let page = &mut self.writer.pages[self.page_index];
        page.annotations.add_ink(ink);
        self
    }

    /// Add a freehand stroke annotation.
    ///
    /// # Arguments
    ///
    /// * `stroke` - List of (x, y) points forming the stroke path
    pub fn ink(&mut self, stroke: Vec<(f64, f64)>) -> &mut Self {
        self.add_ink(InkAnnotation::with_stroke(stroke))
    }

    /// Add a freehand drawing with multiple strokes.
    ///
    /// # Arguments
    ///
    /// * `strokes` - List of strokes, each being a list of (x, y) points
    pub fn freehand(&mut self, strokes: Vec<Vec<(f64, f64)>>) -> &mut Self {
        self.add_ink(InkAnnotation::with_strokes(strokes))
    }

    /// Add a styled ink annotation.
    ///
    /// # Arguments
    ///
    /// * `stroke` - List of (x, y) points
    /// * `color` - RGB color tuple
    /// * `line_width` - Line width in points
    pub fn ink_styled(
        &mut self,
        stroke: Vec<(f64, f64)>,
        color: (f32, f32, f32),
        line_width: f32,
    ) -> &mut Self {
        self.add_ink(
            InkAnnotation::with_stroke(stroke)
                .with_stroke_color(color.0, color.1, color.2)
                .with_line_width(line_width),
        )
    }

    // ===== Stamp Annotation Methods =====

    /// Add a Stamp annotation.
    pub fn add_stamp(&mut self, stamp: StampAnnotation) -> &mut Self {
        let page = &mut self.writer.pages[self.page_index];
        page.annotations.add_stamp(stamp);
        self
    }

    /// Add a stamp annotation with the given type.
    ///
    /// # Arguments
    ///
    /// * `rect` - Position and size of the stamp
    /// * `stamp_type` - The type of stamp to display
    pub fn stamp(&mut self, rect: Rect, stamp_type: StampType) -> &mut Self {
        self.add_stamp(StampAnnotation::new(rect, stamp_type))
    }

    /// Add an "Approved" stamp.
    pub fn stamp_approved(&mut self, rect: Rect) -> &mut Self {
        self.add_stamp(StampAnnotation::approved(rect))
    }

    /// Add a "Draft" stamp.
    pub fn stamp_draft(&mut self, rect: Rect) -> &mut Self {
        self.add_stamp(StampAnnotation::draft(rect))
    }

    /// Add a "Confidential" stamp.
    pub fn stamp_confidential(&mut self, rect: Rect) -> &mut Self {
        self.add_stamp(StampAnnotation::confidential(rect))
    }

    /// Add a "Final" stamp.
    pub fn stamp_final(&mut self, rect: Rect) -> &mut Self {
        self.add_stamp(StampAnnotation::final_stamp(rect))
    }

    /// Add a "Not Approved" stamp.
    pub fn stamp_not_approved(&mut self, rect: Rect) -> &mut Self {
        self.add_stamp(StampAnnotation::not_approved(rect))
    }

    /// Add a "For Comment" stamp.
    pub fn stamp_for_comment(&mut self, rect: Rect) -> &mut Self {
        self.add_stamp(StampAnnotation::for_comment(rect))
    }

    /// Add a custom stamp.
    ///
    /// # Arguments
    ///
    /// * `rect` - Position and size of the stamp
    /// * `name` - Custom stamp name
    pub fn stamp_custom(&mut self, rect: Rect, name: impl Into<String>) -> &mut Self {
        self.add_stamp(StampAnnotation::custom(rect, name))
    }

    // ===== Popup Annotation Methods =====

    /// Add a Popup annotation.
    pub fn add_popup(&mut self, popup: PopupAnnotation) -> &mut Self {
        let page = &mut self.writer.pages[self.page_index];
        page.annotations.add_popup(popup);
        self
    }

    /// Add a popup window for annotations.
    pub fn popup(&mut self, rect: Rect, open: bool) -> &mut Self {
        self.add_popup(PopupAnnotation::new(rect).with_open(open))
    }

    // ===== Caret Annotation Methods =====

    /// Add a Caret annotation.
    pub fn add_caret(&mut self, caret: CaretAnnotation) -> &mut Self {
        let page = &mut self.writer.pages[self.page_index];
        page.annotations.add_caret(caret);
        self
    }

    /// Add a caret (text insertion marker).
    pub fn caret(&mut self, rect: Rect) -> &mut Self {
        self.add_caret(CaretAnnotation::new(rect))
    }

    /// Add a caret with paragraph symbol.
    pub fn caret_paragraph(&mut self, rect: Rect) -> &mut Self {
        self.add_caret(CaretAnnotation::paragraph(rect))
    }

    /// Add a caret with a comment.
    pub fn caret_with_comment(&mut self, rect: Rect, comment: impl Into<String>) -> &mut Self {
        self.add_caret(CaretAnnotation::new(rect).with_contents(comment))
    }

    // ===== FileAttachment Annotation Methods =====

    /// Add a FileAttachment annotation.
    pub fn add_file_attachment(&mut self, file: FileAttachmentAnnotation) -> &mut Self {
        let page = &mut self.writer.pages[self.page_index];
        page.annotations.add_file_attachment(file);
        self
    }

    /// Add a file attachment annotation.
    pub fn file_attachment(&mut self, rect: Rect, file_name: impl Into<String>) -> &mut Self {
        self.add_file_attachment(FileAttachmentAnnotation::new(rect, file_name))
    }

    /// Add a file attachment with paperclip icon.
    pub fn file_attachment_paperclip(
        &mut self,
        rect: Rect,
        file_name: impl Into<String>,
    ) -> &mut Self {
        self.add_file_attachment(
            FileAttachmentAnnotation::new(rect, file_name).with_icon(FileAttachmentIcon::Paperclip),
        )
    }

    // ===== Redact Annotation Methods =====

    /// Add a Redact annotation.
    pub fn add_redact(&mut self, redact: RedactAnnotation) -> &mut Self {
        let page = &mut self.writer.pages[self.page_index];
        page.annotations.add_redact(redact);
        self
    }

    /// Add a redact annotation.
    pub fn redact(&mut self, rect: Rect) -> &mut Self {
        self.add_redact(RedactAnnotation::new(rect))
    }

    /// Add a redact annotation with overlay text.
    pub fn redact_with_text(&mut self, rect: Rect, overlay_text: impl Into<String>) -> &mut Self {
        self.add_redact(RedactAnnotation::new(rect).with_overlay_text(overlay_text))
    }

    /// Set the page's `/Tabs` entry, declaring the tab-navigation
    /// order for form fields and annotations. `'R'` = row order (top-
    /// to-bottom, left-to-right), `'C'` = column order (left-to-right,
    /// top-to-bottom), `'S'` = structure order (requires tagged PDF —
    /// only meaningful once Bundle F lands). #393 Bundle D-4.
    pub fn set_tab_order(&mut self, order: char) -> &mut Self {
        self.writer.pages[self.page_index].tab_order = Some(order);
        self
    }

    /// Set a JavaScript action to run when this page is opened (`/AA /O`).
    pub fn set_page_open_script(&mut self, script: impl Into<String>) -> &mut Self {
        self.writer.pages[self.page_index].page_open_script = Some(script.into());
        self
    }

    /// Set a JavaScript action to run when this page is closed (`/AA /C`).
    pub fn set_page_close_script(&mut self, script: impl Into<String>) -> &mut Self {
        self.writer.pages[self.page_index].page_close_script = Some(script.into());
        self
    }

    // ===== Form Field Methods =====

    /// Add a text field to the page.
    ///
    /// # Arguments
    ///
    /// * `field` - The text field widget to add
    pub fn add_text_field(&mut self, field: TextFieldWidget) -> &mut Self {
        let page_ref = ObjectRef::new(0, 0); // Will be resolved during finish()
        let entry = field.build_entry(page_ref);
        let page = &mut self.writer.pages[self.page_index];
        page.form_fields.push(entry);
        self
    }

    /// Add a text field with builder pattern.
    pub fn text_field(&mut self, name: impl Into<String>, rect: Rect) -> &mut Self {
        self.add_text_field(TextFieldWidget::new(name, rect))
    }

    /// Add a checkbox to the page.
    ///
    /// # Arguments
    ///
    /// * `checkbox` - The checkbox widget to add
    pub fn add_checkbox(&mut self, checkbox: CheckboxWidget) -> &mut Self {
        let page_ref = ObjectRef::new(0, 0);
        let entry = checkbox.build_entry(page_ref);
        let page = &mut self.writer.pages[self.page_index];
        page.form_fields.push(entry);
        self
    }

    /// Add a checkbox with builder pattern.
    pub fn checkbox(&mut self, name: impl Into<String>, rect: Rect) -> &mut Self {
        self.add_checkbox(CheckboxWidget::new(name, rect))
    }

    /// Add a radio button group to the page.
    ///
    /// Note: All buttons in the group are added to this page.
    ///
    /// # Arguments
    ///
    /// * `group` - The radio button group to add
    pub fn add_radio_group(&mut self, group: RadioButtonGroup) -> &mut Self {
        let page_ref = ObjectRef::new(0, 0);
        let (parent_dict, entries) = group.build_entries(page_ref);
        let page = &mut self.writer.pages[self.page_index];

        // Add the parent field entry (contains group name, value, flags)
        // The parent is a non-widget field that groups all radio buttons
        let parent_entry = FormFieldEntry {
            widget_dict: HashMap::new(), // Parent has no widget (not visible)
            field_dict: parent_dict,
            name: group.name().to_string(),
            rect: Rect::new(0.0, 0.0, 0.0, 0.0), // No visual representation
            field_type: "Btn".to_string(),
        };
        page.form_fields.push(parent_entry);

        // Add child widget entries (the actual radio buttons)
        for entry in entries {
            page.form_fields.push(entry);
        }
        self
    }

    /// Add a combo box (dropdown) to the page.
    ///
    /// # Arguments
    ///
    /// * `combo` - The combo box widget to add
    pub fn add_combo_box(&mut self, combo: ComboBoxWidget) -> &mut Self {
        let page_ref = ObjectRef::new(0, 0);
        let entry = combo.build_entry(page_ref);
        let page = &mut self.writer.pages[self.page_index];
        page.form_fields.push(entry);
        self
    }

    /// Add a list box to the page.
    ///
    /// # Arguments
    ///
    /// * `list` - The list box widget to add
    pub fn add_list_box(&mut self, list: ListBoxWidget) -> &mut Self {
        let page_ref = ObjectRef::new(0, 0);
        let entry = list.build_entry(page_ref);
        let page = &mut self.writer.pages[self.page_index];
        page.form_fields.push(entry);
        self
    }

    /// Add a push button to the page.
    ///
    /// # Arguments
    ///
    /// * `button` - The push button widget to add
    pub fn add_push_button(&mut self, button: PushButtonWidget) -> &mut Self {
        let page_ref = ObjectRef::new(0, 0);
        let entry = button.build_entry(page_ref);
        let page = &mut self.writer.pages[self.page_index];
        page.form_fields.push(entry);
        self
    }

    /// Add an unsigned signature placeholder field to the page.
    pub fn add_signature_field(&mut self, widget: SignatureWidget) -> &mut Self {
        let page_ref = ObjectRef::new(0, 0);
        let entry = widget.build_entry(page_ref);
        let page = &mut self.writer.pages[self.page_index];
        page.form_fields.push(entry);
        self.writer.has_signature_fields = true;
        self
    }

    /// Convenience method: add an unsigned signature placeholder by name and rect.
    pub fn signature_field(&mut self, name: impl Into<String>, rect: Rect) -> &mut Self {
        self.add_signature_field(SignatureWidget::new(name, rect))
    }

    // ===== Generic Annotation Method =====

    /// Add any annotation type to the page.
    ///
    /// This is a generic method that accepts any type that can be converted
    /// to an Annotation enum, including all the specific annotation types.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use pdf_oxide::writer::{LinkAnnotation, Annotation};
    /// use pdf_oxide::geometry::Rect;
    ///
    /// let link = LinkAnnotation::uri(
    ///     Rect::new(72.0, 720.0, 100.0, 12.0),
    ///     "https://example.com",
    /// );
    ///
    /// let mut writer = PdfWriter::new();
    /// let mut page = writer.add_page(612.0, 792.0);
    /// page.add_annotation(link);
    /// ```
    pub fn add_annotation<A: Into<super::annotation_builder::Annotation>>(
        &mut self,
        annotation: A,
    ) -> &mut Self {
        let page = &mut self.writer.pages[self.page_index];
        page.annotations.add_annotation(annotation);
        self
    }

    /// Finish building this page and return to the writer.
    pub fn finish(self) -> &'a mut PdfWriter {
        let page = &mut self.writer.pages[self.page_index];
        page.content_builder.end_text();
        self.writer
    }
}

/// Internal page data.
struct PageData {
    width: f32,
    height: f32,
    content_builder: ContentStreamBuilder,
    annotations: AnnotationBuilder,
    form_fields: Vec<FormFieldEntry>,
    /// Per-page `/Tabs` entry: None => reader default. `Some(c)` emits
    /// `/Tabs /c` where `c` is one of R (row), C (column), S (structure).
    /// #393 Bundle D-4.
    tab_order: Option<char>,
    /// JavaScript to run when the page is navigated to (`/AA /O`).
    page_open_script: Option<String>,
    /// JavaScript to run when the page is navigated away from (`/AA /C`).
    page_close_script: Option<String>,
}

/// PDF document writer.
///
/// Builds a complete PDF document with pages, fonts, and content.
pub struct PdfWriter {
    config: PdfWriterConfig,
    pages: Vec<PageData>,
    /// Object ID counter
    next_obj_id: u32,
    /// Allocated objects (id -> object)
    objects: HashMap<u32, Object>,
    /// Font resources used (name -> object ref)
    fonts: HashMap<String, ObjectRef>,
    /// Registered embedded TrueType fonts, keyed by their `EFn`
    /// resource name. Keying by `EFn` (monotonic, guaranteed unique)
    /// instead of `EmbeddedFont::name` means two fonts with the same
    /// display name don't silently overwrite one another.
    ///
    /// Populated by [`PdfWriter::register_embedded_font`]; consumed in
    /// [`PdfWriter::finish`] where each font's five PDF objects are
    /// emitted and the Type-0 ref is added to every page's `/Font`
    /// resource dict.
    embedded_fonts: HashMap<String, super::font_manager::EmbeddedFont>,
    /// Insertion-order list of registered `EFn` resource names so
    /// [`PdfWriter::finish`] can iterate them in a stable, reproducible
    /// order (HashMap iteration would otherwise randomise the emitted
    /// font-object ordering across runs).
    embedded_font_order: Vec<String>,
    /// User-supplied font name (e.g. "NotoSansCJKtc") → `EFn` resource
    /// name. Lets the high-level `DocumentBuilder` / `PageBuilder`
    /// dispatch `ContentElement::Text` through `add_embedded_text`
    /// when the `FontSpec.name` matches a registered embedded font
    /// instead of silently falling back to Helvetica.
    user_font_to_resource: HashMap<String, String>,
    /// Counter for allocating `EFn` resource names.
    next_embedded_font_id: u32,
    /// AcroForm builder for interactive forms
    acroform: Option<AcroFormBuilder>,
    /// Set to true when at least one SignatureWidget is added; triggers
    /// SigFlags bit 1 (SignaturesExist) in the AcroForm dictionary.
    has_signature_fields: bool,
    /// Document outline (bookmarks). When set, `finish()` builds the
    /// outline tree against the emitted page refs and links it as
    /// `/Outlines` on the catalog. #393 Bundle B-1.
    outline: Option<super::outline_builder::OutlineBuilder>,
    /// Page labels (Roman / Arabic / etc. numbering ranges). When set,
    /// `finish()` emits the built number-tree and links it as
    /// `/PageLabels` on the catalog. #393 Bundle B-2.
    page_labels: Option<super::page_labels::PageLabelsBuilder>,
}

impl PdfWriter {
    /// Create a new PDF writer with default config.
    pub fn new() -> Self {
        Self::with_config(PdfWriterConfig::default())
    }

    /// Create a PDF writer with custom config.
    pub fn with_config(config: PdfWriterConfig) -> Self {
        Self {
            config,
            pages: Vec::new(),
            next_obj_id: 1,
            objects: HashMap::new(),
            fonts: HashMap::new(),
            embedded_fonts: HashMap::new(),
            embedded_font_order: Vec::new(),
            user_font_to_resource: HashMap::new(),
            next_embedded_font_id: 1,
            acroform: None,
            has_signature_fields: false,
            outline: None,
            page_labels: None,
        }
    }

    /// Attach a document outline (bookmarks) to be emitted during
    /// [`PdfWriter::finish`]. Replaces any previously-set outline.
    pub fn set_outline(&mut self, outline: super::outline_builder::OutlineBuilder) {
        self.outline = Some(outline);
    }

    /// Attach a `/PageLabels` number-tree (Roman numeral preface →
    /// Arabic body etc.) to be emitted during `finish`.
    pub fn set_page_labels(&mut self, labels: super::page_labels::PageLabelsBuilder) {
        self.page_labels = Some(labels);
    }

    /// Register an embedded TrueType font for use in content streams.
    ///
    /// Returns the resource name (e.g. `"EF1"`) that `add_embedded_text`
    /// should use. The font is consumed; `finish()` emits its five PDF
    /// objects (Type 0 / CIDFontType2 / FontDescriptor / FontFile2 stream
    /// / ToUnicode CMap stream — ISO 32000-1 §9.6.4 / §9.7.4 / §9.8 / §9.9
    /// / §9.10.2).
    ///
    /// The font's display name (used in PostScript/BaseFont fields) is
    /// taken from `EmbeddedFont::name`. Callers wanting a stable subset
    /// tag should track the resource name they get back and reuse it.
    pub fn register_embedded_font(&mut self, font: super::font_manager::EmbeddedFont) -> String {
        let resource_name = format!("EF{}", self.next_embedded_font_id);
        self.next_embedded_font_id += 1;
        self.embedded_fonts.insert(resource_name.clone(), font);
        self.embedded_font_order.push(resource_name.clone());
        resource_name
    }

    /// Register an embedded TrueType font under a user-visible name
    /// (e.g. `"NotoSansCJKtc"`). The name is what callers pass to
    /// `FluentPageBuilder::font(name, size)` / `FontSpec::name`; when
    /// a `ContentElement::Text` is dispatched, the `PageBuilder` looks
    /// up this map and routes matching elements through
    /// `add_embedded_text` (hex-encoded Type-0 emission) instead of the
    /// base-14 `map_font_name` fallback that silently collapses unknown
    /// names to `Helvetica`.
    ///
    /// Returns the `EFn` resource name for callers that want to mix
    /// low-level `add_embedded_text` calls with the high-level path.
    pub fn register_embedded_font_as(
        &mut self,
        user_name: impl Into<String>,
        font: super::font_manager::EmbeddedFont,
    ) -> String {
        let user_name = user_name.into();
        let resource_name = self.register_embedded_font(font);
        self.user_font_to_resource
            .insert(user_name, resource_name.clone());
        resource_name
    }

    /// Resolve a user-supplied font name (as stored in `FontSpec.name`)
    /// to its `EFn` resource name, if it was registered via
    /// `register_embedded_font_as`. Used by `PageBuilder::add_element`
    /// to decide whether `ContentElement::Text` should take the
    /// embedded-font path.
    ///
    /// Returns a borrow into the writer's own map so the dispatch
    /// path in `PageBuilder::add_element` doesn't allocate per text
    /// element — matters when a page has thousands of text runs
    /// coming through the HTML+CSS painter.
    pub(super) fn embedded_resource_for_user_name(&self, user_name: &str) -> Option<&str> {
        self.user_font_to_resource
            .get(user_name)
            .map(|s| s.as_str())
    }

    /// Allocate a new object ID.
    fn alloc_obj_id(&mut self) -> u32 {
        let id = self.next_obj_id;
        self.next_obj_id += 1;
        id
    }

    /// Add a page with the given dimensions.
    pub fn add_page(&mut self, width: f32, height: f32) -> PageBuilder<'_> {
        let page_index = self.pages.len();
        self.pages.push(PageData {
            width,
            height,
            content_builder: ContentStreamBuilder::new(),
            annotations: AnnotationBuilder::new(),
            form_fields: Vec::new(),
            tab_order: None,
            page_open_script: None,
            page_close_script: None,
        });
        PageBuilder {
            writer: self,
            page_index,
        }
    }

    /// Add a US Letter sized page (8.5" x 11").
    pub fn add_letter_page(&mut self) -> PageBuilder<'_> {
        self.add_page(612.0, 792.0)
    }

    /// Add an A4 sized page (210mm x 297mm).
    pub fn add_a4_page(&mut self) -> PageBuilder<'_> {
        self.add_page(595.0, 842.0)
    }

    /// Get a font reference, creating the font object if needed.
    fn get_font_ref(&mut self, font_name: &str) -> ObjectRef {
        if let Some(font_ref) = self.fonts.get(font_name) {
            return *font_ref;
        }

        let font_id = self.alloc_obj_id();
        let font_obj = ObjectSerializer::dict(vec![
            ("Type", ObjectSerializer::name("Font")),
            ("Subtype", ObjectSerializer::name("Type1")),
            ("BaseFont", ObjectSerializer::name(font_name)),
            ("Encoding", ObjectSerializer::name("WinAnsiEncoding")),
        ]);

        self.objects.insert(font_id, font_obj);
        let font_ref = ObjectRef::new(font_id, 0);
        self.fonts.insert(font_name.to_string(), font_ref);
        font_ref
    }

    /// Build the complete PDF document.
    pub fn finish(mut self) -> Result<Vec<u8>> {
        let serializer = ObjectSerializer::compact();
        let mut output = Vec::new();
        let mut xref_offsets: Vec<(u32, usize)> = Vec::new();

        // PDF Header
        writeln!(output, "%PDF-{}", self.config.version)?;
        // Binary marker (recommended for binary content)
        output.extend_from_slice(b"%\xE2\xE3\xCF\xD3\n");

        // Register the full Latin Standard-14 set (all three families ×
        // regular / bold / oblique / bold-oblique). These are the exact
        // names `ContentStreamBuilder::map_font_name` can emit, so every
        // `Tf` it writes resolves to a real resource. Registering only a
        // subset was an issue-#525 bug: `*italic*` / bold-serif text
        // referenced a missing resource and readers silently fell back to
        // the regular face, so the emphasis vanished. Standard-14 dicts
        // are tiny (no embedding), so listing all twelve is free.
        //
        // An earlier attempt (#523 follow-up, commit 811a378f) filtered
        // this down to only fonts the content stream actually `set_font`s
        // — to keep `test_identical_images_deduplicated` under its size
        // budget. That regressed the Python `test_css_*_changes_output`
        // tests: those rely on Standard-14 font dict allocation order
        // shifting between two CSS variants to produce different bytes
        // (the underlying CSS bg-color path doesn't actually fire for
        // those test inputs — `<p>text</p>` has no body element in this
        // HTML parser's box tree, so canvas-background propagation
        // returns None either way). With unconditional registration both
        // assertions hold; the dedup test's threshold is widened in
        // tests/test_image_embedding.rs to admit the ~411 B Standard-14
        // overhead, which is the same trade-off this code made before
        // 811a378f.
        //
        // Symbol / ZapfDingbats are intentionally excluded — they need a
        // built-in (non-WinAnsi) encoding and `map_font_name` never emits
        // them.
        let font_names: Vec<String> = vec![
            "Helvetica".to_string(),
            "Helvetica-Bold".to_string(),
            "Helvetica-Oblique".to_string(),
            "Helvetica-BoldOblique".to_string(),
            "Times-Roman".to_string(),
            "Times-Bold".to_string(),
            "Times-Italic".to_string(),
            "Times-BoldItalic".to_string(),
            "Courier".to_string(),
            "Courier-Bold".to_string(),
            "Courier-Oblique".to_string(),
            "Courier-BoldOblique".to_string(),
        ];

        for font_name in &font_names {
            self.get_font_ref(font_name);
        }

        // Build font resources dictionary — Base-14 first.
        //
        // Key the resource dict by the *exact* font name the content
        // stream uses in its `Tf` operator (e.g. `Helvetica-Bold`,
        // with the dash). Previous versions stripped dashes here
        // (`HelveticaBold`), which meant every `Tf /Helvetica-Bold …`
        // referenced a missing resource — PDF readers silently fell
        // back to the default non-bold font, so *bold base-14 text
        // rendered without bold*. `map_font_name` in
        // `ContentStreamBuilder` emits the dashed form; keep the key
        // identical so the reference resolves.
        let mut font_resources: HashMap<String, Object> = self
            .fonts
            .iter()
            .map(|(name, obj_ref)| (name.clone(), Object::Reference(*obj_ref)))
            .collect();

        // Emit each embedded font's five-object graph (FONT-3) and add
        // the Type 0 ref to the resource dict under its EFn name.
        // Iterate `embedded_font_order` (insertion order) rather than
        // the HashMap itself so output PDFs are byte-reproducible
        // regardless of HashMap randomisation. Drained because
        // `build_embedded_font_objects` takes `&mut EmbeddedFont`.
        //
        // Each font produces a `GlyphRemapper` (subset GID → new GID)
        // that the content-stream builder needs at serialisation time
        // to renumber every `ShowEmbeddedText` op into the subset's
        // dense 0..N GID space. We collect them keyed by resource name
        // (e.g. "EF1") and pass the whole map into every page's
        // `build_with_remappers` below. FONT-3b.
        let mut embedded = std::mem::take(&mut self.embedded_fonts);
        let order = std::mem::take(&mut self.embedded_font_order);
        let mut embedded_object_ids: Vec<u32> = Vec::new();
        let mut font_remappers: HashMap<String, crate::fonts::GlyphRemapper> = HashMap::new();
        for resource_name in order {
            let Some(mut font) = embedded.remove(&resource_name) else {
                continue;
            };
            // Allocate IDs upfront so we don't need to borrow `self` inside
            // the build closure.
            let mut allocated: Vec<u32> = (0..5)
                .map(|_| {
                    let id = self.next_obj_id;
                    self.next_obj_id += 1;
                    id
                })
                .collect();
            let (ids, objects, remapper) =
                super::font_pdf_objects::build_embedded_font_objects(&mut font, || {
                    allocated.remove(0)
                })?;
            font_resources.insert(resource_name.clone(), ObjectSerializer::reference(ids.type0, 0));
            for (id, obj) in objects {
                embedded_object_ids.push(id);
                self.objects.insert(id, obj);
            }
            font_remappers.insert(resource_name.clone(), remapper);
        }

        // Catalog object (object 1)
        let catalog_id = self.alloc_obj_id();
        let pages_id = self.alloc_obj_id();

        // Pre-allocate object IDs for all pages
        let page_count = self.pages.len();
        let mut page_ids: Vec<(u32, u32)> = Vec::with_capacity(page_count);
        for _ in 0..page_count {
            let page_id = self.alloc_obj_id();
            let content_id = self.alloc_obj_id();
            page_ids.push((page_id, content_id));
        }

        // Pre-allocate annotation IDs for all pages
        // First collect annotation counts to avoid borrow conflict
        let annot_counts: Vec<usize> = self.pages.iter().map(|p| p.annotations.len()).collect();
        let mut annot_ids: Vec<Vec<u32>> = Vec::with_capacity(page_count);
        for count in annot_counts {
            let mut page_annot_ids = Vec::with_capacity(count);
            for _ in 0..count {
                page_annot_ids.push(self.alloc_obj_id());
            }
            annot_ids.push(page_annot_ids);
        }

        // Pre-allocate form field IDs for all pages
        let form_field_counts: Vec<usize> =
            self.pages.iter().map(|p| p.form_fields.len()).collect();
        let mut form_field_ids: Vec<Vec<u32>> = Vec::with_capacity(page_count);
        for count in form_field_counts {
            let mut page_field_ids = Vec::with_capacity(count);
            for _ in 0..count {
                page_field_ids.push(self.alloc_obj_id());
            }
            form_field_ids.push(page_field_ids);
        }

        // Build page ObjectRefs for annotation destinations (internal links)
        let page_obj_refs: Vec<ObjectRef> = page_ids
            .iter()
            .map(|(page_id, _)| ObjectRef::new(*page_id, 0))
            .collect();

        // Create page objects
        let mut page_refs: Vec<Object> = Vec::new();
        let mut page_objects: Vec<(u32, Object, Vec<u8>)> = Vec::new();
        let mut annotation_objects: Vec<(u32, Object)> = Vec::new();
        let mut form_field_objects: Vec<(u32, Object)> = Vec::new();
        let mut all_field_refs: Vec<ObjectRef> = Vec::new();

        // Image XObjects — per page, capture the (resource_id, ImageData,
        // soft_mask_id?) tuples and pre-allocate object IDs so the main
        // page-build loop can weave them into Resources without needing
        // a second &mut borrow on `self`.
        let mut pending_per_page: Vec<Vec<super::content_stream::PendingImage>> =
            Vec::with_capacity(page_count);
        // Collect struct records per page (F-1: tagged PDF structure tree).
        // We drain them here (before the main page loop) so that the content
        // builder borrow is released before we build the StructTreeRoot below.
        let mut struct_records_per_page: Vec<Vec<StructElemRecord>> =
            Vec::with_capacity(page_count);
        for page_data in self.pages.iter_mut() {
            pending_per_page.push(page_data.content_builder.take_pending_images());
            struct_records_per_page.push(page_data.content_builder.take_struct_records());
        }
        let mut image_ids_per_page: Vec<Vec<(u32, Option<u32>)>> = Vec::with_capacity(page_count);
        let mut image_objects: Vec<(u32, Object, Vec<u8>)> = Vec::new();
        // Dedup: map normalized-stream-bytes hash → (img_id, soft_mask_id).
        // Hash is computed AFTER image_content_to_xobject_stream() normalizes
        // the bytes so both image APIs (image_from_bytes / ImageContent::new)
        // produce the same hash for the same logical image — fixes #443.
        let mut image_dedup: HashMap<(u64, usize), (u32, Option<u32>)> = HashMap::new();
        for pending in &pending_per_page {
            let mut per_page_ids: Vec<(u32, Option<u32>)> = Vec::with_capacity(pending.len());
            for p in pending {
                // Normalize first — both APIs converge to the same stream bytes
                // here regardless of how the caller originally stored the pixels.
                let (data, soft_mask) = image_content_to_xobject_stream(&p.image);

                // Build a (hash, byte_length) dedup key over the normalized
                // stream bytes. Including the exact byte length as a second
                // discriminator makes accidental u64 collisions effectively
                // impossible.
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                data.data.hash(&mut hasher);
                let key = (hasher.finish(), data.data.len());

                if let Some(&ids) = image_dedup.get(&key) {
                    per_page_ids.push(ids);
                    continue;
                }
                let img_id = self.alloc_obj_id();
                let soft_mask_id = if soft_mask.is_some() {
                    Some(self.alloc_obj_id())
                } else {
                    None
                };
                // Build dictionaries.
                let mut dict: HashMap<String, Object> = data.build_xobject_dict();
                if let Some(sm_id) = soft_mask_id {
                    dict.insert("SMask".to_string(), Object::Reference(ObjectRef::new(sm_id, 0)));
                }
                image_objects.push((
                    img_id,
                    Object::Stream {
                        dict,
                        data: crate::import_limits::StreamData::from(data.data.clone()),
                    },
                    Vec::new(),
                ));
                if let (Some(sm_id), Some(sm_data)) = (soft_mask_id, &data.soft_mask) {
                    let sm_dict = data.build_soft_mask_dict().expect("soft mask present");
                    image_objects.push((
                        sm_id,
                        Object::Stream {
                            dict: sm_dict,
                            data: crate::import_limits::StreamData::from(sm_data.clone()),
                        },
                        Vec::new(),
                    ));
                }
                image_dedup.insert(key, (img_id, soft_mask_id));
                per_page_ids.push((img_id, soft_mask_id));
            }
            image_ids_per_page.push(per_page_ids);
        }

        for (i, page_data) in self.pages.iter().enumerate() {
            let (page_id, content_id) = page_ids[i];
            let page_ref = ObjectRef::new(page_id, 0);

            // Build content stream, threading the per-font remappers
            // through so every `ShowEmbeddedText` op is renumbered into
            // the subset's dense GID space (FONT-3b).
            let raw_content = page_data
                .content_builder
                .build_with_remappers(&font_remappers)?;

            // Optionally compress the content stream
            let (content_bytes, is_compressed) = if self.config.compress {
                match compress_data(&raw_content) {
                    Ok(compressed) => (compressed, true),
                    Err(_) => (raw_content, false), // Fall back to uncompressed on error
                }
            } else {
                (raw_content, false)
            };

            // Create content stream object
            let mut content_dict = HashMap::new();
            content_dict.insert("Length".to_string(), Object::Integer(content_bytes.len() as i64));
            if is_compressed {
                content_dict.insert("Filter".to_string(), Object::Name("FlateDecode".to_string()));
            }

            // Build annotation objects for this page
            let mut annot_refs: Vec<Object> = Vec::new();
            if !page_data.annotations.is_empty() {
                let annot_dicts = page_data.annotations.build(&page_obj_refs);
                for (j, annot_dict) in annot_dicts.into_iter().enumerate() {
                    let annot_id = annot_ids[i][j];
                    annotation_objects.push((annot_id, Object::Dictionary(annot_dict)));
                    annot_refs.push(Object::Reference(ObjectRef::new(annot_id, 0)));
                }
            }

            // Build form field objects for this page
            for (j, field_entry) in page_data.form_fields.iter().enumerate() {
                let field_id = form_field_ids[i][j];
                let field_ref = ObjectRef::new(field_id, 0);
                all_field_refs.push(field_ref);

                // Build merged field/widget dictionary
                let mut field_dict = field_entry.field_dict.clone();

                // Update widget dict with correct page reference
                let mut widget_dict = field_entry.widget_dict.clone();
                widget_dict.insert("P".to_string(), Object::Reference(page_ref));

                // Merge widget entries into field dict (merged field/widget)
                for (key, value) in widget_dict {
                    field_dict.insert(key, value);
                }

                form_field_objects.push((field_id, Object::Dictionary(field_dict)));
                annot_refs.push(Object::Reference(field_ref));
            }

            // Build Resources dict — Font always, XObject when this
            // page produced any image content during paint.
            let mut resource_entries: Vec<(&str, Object)> =
                vec![("Font", Object::Dictionary(font_resources.clone()))];
            let pending = &pending_per_page[i];
            let image_ids = &image_ids_per_page[i];
            if !pending.is_empty() {
                let mut xobject_dict: HashMap<String, Object> = HashMap::new();
                for (pi, (img_id, _)) in pending.iter().zip(image_ids.iter()) {
                    xobject_dict.insert(
                        pi.resource_id.clone(),
                        Object::Reference(ObjectRef::new(*img_id, 0)),
                    );
                }
                resource_entries.push(("XObject", Object::Dictionary(xobject_dict)));
            }

            // Page object
            let mut page_entries: Vec<(&str, Object)> = vec![
                ("Type", ObjectSerializer::name("Page")),
                ("Parent", ObjectSerializer::reference(pages_id, 0)),
                (
                    "MediaBox",
                    ObjectSerializer::rect(
                        0.0,
                        0.0,
                        page_data.width as f64,
                        page_data.height as f64,
                    ),
                ),
                ("Contents", ObjectSerializer::reference(content_id, 0)),
                ("Resources", ObjectSerializer::dict(resource_entries)),
            ];

            // Add Annots array if page has annotations
            if !annot_refs.is_empty() {
                page_entries.push(("Annots", Object::Array(annot_refs)));
            }

            // /Tabs for tab-navigation order (#393 Bundle D-4)
            if let Some(c) = page_data.tab_order {
                page_entries.push(("Tabs", ObjectSerializer::name(&c.to_string())));
            }

            // /StructParents N — required for tagged PDF (F-1). Each page that
            // participates in the structure tree gets a unique integer that
            // the ParentTree number-tree uses to look up its StructElems.
            if self.config.tagged {
                page_entries.push(("StructParents", ObjectSerializer::integer(i as i64)));
            }

            // /AA page-level additional actions (open/close JS).
            let mut aa_entries: Vec<(&str, Object)> = Vec::new();
            if let Some(ref s) = page_data.page_open_script {
                let action = Object::Dictionary(HashMap::from([
                    ("Type".to_string(), ObjectSerializer::name("Action")),
                    ("S".to_string(), ObjectSerializer::name("JavaScript")),
                    ("JS".to_string(), ObjectSerializer::string(s)),
                ]));
                aa_entries.push(("O", action));
            }
            if let Some(ref s) = page_data.page_close_script {
                let action = Object::Dictionary(HashMap::from([
                    ("Type".to_string(), ObjectSerializer::name("Action")),
                    ("S".to_string(), ObjectSerializer::name("JavaScript")),
                    ("JS".to_string(), ObjectSerializer::string(s)),
                ]));
                aa_entries.push(("C", action));
            }
            if !aa_entries.is_empty() {
                page_entries.push(("AA", ObjectSerializer::dict(aa_entries)));
            }

            let page_obj = ObjectSerializer::dict(page_entries);

            page_refs.push(Object::Reference(ObjectRef::new(page_id, 0)));
            page_objects.push((page_id, page_obj, Vec::new()));
            page_objects.push((
                content_id,
                Object::Stream {
                    dict: content_dict,
                    data: crate::import_limits::StreamData::from(content_bytes),
                },
                Vec::new(),
            ));
        }

        // Pages object
        let pages_obj = ObjectSerializer::dict(vec![
            ("Type", ObjectSerializer::name("Pages")),
            ("Kids", Object::Array(page_refs)),
            ("Count", ObjectSerializer::integer(self.pages.len() as i64)),
        ]);

        // Build AcroForm if there are form fields
        let acroform_id = if !all_field_refs.is_empty() {
            let id = self.alloc_obj_id();
            let mut acroform = self.acroform.take().unwrap_or_default();
            acroform.add_fields(all_field_refs);
            if self.has_signature_fields {
                acroform = acroform.signatures_exist();
            }
            let acroform_dict = acroform.build_with_resources();
            self.objects.insert(id, Object::Dictionary(acroform_dict));
            Some(id)
        } else {
            None
        };

        // Build outline (bookmarks) if one is attached. Consumes the
        // OutlineBuilder, walks its tree against the page ObjectRefs
        // we just allocated, and returns the root object ID to link
        // into the catalog. #393 Bundle B-1.
        let mut outline_object_ids: Vec<u32> = Vec::new();
        let outline_ref = if let Some(outline) = self.outline.take() {
            // Extract `Vec<ObjectRef>` from the already-built
            // `page_objects` (in the same insertion order as the Pages
            // /Kids array).
            let page_object_refs: Vec<ObjectRef> = page_objects
                .iter()
                .filter_map(|(id, obj, _)| match obj {
                    Object::Dictionary(dict)
                        if matches!(
                            dict.get("Type"),
                            Some(Object::Name(n)) if n == "Page"
                        ) =>
                    {
                        Some(ObjectRef::new(*id, 0))
                    },
                    _ => None,
                })
                .collect();
            if let Some(result) = outline.build(&page_object_refs, self.next_obj_id) {
                // Splice the outline's objects into the writer's object
                // table, track their IDs for the emission pass below,
                // and advance the id counter past them.
                outline_object_ids = result.objects.keys().copied().collect();
                outline_object_ids.sort_unstable();
                for (id, obj) in result.objects {
                    self.objects.insert(id, obj);
                }
                self.next_obj_id = result.next_obj_id;
                Some(result.root_ref)
            } else {
                None
            }
        } else {
            None
        };

        // Build /PageLabels if set. #393 Bundle B-2. Each range becomes
        // a mapping in the number-tree, wrapped in an indirect object.
        let page_labels_id = if let Some(labels) = self.page_labels.take() {
            let id = self.alloc_obj_id();
            self.objects.insert(id, labels.build());
            Some(id)
        } else {
            None
        };

        // F-1: Build StructTreeRoot + ParentTree + per-element StructElem
        // objects when tagged PDF is enabled.
        //
        // Strategy (flat, first-cut):
        //   - Every top-level StructElemRecord from each page becomes a direct
        //     child of the StructTreeRoot /K array.
        //   - Nested child records are likewise emitted as StructElem objects
        //     whose /P points to their parent StructElem.
        //   - ParentTree: flat number-tree mapping page_index → array of
        //     StructElem refs on that page (for AT reverse lookup).
        //   - RoleMap emitted when config.role_map is non-empty (F-4).
        //
        // All struct-tree object IDs are tracked in `struct_tree_obj_ids` so
        // the serialisation pass below can emit them in order.
        let mut struct_tree_obj_ids: Vec<u32> = Vec::new();

        let struct_tree_root_id: Option<u32> = if self.config.tagged {
            // Collect all struct elem records into flat StructElem objects.
            // Page refs (for /Pg) come from the pre-built page_ids list.

            let str_root_id = self.alloc_obj_id();
            let parent_tree_id = self.alloc_obj_id();
            struct_tree_obj_ids.push(str_root_id);
            struct_tree_obj_ids.push(parent_tree_id);

            // Recursive helper: emit StructElem dicts for a record tree.
            // Returns the ObjectRef of the root element for this record.
            fn emit_struct_elems(
                record: &StructElemRecord,
                parent_ref: ObjectRef,
                page_ref: ObjectRef,
                next_id: &mut u32,
                obj_ids: &mut Vec<u32>,
                out: &mut Vec<(u32, Object)>,
            ) -> ObjectRef {
                let my_id = *next_id;
                *next_id += 1;
                let my_ref = ObjectRef::new(my_id, 0);
                obj_ids.push(my_id);

                // Recurse into children first so we know their refs for /K
                let child_refs: Vec<ObjectRef> = record
                    .children
                    .iter()
                    .map(|child| emit_struct_elems(child, my_ref, page_ref, next_id, obj_ids, out))
                    .collect();

                let mut dict: HashMap<String, Object> = HashMap::new();
                dict.insert("Type".to_string(), Object::Name("StructElem".to_string()));
                dict.insert("S".to_string(), Object::Name(record.structure_type.clone()));
                dict.insert("P".to_string(), Object::Reference(parent_ref));
                dict.insert("Pg".to_string(), Object::Reference(page_ref));
                // /K: either array of MCIDs + child refs, or just the MCID
                if child_refs.is_empty() {
                    // Leaf: /K is just the integer MCID
                    dict.insert("K".to_string(), Object::Integer(record.mcid as i64));
                } else {
                    // Has children: /K is an array of the MCID integer + child refs
                    let mut k_array: Vec<Object> = Vec::new();
                    k_array.push(Object::Integer(record.mcid as i64));
                    for cr in &child_refs {
                        k_array.push(Object::Reference(*cr));
                    }
                    dict.insert("K".to_string(), Object::Array(k_array));
                }
                if let Some(ref alt) = record.alt_text {
                    dict.insert("Alt".to_string(), ObjectSerializer::string(alt));
                }
                if let Some(ref lang) = record.language {
                    dict.insert("Lang".to_string(), ObjectSerializer::string(lang));
                }

                out.push((my_id, Object::Dictionary(dict)));
                my_ref
            }

            let mut all_struct_elem_objs: Vec<(u32, Object)> = Vec::new();
            // top-level refs → direct children of StructTreeRoot's /K
            let mut top_level_refs: Vec<Object> = Vec::new();
            // ParentTree entries: page_index → [StructElem refs on that page]
            let mut parent_tree_entries: Vec<Object> = Vec::new();

            let str_root_ref = ObjectRef::new(str_root_id, 0);
            for (page_idx, records) in struct_records_per_page.iter().enumerate() {
                let page_ref = ObjectRef::new(page_ids[page_idx].0, 0);

                let mut page_elem_refs: Vec<Object> = Vec::new();

                for record in records {
                    let elem_ref = emit_struct_elems(
                        record,
                        str_root_ref,
                        page_ref,
                        &mut self.next_obj_id,
                        &mut struct_tree_obj_ids,
                        &mut all_struct_elem_objs,
                    );
                    top_level_refs.push(Object::Reference(elem_ref));
                    page_elem_refs.push(Object::Reference(elem_ref));
                }

                // ParentTree entry for this page (even if empty, keep indexing stable)
                parent_tree_entries.push(Object::Integer(page_idx as i64));
                parent_tree_entries.push(Object::Array(page_elem_refs));
            }

            // ParentTree number-tree (flat /Nums array form)
            let parent_tree_dict: HashMap<String, Object> =
                HashMap::from([("Nums".to_string(), Object::Array(parent_tree_entries))]);
            self.objects
                .insert(parent_tree_id, Object::Dictionary(parent_tree_dict));

            // StructTreeRoot dict
            let mut str_dict: HashMap<String, Object> = HashMap::new();
            str_dict.insert("Type".to_string(), Object::Name("StructTreeRoot".to_string()));
            str_dict.insert("K".to_string(), Object::Array(top_level_refs));
            str_dict.insert(
                "ParentTree".to_string(),
                Object::Reference(ObjectRef::new(parent_tree_id, 0)),
            );
            // ISO 14289-1 §7.1 / PDF Ref §10.6.6: ParentTreeNextKey must equal
            // the next key that would be assigned (i.e. the page count).
            str_dict.insert("ParentTreeNextKey".to_string(), Object::Integer(page_count as i64));

            // F-4: /RoleMap
            if !self.config.role_map.is_empty() {
                let mut role_map_dict: HashMap<String, Object> = HashMap::new();
                for (custom, standard) in &self.config.role_map {
                    role_map_dict.insert(custom.clone(), Object::Name(standard.clone()));
                }
                str_dict.insert("RoleMap".to_string(), Object::Dictionary(role_map_dict));
            }

            self.objects
                .insert(str_root_id, Object::Dictionary(str_dict));

            // Store all StructElem objects
            for (id, obj) in all_struct_elem_objs {
                self.objects.insert(id, obj);
            }

            Some(str_root_id)
        } else {
            None
        };

        // Catalog object
        let mut catalog_entries = vec![
            ("Type", ObjectSerializer::name("Catalog")),
            ("Pages", ObjectSerializer::reference(pages_id, 0)),
        ];
        if let Some(acroform_id) = acroform_id {
            catalog_entries.push(("AcroForm", ObjectSerializer::reference(acroform_id, 0)));
        }
        if let Some(root_ref) = outline_ref {
            catalog_entries
                .push(("Outlines", ObjectSerializer::reference(root_ref.id, root_ref.gen)));
        }
        if let Some(labels_id) = page_labels_id {
            catalog_entries.push(("PageLabels", ObjectSerializer::reference(labels_id, 0)));
        }
        if let Some(ref script) = self.config.open_action_script {
            let action = Object::Dictionary(HashMap::from([
                ("Type".to_string(), ObjectSerializer::name("Action")),
                ("S".to_string(), ObjectSerializer::name("JavaScript")),
                ("JS".to_string(), ObjectSerializer::string(script)),
            ]));
            catalog_entries.push(("OpenAction", action));
        }
        // F-1/F-2: Tagged PDF catalog entries
        // Build XMP metadata stream for pdfuaid:part (PDF/UA-1 ISO 14289-1 §6.7.11).
        let xmp_metadata_id: Option<u32> = if self.config.tagged {
            // /MarkInfo << /Marked true >>
            let mark_info =
                Object::Dictionary(HashMap::from([("Marked".to_string(), Object::Boolean(true))]));
            catalog_entries.push(("MarkInfo", mark_info));

            // /StructTreeRoot ref
            if let Some(str_id) = struct_tree_root_id {
                catalog_entries.push(("StructTreeRoot", ObjectSerializer::reference(str_id, 0)));
            }

            // F-2: /Lang
            let lang = self
                .config
                .language
                .as_deref()
                .unwrap_or("en-US")
                .to_string();
            catalog_entries.push(("Lang", ObjectSerializer::string(&lang)));

            // /ViewerPreferences << /DisplayDocTitle true >>
            let viewer_prefs = Object::Dictionary(HashMap::from([(
                "DisplayDocTitle".to_string(),
                Object::Boolean(true),
            )]));
            catalog_entries.push(("ViewerPreferences", viewer_prefs));

            // ISO 14289-1 §6.7.11: PDF/UA documents must carry an XMP metadata
            // stream in the document catalog with pdfuaid:part set to 1 (UA-1).
            let title = self.config.title.as_deref().unwrap_or("").to_string();
            let creator = self
                .config
                .creator
                .as_deref()
                .unwrap_or("pdf_oxide")
                .to_string();
            let xmp = build_pdfua_xmp(&title, &creator, &lang);
            let xmp_id = self.alloc_obj_id();
            let mut xmp_dict: HashMap<String, Object> = HashMap::new();
            xmp_dict.insert("Type".to_string(), Object::Name("Metadata".to_string()));
            xmp_dict.insert("Subtype".to_string(), Object::Name("XML".to_string()));
            xmp_dict.insert("Length".to_string(), Object::Integer(xmp.len() as i64));
            self.objects.insert(
                xmp_id,
                Object::Stream {
                    dict: xmp_dict,
                    data: crate::import_limits::StreamData::from(xmp),
                },
            );
            catalog_entries.push(("Metadata", ObjectSerializer::reference(xmp_id, 0)));
            Some(xmp_id)
        } else {
            None
        };
        let catalog_obj = ObjectSerializer::dict(catalog_entries);

        // Info object (optional metadata)
        let info_id = self.alloc_obj_id();
        let mut info_entries = Vec::new();
        if let Some(title) = &self.config.title {
            info_entries.push(("Title", ObjectSerializer::string(title)));
        }
        if let Some(author) = &self.config.author {
            info_entries.push(("Author", ObjectSerializer::string(author)));
        }
        if let Some(subject) = &self.config.subject {
            info_entries.push(("Subject", ObjectSerializer::string(subject)));
        }
        if let Some(creator) = &self.config.creator {
            info_entries.push(("Creator", ObjectSerializer::string(creator)));
        }
        let info_obj = ObjectSerializer::dict(info_entries);

        // Write all objects
        // Catalog
        xref_offsets.push((catalog_id, output.len()));
        output.extend_from_slice(&serializer.serialize_indirect(catalog_id, 0, &catalog_obj));

        // Pages
        xref_offsets.push((pages_id, output.len()));
        output.extend_from_slice(&serializer.serialize_indirect(pages_id, 0, &pages_obj));

        // Font objects (Base-14)
        for font_ref in self.fonts.values() {
            if let Some(font_obj) = self.objects.get(&font_ref.id) {
                xref_offsets.push((font_ref.id, output.len()));
                output.extend_from_slice(&serializer.serialize_indirect(font_ref.id, 0, font_obj));
            }
        }

        // Embedded font objects (FONT-3): the five-object graph per font
        // (Type 0, CIDFontType2, FontDescriptor, FontFile2 stream,
        // ToUnicode stream).
        for &id in &embedded_object_ids {
            if let Some(obj) = self.objects.get(&id) {
                xref_offsets.push((id, output.len()));
                output.extend_from_slice(&serializer.serialize_indirect(id, 0, obj));
            }
        }

        // Page and content objects
        for (obj_id, obj, _) in &page_objects {
            xref_offsets.push((*obj_id, output.len()));
            output.extend_from_slice(&serializer.serialize_indirect(*obj_id, 0, obj));
        }

        // Image XObject streams (from HTML <img> / add_element Image).
        for (obj_id, obj, _) in &image_objects {
            xref_offsets.push((*obj_id, output.len()));
            output.extend_from_slice(&serializer.serialize_indirect(*obj_id, 0, obj));
        }

        // Hoist any inline appearance streams (e.g. a watermark's `/AP /N`)
        // into their own indirect objects — a stream may not be a direct
        // dictionary value. New ids come from `next_obj_id` and the extracted
        // streams are appended so they get xref entries below.
        let mut hoisted_ap_objects: Vec<(u32, Object)> = Vec::new();
        for (_, annot_obj) in annotation_objects.iter_mut() {
            if let Object::Dictionary(annot_dict) = annot_obj {
                hoisted_ap_objects
                    .extend(hoist_appearance_streams(annot_dict, &mut self.next_obj_id));
            }
        }
        annotation_objects.extend(hoisted_ap_objects);

        // Annotation objects
        for (annot_id, annot_obj) in &annotation_objects {
            xref_offsets.push((*annot_id, output.len()));
            output.extend_from_slice(&serializer.serialize_indirect(*annot_id, 0, annot_obj));
        }

        // Form field objects
        for (field_id, field_obj) in &form_field_objects {
            xref_offsets.push((*field_id, output.len()));
            output.extend_from_slice(&serializer.serialize_indirect(*field_id, 0, field_obj));
        }

        // AcroForm object (if present)
        if let Some(acroform_id) = acroform_id {
            if let Some(acroform_obj) = self.objects.get(&acroform_id) {
                xref_offsets.push((acroform_id, output.len()));
                output.extend_from_slice(&serializer.serialize_indirect(
                    acroform_id,
                    0,
                    acroform_obj,
                ));
            }
        }

        // Outline objects (root + every item). #393 Bundle B-1.
        for &id in &outline_object_ids {
            if let Some(obj) = self.objects.get(&id) {
                xref_offsets.push((id, output.len()));
                output.extend_from_slice(&serializer.serialize_indirect(id, 0, obj));
            }
        }

        // PageLabels number tree (if set). #393 Bundle B-2.
        if let Some(id) = page_labels_id {
            if let Some(obj) = self.objects.get(&id) {
                xref_offsets.push((id, output.len()));
                output.extend_from_slice(&serializer.serialize_indirect(id, 0, obj));
            }
        }

        // F-1: StructTreeRoot + ParentTree + StructElem objects (tagged PDF).
        // Emit in insertion order (struct_tree_obj_ids preserves this).
        for &id in &struct_tree_obj_ids {
            if let Some(obj) = self.objects.get(&id) {
                xref_offsets.push((id, output.len()));
                output.extend_from_slice(&serializer.serialize_indirect(id, 0, obj));
            }
        }

        // ISO 14289-1 §6.7.11: XMP metadata stream (tagged PDF only).
        if let Some(xmp_id) = xmp_metadata_id {
            if let Some(obj) = self.objects.get(&xmp_id) {
                xref_offsets.push((xmp_id, output.len()));
                output.extend_from_slice(&serializer.serialize_indirect(xmp_id, 0, obj));
            }
        }

        // Info object
        xref_offsets.push((info_id, output.len()));
        output.extend_from_slice(&serializer.serialize_indirect(info_id, 0, &info_obj));

        // Write xref table
        let xref_start = output.len();
        writeln!(output, "xref")?;
        writeln!(output, "0 {}", self.next_obj_id)?;

        // Object 0 is always free
        writeln!(output, "0000000000 65535 f ")?;

        // Sort xref entries by object ID
        xref_offsets.sort_by_key(|(id, _)| *id);

        for (_, offset) in &xref_offsets {
            writeln!(output, "{:010} 00000 n ", offset)?;
        }

        // Write trailer
        let trailer = ObjectSerializer::dict(vec![
            ("Size", ObjectSerializer::integer(self.next_obj_id as i64)),
            ("Root", ObjectSerializer::reference(catalog_id, 0)),
            ("Info", ObjectSerializer::reference(info_id, 0)),
        ]);

        writeln!(output, "trailer")?;
        output.extend_from_slice(&serializer.serialize(&trailer));
        writeln!(output)?;
        writeln!(output, "startxref")?;
        writeln!(output, "{}", xref_start)?;
        write!(output, "%%EOF")?;

        Ok(output)
    }

    /// Save the PDF to a file.
    pub fn save(self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let bytes = self.finish()?;
        std::fs::write(path, bytes)?;
        Ok(())
    }
}

impl Default for PdfWriter {
    fn default() -> Self {
        Self::new()
    }
}

/// Build a minimal XMP packet that satisfies ISO 14289-1 §6.7.11 (PDF/UA-1).
/// Returns raw UTF-8 bytes (no BOM, no padding).
fn build_pdfua_xmp(title: &str, creator: &str, lang: &str) -> Vec<u8> {
    // Escape the three free-text fields for XML attribute/element contexts.
    fn xml_escape(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
    }
    let title_e = xml_escape(title);
    let creator_e = xml_escape(creator);
    let lang_e = xml_escape(lang);

    let xmp = format!(
        r#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about=""
      xmlns:dc="http://purl.org/dc/elements/1.1/"
      xmlns:xmp="http://ns.adobe.com/xap/1.0/"
      xmlns:pdf="http://ns.adobe.com/pdf/1.3/"
      xmlns:pdfuaid="http://www.aiim.org/pdfua/ns/id/">
   <dc:title><rdf:Alt><rdf:li xml:lang="{lang}">{title}</rdf:li></rdf:Alt></dc:title>
   <dc:creator><rdf:Seq><rdf:li>{creator}</rdf:li></rdf:Seq></dc:creator>
   <xmp:CreatorTool>{creator}</xmp:CreatorTool>
   <pdf:Producer>{creator}</pdf:Producer>
   <pdfuaid:part>1</pdfuaid:part>
   <pdfuaid:amd>2005</pdfuaid:amd>
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#,
        lang = lang_e,
        title = title_e,
        creator = creator_e,
    );
    xmp.into_bytes()
}

/// Convert an [`elements::ImageContent`] (what `PageBuilder::add_element`
/// accepts) into the matching [`super::image_handler::ImageData`] so
/// the XObject stream dictionary + soft mask dict can be emitted. The
/// two structs carry the same payload but are owned by different
/// layers — this keeps the paint pipeline plugged into the standard
/// writer-side serializer without reaching across module boundaries.
pub(crate) fn image_content_to_xobject_stream(
    image: &crate::elements::ImageContent,
) -> (super::image_handler::ImageData, Option<Vec<u8>>) {
    use super::image_handler::{ColorSpace as WColorSpace, ImageData, ImageFormat as WImageFormat};

    // If the caller passed raw PNG or JPEG file bytes directly (e.g. via
    // `ImageContent::new()` without going through `image_from_bytes()`),
    // decode them now so the XObject gets the correct pixel dimensions,
    // colour space, filter params, and PNG per-row filter bytes.
    //
    // Detection is by magic number rather than `image.format` so we catch
    // both cases: (a) format tag matches bytes, (b) caller passed a PNG
    // buffer but labelled it as something else.
    let raw = &image.data;

    // PNG magic: \x89 P N G \r \n \x1a \n
    if raw.len() >= 8 && raw[..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
        if let Ok(decoded) = ImageData::from_png(raw) {
            let soft_mask = decoded.soft_mask.clone();
            return (decoded, soft_mask);
        }
    }

    // JPEG magic: \xFF \xD8
    if raw.len() >= 2 && raw[0] == 0xFF && raw[1] == 0xD8 {
        if let Ok(decoded) = ImageData::from_jpeg(raw.to_vec()) {
            // JPEG has no alpha channel.
            return (decoded, None);
        }
    }

    // Pre-processed path: data was already encoded by `ImageData::from_png()`
    // (Flate-compressed pixels with per-row filter bytes) or is raw pixels.
    let color_space = match image.color_space {
        crate::elements::ColorSpace::Gray => WColorSpace::DeviceGray,
        crate::elements::ColorSpace::CMYK => WColorSpace::DeviceCMYK,
        crate::elements::ColorSpace::RGB => WColorSpace::DeviceRGB,
        // The writer's ImageData doesn't currently model Indexed or
        // Lab. The html_css paint pipeline only produces Gray / RGB /
        // CMYK ImageContents so this branch is latent — but if a
        // caller constructs an ImageContent with Indexed or Lab
        // directly (and routes it through `PageBuilder::add_element`),
        // silently coercing to RGB would produce wrong colours.
        // Fall back to RGB to keep the XObject emittable, and emit a
        // warning so the miscoloration is diagnosable.
        crate::elements::ColorSpace::Indexed | crate::elements::ColorSpace::Lab => {
            log::warn!(
                "image_content_to_xobject_stream: ColorSpace::{:?} is not yet supported by \
                 the writer pipeline; falling back to DeviceRGB (colours may be wrong)",
                image.color_space
            );
            WColorSpace::DeviceRGB
        },
    };
    let format = match image.format {
        crate::elements::ImageFormat::Jpeg => WImageFormat::Jpeg,
        crate::elements::ImageFormat::Png => WImageFormat::Png,
        _ => WImageFormat::Raw,
    };
    // Carry the alpha channel forward if the caller attached one
    // (PNG RGBA / LA). `ImageData::from_png` compresses alpha upstream,
    // so `soft_mask` here is already the FlateDecode payload ready to
    // stream straight into the /SMask XObject.
    let soft_mask = image.soft_mask.clone();
    let data = ImageData {
        width: image.width,
        height: image.height,
        bits_per_component: image.bits_per_component,
        color_space,
        format,
        data: image.data.clone(),
        soft_mask: soft_mask.clone(),
    };
    (data, soft_mask)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_empty_pdf() {
        let writer = PdfWriter::new();
        let mut writer = writer;
        writer.add_letter_page().finish();
        let bytes = writer.finish().unwrap();

        let content = String::from_utf8_lossy(&bytes);
        assert!(content.starts_with("%PDF-1.7"));
        assert!(content.contains("/Type /Catalog"));
        assert!(content.contains("/Type /Pages"));
        assert!(content.contains("/Type /Page"));
        assert!(content.contains("%%EOF"));
    }

    #[test]
    fn test_pdf_with_text() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            page.add_text("Hello, World!", 72.0, 720.0, "Helvetica", 12.0);
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Type /Font"));
        assert!(content.contains("/BaseFont /Helvetica"));
        assert!(content.contains("BT"));
        assert!(content.contains("(Hello, World!) Tj"));
        assert!(content.contains("ET"));
    }

    #[test]
    fn test_pdf_with_metadata() {
        let config = PdfWriterConfig::default()
            .with_title("Test Document")
            .with_author("Test Author");

        let mut writer = PdfWriter::with_config(config);
        writer.add_letter_page().finish();

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Title (Test Document)"));
        assert!(content.contains("/Author (Test Author)"));
    }

    #[test]
    fn test_multiple_pages() {
        let mut writer = PdfWriter::new();
        writer.add_letter_page().finish();
        writer.add_a4_page().finish();

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Count 2"));
        // Two MediaBox entries for different page sizes
        assert!(content.contains("[0 0 612 792]")); // Letter
        assert!(content.contains("[0 0 595 842]")); // A4
    }

    #[test]
    fn test_page_builder() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            page.add_text("Line 1", 72.0, 720.0, "Helvetica", 12.0);
            page.add_text("Line 2", 72.0, 700.0, "Helvetica", 12.0);
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_pdf_with_link_annotation() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            page.add_text("Click here to visit Rust", 72.0, 720.0, "Helvetica", 12.0);
            page.link(Rect::new(72.0, 720.0, 150.0, 12.0), "https://www.rust-lang.org");
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        // Verify annotation structure
        assert!(content.contains("/Type /Annot"));
        assert!(content.contains("/Subtype /Link"));
        assert!(content.contains("/Annots"));
        assert!(content.contains("rust-lang.org"));
    }

    #[test]
    fn test_pdf_with_internal_link() {
        let mut writer = PdfWriter::new();

        // Page 1 with link to page 2
        {
            let mut page = writer.add_letter_page();
            page.add_text("Go to page 2", 72.0, 720.0, "Helvetica", 12.0);
            page.internal_link(Rect::new(72.0, 720.0, 100.0, 12.0), 1);
            page.finish();
        }

        // Page 2 (target)
        {
            let mut page = writer.add_letter_page();
            page.add_text("This is page 2", 72.0, 720.0, "Helvetica", 12.0);
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Type /Annot"));
        assert!(content.contains("/Subtype /Link"));
        assert!(content.contains("/Dest")); // Destination for internal link
        assert!(content.contains("/Fit")); // Fit mode
    }

    #[test]
    fn test_pdf_with_multiple_annotations() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            page.link(Rect::new(72.0, 720.0, 100.0, 12.0), "https://example1.com");
            page.link(Rect::new(72.0, 700.0, 100.0, 12.0), "https://example2.com");
            page.link(Rect::new(72.0, 680.0, 100.0, 12.0), "https://example3.com");
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        // Count occurrences of /Type /Annot
        let annot_count = content.matches("/Type /Annot").count();
        assert_eq!(annot_count, 3, "Expected 3 annotations");
    }

    #[test]
    fn test_pdf_with_highlight() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            page.add_text("Important text to highlight", 72.0, 720.0, "Helvetica", 12.0);
            page.highlight_rect(Rect::new(72.0, 720.0, 150.0, 12.0));
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Type /Annot"));
        assert!(content.contains("/Subtype /Highlight"));
        assert!(content.contains("/QuadPoints"));
        assert!(content.contains("/Annots"));
    }

    #[test]
    fn test_pdf_with_all_text_markup_types() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            // Add all four text markup types
            page.highlight_rect(Rect::new(72.0, 720.0, 100.0, 12.0));
            page.underline_rect(Rect::new(72.0, 700.0, 100.0, 12.0));
            page.strikeout_rect(Rect::new(72.0, 680.0, 100.0, 12.0));
            page.squiggly_rect(Rect::new(72.0, 660.0, 100.0, 12.0));
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Subtype /Highlight"));
        assert!(content.contains("/Subtype /Underline"));
        assert!(content.contains("/Subtype /StrikeOut"));
        assert!(content.contains("/Subtype /Squiggly"));

        // Should have 4 annotations
        let annot_count = content.matches("/Type /Annot").count();
        assert_eq!(annot_count, 4, "Expected 4 text markup annotations");
    }

    #[test]
    fn test_pdf_with_mixed_annotations() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            // Mix link and text markup annotations
            page.link(Rect::new(72.0, 720.0, 100.0, 12.0), "https://example.com");
            page.highlight_rect(Rect::new(72.0, 700.0, 100.0, 12.0));
            page.underline_rect(Rect::new(72.0, 680.0, 100.0, 12.0));
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        // Should have 3 annotations total
        let annot_count = content.matches("/Type /Annot").count();
        assert_eq!(annot_count, 3, "Expected 3 mixed annotations");

        assert!(content.contains("/Subtype /Link"));
        assert!(content.contains("/Subtype /Highlight"));
        assert!(content.contains("/Subtype /Underline"));
    }

    #[test]
    fn test_pdf_with_sticky_note() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            page.add_text("Document with a note", 72.0, 720.0, "Helvetica", 12.0);
            page.sticky_note(Rect::new(72.0, 700.0, 24.0, 24.0), "This is an important note!");
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Type /Annot"));
        assert!(content.contains("/Subtype /Text"));
        assert!(content.contains("/Name /Note"));
        assert!(content.contains("/Annots"));
        assert!(content.contains("important note"));
    }

    #[test]
    fn test_pdf_with_comment_annotation() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            page.comment(Rect::new(72.0, 720.0, 24.0, 24.0), "Review comment here");
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Type /Annot"));
        assert!(content.contains("/Subtype /Text"));
        assert!(content.contains("/Name /Comment"));
    }

    #[test]
    fn test_pdf_with_text_note_icons() {
        use crate::annotation_types::TextAnnotationIcon;

        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            // Add notes with different icons
            page.text_note_with_icon(
                Rect::new(72.0, 720.0, 24.0, 24.0),
                "Help note",
                TextAnnotationIcon::Help,
            );
            page.text_note_with_icon(
                Rect::new(100.0, 720.0, 24.0, 24.0),
                "Key note",
                TextAnnotationIcon::Key,
            );
            page.text_note_with_icon(
                Rect::new(128.0, 720.0, 24.0, 24.0),
                "Insert note",
                TextAnnotationIcon::Insert,
            );
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Name /Help"));
        assert!(content.contains("/Name /Key"));
        assert!(content.contains("/Name /Insert"));

        // Should have 3 text annotations
        let annot_count = content.matches("/Subtype /Text").count();
        assert_eq!(annot_count, 3, "Expected 3 text annotations with different icons");
    }

    #[test]
    fn test_pdf_with_all_annotation_types() {
        use crate::annotation_types::TextAnnotationIcon;

        let mut writer = PdfWriter::new();

        // Page 1 with link to page 2
        {
            let mut page = writer.add_letter_page();
            page.add_text("Comprehensive annotation test", 72.0, 750.0, "Helvetica", 14.0);

            // Link annotation
            page.link(Rect::new(72.0, 720.0, 100.0, 12.0), "https://example.com");

            // Text markup annotations
            page.highlight_rect(Rect::new(72.0, 700.0, 100.0, 12.0));
            page.underline_rect(Rect::new(72.0, 680.0, 100.0, 12.0));
            page.strikeout_rect(Rect::new(72.0, 660.0, 100.0, 12.0));
            page.squiggly_rect(Rect::new(72.0, 640.0, 100.0, 12.0));

            // Text annotations (sticky notes)
            page.sticky_note(Rect::new(200.0, 720.0, 24.0, 24.0), "A sticky note");
            page.comment(Rect::new(200.0, 680.0, 24.0, 24.0), "A comment");
            page.text_note_with_icon(
                Rect::new(200.0, 640.0, 24.0, 24.0),
                "Help text",
                TextAnnotationIcon::Help,
            );

            // Internal link
            page.internal_link(Rect::new(72.0, 600.0, 100.0, 12.0), 1);

            page.finish();
        }

        // Page 2 (target)
        {
            let mut page = writer.add_letter_page();
            page.add_text("Page 2", 72.0, 720.0, "Helvetica", 12.0);
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        // Verify all annotation types are present
        assert!(content.contains("/Subtype /Link"));
        assert!(content.contains("/Subtype /Highlight"));
        assert!(content.contains("/Subtype /Underline"));
        assert!(content.contains("/Subtype /StrikeOut"));
        assert!(content.contains("/Subtype /Squiggly"));
        assert!(content.contains("/Subtype /Text"));

        // Should have 9 annotations on page 1:
        // 2 links + 4 text markup + 3 sticky notes = 9
        let annot_count = content.matches("/Type /Annot").count();
        assert_eq!(annot_count, 9, "Expected 9 annotations total");
    }

    #[test]
    fn test_pdf_with_textbox() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            page.add_text("Document with text box", 72.0, 750.0, "Helvetica", 14.0);
            page.textbox(Rect::new(72.0, 650.0, 200.0, 80.0), "This is a text box annotation");
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Type /Annot"));
        assert!(content.contains("/Subtype /FreeText"));
        assert!(content.contains("/DA")); // Default Appearance
        assert!(content.contains("/Annots"));
    }

    #[test]
    fn test_pdf_with_styled_textbox() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            page.textbox_styled(
                Rect::new(72.0, 600.0, 250.0, 60.0),
                "Styled text content",
                "Courier",
                14.0,
            );
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Subtype /FreeText"));
        assert!(content.contains("/Cour")); // Courier font
        assert!(content.contains("14")); // Font size
    }

    #[test]
    fn test_pdf_with_centered_textbox() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            page.textbox_centered(Rect::new(100.0, 500.0, 200.0, 40.0), "Centered text");
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Subtype /FreeText"));
        assert!(content.contains("/Q 1")); // Center alignment
    }

    #[test]
    fn test_pdf_with_callout() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            // Callout with leader line from (50, 550) to (72, 600)
            page.callout(
                Rect::new(72.0, 600.0, 150.0, 50.0),
                "Callout annotation",
                vec![50.0, 550.0, 72.0, 600.0],
            );
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Subtype /FreeText"));
        assert!(content.contains("/IT /FreeTextCallout")); // Intent
        assert!(content.contains("/CL")); // Callout line
    }

    #[test]
    fn test_pdf_with_typewriter() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            page.typewriter(Rect::new(72.0, 500.0, 300.0, 20.0), "Typewriter text");
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Subtype /FreeText"));
        assert!(content.contains("/IT /FreeTextTypeWriter")); // Intent
    }

    #[test]
    fn test_pdf_with_multiple_freetext_types() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            page.textbox(Rect::new(72.0, 700.0, 150.0, 40.0), "Basic text box");
            page.textbox_centered(Rect::new(72.0, 640.0, 150.0, 40.0), "Centered box");
            page.typewriter(Rect::new(72.0, 580.0, 200.0, 20.0), "Typewriter");
            page.callout(
                Rect::new(300.0, 700.0, 150.0, 40.0),
                "Callout",
                vec![250.0, 680.0, 300.0, 720.0],
            );
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        // Should have 4 FreeText annotations
        let freetext_count = content.matches("/Subtype /FreeText").count();
        assert_eq!(freetext_count, 4, "Expected 4 FreeText annotations");
    }

    #[test]
    fn test_pdf_with_line_annotation() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            page.line((100.0, 100.0), (300.0, 100.0));
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Subtype /Line"));
        assert!(content.contains("/L ")); // Line coordinates
    }

    #[test]
    fn test_pdf_with_arrow_annotation() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            page.arrow((100.0, 200.0), (300.0, 200.0));
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Subtype /Line"));
        assert!(content.contains("/LE")); // Line endings
        assert!(content.contains("/OpenArrow"));
    }

    #[test]
    fn test_pdf_with_rectangle_annotation() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            page.rectangle(Rect::new(100.0, 400.0, 150.0, 100.0));
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Subtype /Square"));
    }

    #[test]
    fn test_pdf_with_circle_annotation() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            page.circle(Rect::new(300.0, 400.0, 100.0, 100.0));
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Subtype /Circle"));
    }

    #[test]
    fn test_pdf_with_filled_shapes() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            page.rectangle_filled(
                Rect::new(100.0, 300.0, 100.0, 80.0),
                (0.0, 0.0, 1.0), // Blue stroke
                (0.8, 0.8, 1.0), // Light blue fill
            );
            page.circle_filled(
                Rect::new(250.0, 300.0, 80.0, 80.0),
                (1.0, 0.0, 0.0), // Red stroke
                (1.0, 0.8, 0.8), // Light red fill
            );
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Subtype /Square"));
        assert!(content.contains("/Subtype /Circle"));
        assert!(content.contains("/IC")); // Interior color
    }

    #[test]
    fn test_pdf_with_polygon() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            // Triangle
            page.polygon(vec![(100.0, 100.0), (150.0, 200.0), (50.0, 200.0)]);
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Subtype /Polygon"));
        assert!(content.contains("/Vertices"));
    }

    #[test]
    fn test_pdf_with_polyline() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            page.polyline(vec![
                (100.0, 500.0),
                (200.0, 550.0),
                (300.0, 500.0),
                (400.0, 550.0),
            ]);
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Subtype /PolyLine"));
        assert!(content.contains("/Vertices"));
    }

    #[test]
    fn test_pdf_with_all_shape_types() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            // Line
            page.line((72.0, 750.0), (200.0, 750.0));
            // Arrow
            page.arrow((72.0, 700.0), (200.0, 700.0));
            // Rectangle
            page.rectangle(Rect::new(72.0, 600.0, 100.0, 50.0));
            // Circle
            page.circle(Rect::new(200.0, 600.0, 50.0, 50.0));
            // Polygon
            page.polygon(vec![(300.0, 600.0), (350.0, 650.0), (250.0, 650.0)]);
            // Polyline
            page.polyline(vec![(72.0, 500.0), (150.0, 550.0), (250.0, 500.0)]);
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        // Verify all shape types
        assert!(content.contains("/Subtype /Line"));
        assert!(content.contains("/Subtype /Square"));
        assert!(content.contains("/Subtype /Circle"));
        assert!(content.contains("/Subtype /Polygon"));
        assert!(content.contains("/Subtype /PolyLine"));

        // Should have 6 shape annotations
        let annot_count = content.matches("/Type /Annot").count();
        assert_eq!(annot_count, 6, "Expected 6 shape annotations");
    }

    #[test]
    fn test_pdf_with_ink_annotation() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            page.ink(vec![(100.0, 100.0), (150.0, 120.0), (200.0, 100.0)]);
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Type /Annot"));
        assert!(content.contains("/Subtype /Ink"));
        assert!(content.contains("/InkList"));
    }

    #[test]
    fn test_pdf_with_freehand_multiple_strokes() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            page.freehand(vec![
                vec![(100.0, 100.0), (150.0, 120.0), (200.0, 100.0)],
                vec![(100.0, 200.0), (200.0, 200.0)],
                vec![(150.0, 150.0), (150.0, 250.0)],
            ]);
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Subtype /Ink"));
        assert!(content.contains("/InkList"));
        // Should have 1 ink annotation
        let ink_count = content.matches("/Subtype /Ink").count();
        assert_eq!(ink_count, 1, "Expected 1 Ink annotation");
    }

    #[test]
    fn test_pdf_with_styled_ink() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            page.ink_styled(
                vec![(100.0, 300.0), (200.0, 350.0), (300.0, 300.0)],
                (1.0, 0.0, 0.0), // Red
                3.0,             // 3pt line width
            );
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Subtype /Ink"));
        assert!(content.contains("/C")); // Color
        assert!(content.contains("/BS")); // Border style
    }

    #[test]
    fn test_pdf_with_multiple_ink_annotations() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            // Add multiple separate ink annotations
            page.ink(vec![(100.0, 100.0), (150.0, 120.0)]);
            page.ink(vec![(200.0, 100.0), (250.0, 120.0)]);
            page.ink_styled(
                vec![(300.0, 100.0), (350.0, 120.0)],
                (0.0, 0.0, 1.0), // Blue
                2.0,
            );
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        // Should have 3 ink annotations
        let ink_count = content.matches("/Subtype /Ink").count();
        assert_eq!(ink_count, 3, "Expected 3 Ink annotations");
    }

    #[test]
    fn test_pdf_with_ink_and_other_annotations() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            // Mix ink with other annotations
            page.ink(vec![(100.0, 100.0), (200.0, 150.0)]);
            page.highlight_rect(Rect::new(72.0, 700.0, 100.0, 12.0));
            page.sticky_note(Rect::new(300.0, 700.0, 24.0, 24.0), "Note");
            page.line((72.0, 600.0), (200.0, 600.0));
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Subtype /Ink"));
        assert!(content.contains("/Subtype /Highlight"));
        assert!(content.contains("/Subtype /Text"));
        assert!(content.contains("/Subtype /Line"));

        // Should have 4 annotations
        let annot_count = content.matches("/Type /Annot").count();
        assert_eq!(annot_count, 4, "Expected 4 mixed annotations");
    }

    #[test]
    fn test_pdf_with_approved_stamp() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            page.stamp_approved(Rect::new(400.0, 700.0, 150.0, 50.0));
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Type /Annot"));
        assert!(content.contains("/Subtype /Stamp"));
        assert!(content.contains("/Name /Approved"));
    }

    #[test]
    fn test_pdf_with_draft_stamp() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            page.stamp_draft(Rect::new(400.0, 650.0, 120.0, 40.0));
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Subtype /Stamp"));
        assert!(content.contains("/Name /Draft"));
    }

    #[test]
    fn test_pdf_with_confidential_stamp() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            page.stamp_confidential(Rect::new(400.0, 600.0, 150.0, 50.0));
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Subtype /Stamp"));
        assert!(content.contains("/Name /Confidential"));
    }

    #[test]
    fn test_pdf_with_custom_stamp() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            page.stamp_custom(Rect::new(400.0, 550.0, 150.0, 50.0), "ReviewPending");
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Subtype /Stamp"));
        assert!(content.contains("/Name /ReviewPending"));
    }

    #[test]
    fn test_pdf_with_multiple_stamps() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            page.stamp_approved(Rect::new(400.0, 700.0, 100.0, 40.0));
            page.stamp_draft(Rect::new(400.0, 650.0, 100.0, 40.0));
            page.stamp_final(Rect::new(400.0, 600.0, 100.0, 40.0));
            page.stamp_for_comment(Rect::new(400.0, 550.0, 100.0, 40.0));
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        // Should have 4 stamp annotations
        let stamp_count = content.matches("/Subtype /Stamp").count();
        assert_eq!(stamp_count, 4, "Expected 4 Stamp annotations");

        assert!(content.contains("/Name /Approved"));
        assert!(content.contains("/Name /Draft"));
        assert!(content.contains("/Name /Final"));
        assert!(content.contains("/Name /ForComment"));
    }

    #[test]
    fn test_pdf_with_stamp_and_other_annotations() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            page.stamp_approved(Rect::new(400.0, 700.0, 150.0, 50.0));
            page.highlight_rect(Rect::new(72.0, 700.0, 100.0, 12.0));
            page.sticky_note(Rect::new(200.0, 700.0, 24.0, 24.0), "Note");
            page.line((72.0, 600.0), (200.0, 600.0));
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Subtype /Stamp"));
        assert!(content.contains("/Subtype /Highlight"));
        assert!(content.contains("/Subtype /Text"));
        assert!(content.contains("/Subtype /Line"));

        // Should have 4 annotations
        let annot_count = content.matches("/Type /Annot").count();
        assert_eq!(annot_count, 4, "Expected 4 mixed annotations");
    }

    // ============ Special Annotations Tests ============

    #[test]
    fn test_pdf_with_popup_annotation() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            page.popup(Rect::new(200.0, 600.0, 200.0, 100.0), true);
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Type /Annot"));
        assert!(content.contains("/Subtype /Popup"));
        assert!(content.contains("/Rect"));
        assert!(content.contains("/Open true"));
    }

    #[test]
    fn test_pdf_with_caret_annotation() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            page.caret(Rect::new(100.0, 700.0, 20.0, 20.0));
            page.caret_paragraph(Rect::new(100.0, 650.0, 20.0, 20.0));
            page.caret_with_comment(
                Rect::new(100.0, 600.0, 20.0, 20.0),
                "Insert new paragraph here",
            );
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        let caret_count = content.matches("/Subtype /Caret").count();
        assert_eq!(caret_count, 3, "Expected 3 Caret annotations");

        assert!(content.contains("/Sy /None"));
        assert!(content.contains("/Sy /P"));
        assert!(content.contains("Insert new paragraph here"));
    }

    #[test]
    fn test_pdf_with_file_attachment_annotation() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            page.file_attachment(Rect::new(50.0, 700.0, 24.0, 24.0), "document.pdf");
            page.file_attachment_paperclip(Rect::new(50.0, 650.0, 24.0, 24.0), "notes.txt");
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        let attach_count = content.matches("/Subtype /FileAttachment").count();
        assert_eq!(attach_count, 2, "Expected 2 FileAttachment annotations");

        assert!(content.contains("/Name /PushPin"));
        assert!(content.contains("/Name /Paperclip"));
        assert!(content.contains("document.pdf"));
        assert!(content.contains("notes.txt"));
    }

    #[test]
    fn test_pdf_with_redact_annotation() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            page.redact(Rect::new(100.0, 700.0, 200.0, 20.0));
            page.redact_with_text(Rect::new(100.0, 650.0, 200.0, 20.0), "REDACTED");
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        let redact_count = content.matches("/Subtype /Redact").count();
        assert_eq!(redact_count, 2, "Expected 2 Redact annotations");

        assert!(content.contains("REDACTED"));
    }

    #[test]
    fn test_pdf_with_mixed_special_annotations() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            page.popup(Rect::new(200.0, 700.0, 150.0, 80.0), false);
            page.caret(Rect::new(100.0, 650.0, 20.0, 20.0));
            page.file_attachment(Rect::new(50.0, 600.0, 24.0, 24.0), "report.pdf");
            page.redact(Rect::new(100.0, 550.0, 200.0, 20.0));
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Subtype /Popup"));
        assert!(content.contains("/Subtype /Caret"));
        assert!(content.contains("/Subtype /FileAttachment"));
        assert!(content.contains("/Subtype /Redact"));

        let annot_count = content.matches("/Type /Annot").count();
        assert_eq!(annot_count, 4, "Expected 4 special annotations");
    }

    #[test]
    fn test_pdf_with_complete_annotation_coverage() {
        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_letter_page();
            // Link
            page.link(Rect::new(72.0, 750.0, 100.0, 20.0), "https://example.com");
            // Text markup
            page.highlight_rect(Rect::new(72.0, 720.0, 100.0, 12.0));
            page.underline_rect(Rect::new(72.0, 700.0, 100.0, 12.0));
            // Sticky note
            page.sticky_note(Rect::new(200.0, 720.0, 24.0, 24.0), "Note");
            // FreeText
            page.textbox(Rect::new(72.0, 660.0, 150.0, 30.0), "Comment here");
            // Shapes
            page.line((72.0, 620.0), (200.0, 620.0));
            page.rectangle(Rect::new(72.0, 570.0, 50.0, 50.0));
            page.circle(Rect::new(140.0, 570.0, 50.0, 50.0));
            // Ink
            page.ink(vec![(72.0, 520.0), (100.0, 540.0), (130.0, 520.0)]);
            // Stamp
            page.stamp_approved(Rect::new(400.0, 700.0, 100.0, 40.0));
            // Special
            page.popup(Rect::new(400.0, 600.0, 150.0, 80.0), false);
            page.caret(Rect::new(400.0, 550.0, 20.0, 20.0));
            page.file_attachment(Rect::new(400.0, 500.0, 24.0, 24.0), "data.xlsx");
            page.redact(Rect::new(400.0, 450.0, 150.0, 20.0));
            page.finish();
        }

        let bytes = writer.finish().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        // Verify all annotation types are present
        assert!(content.contains("/Subtype /Link"));
        assert!(content.contains("/Subtype /Highlight"));
        assert!(content.contains("/Subtype /Underline"));
        assert!(content.contains("/Subtype /Text"));
        assert!(content.contains("/Subtype /FreeText"));
        assert!(content.contains("/Subtype /Line"));
        assert!(content.contains("/Subtype /Square"));
        assert!(content.contains("/Subtype /Circle"));
        assert!(content.contains("/Subtype /Ink"));
        assert!(content.contains("/Subtype /Stamp"));
        assert!(content.contains("/Subtype /Popup"));
        assert!(content.contains("/Subtype /Caret"));
        assert!(content.contains("/Subtype /FileAttachment"));
        assert!(content.contains("/Subtype /Redact"));

        let annot_count = content.matches("/Type /Annot").count();
        assert_eq!(annot_count, 14, "Expected 14 different annotation types");
    }

    // ── issue #425: image rendering regression tests ───────────────────────

    fn make_png_bytes(width: u32, height: u32, pixels_rgb: &[u8]) -> Vec<u8> {
        use image::RgbImage;
        let img = RgbImage::from_raw(width, height, pixels_rgb.to_vec())
            .expect("pixel buffer size mismatch");
        let mut buf = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .expect("encode PNG");
        buf
    }

    fn make_jpeg_bytes(width: u32, height: u32, pixels_rgb: &[u8]) -> Vec<u8> {
        use image::RgbImage;
        let img = RgbImage::from_raw(width, height, pixels_rgb.to_vec())
            .expect("pixel buffer size mismatch");
        let mut buf = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Jpeg)
            .expect("encode JPEG");
        buf
    }

    /// `ImageContent::new()` with raw PNG file bytes must produce an XObject
    /// whose Width/Height come from the PNG header, not from the caller-
    /// supplied width/height parameters (issue #425 bug 2).
    #[test]
    fn test_image_content_raw_png_bytes_decoded_correctly() {
        use crate::elements::{ImageContent, ImageFormat};
        use crate::geometry::Rect;

        let png = make_png_bytes(4, 3, &[128u8; 4 * 3 * 3]); // 4×3 solid grey
        let content = ImageContent::new(
            Rect::new(0.0, 0.0, 100.0, 100.0),
            ImageFormat::Png,
            png,
            99, // wrong width — should be overridden by PNG header
            99, // wrong height — should be overridden by PNG header
        );

        let (image_data, _soft_mask) = image_content_to_xobject_stream(&content);
        assert_eq!(image_data.width, 4, "width must come from PNG header (4), not user arg (99)");
        assert_eq!(image_data.height, 3, "height must come from PNG header (3), not user arg (99)");

        // XObject dict must have FlateDecode + Predictor=15 (not raw PNG bytes).
        use crate::object::Object;
        let dict = image_data.build_xobject_dict();
        assert_eq!(dict["Filter"], Object::Name("FlateDecode".to_string()));
        let parms = match dict.get("DecodeParms") {
            Some(Object::Dictionary(d)) => d,
            _ => panic!("DecodeParms missing"),
        };
        assert_eq!(parms["Predictor"], Object::Integer(15));
    }

    /// `ImageContent::new()` with raw JPEG file bytes must produce an XObject
    /// whose Width/Height come from the JPEG header, not from the caller-
    /// supplied width/height (issue #425 bug 3 — "zoomed in" JPEG).
    #[test]
    fn test_image_content_raw_jpeg_bytes_uses_real_dimensions() {
        use crate::elements::{ImageContent, ImageFormat};
        use crate::geometry::Rect;

        // Encode a 6×5 JPEG.
        let jpeg = make_jpeg_bytes(6, 5, &[200u8; 6 * 5 * 3]);
        let content = ImageContent::new(
            Rect::new(0.0, 0.0, 200.0, 380.0),
            ImageFormat::Jpeg,
            jpeg,
            200, // wrong — should be 6
            380, // wrong — should be 5
        );

        let (image_data, _soft_mask) = image_content_to_xobject_stream(&content);
        assert_eq!(image_data.width, 6, "width must come from JPEG header (6), not user arg (200)");
        assert_eq!(
            image_data.height, 5,
            "height must come from JPEG header (5), not user arg (380)"
        );

        // XObject dict must use DCTDecode.
        use crate::object::Object;
        let dict = image_data.build_xobject_dict();
        assert_eq!(dict["Filter"], Object::Name("DCTDecode".to_string()));
    }

    /// End-to-end: `image_from_bytes()` with a real PNG must produce a PDF
    /// that contains a properly-formed FlateDecode+Predictor=15 image stream
    /// (issue #425 bug 1 — colour loss).
    #[test]
    fn test_image_from_bytes_png_produces_valid_flate_stream() {
        use crate::geometry::Rect;
        use crate::writer::document_builder::DocumentBuilder;

        let png = make_png_bytes(2, 2, &[255u8, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 0]);

        let mut builder = DocumentBuilder::new();
        let _ = builder
            .letter_page()
            .image_from_bytes(&png, Rect::new(0.0, 0.0, 100.0, 100.0))
            .expect("image_from_bytes failed")
            .done();

        let pdf_bytes = builder.build().expect("build failed");

        // The PDF must contain FlateDecode and Predictor 15 (for PNG images).
        let pdf_text = String::from_utf8_lossy(&pdf_bytes);
        assert!(pdf_text.contains("/FlateDecode"), "must use FlateDecode");
        assert!(pdf_text.contains("/Predictor 15"), "must declare Predictor 15");
        // Must NOT use DCTDecode for a PNG source.
        assert!(!pdf_text.contains("/DCTDecode"), "must not use DCTDecode for PNG");
    }

    /// Reproduces the exact code from issue #425 using the reporter's actual
    /// attached images (downloaded to /tmp by the developer).
    ///
    /// Run with:
    ///   cargo test -p pdf_oxide --lib -- write_issue_425_reporter_images --nocapture --ignored
    #[test]
    #[ignore]
    fn write_issue_425_reporter_images() {
        use crate::elements::{ContentElement, ImageContent, ImageFormat};
        use crate::geometry::Rect;
        use crate::writer::document_builder::DocumentBuilder;

        let cats_png = std::fs::read("/tmp/issue_425_img1.png")
            .expect("PNG not found — run the curl download step first");
        let cats_jpg = std::fs::read("/tmp/issue_425_img2.jpg")
            .expect("JPEG not found — run the curl download step first");

        std::fs::create_dir_all("output").unwrap();

        let mut document_builder = DocumentBuilder::new();
        let page_builder = document_builder.a4_page();

        // Exact layout from the reporter's code:
        // Top half — image_from_bytes() for both PNG and JPEG
        let page_builder = page_builder
            .image_from_bytes(
                &cats_png,
                Rect {
                    x: 50.0,
                    y: 450.0,
                    width: 280.0,
                    height: 380.0,
                },
            )
            .unwrap()
            .image_from_bytes(
                &cats_jpg,
                Rect {
                    x: 380.0,
                    y: 450.0,
                    width: 200.0,
                    height: 380.0,
                },
            )
            .unwrap();

        // Bottom half — ImageContent::new() for both PNG and JPEG
        let page_builder = page_builder
            .element(ContentElement::Image(ImageContent::new(
                Rect {
                    x: 50.0,
                    y: 50.0,
                    width: 280.0,
                    height: 380.0,
                },
                ImageFormat::Png,
                cats_png,
                200,
                380,
            )))
            .element(ContentElement::Image(ImageContent::new(
                Rect {
                    x: 380.0,
                    y: 50.0,
                    width: 200.0,
                    height: 380.0,
                },
                ImageFormat::Jpeg,
                cats_jpg,
                200,
                380,
            )));

        let _ = page_builder.done();

        let pdf_bytes = document_builder.build().unwrap();
        let path = "output/issue_425_reporter_images.pdf";
        std::fs::write(path, &pdf_bytes).unwrap();
        println!("\n✓ Written: {}", std::fs::canonicalize(path).unwrap().display());
        println!("  Top-left:  PNG via image_from_bytes()   — should show cats PNG correctly");
        println!("  Top-right: JPEG via image_from_bytes()  — should show cats JPEG correctly");
        println!(
            "  Bot-left:  PNG via ImageContent::new()  — should show same cats PNG (was blank)"
        );
        println!(
            "  Bot-right: JPEG via ImageContent::new() — should show same cats JPEG (was zoomed)"
        );
    }

    /// Visual verification PDF for issue #425.
    ///
    /// Run with:
    ///   cargo test -p pdf_oxide --lib -- write_issue_425_visual_verification --nocapture --ignored
    ///
    /// Opens `output/issue_425_images.pdf` — check all 4 images display correctly.
    #[test]
    #[ignore]
    fn write_issue_425_visual_verification() {
        use crate::elements::{ImageContent, ImageFormat as EImageFormat};
        use crate::geometry::Rect;
        use crate::writer::document_builder::DocumentBuilder;

        std::fs::create_dir_all("output").unwrap();

        // Build vivid test images so it's immediately obvious if colours are wrong.
        // 100×100 solid red PNG
        let red_png = make_png_bytes(100, 100, &[255u8, 0, 0].repeat(100 * 100));
        // 100×100 solid blue PNG
        let blue_png = make_png_bytes(100, 100, &[0u8, 0, 255].repeat(100 * 100));
        // 100×100 solid green JPEG
        let green_jpg = make_jpeg_bytes(100, 100, &[0u8, 180, 0].repeat(100 * 100));
        // 100×100 purple JPEG (for ImageContent path)
        let purple_jpg = make_jpeg_bytes(100, 100, &[160u8, 0, 160].repeat(100 * 100));

        let mut builder = DocumentBuilder::new();
        let page = builder.letter_page();

        // Row 1 labels & images via image_from_bytes() ----------------------
        //   Col 1: red PNG  (bug 1 — colour loss before fix)
        //   Col 2: green JPEG
        let page = page
            .font("Helvetica", 10.0)
            .at(50.0, 700.0)
            .text("image_from_bytes() — PNG (should be solid RED):")
            .at(50.0, 580.0)
            .text("image_from_bytes() — JPEG (should be solid GREEN):")
            .image_from_bytes(&red_png, Rect::new(50.0, 450.0, 200.0, 200.0))
            .unwrap()
            .image_from_bytes(&green_jpg, Rect::new(300.0, 450.0, 200.0, 200.0))
            .unwrap();

        // Row 2 labels & images via ImageContent::new() ----------------------
        //   Col 1: blue PNG  (bug 2 — blank before fix)
        //   Col 2: purple JPEG (bug 3 — wrong dimensions / zoom before fix)
        let page = page
            .at(50.0, 420.0)
            .text("ImageContent::new() — PNG (should be solid BLUE):")
            .at(50.0, 300.0)
            .text("ImageContent::new() — JPEG (should be solid PURPLE):");

        let blue_content = ImageContent::new(
            Rect::new(50.0, 170.0, 200.0, 200.0),
            EImageFormat::Png,
            blue_png,
            100,
            100,
        );
        let purple_content = ImageContent::new(
            Rect::new(300.0, 170.0, 200.0, 200.0),
            EImageFormat::Jpeg,
            purple_jpg,
            999, // deliberately wrong pixel dims — fix must use real JPEG dims
            999,
        );

        use crate::elements::ContentElement;
        let _ = page
            .element(ContentElement::Image(blue_content))
            .element(ContentElement::Image(purple_content))
            .done();

        let pdf_bytes = builder.build().unwrap();
        let path = "output/issue_425_images.pdf";
        std::fs::write(path, &pdf_bytes).unwrap();
        println!("\n✓ Written: {}", std::fs::canonicalize(path).unwrap().display());
        println!("  Open in any PDF viewer and check:");
        println!("  Top-left  → solid RED   (image_from_bytes PNG)");
        println!("  Top-right → solid GREEN (image_from_bytes JPEG)");
        println!("  Bot-left  → solid BLUE  (ImageContent::new PNG)");
        println!("  Bot-right → solid PURPLE (ImageContent::new JPEG)");
    }
}
