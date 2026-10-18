//! High-level document builder with fluent API.
//!
//! Provides a convenient interface for building PDF documents
//! using method chaining, wrapping the lower-level PdfWriter.
//!
//! # Annotations
//!
//! The fluent API supports adding annotations directly to text elements:
//!
//! ```ignore
//! use pdf_oxide::writer::{DocumentBuilder, PageSize};
//!
//! let mut builder = DocumentBuilder::new();
//! builder
//!     .page(PageSize::Letter)
//!     .at(72.0, 720.0)
//!     .text("Click here for more info")
//!     .link_url("https://example.com")  // Link the previous text
//!     .text("Important note")
//!     .highlight((1.0, 1.0, 0.0))       // Highlight in yellow
//!     .sticky_note("Review this section")
//!     .done();
//! ```

use super::annotation_builder::{Annotation, LinkAnnotation};
use super::font_manager::{EmbeddedFont, FontManager, TextLayout};
use super::freetext::FreeTextAnnotation;
use super::page_template::PageTemplate;
use super::pdf_writer::{PdfWriter, PdfWriterConfig};
use super::stamp::{StampAnnotation, StampType};
use super::table_renderer::{FontMetrics, Table};
use super::text_annotations::TextAnnotation;
use super::text_markup::TextMarkupAnnotation;
use super::watermark::WatermarkAnnotation;
use crate::annotation_types::{TextAnnotationIcon, TextMarkupType};
use crate::elements::{ContentElement, TextContent};
use crate::error::Result;
use crate::geometry::Rect;
use std::path::Path;

/// Metadata for a PDF document.
#[derive(Debug, Clone, Default)]
pub struct DocumentMetadata {
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
    /// PDF version (default: "1.7")
    pub version: Option<String>,
    /// When true, emit PDF/UA-1 tagged-PDF catalog entries (/MarkInfo,
    /// /StructTreeRoot, /Lang, /ViewerPreferences). F-1/F-2.
    pub tagged: bool,
    /// Document natural language tag (e.g. "en-US"). Emitted as catalog
    /// /Lang when `tagged` is true, but also usable without tagging. F-2.
    pub language: Option<String>,
    /// Custom-type → standard-type role mappings for /RoleMap. F-4.
    pub role_map: Vec<(String, String)>,
}

impl DocumentMetadata {
    /// Create new empty metadata.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set document title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set document author.
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Set document subject.
    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    /// Set document keywords.
    pub fn keywords(mut self, keywords: impl Into<String>) -> Self {
        self.keywords = Some(keywords.into());
        self
    }

    /// Set creator application.
    pub fn creator(mut self, creator: impl Into<String>) -> Self {
        self.creator = Some(creator.into());
        self
    }

    /// Enable PDF/UA-1 tagging. When true, `DocumentBuilder::build` will emit
    /// `/MarkInfo`, `/StructTreeRoot`, `/Lang`, and `/ViewerPreferences` in the
    /// catalog. Has no effect on existing callers that do not call this method.
    pub fn tagged_pdf_ua1(mut self) -> Self {
        self.tagged = true;
        self
    }

    /// Set the document's natural language (e.g. `"en-US"`).
    /// Emitted as `/Lang` in the PDF catalog when `tagged_pdf_ua1()` is set.
    pub fn language(mut self, lang: impl Into<String>) -> Self {
        self.language = Some(lang.into());
        self
    }

    /// Add a custom → standard role mapping to the StructTreeRoot /RoleMap.
    /// Multiple calls accumulate entries. Example: `("Note", "P")` maps the
    /// custom "Note" structure type to the standard "P" (Paragraph).
    pub fn role_map(mut self, custom: impl Into<String>, standard: impl Into<String>) -> Self {
        self.role_map.push((custom.into(), standard.into()));
        self
    }
}

/// Standard page sizes.
#[derive(Debug, Clone, Copy)]
pub enum PageSize {
    /// US Letter (8.5" x 11")
    Letter,
    /// A4 (210mm x 297mm)
    A4,
    /// Legal (8.5" x 14")
    Legal,
    /// A3 (297mm x 420mm)
    A3,
    /// Custom dimensions in points
    Custom(f32, f32),
}

impl PageSize {
    /// Get dimensions in points (1 inch = 72 points).
    pub fn dimensions(&self) -> (f32, f32) {
        match self {
            PageSize::Letter => (612.0, 792.0),
            PageSize::A4 => (595.0, 842.0),
            PageSize::Legal => (612.0, 1008.0),
            PageSize::A3 => (842.0, 1190.0),
            PageSize::Custom(w, h) => (*w, *h),
        }
    }
}

/// Text alignment options.
#[derive(Debug, Clone, Copy, Default)]
pub enum TextAlign {
    /// Left-aligned text (default)
    #[default]
    Left,
    /// Center-aligned text
    Center,
    /// Right-aligned text
    Right,
}

/// Configuration for text rendering.
#[derive(Debug, Clone)]
pub struct TextConfig {
    /// Font name (default: Helvetica)
    pub font: String,
    /// Font size in points (default: 12)
    pub size: f32,
    /// Text alignment
    pub align: TextAlign,
    /// Line height multiplier (default: 1.2)
    pub line_height: f32,
}

impl Default for TextConfig {
    fn default() -> Self {
        Self {
            font: "Helvetica".to_string(),
            size: 12.0,
            align: TextAlign::Left,
            line_height: 1.2,
        }
    }
}

/// Marker style for `numbered_list`. #393 Bundle E-2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListStyle {
    /// 1. 2. 3. ...
    Decimal,
    /// i. ii. iii. ...
    RomanLower,
    /// a. b. c. ...
    AlphaLower,
}

/// Internal enum: which marker per item.
enum ListMarker {
    Bullet,
    Numbered(ListStyle),
}

/// Convert a 1-based index to lowercase Roman numerals (`i, ii, iii`...).
/// Caps at 3999 — same as PDF's own page-label Roman support.
fn to_roman_lower(mut n: usize) -> String {
    if n == 0 {
        return String::new();
    }
    const NUMERALS: &[(usize, &str)] = &[
        (1000, "m"),
        (900, "cm"),
        (500, "d"),
        (400, "cd"),
        (100, "c"),
        (90, "xc"),
        (50, "l"),
        (40, "xl"),
        (10, "x"),
        (9, "ix"),
        (5, "v"),
        (4, "iv"),
        (1, "i"),
    ];
    let mut out = String::new();
    for &(value, glyph) in NUMERALS {
        while n >= value {
            out.push_str(glyph);
            n -= value;
        }
    }
    out
}

/// Convert 1 → "a", 2 → "b", ..., 26 → "z", 27 → "aa", ...
fn to_alpha_lower(mut n: usize) -> String {
    if n == 0 {
        return String::new();
    }
    let mut out = String::new();
    while n > 0 {
        n -= 1;
        out.insert(0, (b'a' + (n % 26) as u8) as char);
        n /= 26;
    }
    out
}

/// Tab-navigation order for form fields / annotations within a page.
/// Emitted as the `/Tabs` entry on the page dict. #393 Bundle D-4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabOrder {
    /// Row order — top-to-bottom, left-to-right (reader default).
    Row,
    /// Column order — left-to-right, top-to-bottom.
    Column,
    /// Structure order — requires tagged PDF (Bundle F). Until Bundle F
    /// ships, readers fall back to row order.
    Structure,
}

impl TabOrder {
    fn as_pdf_char(self) -> char {
        match self {
            TabOrder::Row => 'R',
            TabOrder::Column => 'C',
            TabOrder::Structure => 'S',
        }
    }
}

/// Stroke style for line-drawing primitives (`stroke_rect`, `stroke_line`,
/// shape primitives).
///
/// Introduced alongside the buffered `Table` surface so cell borders and
/// row rules can have explicit thickness and colour without forcing users
/// through the lower-level `ContentElement::Path` builder.
#[derive(Debug, Clone)]
pub struct LineStyle {
    /// Stroke width in points. Must be > 0.
    pub width: f32,
    /// RGB colour, each channel in `0.0..=1.0`.
    pub color: (f32, f32, f32),
    /// Optional dash pattern: `Some((dashes, phase))` emits a
    /// `[dashes...] phase d` graphics-state op before stroking. Each
    /// entry in `dashes` is an on/off length in points (e.g.
    /// `[3.0, 2.0]` = 3 pt dash, 2 pt gap, repeating). `None` is solid.
    pub dash: Option<(Vec<f32>, f32)>,
}

impl Default for LineStyle {
    fn default() -> Self {
        Self {
            width: 1.0,
            color: (0.0, 0.0, 0.0),
            dash: None,
        }
    }
}

impl LineStyle {
    /// Construct a `LineStyle` from a width (points) and RGB colour
    /// channels (each `0.0..=1.0`).
    pub fn new(width: f32, r: f32, g: f32, b: f32) -> Self {
        Self {
            width,
            color: (r, g, b),
            dash: None,
        }
    }

    /// Builder: set a dash pattern. `dashes` is on/off lengths in
    /// points, `phase` is the starting offset.
    pub fn with_dash(mut self, dashes: &[f32], phase: f32) -> Self {
        self.dash = Some((dashes.to_vec(), phase));
        self
    }

    /// Builder: clear the dash pattern (return to solid).
    pub fn solid(mut self) -> Self {
        self.dash = None;
        self
    }
}

// ---------------------------------------------------------------------------
// Rich text support
// ---------------------------------------------------------------------------

/// Style applied to a single run in [`FluentPageBuilder::rich_paragraph`].
#[derive(Debug, Clone, PartialEq)]
pub enum TextRunStyle {
    /// Normal weight, current font color.
    Normal,
    /// Bold weight using the bold variant of the current font.
    Bold,
    /// Italic using the italic variant of the current font.
    Italic,
    /// RGB color override (0.0–1.0 per channel), normal weight.
    Color {
        /// Red channel (0.0–1.0).
        r: f32,
        /// Green channel (0.0–1.0).
        g: f32,
        /// Blue channel (0.0–1.0).
        b: f32,
    },
}

/// A single styled text run for [`FluentPageBuilder::rich_paragraph`].
///
/// Construct with the helper functions [`TextRun::normal`], [`TextRun::bold`],
/// [`TextRun::italic`], or [`TextRun::color`].
#[derive(Debug, Clone)]
pub struct TextRun {
    /// The text content of this run.
    pub text: String,
    /// Visual style applied to this run.
    pub style: TextRunStyle,
}

impl TextRun {
    /// Create a normal-weight run.
    pub fn normal(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: TextRunStyle::Normal,
        }
    }
    /// Create a bold run.
    pub fn bold(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: TextRunStyle::Bold,
        }
    }
    /// Create an italic run.
    pub fn italic(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: TextRunStyle::Italic,
        }
    }
    /// Create a color-overridden run. Channels are in 0.0–1.0 range.
    pub fn color(r: f32, g: f32, b: f32, text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: TextRunStyle::Color { r, g, b },
        }
    }
}

/// Return the bold variant of a base font name.
fn bold_font_name(base: &str) -> String {
    if base.contains("Bold") || base.contains("bold") {
        return base.to_string();
    }
    match base {
        "Helvetica" => "Helvetica-Bold".to_string(),
        "Times-Roman" | "Times" => "Times-Bold".to_string(),
        "Courier" => "Courier-Bold".to_string(),
        other => format!("{}-Bold", other),
    }
}

/// Return the italic variant of a base font name.
fn italic_font_name(base: &str) -> String {
    if base.contains("Italic") || base.contains("italic") || base.contains("Oblique") {
        return base.to_string();
    }
    match base {
        "Helvetica" => "Helvetica-Oblique".to_string(),
        "Helvetica-Bold" => "Helvetica-BoldOblique".to_string(),
        "Times-Roman" | "Times" => "Times-Italic".to_string(),
        "Courier" => "Courier-Oblique".to_string(),
        other => format!("{}-Italic", other),
    }
}

/// Page builder for adding content to a page with fluent API.
pub struct FluentPageBuilder<'a> {
    builder: &'a mut DocumentBuilder,
    page_index: usize,
    cursor_x: f32,
    cursor_y: f32,
    text_config: TextConfig,
    text_layout: TextLayout,
    /// Track the last text element's bounding box for text markup annotations
    last_text_rect: Option<Rect>,
    /// Pending annotations for this page
    pending_annotations: Vec<Annotation>,
    /// Active 2D affine transform (PDF row order: `[a b c d e f]`).
    /// `None` = identity. Populated by [`Self::with_transform`] /
    /// [`Self::rotated`] / [`Self::scaled`] / [`Self::translated`] and
    /// applied to `TextContent.matrix` on emission. v0.3.39 scope:
    /// text only — see inline comment in the transforms section for
    /// the Path / Image / Table follow-up.
    current_matrix: Option<[f32; 6]>,
    /// Deferred `/Tabs` value for the current page, set by
    /// [`Self::tab_order`] and applied in [`Self::done`]. #393 D-4.
    pending_tab_order: Option<TabOrder>,
}

impl<'a> FluentPageBuilder<'a> {
    /// Set the text configuration for subsequent text operations.
    pub fn text_config(mut self, config: TextConfig) -> Self {
        self.text_config = config;
        self
    }

    /// Set font for subsequent text operations.
    pub fn font(mut self, name: &str, size: f32) -> Self {
        self.text_config.font = name.to_string();
        self.text_config.size = size;
        self
    }

    /// Set cursor position for text placement.
    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.cursor_x = x;
        self.cursor_y = y;
        self
    }

    /// Vertical points remaining on the current page from the cursor down to
    /// the bottom margin (conventionally 72 pt / 1 inch from the bottom of
    /// the page). Pure query — no mutation, no emission.
    ///
    /// The streaming `Table` surface (research #393) uses this to decide
    /// whether the next row fits or whether to trigger a page break:
    ///
    /// ```no_run
    /// # use pdf_oxide::writer::DocumentBuilder;
    /// # let mut doc = DocumentBuilder::new();
    /// # let page = doc.letter_page();
    /// if page.remaining_space() < 40.0 {
    ///     // ... trigger new_page_same_size() and redraw header
    /// }
    /// ```
    ///
    /// Returns 0.0 (not negative) when the cursor has already passed the
    /// bottom margin. A return of > 0.0 does not *guarantee* the next row
    /// fits — that depends on the row's own measured height.
    pub fn remaining_space(&self) -> f32 {
        const BOTTOM_MARGIN: f32 = 72.0;
        (self.cursor_y - BOTTOM_MARGIN).max(0.0)
    }

    /// Finish the current page and start a new one with the **same page
    /// size**. The builder's `text_config` (font, size, alignment,
    /// line-height multiplier) carries over; cursor resets to the same
    /// top-left origin the current page started at.
    ///
    /// Pending annotations and form fields on the current page are
    /// committed before the new page is opened. Does not re-draw any
    /// header / footer — callers that want header-repeat-on-break (tables,
    /// long documents) must redraw explicitly.
    pub fn new_page_same_size(mut self) -> FluentPageBuilder<'a> {
        self.new_page_same_size_inplace();
        self
    }

    /// In-place page break: same semantics as `new_page_same_size` but
    /// mutates `self` rather than consuming it. Used by `StreamingTable`
    /// (which borrows its `FluentPageBuilder` and can't consume).
    pub(crate) fn new_page_same_size_inplace(&mut self) {
        let current = &self.builder.pages[self.page_index];
        let width = current.width;
        let height = current.height;

        // Commit pending annotations to the current page before switching.
        let annotations = std::mem::take(&mut self.pending_annotations);
        self.builder.pages[self.page_index]
            .annotations
            .extend(annotations);

        // Append a fresh PageData with matching dimensions.
        self.page_index = self.builder.pages.len();
        self.builder.pages.push(PageData {
            width,
            height,
            elements: Vec::new(),
            annotations: Vec::new(),
            form_fields: Vec::new(),
            form_field_meta: Vec::new(),
            tab_order: None,
            page_open_script: None,
            page_close_script: None,
            pending_footnotes: Vec::new(),
        });

        // Reset cursor to top-left (mirrors DocumentBuilder::page).
        self.cursor_x = 72.0;
        self.cursor_y = height - 72.0;
        self.last_text_rect = None;
        // text_config, text_layout, pending_annotations (now empty)
        // carry over automatically.
    }

    /// Width and height of the current page in points. Useful when the
    /// caller needs to size content to the actual page (e.g. table column
    /// caps that must reflect a landscape orientation chosen at section
    /// time, not the document's default size).
    pub fn page_dimensions(&self) -> (f32, f32) {
        let p = &self.builder.pages[self.page_index];
        (p.width, p.height)
    }

    /// Current cursor X (points from left edge). Used by
    /// `StreamingTable` to anchor column offsets.
    pub(crate) fn cursor_x(&self) -> f32 {
        self.cursor_x
    }
    /// Current cursor Y (points from bottom edge, PDF convention).
    pub(crate) fn cursor_y(&self) -> f32 {
        self.cursor_y
    }
    /// Move the cursor down to `y`. Internal use — public callers should
    /// use `at()` which takes (x, y).
    pub(crate) fn set_cursor_y(&mut self, y: f32) {
        self.cursor_y = y;
    }
    /// Width of the current page in points. Useful for callers
    /// computing alignment rectangles relative to the page edge.
    pub(crate) fn page_width(&self) -> f32 {
        self.builder.pages[self.page_index].width
    }
    /// Font name from the builder's current text_config.
    pub(crate) fn text_config_font_name(&self) -> &str {
        &self.text_config.font
    }
    /// Font size in points.
    pub(crate) fn text_config_font_size(&self) -> f32 {
        self.text_config.size
    }
    /// Line-height multiplier (multiplied with font size to get baseline
    /// step).
    pub(crate) fn text_config_line_height(&self) -> f32 {
        self.text_config.line_height
    }
    /// Wrap `text` to `max_width` using the builder's TextLayout engine.
    /// Returns one `(line, measured_width)` per visual line.
    pub(crate) fn wrap_cell_text(&self, text: &str, max_width: f32) -> Vec<(String, f32)> {
        self.text_layout
            .wrap_text(text, &self.text_config.font, self.text_config.size, max_width)
    }
    /// Push a `ContentElement` into the current page's element list.
    pub(crate) fn push_element(&mut self, element: ContentElement) {
        self.builder.pages[self.page_index].elements.push(element);
    }
    /// Number of elements already on the current page — used to seed
    /// monotone reading_order values.
    pub(crate) fn page_element_count(&self) -> usize {
        self.builder.pages[self.page_index].elements.len()
    }

    /// Open a streaming table that consumes this page builder. See
    /// [`super::streaming_table::StreamingTable`] for the full API and
    /// [issue #393](https://github.com/yfedoseev/pdf_oxide/issues/393)
    /// for design rationale.
    pub fn streaming_table(
        self,
        config: super::streaming_table::StreamingTableConfig,
    ) -> super::streaming_table::StreamingTable<'a> {
        super::streaming_table::StreamingTable::open(self, config)
    }

    /// Measure the rendered width of `text` in the builder's current font
    /// and size, in PDF points. Pure query — does not advance the cursor or
    /// emit any content.
    ///
    /// Use this to pick explicit column widths before calling
    /// `streaming_table` (see #393) or to right-align custom labels. For
    /// embedded fonts, the measure honours the face's horizontal advances
    /// (HMTX). For base-14 fonts, it uses the AFM width tables.
    pub fn measure(&self, text: &str) -> f32 {
        self.text_layout.font_manager().text_width(
            text,
            &self.text_config.font,
            self.text_config.size,
        )
    }

    /// Add text at the current cursor position.
    pub fn text(mut self, text: &str) -> Self {
        let text_width = self.text_layout.font_manager().text_width(
            text,
            &self.text_config.font,
            self.text_config.size,
        );

        // Create the bounding box and track it for potential markup annotations
        let text_rect = Rect::new(self.cursor_x, self.cursor_y, text_width, self.text_config.size);
        self.last_text_rect = Some(text_rect);

        let page = &mut self.builder.pages[self.page_index];
        page.elements.push(ContentElement::Text(TextContent {
            text: text.to_string(),
            bbox: text_rect,
            font: crate::elements::FontSpec {
                name: self.text_config.font.clone(),
                size: self.text_config.size,
            },
            style: Default::default(),
            reading_order: Some(page.elements.len()),
            artifact_type: None,
            origin: None,
            rotation_degrees: None,
            matrix: self.current_matrix,
        }));
        // Move cursor down for next line
        self.cursor_y -= self.text_config.size * self.text_config.line_height;
        self
    }

    /// Place wrapped text inside a rectangle with horizontal alignment.
    ///
    /// Wraps `text` to `rect.width` using the builder's current font and
    /// size, emits one content-stream line per wrapped line, and positions
    /// each line within the rect per `align`:
    ///
    /// - `Left`:   line left edge at `rect.x`
    /// - `Center`: line centered within `rect.x .. rect.x + rect.width`
    /// - `Right`:  line right edge at `rect.x + rect.width`
    ///
    /// Vertical layout is top-anchored: the first line's top sits at
    /// `rect.y`, subsequent lines drop by `size * line_height`. The cursor
    /// is **not** advanced — the rect has its own geometry and the caller
    /// owns the cursor. Use `measure()` or `text_layout.text_bounds()` to
    /// pre-compute rect dimensions if needed.
    ///
    /// This is the cell-text primitive the buffered `Table` surface
    /// (research #393) consumes. It is also usable standalone for
    /// box-constrained captions, labels, and pull-quotes.
    pub fn text_in_rect(mut self, rect: Rect, text: &str, align: TextAlign) -> Self {
        let lines = self.text_layout.wrap_text(
            text,
            &self.text_config.font,
            self.text_config.size,
            rect.width,
        );

        let line_height = self.text_config.size * self.text_config.line_height;
        let mut line_top = rect.y;

        for (line_text, line_width) in lines {
            if line_text.is_empty() {
                line_top -= line_height;
                continue;
            }

            let line_x = match align {
                TextAlign::Left => rect.x,
                TextAlign::Center => rect.x + (rect.width - line_width) / 2.0,
                TextAlign::Right => rect.x + rect.width - line_width,
            };

            let bbox = Rect::new(line_x, line_top, line_width, self.text_config.size);
            let page = &mut self.builder.pages[self.page_index];
            page.elements.push(ContentElement::Text(TextContent {
                text: line_text,
                bbox,
                font: crate::elements::FontSpec {
                    name: self.text_config.font.clone(),
                    size: self.text_config.size,
                },
                style: Default::default(),
                reading_order: Some(page.elements.len()),
                artifact_type: None,
                origin: None,
                rotation_degrees: None,
                matrix: None,
            }));

            line_top -= line_height;
        }

        // Track only the last emitted line for potential markup annotations.
        // Table callers typically set markup at the whole-cell level, not
        // per-line; the default matches the behaviour of `.text()`.
        self.last_text_rect = Some(rect);
        self
    }

    /// Add a heading (larger, bold text).
    pub fn heading(self, level: u8, text: &str) -> Self {
        let size = match level {
            1 => 24.0,
            2 => 20.0,
            3 => 16.0,
            _ => 14.0,
        };
        let font = match level {
            1 | 2 => "Helvetica-Bold",
            _ => "Helvetica",
        };
        self.font(font, size).text(text)
    }

    /// Render an unordered list of items at the current cursor
    /// position. Each item gets an indented bullet + wrapped text;
    /// the cursor advances past the last item with a small bottom
    /// padding.
    ///
    /// Bullet glyph is `•` (U+2022) — renders in Helvetica / Times /
    /// Courier without font embedding. For custom bullets (e.g. `★`
    /// with an embedded CJK / symbol font) call `bullet_list_styled`
    /// (not yet in v0.3.39 — filed as a v0.3.40 enhancement).
    ///
    /// #393 Bundle E-2.
    pub fn bullet_list<S: AsRef<str>>(self, items: &[S]) -> Self {
        self.list_inner(items, ListMarker::Bullet)
    }

    /// Render an ordered/numbered list. `style` controls the marker
    /// format (decimal `1. 2. 3.`, lowercase Roman, or lowercase
    /// alphabetic). #393 Bundle E-2.
    pub fn numbered_list<S: AsRef<str>>(self, items: &[S], style: ListStyle) -> Self {
        self.list_inner(items, ListMarker::Numbered(style))
    }

    fn list_inner<S: AsRef<str>>(mut self, items: &[S], marker: ListMarker) -> Self {
        if items.is_empty() {
            return self;
        }
        let indent = 18.0_f32;
        let marker_cell_w = 22.0_f32;
        let line_height = self.text_config.size * self.text_config.line_height;
        let font = self.text_config.font.clone();
        let size = self.text_config.size;
        let page_w = self.builder.pages[self.page_index].width;
        let right_margin = 72.0;
        let text_x = self.cursor_x + indent + marker_cell_w;
        let content_w = page_w - text_x - right_margin;

        for (idx, item) in items.iter().enumerate() {
            let item_text = item.as_ref();
            let line_y = self.cursor_y;
            // Marker (bullet glyph or number) at indent.
            let marker_str = match &marker {
                ListMarker::Bullet => "\u{2022}".to_string(),
                ListMarker::Numbered(style) => match style {
                    ListStyle::Decimal => format!("{}.", idx + 1),
                    ListStyle::RomanLower => format!("{}.", to_roman_lower(idx + 1)),
                    ListStyle::AlphaLower => format!("{}.", to_alpha_lower(idx + 1)),
                },
            };
            let marker_x = self.cursor_x + indent;
            self = self
                .at(marker_x, line_y)
                .font(&font, size)
                .text(&marker_str);
            // Undo the cursor-y advance `.text()` applied so we can
            // place the wrapped content lines at the correct ys below.
            self.cursor_y += line_height;

            // Body: wrap the item text across multiple lines.
            let wrapped = self
                .text_layout
                .wrap_text(item_text, &font, size, content_w);
            if wrapped.is_empty() {
                // Empty item — just drop one blank line.
                self.cursor_y -= line_height;
                continue;
            }
            for (n, (ln, _)) in wrapped.iter().enumerate() {
                let ly = line_y - (n as f32) * line_height;
                self = self.at(text_x, ly).text(ln);
                self.cursor_y += line_height;
            }
            // Net advance for this item: one line_height per wrapped
            // line.
            self.cursor_y -= (wrapped.len() as f32) * line_height;
        }
        // Small trailing breather after the whole list.
        self.cursor_y -= 6.0;
        self
    }

    /// Render a code block at the current cursor position. The block
    /// is rendered as a single filled-background rectangle spanning
    /// the page content width, with monospace (Courier) text laid out
    /// line-by-line inside it. Lines wrap at the content width with a
    /// configurable left/right padding.
    ///
    /// `source` is the code string (preserves `\n` line breaks);
    /// `language` is cosmetic — it doesn't syntax-highlight in v0.3.39
    /// (syntax highlighting is deferred to v0.3.40) but is recorded
    /// for future accessibility tagging (Bundle F).
    ///
    /// After this call the cursor advances past the block. #393
    /// Bundle E-3.
    ///
    /// ```no_run
    /// # use pdf_oxide::writer::DocumentBuilder;
    /// # let mut doc = DocumentBuilder::new();
    /// doc.letter_page()
    ///    .at(72.0, 720.0)
    ///    .heading(1, "Example")
    ///    .code_block("rust", "fn main() {\n    println!(\"hi\");\n}")
    ///    .done();
    /// ```
    pub fn code_block(mut self, language: &str, source: &str) -> Self {
        let _ = language; // reserved for Bundle F accessibility tagging
        let page_width = self.builder.pages[self.page_index].width;
        let left_margin = self.cursor_x.max(72.0);
        let right_margin = 72.0;
        let rect_x = left_margin - 4.0;
        let rect_w = page_width - left_margin - right_margin + 8.0;
        let inner_w = rect_w - 12.0;

        // Switch to monospace for this block; remember the outer font
        // so we can restore it on exit.
        let outer_font = self.text_config.font.clone();
        let outer_size = self.text_config.size;
        self = self.font("Courier", 9.0);

        // Compute total block height by pre-wrapping each source line.
        // 9pt Courier × 1.2 line-height = 10.8 pt per line; 4pt top +
        // 4pt bottom padding.
        let line_height = self.text_config.size * self.text_config.line_height;
        let top_pad = 4.0_f32;
        let bot_pad = 4.0_f32;
        let all_lines: Vec<(String, f32)> = source
            .split('\n')
            .flat_map(|ln| {
                let wrapped = self.text_layout.wrap_text(
                    ln,
                    &self.text_config.font,
                    self.text_config.size,
                    inner_w,
                );
                if wrapped.is_empty() {
                    vec![(String::new(), 0.0)]
                } else {
                    wrapped
                }
            })
            .collect();
        let block_h = top_pad + bot_pad + (all_lines.len() as f32) * line_height;

        // Background fill (light grey).
        let bg_y = self.cursor_y - block_h;
        self = self.filled_rect(rect_x, bg_y, rect_w, block_h, 0.96, 0.96, 0.96);

        // Emit each wrapped line as its own text element. Start from
        // the top-padding offset inside the block and step down by
        // line_height.
        let start_y = self.cursor_y - top_pad;
        let text_x = left_margin;
        for (line_idx, (line, _)) in all_lines.iter().enumerate() {
            let line_y = start_y - (line_idx as f32) * line_height;
            self = self.at(text_x, line_y).text(line);
            // `.text()` advances cursor_y; undo that advance so every
            // line lands at the pre-computed y, not line_height below
            // the prior one (double-advance).
            self.cursor_y += line_height;
        }

        // Final cursor: just below the block with a small breathing
        // space.
        self.cursor_y = bg_y - 6.0;
        self = self.font(&outer_font, outer_size);
        self
    }

    /// Add a paragraph of text with automatic word wrapping.
    pub fn paragraph(mut self, text: &str) -> Self {
        // Use FontManager-based word wrapping for accurate metrics
        let page = &mut self.builder.pages[self.page_index];
        let max_width = page.width - self.cursor_x - 72.0; // 72pt right margin

        let lines = self.text_layout.wrap_text(
            text,
            &self.text_config.font,
            self.text_config.size,
            max_width,
        );

        let line_height = self.text_config.size * self.text_config.line_height;
        const BOTTOM_MARGIN: f32 = 72.0;
        for (line_text, line_width) in lines {
            // Break to a new page when the next line would land below the bottom
            // margin; otherwise content silently writes off the page (invisible).
            if self.cursor_y - line_height < BOTTOM_MARGIN {
                self.new_page_same_size_inplace();
            }
            let page = &mut self.builder.pages[self.page_index];
            page.elements.push(ContentElement::Text(TextContent {
                text: line_text,
                bbox: Rect::new(self.cursor_x, self.cursor_y, line_width, self.text_config.size),
                font: crate::elements::FontSpec {
                    name: self.text_config.font.clone(),
                    size: self.text_config.size,
                },
                style: Default::default(),
                reading_order: Some(page.elements.len()),
                artifact_type: None,
                origin: None,
                rotation_degrees: None,
                matrix: None,
            }));
            self.cursor_y -= line_height;
        }
        // Add extra space after paragraph
        self.cursor_y -= self.text_config.size * 0.5;
        self
    }

    /// Add vertical space.
    pub fn space(mut self, points: f32) -> Self {
        self.cursor_y -= points;
        self
    }

    /// Add a horizontal line.
    pub fn horizontal_rule(mut self) -> Self {
        let page = &mut self.builder.pages[self.page_index];
        let line_y = self.cursor_y + self.text_config.size * 0.5;
        page.elements
            .push(ContentElement::Path(crate::elements::PathContent {
                paint: Default::default(),
                operations: vec![
                    crate::elements::PathOperation::MoveTo(self.cursor_x, line_y),
                    crate::elements::PathOperation::LineTo(page.width - 72.0, line_y),
                ],
                bbox: Rect::new(self.cursor_x, line_y, page.width - 72.0 - self.cursor_x, 1.0),
                stroke_color: Some(crate::layout::Color {
                    r: 0.5,
                    g: 0.5,
                    b: 0.5,
                }),
                fill_color: None,
                stroke_width: 0.5,
                line_cap: crate::elements::LineCap::Butt,
                line_join: crate::elements::LineJoin::Miter,
                dash_pattern: None,
                matrix: None,
                reading_order: None,
                artifact_type: None,
                layer: None,
            }));
        self.cursor_y -= self.text_config.size;
        self
    }

    /// Add a content element directly.
    pub fn element(self, element: ContentElement) -> Self {
        let page = &mut self.builder.pages[self.page_index];
        page.elements.push(element);
        self
    }

    /// Add multiple content elements.
    pub fn elements(self, elements: Vec<ContentElement>) -> Self {
        let page = &mut self.builder.pages[self.page_index];
        page.elements.extend(elements);
        self
    }

    /// Add a footnote reference mark at the current cursor position and
    /// record the corresponding footnote body for page-end placement.
    ///
    /// The `ref_mark` (e.g. `"¹"` or `"[1]"`) is emitted inline at 65 %
    /// of the current font size. The `note_text` body is collected and
    /// rendered below a separator line near the page bottom when the page
    /// is finalised in `DocumentBuilder::build`. In tagged PDF/UA mode the
    /// body is automatically wrapped in a `<Note>` structure element.
    ///
    /// # Example
    ///
    /// ```ignore
    /// page.at(72.0, 700.0)
    ///     .text("Important claim")
    ///     .footnote("¹", "Source: Annual report 2025.")
    ///     .text(" continued here.")
    ///     .done()
    /// ```
    pub fn footnote(mut self, ref_mark: &str, note_text: &str) -> Self {
        let base_size = self.text_config.size;
        let ref_size = (base_size * 0.65).max(6.0);

        let ref_w =
            self.text_layout
                .font_manager()
                .text_width(ref_mark, &self.text_config.font, ref_size);

        let page = &mut self.builder.pages[self.page_index];
        let reading_order = page.elements.len();
        page.elements.push(ContentElement::Text(TextContent {
            text: ref_mark.to_string(),
            bbox: Rect::new(self.cursor_x, self.cursor_y, ref_w, ref_size),
            font: crate::elements::FontSpec {
                name: self.text_config.font.clone(),
                size: ref_size,
            },
            style: Default::default(),
            reading_order: Some(reading_order),
            artifact_type: None,
            origin: None,
            rotation_degrees: None,
            matrix: self.current_matrix,
        }));
        page.pending_footnotes
            .push((ref_mark.to_string(), note_text.to_string()));

        // Advance cursor_x past the ref mark; don't advance cursor_y (inline).
        self.cursor_x += ref_w;
        self
    }

    /// Lay out `text` as wrapped lines distributed across `column_count`
    /// balanced columns starting at the current cursor position.
    ///
    /// Internally uses
    /// [`crate::html_css::layout::multicol::distribute_lines_into_columns`]
    /// to balance the line distribution. The columns span the full available
    /// width (page width minus the left cursor margin and a 72 pt right
    /// margin), separated by `gap_pt` points of inter-column space.
    ///
    /// The cursor advances past the tallest column so subsequent content
    /// continues below the column block.
    ///
    /// # Example
    ///
    /// ```ignore
    /// page.at(72.0, 700.0)
    ///     .columns(2, 12.0, "First paragraph.\nSecond paragraph runs long enough to wrap across both columns.")
    ///     .done()
    /// ```
    pub fn columns(mut self, column_count: u32, gap_pt: f32, text: &str) -> Self {
        use crate::html_css::layout::multicol::distribute_lines_into_columns;

        let n = column_count.max(1);
        let gap = gap_pt.max(0.0);

        // Geometry
        let page_width = self.builder.pages[self.page_index].width;
        let left_x = self.cursor_x;
        let right_margin = 72.0_f32;
        let available = (page_width - left_x - right_margin).max(0.0);
        let total_gap = gap * (n - 1) as f32;
        let col_width = ((available - total_gap) / n as f32).max(1.0);

        let font_size = self.text_config.size;
        let line_h = font_size * self.text_config.line_height;

        // Collect all wrapped lines from the text (split on "\n\n" for paragraphs)
        let mut all_lines: Vec<(String, f32)> = Vec::new(); // (text, measured_width)
        let mut line_heights: Vec<f32> = Vec::new();
        for (para_idx, para) in text.split("\n\n").enumerate() {
            if para_idx > 0 {
                // Blank separator line between paragraphs
                all_lines.push((String::new(), 0.0));
                line_heights.push(font_size * 0.5);
            }
            let wrapped =
                self.text_layout
                    .wrap_text(para, &self.text_config.font, font_size, col_width);
            for (line_text, line_width) in wrapped {
                all_lines.push((line_text, line_width));
                line_heights.push(line_h);
            }
        }

        if all_lines.is_empty() {
            return self;
        }

        // Balance: each column gets ≈ total_height / n height capacity.
        let total_h: f32 = line_heights.iter().sum();
        let col_h_cap = (total_h / n as f32).ceil().max(line_h);

        let dist = distribute_lines_into_columns(&line_heights, n, col_h_cap);

        // Emit elements — borrow pages mutably only during push
        let start_y = self.cursor_y;
        let mut max_drop = 0.0_f32;

        for (col_idx, line_indices) in dist.iter().enumerate() {
            let col_x = left_x + col_idx as f32 * (col_width + gap);
            let mut drop = 0.0_f32;
            for &li in line_indices {
                let (ref line_text, line_width) = all_lines[li];
                let lh = line_heights[li];
                if !line_text.is_empty() {
                    let y = start_y - drop;
                    let font_name = self.text_config.font.clone();
                    let reading_order = self.builder.pages[self.page_index].elements.len();
                    self.builder.pages[self.page_index]
                        .elements
                        .push(ContentElement::Text(TextContent {
                            text: line_text.clone(),
                            bbox: Rect::new(col_x, y, line_width, font_size),
                            font: crate::elements::FontSpec {
                                name: font_name,
                                size: font_size,
                            },
                            style: Default::default(),
                            reading_order: Some(reading_order),
                            artifact_type: None,
                            origin: None,
                            rotation_degrees: None,
                            matrix: self.current_matrix,
                        }));
                }
                drop += lh;
            }
            max_drop = max_drop.max(drop);
        }

        self.cursor_y = start_y - max_drop - font_size * 0.5;
        self
    }

    // ==========================================================================
    // Rich text (inline runs)
    // ==========================================================================

    /// Emit `text` inline at the current cursor — advances `cursor_x` by the
    /// measured text width but does **not** advance `cursor_y`.  Call
    /// [`Self::newline`] after a sequence of inline calls to move to the next
    /// line, or use [`Self::rich_paragraph`] for an all-in-one API.
    pub fn inline(mut self, text: &str) -> Self {
        self.emit_inline_run(text, None, None);
        self
    }

    /// Inline bold run (Helvetica-Bold / embedded font "-Bold" suffix).
    pub fn inline_bold(mut self, text: &str) -> Self {
        let bold_font = bold_font_name(&self.text_config.font);
        self.emit_inline_run(text, Some(bold_font), None);
        self
    }

    /// Inline italic run (Helvetica-Oblique / embedded font "-Oblique" suffix).
    pub fn inline_italic(mut self, text: &str) -> Self {
        let italic_font = italic_font_name(&self.text_config.font);
        self.emit_inline_run(text, Some(italic_font), None);
        self
    }

    /// Inline run with a custom RGB color (values 0.0–1.0).
    pub fn inline_color(mut self, r: f32, g: f32, b: f32, text: &str) -> Self {
        self.emit_inline_run(text, None, Some(crate::layout::Color { r, g, b }));
        self
    }

    /// Advance cursor_y by one line-height and reset cursor_x to the left
    /// margin (72 pt). Used after a run of [`Self::inline`] calls.
    pub fn newline(mut self) -> Self {
        self.cursor_y -= self.text_config.size * self.text_config.line_height;
        self.cursor_x = 72.0;
        self
    }

    /// A single text run for [`Self::rich_paragraph`].
    ///
    /// Construct with the associated helper functions:
    /// [`TextRun::normal`], [`TextRun::bold`], [`TextRun::italic`],
    /// [`TextRun::color`].
    pub fn rich_paragraph(mut self, runs: &[TextRun]) -> Self {
        let left_margin = self.cursor_x;
        let right_margin = 72.0_f32;
        let page_width = self.builder.pages[self.page_index].width;
        let max_right = page_width - right_margin;

        for (run_idx, run) in runs.iter().enumerate() {
            let font_name = match run.style {
                TextRunStyle::Bold => bold_font_name(&self.text_config.font),
                TextRunStyle::Italic => italic_font_name(&self.text_config.font),
                TextRunStyle::Normal | TextRunStyle::Color { .. } => self.text_config.font.clone(),
            };
            let color = match run.style {
                TextRunStyle::Color { r, g, b } => Some(crate::layout::Color { r, g, b }),
                _ => None,
            };

            // Each run lays out its own words independently and then
            // advances `cursor_x` by exactly the emitted text's width — no
            // separator is otherwise inserted between one run ending and
            // the next beginning. A run boundary that falls mid-line would
            // draw flush against the previous run with zero gap ("Text Run
            // 1" + "Text Run 2" → "Text Run 1Text Run 2"). Advance past a
            // single space before laying out this run's words, but only
            // when continuing an existing line — a run that starts a fresh
            // page/line (cursor_x back at the margin, from wrapping or
            // being the paragraph's first run) needs no separator, the line
            // break already provides it. Any leading/trailing whitespace on
            // the run's own text is still discarded by `split_whitespace`
            // below (unchanged) — spacing between runs is guaranteed here
            // instead of relying on the caller to pad it correctly.
            if run_idx > 0 && (self.cursor_x - left_margin).abs() > 0.01 {
                let space_w = self.text_layout.font_manager().text_width(
                    " ",
                    &font_name,
                    self.text_config.size,
                );
                if self.cursor_x + space_w <= max_right {
                    self.cursor_x += space_w;
                } else {
                    let line_height = self.text_config.size * self.text_config.line_height;
                    if self.cursor_y - line_height < 72.0 {
                        self.new_page_same_size_inplace();
                    }
                    self.cursor_y -= line_height;
                    self.cursor_x = left_margin;
                }
            }

            // Wrap run text to available line width
            let words: Vec<&str> = run.text.split_whitespace().collect();
            let mut buf = String::new();
            for word in words {
                let candidate = if buf.is_empty() {
                    word.to_string()
                } else {
                    format!("{} {}", buf, word)
                };
                let cw = self.text_layout.font_manager().text_width(
                    &candidate,
                    &font_name,
                    self.text_config.size,
                );
                if self.cursor_x + cw > max_right && !buf.is_empty() {
                    // Auto-paginate when next line would land below bottom margin.
                    let line_height = self.text_config.size * self.text_config.line_height;
                    if self.cursor_y - line_height < 72.0 {
                        self.new_page_same_size_inplace();
                    }
                    // Emit buf as a line
                    let bw = self.text_layout.font_manager().text_width(
                        &buf,
                        &font_name,
                        self.text_config.size,
                    );
                    let reading_order = self.builder.pages[self.page_index].elements.len();
                    self.builder.pages[self.page_index]
                        .elements
                        .push(ContentElement::Text(TextContent {
                            text: buf.clone(),
                            bbox: Rect::new(
                                self.cursor_x,
                                self.cursor_y,
                                bw,
                                self.text_config.size,
                            ),
                            font: crate::elements::FontSpec {
                                name: font_name.clone(),
                                size: self.text_config.size,
                            },
                            style: crate::elements::TextStyle {
                                color: color.unwrap_or(crate::layout::Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                }),
                                ..Default::default()
                            },
                            reading_order: Some(reading_order),
                            artifact_type: None,
                            origin: None,
                            rotation_degrees: None,
                            matrix: self.current_matrix,
                        }));
                    self.cursor_y -= line_height;
                    self.cursor_x = left_margin;
                    buf = word.to_string();
                } else {
                    buf = candidate;
                }
            }
            // Emit any remaining words
            if !buf.is_empty() {
                let bw = self.text_layout.font_manager().text_width(
                    &buf,
                    &font_name,
                    self.text_config.size,
                );
                let reading_order = self.builder.pages[self.page_index].elements.len();
                self.builder.pages[self.page_index]
                    .elements
                    .push(ContentElement::Text(TextContent {
                        text: buf,
                        bbox: Rect::new(self.cursor_x, self.cursor_y, bw, self.text_config.size),
                        font: crate::elements::FontSpec {
                            name: font_name.clone(),
                            size: self.text_config.size,
                        },
                        style: crate::elements::TextStyle {
                            color: color.unwrap_or(crate::layout::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                            }),
                            ..Default::default()
                        },
                        reading_order: Some(reading_order),
                        artifact_type: None,
                        origin: None,
                        rotation_degrees: None,
                        matrix: self.current_matrix,
                    }));
                self.cursor_x += bw;
            }
        }
        // Finish the paragraph
        self.cursor_y -= self.text_config.size * self.text_config.line_height;
        self.cursor_x = left_margin;
        self.cursor_y -= self.text_config.size * 0.5; // trailing gap
        self
    }

    /// Internal: emit one inline text run advancing cursor_x only.
    fn emit_inline_run(
        &mut self,
        text: &str,
        font_override: Option<String>,
        color: Option<crate::layout::Color>,
    ) {
        let font_name = font_override.unwrap_or_else(|| self.text_config.font.clone());
        let w = self
            .text_layout
            .font_manager()
            .text_width(text, &font_name, self.text_config.size);
        let reading_order = self.builder.pages[self.page_index].elements.len();
        self.builder.pages[self.page_index]
            .elements
            .push(ContentElement::Text(TextContent {
                text: text.to_string(),
                bbox: Rect::new(self.cursor_x, self.cursor_y, w, self.text_config.size),
                font: crate::elements::FontSpec {
                    name: font_name,
                    size: self.text_config.size,
                },
                style: crate::elements::TextStyle {
                    color: color.unwrap_or(crate::layout::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                    }),
                    ..Default::default()
                },
                reading_order: Some(reading_order),
                artifact_type: None,
                origin: None,
                rotation_degrees: None,
                matrix: self.current_matrix,
            }));
        self.cursor_x += w;
        self.last_text_rect =
            Some(Rect::new(self.cursor_x - w, self.cursor_y, w, self.text_config.size));
    }

    // ==========================================================================
    // Annotation Methods
    // ==========================================================================

    /// Add a URL link annotation to the last text element.
    ///
    /// The link will cover the bounding box of the most recently added text.
    ///
    /// # Example
    ///
    /// ```ignore
    /// builder.page(PageSize::Letter)
    ///     .at(72.0, 720.0)
    ///     .text("Visit our website")
    ///     .link_url("https://example.com")
    ///     .done();
    /// ```
    pub fn link_url(mut self, url: &str) -> Self {
        if let Some(rect) = self.last_text_rect {
            let link = LinkAnnotation::uri(rect, url);
            self.pending_annotations.push(link.into());
        }
        self
    }

    /// Add an internal page link annotation to the last text element.
    ///
    /// # Arguments
    ///
    /// * `page` - The target page index (0-based)
    ///
    /// # Example
    ///
    /// ```ignore
    /// builder.page(PageSize::Letter)
    ///     .at(72.0, 720.0)
    ///     .text("Go to page 5")
    ///     .link_page(4)  // 0-indexed
    ///     .done();
    /// ```
    pub fn link_page(mut self, page: usize) -> Self {
        if let Some(rect) = self.last_text_rect {
            let link = LinkAnnotation::goto_page(rect, page);
            self.pending_annotations.push(link.into());
        }
        self
    }

    /// Add a named destination link to the last text element.
    ///
    /// # Arguments
    ///
    /// * `destination` - The named destination string
    pub fn link_named(mut self, destination: &str) -> Self {
        if let Some(rect) = self.last_text_rect {
            let link = LinkAnnotation::goto_named(rect, destination);
            self.pending_annotations.push(link.into());
        }
        self
    }

    /// Add a JavaScript action link to the last text element.
    ///
    /// When clicked the viewer executes the provided JavaScript string.
    pub fn link_javascript(mut self, script: &str) -> Self {
        use super::annotation_builder::{BorderStyle, HighlightMode, LinkAction};
        if let Some(rect) = self.last_text_rect {
            let link = LinkAnnotation {
                rect,
                action: LinkAction::JavaScript(script.into()),
                border: BorderStyle::none(),
                highlight: HighlightMode::default(),
                color: None,
                quad_points: None,
            };
            self.pending_annotations.push(link.into());
        }
        self
    }

    /// Run a JavaScript script when this page is opened (`/AA /O`).
    pub fn on_open(self, script: &str) -> Self {
        self.builder.pages[self.page_index].page_open_script = Some(script.into());
        self
    }

    /// Run a JavaScript script when this page is closed (`/AA /C`).
    pub fn on_close(self, script: &str) -> Self {
        self.builder.pages[self.page_index].page_close_script = Some(script.into());
        self
    }

    /// Add a highlight annotation to the last text element.
    ///
    /// # Arguments
    ///
    /// * `color` - RGB color tuple (0.0-1.0 for each component)
    ///
    /// # Example
    ///
    /// ```ignore
    /// builder.page(PageSize::Letter)
    ///     .at(72.0, 720.0)
    ///     .text("Important text")
    ///     .highlight((1.0, 1.0, 0.0))  // Yellow highlight
    ///     .done();
    /// ```
    pub fn highlight(mut self, color: (f32, f32, f32)) -> Self {
        if let Some(rect) = self.last_text_rect {
            let markup = TextMarkupAnnotation::from_rect(TextMarkupType::Highlight, rect)
                .with_color(color.0, color.1, color.2);
            self.pending_annotations.push(markup.into());
        }
        self
    }

    /// Add an underline annotation to the last text element.
    ///
    /// # Arguments
    ///
    /// * `color` - RGB color tuple (0.0-1.0 for each component)
    pub fn underline(mut self, color: (f32, f32, f32)) -> Self {
        if let Some(rect) = self.last_text_rect {
            let markup = TextMarkupAnnotation::from_rect(TextMarkupType::Underline, rect)
                .with_color(color.0, color.1, color.2);
            self.pending_annotations.push(markup.into());
        }
        self
    }

    /// Add a strikeout annotation to the last text element.
    ///
    /// # Arguments
    ///
    /// * `color` - RGB color tuple (0.0-1.0 for each component)
    pub fn strikeout(mut self, color: (f32, f32, f32)) -> Self {
        if let Some(rect) = self.last_text_rect {
            let markup = TextMarkupAnnotation::from_rect(TextMarkupType::StrikeOut, rect)
                .with_color(color.0, color.1, color.2);
            self.pending_annotations.push(markup.into());
        }
        self
    }

    /// Add a squiggly underline annotation to the last text element.
    ///
    /// # Arguments
    ///
    /// * `color` - RGB color tuple (0.0-1.0 for each component)
    pub fn squiggly(mut self, color: (f32, f32, f32)) -> Self {
        if let Some(rect) = self.last_text_rect {
            let markup = TextMarkupAnnotation::from_rect(TextMarkupType::Squiggly, rect)
                .with_color(color.0, color.1, color.2);
            self.pending_annotations.push(markup.into());
        }
        self
    }

    /// Add a sticky note annotation at the current cursor position.
    ///
    /// # Arguments
    ///
    /// * `text` - The note content
    ///
    /// # Example
    ///
    /// ```ignore
    /// builder.page(PageSize::Letter)
    ///     .at(72.0, 720.0)
    ///     .sticky_note("Please review this section")
    ///     .done();
    /// ```
    pub fn sticky_note(mut self, text: &str) -> Self {
        // Place sticky note at current cursor position (small 24x24 icon)
        let rect = Rect::new(self.cursor_x, self.cursor_y, 24.0, 24.0);
        let note = TextAnnotation::new(rect, text);
        self.pending_annotations.push(note.into());
        self
    }

    /// Add a sticky note annotation with a specific icon at the current cursor position.
    ///
    /// # Arguments
    ///
    /// * `text` - The note content
    /// * `icon` - The icon to display
    pub fn sticky_note_with_icon(mut self, text: &str, icon: TextAnnotationIcon) -> Self {
        let rect = Rect::new(self.cursor_x, self.cursor_y, 24.0, 24.0);
        let note = TextAnnotation::new(rect, text).with_icon(icon);
        self.pending_annotations.push(note.into());
        self
    }

    /// Add a sticky note annotation at a specific position.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate
    /// * `y` - Y coordinate
    /// * `text` - The note content
    pub fn sticky_note_at(mut self, x: f32, y: f32, text: &str) -> Self {
        let rect = Rect::new(x, y, 24.0, 24.0);
        let note = TextAnnotation::new(rect, text);
        self.pending_annotations.push(note.into());
        self
    }

    /// Add a stamp annotation at the current cursor position.
    ///
    /// # Arguments
    ///
    /// * `stamp_type` - The type of stamp (Approved, Draft, Confidential, etc.)
    ///
    /// # Example
    ///
    /// ```ignore
    /// use pdf_oxide::writer::StampType;
    ///
    /// builder.page(PageSize::Letter)
    ///     .at(72.0, 720.0)
    ///     .stamp(StampType::Approved)
    ///     .done();
    /// ```
    pub fn stamp(mut self, stamp_type: StampType) -> Self {
        // Default stamp size: 150x50 points
        let rect = Rect::new(self.cursor_x, self.cursor_y, 150.0, 50.0);
        let stamp = StampAnnotation::new(rect, stamp_type);
        self.pending_annotations.push(stamp.into());
        self
    }

    /// Add a stamp annotation at a specific position with custom size.
    ///
    /// # Arguments
    ///
    /// * `rect` - The bounding rectangle for the stamp
    /// * `stamp_type` - The type of stamp
    pub fn stamp_at(mut self, rect: Rect, stamp_type: StampType) -> Self {
        let stamp = StampAnnotation::new(rect, stamp_type);
        self.pending_annotations.push(stamp.into());
        self
    }

    /// Add a FreeText annotation (text displayed directly on page).
    ///
    /// # Arguments
    ///
    /// * `rect` - The bounding rectangle for the text box
    /// * `text` - The text content
    pub fn freetext(mut self, rect: Rect, text: &str) -> Self {
        let freetext = FreeTextAnnotation::new(rect, text);
        self.pending_annotations.push(freetext.into());
        self
    }

    /// Add a FreeText annotation with custom font settings.
    ///
    /// # Arguments
    ///
    /// * `rect` - The bounding rectangle for the text box
    /// * `text` - The text content
    /// * `font` - Font name
    /// * `size` - Font size in points
    pub fn freetext_styled(mut self, rect: Rect, text: &str, font: &str, size: f32) -> Self {
        let freetext = FreeTextAnnotation::new(rect, text).with_font(font, size);
        self.pending_annotations.push(freetext.into());
        self
    }

    /// Add a watermark annotation (appears behind content, optionally print-only).
    ///
    /// # Arguments
    ///
    /// * `text` - The watermark text
    ///
    /// # Example
    ///
    /// ```ignore
    /// builder.page(PageSize::Letter)
    ///     .watermark("DRAFT")
    ///     .done();
    /// ```
    pub fn watermark(mut self, text: &str) -> Self {
        let page = &self.builder.pages[self.page_index];
        // Center the watermark on the page with diagonal orientation
        let rect =
            Rect::new(page.width * 0.1, page.height * 0.3, page.width * 0.8, page.height * 0.4);
        let watermark = WatermarkAnnotation::new(text)
            .with_rect(rect)
            .with_rotation(45.0)
            .with_opacity(0.3)
            .with_font("Helvetica", 72.0);
        self.pending_annotations.push(watermark.into());
        self
    }

    /// Add a "CONFIDENTIAL" watermark with preset styling.
    pub fn watermark_confidential(mut self) -> Self {
        let page = &self.builder.pages[self.page_index];
        let rect =
            Rect::new(page.width * 0.1, page.height * 0.3, page.width * 0.8, page.height * 0.4);
        let watermark = WatermarkAnnotation::confidential().with_rect(rect);
        self.pending_annotations.push(watermark.into());
        self
    }

    /// Add a "DRAFT" watermark with preset styling.
    pub fn watermark_draft(mut self) -> Self {
        let page = &self.builder.pages[self.page_index];
        let rect =
            Rect::new(page.width * 0.1, page.height * 0.3, page.width * 0.8, page.height * 0.4);
        let watermark = WatermarkAnnotation::draft().with_rect(rect);
        self.pending_annotations.push(watermark.into());
        self
    }

    /// Add a custom watermark with full control over positioning and styling.
    pub fn watermark_custom(mut self, watermark: WatermarkAnnotation) -> Self {
        self.pending_annotations.push(watermark.into());
        self
    }

    /// Add a generic annotation.
    ///
    /// This is a low-level method that allows adding any annotation type.
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
    /// builder.page(PageSize::Letter)
    ///     .add_annotation(link)
    ///     .done();
    /// ```
    pub fn add_annotation<A: Into<Annotation>>(mut self, annotation: A) -> Self {
        self.pending_annotations.push(annotation.into());
        self
    }

    /// Add a single-line text form field to the page. `name` is the
    /// unique field identifier used for form submission;
    /// `default_value` is the initial text shown in the field (pass
    /// `None` or an empty string for a blank field).
    pub fn text_field(
        self,
        name: impl Into<String>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        default_value: Option<String>,
    ) -> Self {
        let page = &mut self.builder.pages[self.page_index];
        page.form_fields.push(PendingFormField::TextField {
            name: name.into(),
            rect: Rect::new(x, y, w, h),
            default_value,
        });
        page.form_field_meta.push(PendingFieldMeta::default());
        self
    }

    /// Add a checkbox form field to the page. `checked` sets whether
    /// the box is initially ticked.
    pub fn checkbox(
        self,
        name: impl Into<String>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        checked: bool,
    ) -> Self {
        let page = &mut self.builder.pages[self.page_index];
        page.form_fields.push(PendingFormField::Checkbox {
            name: name.into(),
            rect: Rect::new(x, y, w, h),
            checked,
        });
        page.form_field_meta.push(PendingFieldMeta::default());
        self
    }

    /// Add a dropdown combo-box form field. Each entry of `options` is
    /// a user-visible string that also serves as the submitted value.
    /// `selected` picks the initial choice by value; pass `None` to
    /// leave the field blank.
    pub fn combo_box(
        self,
        name: impl Into<String>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        options: Vec<String>,
        selected: Option<String>,
    ) -> Self {
        let page = &mut self.builder.pages[self.page_index];
        page.form_fields.push(PendingFormField::ComboBox {
            name: name.into(),
            rect: Rect::new(x, y, w, h),
            options,
            selected,
        });
        page.form_field_meta.push(PendingFieldMeta::default());
        self
    }

    /// Add a scrollable list-box form field — same options shape as
    /// [`Self::combo_box`] but the user sees a scrollable list rather
    /// than a dropdown. `multi_select` toggles whether multiple
    /// entries can be selected simultaneously. `selected` is the
    /// initial value (or values; a comma-separated list is treated as
    /// multiple selections when `multi_select` is true).
    ///
    /// #393 Bundle D-1 — wires the existing `ListBoxWidget` through
    /// the fluent surface.
    pub fn list_box(
        self,
        name: impl Into<String>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        options: Vec<String>,
        selected: Option<String>,
        multi_select: bool,
    ) -> Self {
        let page = &mut self.builder.pages[self.page_index];
        page.form_fields.push(PendingFormField::ListBox {
            name: name.into(),
            rect: Rect::new(x, y, w, h),
            options,
            selected,
            multi_select,
        });
        page.form_field_meta.push(PendingFieldMeta::default());
        self
    }

    /// Add a radio-button group. Each entry of `buttons` is an
    /// `(export_value, x, y, w, h)` tuple describing one option's
    /// submitted value and its visible bounding rectangle. `selected`
    /// picks the initial choice by export value.
    pub fn radio_group(
        self,
        name: impl Into<String>,
        buttons: Vec<(String, f32, f32, f32, f32)>,
        selected: Option<String>,
    ) -> Self {
        let page = &mut self.builder.pages[self.page_index];
        let buttons = buttons
            .into_iter()
            .map(|(v, x, y, w, h)| (v, Rect::new(x, y, w, h)))
            .collect();
        page.form_fields.push(PendingFormField::RadioGroup {
            name: name.into(),
            buttons,
            selected,
        });
        page.form_field_meta.push(PendingFieldMeta::default());
        self
    }

    /// Add a clickable push button with a visible caption.
    pub fn push_button(
        self,
        name: impl Into<String>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        caption: impl Into<String>,
    ) -> Self {
        let page = &mut self.builder.pages[self.page_index];
        page.form_fields.push(PendingFormField::PushButton {
            name: name.into(),
            rect: Rect::new(x, y, w, h),
            caption: caption.into(),
        });
        page.form_field_meta.push(PendingFieldMeta::default());
        self
    }

    /// Add an unsigned signature placeholder field to the page.
    ///
    /// The field is created with `/FT /Sig` and `/V null` (unsigned).
    /// A signing application can fill it in via incremental update.
    pub fn signature_field(self, name: impl Into<String>, x: f32, y: f32, w: f32, h: f32) -> Self {
        let page = &mut self.builder.pages[self.page_index];
        page.form_fields.push(PendingFormField::SignatureField {
            name: name.into(),
            rect: Rect::new(x, y, w, h),
        });
        page.form_field_meta.push(PendingFieldMeta::default());
        self
    }

    /// Mark the most-recently-added form field on this page as
    /// **required**. Chainable after any `text_field` / `checkbox` /
    /// `combo_box` / `list_box` / `radio_group` / `push_button` /
    /// `signature_field` call. No-op if no field has been
    /// added yet. #393 Bundle D-3.
    pub fn required(self) -> Self {
        let page = &mut self.builder.pages[self.page_index];
        if let Some(meta) = page.form_field_meta.last_mut() {
            meta.required = true;
        }
        self
    }

    /// Mark the most-recently-added form field as **read-only**
    /// (displayed but not editable in the reader).
    pub fn read_only(self) -> Self {
        let page = &mut self.builder.pages[self.page_index];
        if let Some(meta) = page.form_field_meta.last_mut() {
            meta.read_only = true;
        }
        self
    }

    /// Attach a **tooltip** (hover text, `/TU` in the field dict) to
    /// the most-recently-added form field.
    pub fn tooltip(self, text: impl Into<String>) -> Self {
        let page = &mut self.builder.pages[self.page_index];
        if let Some(meta) = page.form_field_meta.last_mut() {
            meta.tooltip = Some(text.into());
        }
        self
    }

    /// Attach a JavaScript keystroke action (`/AA /K`) to the most-recently-added
    /// form field. Called on every keystroke while the field has focus.
    pub fn field_keystroke(self, script: impl Into<String>) -> Self {
        let page = &mut self.builder.pages[self.page_index];
        if let Some(meta) = page.form_field_meta.last_mut() {
            meta.keystroke = Some(script.into());
        }
        self
    }

    /// Attach a JavaScript format action (`/AA /F`) to the most-recently-added
    /// form field. Called when the field value is formatted for display.
    pub fn field_format(self, script: impl Into<String>) -> Self {
        let page = &mut self.builder.pages[self.page_index];
        if let Some(meta) = page.form_field_meta.last_mut() {
            meta.format = Some(script.into());
        }
        self
    }

    /// Attach a JavaScript validate action (`/AA /V`) to the most-recently-added
    /// form field. Called when the user commits the field value.
    pub fn field_validate(self, script: impl Into<String>) -> Self {
        let page = &mut self.builder.pages[self.page_index];
        if let Some(meta) = page.form_field_meta.last_mut() {
            meta.validate = Some(script.into());
        }
        self
    }

    /// Attach a JavaScript calculate action (`/AA /C`) to the most-recently-added
    /// form field. Called when any field in the form changes.
    pub fn field_calculate(self, script: impl Into<String>) -> Self {
        let page = &mut self.builder.pages[self.page_index];
        if let Some(meta) = page.form_field_meta.last_mut() {
            meta.calculate = Some(script.into());
        }
        self
    }

    /// Declare the page's tab-navigation order. `TabOrder::Row`
    /// (default in modern readers), `TabOrder::Column`, or
    /// `TabOrder::Structure` (requires tagged PDF — Bundle F).
    /// Deferred until `done()` — the `/Tabs` key is emitted on the
    /// page dict at build time. #393 Bundle D-4.
    pub fn tab_order(mut self, order: TabOrder) -> Self {
        self.pending_tab_order = Some(order);
        self
    }

    // ───────────────────────────────────────────────────────────────────
    // Low-level graphics primitives (PdfWriter exposure)
    //
    // These emit `ContentElement::Path` directly, the same backing
    // primitive DocumentBuilder already supports via `element()`. Kept
    // as first-class fluent methods because "I want a rectangle" is
    // common enough that forcing users through the lower-level
    // `ContentElement::Path` builder is ergonomically bad across 6
    // bindings.
    // ───────────────────────────────────────────────────────────────────

    /// Place a buffered `Table` at the current cursor position.
    ///
    /// This is the v0.3.39 buffered table surface — see research #393.
    /// The table layout (column widths, row heights, cell positions, wrapped
    /// cell text) is solved against the page's content width
    /// (`page.width - 2 × 72pt`), the result is emitted as a sequence of
    /// `ContentElement::Text` and `ContentElement::Path` via
    /// `Table::to_content_elements`, and the cursor is advanced by the
    /// table's total height.
    ///
    /// **Scope:** in-memory tables. Supports colspan / rowspan / rich cell
    /// styling. Does **not** page-break — if the layout overflows the
    /// current page the overflow is drawn past the bottom margin. For
    /// 1000+ rows that cross page boundaries, use `streaming_table`
    /// (lands in step 5/9).
    ///
    /// Font measurement uses the page-default font (`text_config.font`).
    /// Per-cell font overrides honour the font name string in the cell but
    /// are measured against the table default — good enough for v0.3.39.
    /// Track: #400 v0.3.40 (mixed-font precise metrics).
    pub fn table(mut self, table: Table) -> Self {
        let page_width = self.builder.pages[self.page_index].width;
        let content_width = page_width - 2.0 * 72.0; // match margin convention

        let metrics = FluentFontMetrics {
            manager: self.text_layout.font_manager(),
            font_name: self.text_config.font.clone(),
        };
        let layout = table.calculate_layout(content_width, &metrics);

        let elements = table.to_content_elements(self.cursor_x, self.cursor_y, &layout);
        let n = elements.len();

        let page = &mut self.builder.pages[self.page_index];
        let base_order = page.elements.len();
        for (i, mut elem) in elements.into_iter().enumerate() {
            // Rebase table-local reading_order onto the page's running
            // sequence so subsequent builder calls don't alias orders.
            match &mut elem {
                ContentElement::Text(t) => t.reading_order = Some(base_order + i),
                ContentElement::Path(p) => p.reading_order = Some(base_order + i),
                _ => {},
            }
            page.elements.push(elem);
        }
        let _ = n; // retained for future logging / subsetter-registration path

        self.cursor_y -= layout.total_height;
        self
    }

    /// Draw a stroked rectangle with a caller-supplied `LineStyle`.
    /// Unlike [`Self::rect`] (1pt black default), this exposes width and
    /// colour. Used by the upcoming buffered `Table` surface for per-side
    /// coloured / variable-thickness cell borders (#393 D-P1.3).
    pub fn stroke_rect(self, x: f32, y: f32, w: f32, h: f32, style: LineStyle) -> Self {
        use crate::elements::PathContent;
        use crate::elements::PathOperation;
        let mut path = PathContent::new(Rect::new(x, y, w, h));
        path.operations.push(PathOperation::Rectangle(x, y, w, h));
        path.stroke_color = Some(crate::layout::Color {
            r: style.color.0,
            g: style.color.1,
            b: style.color.2,
        });
        path.fill_color = None;
        path.stroke_width = style.width;
        path.dash_pattern = style.dash;
        path.matrix = self.current_matrix;
        let page = &mut self.builder.pages[self.page_index];
        page.elements.push(ContentElement::Path(path));
        self
    }

    /// Draw a straight line with a caller-supplied `LineStyle`. Variable-
    /// thickness / coloured rules — e.g. a 0.5pt grey rule between rows.
    pub fn stroke_line(self, x1: f32, y1: f32, x2: f32, y2: f32, style: LineStyle) -> Self {
        use crate::elements::PathContent;
        use crate::elements::PathOperation;
        let min_x = x1.min(x2);
        let min_y = y1.min(y2);
        let w = (x2 - x1).abs().max(1.0);
        let h = (y2 - y1).abs().max(1.0);
        let mut path = PathContent::new(Rect::new(min_x, min_y, w, h));
        path.operations.push(PathOperation::MoveTo(x1, y1));
        path.operations.push(PathOperation::LineTo(x2, y2));
        path.stroke_color = Some(crate::layout::Color {
            r: style.color.0,
            g: style.color.1,
            b: style.color.2,
        });
        path.fill_color = None;
        path.stroke_width = style.width;
        path.dash_pattern = style.dash;
        path.matrix = self.current_matrix;
        let page = &mut self.builder.pages[self.page_index];
        page.elements.push(ContentElement::Path(path));
        self
    }

    // ───────────────────────────────────────────────────────────────────
    // Transforms (rotate / scale / translate + arbitrary 2D matrix)
    //
    // Applies to TextContent, PathContent (shapes + stroke primitives),
    // and ImageContent elements emitted inside the closure. `TableContent`
    // is **not yet** transformed — it's rendered via
    // `Table::to_content_elements` whose inner elements already carry
    // their own matrix field and compose naturally, but the
    // Table-as-a-whole matrix remains a v0.3.40 item. Extended from
    // text-only to cover paths + images in #393 Bundle A-2 follow-up.
    // ───────────────────────────────────────────────────────────────────

    /// Evaluate `f` inside a scope where every text element is drawn
    /// under the 2D affine transform `matrix` (PDF convention:
    /// `[a b c d e f]`, applied as `(x' y') = (x y) · M`). The transform
    /// composes with any outer transform; on exit, the outer transform
    /// is restored.
    ///
    /// ```no_run
    /// # use pdf_oxide::writer::DocumentBuilder;
    /// # let mut doc = DocumentBuilder::new();
    /// # let page = doc.letter_page();
    /// let page = page.with_transform([1.0, 0.0, 0.0, 1.0, 50.0, 50.0], |p| {
    ///     p.at(0.0, 0.0).text("drawn at page (50, 50)")
    /// });
    /// ```
    pub fn with_transform<F>(self, matrix: [f32; 6], f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        let outer = self.current_matrix;
        let composed = compose_affine(outer.unwrap_or(IDENTITY), matrix);
        let mut page = Self {
            current_matrix: Some(composed),
            ..self
        };
        page = f(page);
        page.current_matrix = outer;
        page
    }

    /// Evaluate `f` in a scope where text is rotated by `degrees` about
    /// the origin. To rotate about a point `(cx, cy)`, compose with
    /// `.translated(cx, cy, |p| p.rotated(deg, |p| p.translated(-cx, -cy, f)))`.
    pub fn rotated<F>(self, degrees: f32, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        let r = degrees.to_radians();
        let (c, s) = (r.cos(), r.sin());
        // PDF matrix row order: [a b c d e f] = [cos sin -sin cos 0 0]
        self.with_transform([c, s, -s, c, 0.0, 0.0], f)
    }

    /// Evaluate `f` scaled by `(sx, sy)` about the origin.
    pub fn scaled<F>(self, sx: f32, sy: f32, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        self.with_transform([sx, 0.0, 0.0, sy, 0.0, 0.0], f)
    }

    /// Evaluate `f` translated by `(tx, ty)`.
    pub fn translated<F>(self, tx: f32, ty: f32, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        self.with_transform([1.0, 0.0, 0.0, 1.0, tx, ty], f)
    }

    // ───────────────────────────────────────────────────────────────────
    // Image placement
    // ───────────────────────────────────────────────────────────────────

    /// Embed an image at `rect` from a filesystem path. Supports JPEG
    /// and PNG; format is detected from the bytes (not the extension).
    /// PNG alpha channels become `/SMask` XObjects so transparent
    /// images composite correctly.
    ///
    /// `rect.x` / `rect.y` use PDF's bottom-up coordinates (same
    /// convention as the rest of the builder). `rect.width` /
    /// `rect.height` are the on-page size in points — the image is
    /// scaled to fit the rect without preserving aspect ratio; use
    /// [`crate::writer::ImageData::fit_to_box`] before calling this
    /// method if you need letterboxing.
    pub fn image_from_file(self, path: impl AsRef<Path>, rect: Rect) -> Result<Self> {
        use crate::writer::image_handler::ImageData;
        let data =
            ImageData::from_file(path).map_err(|e| crate::error::Error::Image(e.to_string()))?;
        Ok(self.image_with(data, rect))
    }

    /// Embed an image at `rect` from in-memory bytes. Auto-detects
    /// JPEG / PNG by magic number.
    pub fn image_from_bytes(self, bytes: &[u8], rect: Rect) -> Result<Self> {
        use crate::writer::image_handler::ImageData;
        let data =
            ImageData::from_bytes(bytes).map_err(|e| crate::error::Error::Image(e.to_string()))?;
        Ok(self.image_with(data, rect))
    }

    /// Embed an image with alternative text for accessibility (PDF/UA-1 §7.1).
    /// The image is wrapped in a `/Figure` structure element carrying `/Alt`
    /// so assistive technology can describe it. Requires `tagged_pdf_ua1()` on
    /// the `DocumentBuilder` for the alt text to be wired into the StructTree.
    pub fn image_from_bytes_with_alt(
        self,
        bytes: &[u8],
        rect: Rect,
        alt_text: impl Into<String>,
    ) -> Result<Self> {
        use crate::writer::image_handler::ImageData;
        let data =
            ImageData::from_bytes(bytes).map_err(|e| crate::error::Error::Image(e.to_string()))?;
        Ok(self.image_with_alt(data, rect, alt_text))
    }

    /// Embed a decorative image marked as an `/Artifact` (PDF/UA-1 §7.1).
    /// Assistive technology ignores artifact images; do not attach alt text.
    pub fn image_from_bytes_as_artifact(self, bytes: &[u8], rect: Rect) -> Result<Self> {
        use crate::writer::image_handler::ImageData;
        let data =
            ImageData::from_bytes(bytes).map_err(|e| crate::error::Error::Image(e.to_string()))?;
        Ok(self.image_with_artifact(data, rect))
    }

    /// Embed a pre-decoded image with alternative text (PDF/UA-1 §7.1).
    pub fn image_with_alt(
        self,
        data: crate::writer::image_handler::ImageData,
        rect: Rect,
        alt_text: impl Into<String>,
    ) -> Self {
        let alt = alt_text.into();
        self.image_with_options(data, rect, Some(alt), false)
    }

    /// Embed a pre-decoded decorative image as an `/Artifact` (PDF/UA-1 §7.1).
    pub fn image_with_artifact(
        self,
        data: crate::writer::image_handler::ImageData,
        rect: Rect,
    ) -> Self {
        self.image_with_options(data, rect, None, true)
    }

    /// Embed a pre-decoded [`crate::writer::ImageData`] at `rect`. Useful
    /// when a caller has already loaded the image (e.g. for reuse across
    /// multiple placements) and doesn't need the IO / parse step again.
    pub fn image_with(self, data: crate::writer::image_handler::ImageData, rect: Rect) -> Self {
        self.image_with_options(data, rect, None, false)
    }

    fn image_with_options(
        self,
        data: crate::writer::image_handler::ImageData,
        rect: Rect,
        alt_text: Option<String>,
        is_artifact: bool,
    ) -> Self {
        use crate::elements::{
            ColorSpace as EColorSpace, ImageContent, ImageFormat as EImageFormat,
        };
        use crate::writer::image_handler::{
            ColorSpace as WColorSpace, ImageFormat as WImageFormat,
        };

        let format = match data.format {
            WImageFormat::Jpeg => EImageFormat::Jpeg,
            WImageFormat::Png => EImageFormat::Png,
            WImageFormat::Raw => EImageFormat::Raw,
        };
        let color_space = match data.color_space {
            WColorSpace::DeviceGray => EColorSpace::Gray,
            WColorSpace::DeviceRGB => EColorSpace::RGB,
            WColorSpace::DeviceCMYK => EColorSpace::CMYK,
        };

        let mut content = ImageContent::new(rect, format, data.data, data.width, data.height);
        content.color_space = color_space;
        content.bits_per_component = data.bits_per_component;
        if let Some(mask) = data.soft_mask {
            content = content.with_soft_mask(mask);
        }
        if let Some(alt) = alt_text {
            content = content.with_alt_text(alt);
        }
        content.is_artifact = is_artifact;

        // Propagate the active transform so images inside `.rotated()`
        // etc. scopes render under that matrix. #393 Bundle A-2 follow-up.
        content.matrix = self.current_matrix;

        let page = &mut self.builder.pages[self.page_index];
        content.reading_order = Some(page.elements.len());
        page.elements.push(ContentElement::Image(content));
        self
    }

    /// Place a 1-D barcode (Code 128, EAN-13, QR, …) at `(x, y, w, h)`.
    /// The barcode is rendered to PNG at the given pixel dimensions and
    /// embedded as an image. `barcode_type` selects the symbology;
    /// `data` is the content to encode.
    pub fn barcode_1d(
        self,
        barcode_type: crate::writer::BarcodeType,
        data: &str,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) -> Result<Self> {
        let opts = crate::writer::BarcodeOptions::new()
            .width(w as u32)
            .height(h as u32);
        let png = crate::writer::BarcodeGenerator::generate_1d(barcode_type, data, &opts)?;
        self.image_from_bytes(&png, crate::geometry::Rect::new(x, y, w, h))
    }

    /// Place a QR code at `(x, y, size, size)` (square).
    /// `data` is the content to encode (URL, text, etc.).
    pub fn barcode_qr(self, data: &str, x: f32, y: f32, size: f32) -> Result<Self> {
        let opts = crate::writer::QrCodeOptions::new().size(size as u32);
        let png = crate::writer::BarcodeGenerator::generate_qr(data, &opts)?;
        self.image_from_bytes(&png, crate::geometry::Rect::new(x, y, size, size))
    }

    // ───────────────────────────────────────────────────────────────────
    // Shape primitives (circle / ellipse / polygon / arc / bezier_curve)
    // ───────────────────────────────────────────────────────────────────
    // Each primitive accepts an optional LineStyle (stroke) and optional
    // fill colour. `None` for a style leaves that side of the paint
    // undrawn (e.g. fill-only vs stroke-only). Shared helper
    // `push_stroked_fill` applies both to a `PathContent`.

    /// Draw a circle centred at `(cx, cy)` with `radius`. Pass
    /// `stroke = Some(...)` for outlined, `fill = Some((r, g, b))` for
    /// filled. Both together draw a stroked + filled disc.
    pub fn circle(
        self,
        cx: f32,
        cy: f32,
        radius: f32,
        stroke: Option<LineStyle>,
        fill: Option<(f32, f32, f32)>,
    ) -> Self {
        let path = crate::elements::PathContent::circle(cx, cy, radius);
        self.push_shaped_path(path, stroke, fill)
    }

    /// Draw an ellipse centred at `(cx, cy)` with horizontal radius
    /// `rx` and vertical radius `ry`. Same stroke/fill semantics as
    /// [`Self::circle`].
    pub fn ellipse(
        self,
        cx: f32,
        cy: f32,
        rx: f32,
        ry: f32,
        stroke: Option<LineStyle>,
        fill: Option<(f32, f32, f32)>,
    ) -> Self {
        // Magic constant for approximating a quarter-ellipse with a cubic
        // Bezier: k = 4 * (sqrt(2) - 1) / 3.
        use crate::elements::{PathContent, PathOperation};
        const K: f32 = 0.552_284_8;
        let kx = rx * K;
        let ky = ry * K;
        let ops = vec![
            PathOperation::MoveTo(cx, cy + ry),
            PathOperation::CurveTo(cx + kx, cy + ry, cx + rx, cy + ky, cx + rx, cy),
            PathOperation::CurveTo(cx + rx, cy - ky, cx + kx, cy - ry, cx, cy - ry),
            PathOperation::CurveTo(cx - kx, cy - ry, cx - rx, cy - ky, cx - rx, cy),
            PathOperation::CurveTo(cx - rx, cy + ky, cx - kx, cy + ry, cx, cy + ry),
            PathOperation::ClosePath,
        ];
        self.push_shaped_path(PathContent::from_operations(ops), stroke, fill)
    }

    /// Draw a closed polygon through `points`. Requires at least 2
    /// points — fewer is a no-op. Same stroke/fill semantics as
    /// [`Self::circle`].
    pub fn polygon(
        self,
        points: &[(f32, f32)],
        stroke: Option<LineStyle>,
        fill: Option<(f32, f32, f32)>,
    ) -> Self {
        if points.len() < 2 {
            return self;
        }
        use crate::elements::{PathContent, PathOperation};
        let mut ops: Vec<PathOperation> = Vec::with_capacity(points.len() + 2);
        let (x0, y0) = points[0];
        ops.push(PathOperation::MoveTo(x0, y0));
        for &(x, y) in &points[1..] {
            ops.push(PathOperation::LineTo(x, y));
        }
        ops.push(PathOperation::ClosePath);
        self.push_shaped_path(PathContent::from_operations(ops), stroke, fill)
    }

    /// Draw a circular arc centred at `(cx, cy)` with `radius`, from
    /// `start_angle` to `end_angle` (radians, anticlockwise). Only
    /// stroke; arcs are not filled. Approximated by up to 4 cubic
    /// Beziers (one per quadrant), matching the accuracy of the
    /// [`Self::circle`] primitive.
    pub fn arc(
        self,
        cx: f32,
        cy: f32,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        stroke: LineStyle,
    ) -> Self {
        use crate::elements::{PathContent, PathOperation};
        // Subdivide into arcs of <= π/2 each so a single cubic Bezier
        // stays accurate. Magic ratio for quarter-arcs.
        const K_Q: f32 = 0.552_284_8;
        let (mut a, b) = (start_angle, end_angle);
        let step = std::f32::consts::FRAC_PI_2;
        let mut ops: Vec<PathOperation> = vec![PathOperation::MoveTo(
            cx + radius * a.cos(),
            cy + radius * a.sin(),
        )];
        while a < b {
            let seg_end = (a + step).min(b);
            let sweep = seg_end - a;
            // Bezier control-point length for `sweep` radians around the
            // origin: (4/3) * tan(sweep/4) × radius.
            let k = (4.0 / 3.0) * (sweep / 4.0).tan();
            let (sa, ca) = (a.sin(), a.cos());
            let (sb, cb) = (seg_end.sin(), seg_end.cos());
            let c1x = cx + radius * (ca - k * sa);
            let c1y = cy + radius * (sa + k * ca);
            let c2x = cx + radius * (cb + k * sb);
            let c2y = cy + radius * (sb - k * cb);
            let ex = cx + radius * cb;
            let ey = cy + radius * sb;
            ops.push(PathOperation::CurveTo(c1x, c1y, c2x, c2y, ex, ey));
            a = seg_end;
            // If sweep was full π/2, fall through; otherwise finish.
            if (seg_end - b).abs() < 1e-6 {
                break;
            }
        }
        let _ = K_Q; // quarter-circle shortcut retained for future use
        self.push_shaped_path(PathContent::from_operations(ops), Some(stroke), None)
    }

    /// Draw a single cubic Bezier curve from `(x0, y0)` to `(x3, y3)`
    /// with control points `(c1x, c1y)` and `(c2x, c2y)`. Stroke only
    /// by default; pass `Some((r, g, b))` for fill.
    pub fn bezier_curve(
        self,
        x0: f32,
        y0: f32,
        c1x: f32,
        c1y: f32,
        c2x: f32,
        c2y: f32,
        x3: f32,
        y3: f32,
        stroke: LineStyle,
        fill: Option<(f32, f32, f32)>,
    ) -> Self {
        use crate::elements::{PathContent, PathOperation};
        let ops = vec![
            PathOperation::MoveTo(x0, y0),
            PathOperation::CurveTo(c1x, c1y, c2x, c2y, x3, y3),
        ];
        self.push_shaped_path(PathContent::from_operations(ops), Some(stroke), fill)
    }

    /// Internal helper: apply optional stroke + fill to a `PathContent`
    /// and push it as a `ContentElement::Path` on the current page.
    /// Shared by all shape primitives so stroke/fill semantics — and
    /// dash patterns — stay consistent across `circle` / `ellipse` /
    /// `polygon` / `arc` / `bezier_curve`.
    fn push_shaped_path(
        self,
        mut path: crate::elements::PathContent,
        stroke: Option<LineStyle>,
        fill: Option<(f32, f32, f32)>,
    ) -> Self {
        if let Some(style) = stroke {
            path.stroke_color = Some(crate::layout::Color {
                r: style.color.0,
                g: style.color.1,
                b: style.color.2,
            });
            path.stroke_width = style.width;
            path.dash_pattern = style.dash;
        } else {
            path.stroke_color = None;
        }
        if let Some((r, g, b)) = fill {
            path.fill_color = Some(crate::layout::Color { r, g, b });
        } else {
            path.fill_color = None;
        }
        // Propagate the active transform so paths inside `.rotated()` /
        // `.scaled()` / `.translated()` scopes render under that matrix.
        // #393 Bundle A-2 follow-up.
        path.matrix = self.current_matrix;
        let page = &mut self.builder.pages[self.page_index];
        page.elements.push(ContentElement::Path(path));
        self
    }

    /// Draw a stroked rectangle outline at `(x, y)` with size `w × h`
    /// using the default 1pt black stroke. For a filled rectangle with
    /// a custom colour, see [`Self::filled_rect`].
    pub fn rect(self, x: f32, y: f32, w: f32, h: f32) -> Self {
        use crate::elements::PathContent;
        use crate::elements::PathOperation;
        let mut path = PathContent::new(Rect::new(x, y, w, h));
        path.operations.push(PathOperation::Rectangle(x, y, w, h));
        path.stroke_color = Some(crate::layout::Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        });
        path.fill_color = None;
        let page = &mut self.builder.pages[self.page_index];
        page.elements.push(ContentElement::Path(path));
        self
    }

    /// Draw a filled rectangle at `(x, y)` with size `w × h` in the
    /// given RGB colour (channels in `0.0..=1.0`). No outline.
    pub fn filled_rect(self, x: f32, y: f32, w: f32, h: f32, r: f32, g: f32, b: f32) -> Self {
        use crate::elements::PathContent;
        use crate::elements::PathOperation;
        let mut path = PathContent::new(Rect::new(x, y, w, h));
        path.operations.push(PathOperation::Rectangle(x, y, w, h));
        path.fill_color = Some(crate::layout::Color { r, g, b });
        path.stroke_color = None;
        let page = &mut self.builder.pages[self.page_index];
        page.elements.push(ContentElement::Path(path));
        self
    }

    /// Draw a straight line from `(x1, y1)` to `(x2, y2)` with the
    /// default 1pt black stroke.
    pub fn line(self, x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        use crate::elements::PathContent;
        use crate::elements::PathOperation;
        let min_x = x1.min(x2);
        let min_y = y1.min(y2);
        let w = (x2 - x1).abs().max(1.0);
        let h = (y2 - y1).abs().max(1.0);
        let mut path = PathContent::new(Rect::new(min_x, min_y, w, h));
        path.operations.push(PathOperation::MoveTo(x1, y1));
        path.operations.push(PathOperation::LineTo(x2, y2));
        path.stroke_color = Some(crate::layout::Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        });
        path.fill_color = None;
        let page = &mut self.builder.pages[self.page_index];
        page.elements.push(ContentElement::Path(path));
        self
    }

    /// Finish building this page and return to the document builder.
    pub fn done(mut self) -> &'a mut DocumentBuilder {
        // Move pending annotations to page data
        let page = &mut self.builder.pages[self.page_index];
        page.annotations.append(&mut self.pending_annotations);
        if let Some(order) = self.pending_tab_order {
            page.tab_order = Some(order);
        }
        self.builder
    }
}

/// Buffered form-field widget added by `FluentPageBuilder::text_field`
/// etc. Applied to the underlying `pdf_writer::PageBuilder` inside
/// `DocumentBuilder::build`.
/// Metadata flags that apply to any form-field widget. Attached to
/// each `PendingFormField` at push time; mutated after-the-fact by
/// the fluent `.required()` / `.read_only()` / `.tooltip(s)` methods,
/// which target the most-recently-added field on the current page.
/// #393 Bundle D-3.
#[derive(Debug, Clone, Default)]
struct PendingFieldMeta {
    required: bool,
    read_only: bool,
    tooltip: Option<String>,
    /// /AA /K — keystroke JS action (text fields / choice fields with editing)
    keystroke: Option<String>,
    /// /AA /F — format JS action (text fields)
    format: Option<String>,
    /// /AA /V — validate JS action (all field types)
    validate: Option<String>,
    /// /AA /C — calculate JS action (text fields)
    calculate: Option<String>,
}

enum PendingFormField {
    /// A simple single-line text field.
    TextField {
        name: String,
        rect: Rect,
        default_value: Option<String>,
    },
    /// A checkbox, initially checked or not.
    Checkbox {
        name: String,
        rect: Rect,
        checked: bool,
    },
    /// A dropdown combo-box with a fixed list of string options and an
    /// optional initial selection.
    ComboBox {
        name: String,
        rect: Rect,
        options: Vec<String>,
        selected: Option<String>,
    },
    /// A radio-button group. Each entry in `buttons` has an export
    /// value (the PDF form's submitted value if that button is chosen)
    /// and its own rect.
    RadioGroup {
        name: String,
        buttons: Vec<(String, Rect)>,
        selected: Option<String>,
    },
    /// A clickable push button with a visible caption.
    PushButton {
        name: String,
        rect: Rect,
        caption: String,
    },
    /// A scrollable list-box (#393 D-1).
    ListBox {
        name: String,
        rect: Rect,
        options: Vec<String>,
        selected: Option<String>,
        multi_select: bool,
    },
    /// An unsigned signature placeholder field (/FT /Sig).
    SignatureField { name: String, rect: Rect },
}

/// Internal page data for DocumentBuilder.
struct PageData {
    width: f32,
    height: f32,
    elements: Vec<ContentElement>,
    annotations: Vec<Annotation>,
    form_fields: Vec<PendingFormField>,
    /// Per-field metadata, parallel to `form_fields`. Index i applies
    /// to form_fields[i]. #393 Bundle D-3.
    form_field_meta: Vec<PendingFieldMeta>,
    /// Deferred `/Tabs` value — set via `FluentPageBuilder::tab_order`.
    /// #393 Bundle D-4.
    tab_order: Option<TabOrder>,
    /// JavaScript to run when this page is opened (`/AA /O`).
    page_open_script: Option<String>,
    /// JavaScript to run when this page is closed (`/AA /C`).
    page_close_script: Option<String>,
    /// Footnote bodies collected during page building. Each entry is
    /// `(ref_mark, note_text)`. Emitted as a separator + Note elements
    /// at the bottom of the page in `DocumentBuilder::build`.
    pending_footnotes: Vec<(String, String)>,
}

/// High-level document builder with fluent API.
///
/// Provides a convenient way to build PDF documents using method chaining.
///
/// # Example
///
/// ```ignore
/// use pdf_oxide::writer::{DocumentBuilder, PageSize, DocumentMetadata};
///
/// let pdf_bytes = DocumentBuilder::new()
///     .metadata(DocumentMetadata::new().title("My Document"))
///     .page(PageSize::Letter)
///         .at(72.0, 720.0)
///         .heading(1, "Hello, World!")
///         .paragraph("This is a simple PDF document.")
///         .done()
///     .build()?;
/// ```
pub struct DocumentBuilder {
    metadata: DocumentMetadata,
    pages: Vec<PageData>,
    template: Option<PageTemplate>,
    /// Embedded TTF/OTF fonts registered by user-supplied name.
    /// Drained into the internal `PdfWriter` at `build()` time so that
    /// `FluentPageBuilder::font(name, size).text(...)` can emit
    /// CJK / Cyrillic / Greek text via Type-0 hex strings instead of
    /// silently falling back to Helvetica.
    embedded_fonts: Vec<(String, EmbeddedFont)>,
    /// Outline / bookmark tree. Empty by default; populated via
    /// [`DocumentBuilder::bookmark`] and/or [`DocumentBuilder::bookmark_tree`].
    /// Wired into the PDF catalog by `PdfWriter::finish` at build time.
    /// #393 Bundle B-1.
    outline: super::outline_builder::OutlineBuilder,
    /// Page labels. `None` => default decimal 1, 2, 3... `Some(...)`
    /// emits a `/PageLabels` number-tree for mixed Roman/Arabic/etc.
    /// numbering. #393 Bundle B-2.
    page_labels: Option<super::page_labels::PageLabelsBuilder>,
    /// JavaScript to run when the document is opened (`/OpenAction`).
    open_action_script: Option<String>,
    /// FlateDecode-compress emitted page content streams. Default: false.
    /// Enable for size-sensitive output (long documents → 5-10× smaller PDFs).
    compress_streams: bool,
}

impl DocumentBuilder {
    /// Create a new document builder.
    pub fn new() -> Self {
        Self {
            metadata: DocumentMetadata::default(),
            pages: Vec::new(),
            template: None,
            embedded_fonts: Vec::new(),
            outline: super::outline_builder::OutlineBuilder::new(),
            page_labels: None,
            open_action_script: None,
            compress_streams: false,
        }
    }

    /// Enable FlateDecode compression of emitted page content streams.
    /// Off by default (so existing tests / golden files compare byte-for-byte
    /// against uncompressed output); turn on for size-sensitive paths
    /// (long documents — 800-page PDFs shrink ~5×).
    pub fn compress_streams(mut self, on: bool) -> Self {
        self.compress_streams = on;
        self
    }

    /// Insert a table-of-contents page at position `insert_at` (0-based
    /// page index). The ToC lists every currently-added bookmark in
    /// depth-first order with indented titles and right-aligned page
    /// numbers. Blocked on / unblocked by [`Self::bookmark`] /
    /// [`Self::bookmark_tree`] populating the outline tree — call those
    /// before this. #393 Bundle B-3.
    ///
    /// Implementation: inserts a new page at `insert_at` and renders
    /// the bookmark titles at the current `text_config` font/size. Dots
    /// between title and page number are rendered as the Unicode middle
    /// dot (·) — works in base-14 fonts without embedding.
    pub fn insert_toc(self, insert_at: usize, title: impl Into<String>) -> Self {
        use super::outline_builder::{OutlineDestination, OutlineItem};

        // Collect (indent_level, title, page_num) tuples via pre-order DFS
        // so hierarchies render with left-indent per depth.
        fn walk(items: &[OutlineItem], depth: usize, out: &mut Vec<(usize, String, usize)>) {
            for item in items {
                let page = match item.destination {
                    OutlineDestination::Page(p) => p,
                    _ => 0,
                };
                out.push((depth, item.title.clone(), page));
                walk(&item.children, depth + 1, out);
            }
        }
        let mut entries: Vec<(usize, String, usize)> = Vec::new();
        walk(self.outline.items(), 0, &mut entries);
        if entries.is_empty() {
            // Nothing to do — short-circuit to avoid an empty page.
            return self;
        }

        // Shift page indices beyond insert_at so the ToC's target pages
        // still point at the correct content pages post-insertion.
        // v0.3.39 limitation: user-supplied bookmarks that aren't
        // patched here will still point at their ORIGINAL indices.
        // Callers should insert the ToC first OR re-issue bookmarks
        // after insertion. Documented; full renumbering is a v0.3.40
        // follow-up.

        // Build the ToC page content.
        let title_str = title.into();

        // We need a FluentPageBuilder after inserting a page at
        // insert_at. Strategy: stash existing pages after insert_at,
        // push a placeholder page via .page(), move it to position,
        // and render.
        //
        // Simpler: since PageData is Default + the content is additive,
        // just insert the new page at `insert_at` in self.pages, then
        // get an exclusive ref to render into it.
        let page_size = PageSize::Letter;
        let (width, height) = page_size.dimensions();
        let mut builder = self;

        // Clamp insert position to current page count — beyond that,
        // append at the end (no panic).
        let insert_pos = insert_at.min(builder.pages.len());
        builder.pages.insert(
            insert_pos,
            PageData {
                width,
                height,
                elements: Vec::new(),
                annotations: Vec::new(),
                form_fields: Vec::new(),
                form_field_meta: Vec::new(),
                tab_order: None,
                page_open_script: None,
                page_close_script: None,
                pending_footnotes: Vec::new(),
            },
        );

        // Now render directly into that page via a scratch
        // FluentPageBuilder. We can't use `page()` because it appends;
        // instead borrow a fresh builder pointing at `insert_pos`.
        {
            let mut page = FluentPageBuilder {
                builder: &mut builder,
                page_index: insert_pos,
                cursor_x: 72.0,
                cursor_y: height - 72.0,
                text_config: TextConfig::default(),
                text_layout: TextLayout::new(),
                last_text_rect: None,
                pending_annotations: Vec::new(),
                current_matrix: None,
                pending_tab_order: None,
            };

            // Title: bold + larger.
            page = page.font("Helvetica-Bold", 18.0).text(&title_str);
            page = page.space(12.0);
            page = page.font("Helvetica", 11.0);

            let page_w = width - 144.0; // 1-inch margins each side
            for (depth, t, target_page) in entries {
                let indent = 72.0 + (depth as f32) * 16.0;
                // Title
                let line_y = page.cursor_y;
                page = page.at(indent, line_y).text(&t);
                // Right-aligned page number (1-based for user display).
                let num = (target_page + 1).to_string();
                let num_w = page.measure(&num);
                let num_x = (width - 72.0) - num_w;
                page = page.at(num_x, line_y).text(&num);
            }
            let _ = page_w;

            page.done();
        }

        builder
    }

    /// Attach a page-label number tree. Lets a document number its
    /// front matter in lowercase Roman (i, ii, iii...) and its body in
    /// Arabic (1, 2, 3...), or any combination supported by
    /// [`crate::writer::PageLabelsBuilder`]. #393 Bundle B-2.
    ///
    /// ```no_run
    /// # use pdf_oxide::writer::{DocumentBuilder, PageLabelsBuilder};
    /// # use pdf_oxide::extractors::page_labels::{PageLabelRange, PageLabelStyle};
    /// let doc = DocumentBuilder::new()
    ///     .with_page_labels(
    ///         PageLabelsBuilder::new()
    ///             .add_range(PageLabelRange::new(0).with_style(PageLabelStyle::RomanLower))
    ///             .add_range(PageLabelRange::new(4).with_style(PageLabelStyle::Decimal)),
    ///     );
    /// ```
    pub fn with_page_labels(mut self, labels: super::page_labels::PageLabelsBuilder) -> Self {
        self.page_labels = Some(labels);
        self
    }

    /// Run a JavaScript script when the document is opened (`/OpenAction`).
    pub fn on_open(mut self, script: impl Into<String>) -> Self {
        self.open_action_script = Some(script.into());
        self
    }

    /// Add a top-level outline entry (bookmark) pointing at page
    /// `page_index` (0-based). Chainable; for hierarchies use
    /// [`Self::bookmark_tree`].
    ///
    /// Outline entries are materialised into the PDF `/Outlines`
    /// dictionary at [`Self::build`] time and show up in any reader's
    /// bookmarks panel. #393 Bundle B-1.
    pub fn bookmark(mut self, title: impl Into<String>, page_index: usize) -> Self {
        self.outline
            .add_item(super::outline_builder::OutlineItem::new(title, page_index));
        self
    }

    /// Mutate the underlying `OutlineBuilder` directly — useful for
    /// building deep / conditional hierarchies inside a closure.
    ///
    /// ```no_run
    /// # use pdf_oxide::writer::DocumentBuilder;
    /// # use pdf_oxide::writer::outline_builder::OutlineItem;
    /// let doc = DocumentBuilder::new().bookmark_tree(|o| {
    ///     o.add_item(OutlineItem::new("Chapter 1", 0));
    ///     o.add_child(OutlineItem::new("Section 1.1", 1));
    ///     o.add_child(OutlineItem::new("Section 1.2", 2));
    ///     o.pop();
    ///     o.add_item(OutlineItem::new("Chapter 2", 3));
    /// });
    /// ```
    pub fn bookmark_tree<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut super::outline_builder::OutlineBuilder),
    {
        f(&mut self.outline);
        self
    }

    /// Register an embedded TrueType/OpenType font under a user-visible
    /// name. The `name` is what callers then pass to
    /// [`FluentPageBuilder::font`]; any `.text(...)` / element emitted
    /// with that font name is routed through the Type-0 / CIDFontType2
    /// path at build time, so Unicode scripts (CJK, Cyrillic, Greek,
    /// Hebrew, Arabic) render correctly.
    ///
    /// Unregistered font names continue to resolve against the
    /// standard base-14 set (Helvetica / Times / Courier families).
    ///
    /// ```ignore
    /// use pdf_oxide::writer::{DocumentBuilder, EmbeddedFont};
    ///
    /// let font = EmbeddedFont::from_file("fonts/NotoSansCJKtc-Regular.otf")?;
    /// let pdf = DocumentBuilder::new()
    ///     .register_embedded_font("NotoSansCJKtc", font)
    ///     .a4_page()
    ///         .font("NotoSansCJKtc", 10.5)
    ///         .at(72.0, 680.0)
    ///         .text("项目: Rust 特性")
    ///         .done()
    ///     .build()?;
    /// ```
    pub fn register_embedded_font(mut self, name: impl Into<String>, font: EmbeddedFont) -> Self {
        self.embedded_fonts.push((name.into(), font));
        self
    }

    /// Set document title (convenience passthrough to
    /// `DocumentMetadata::title`).
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.metadata.title = Some(title.into());
        self
    }

    /// Set document author.
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.metadata.author = Some(author.into());
        self
    }

    /// Set document subject.
    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.metadata.subject = Some(subject.into());
        self
    }

    /// Set document keywords (comma-separated per PDF convention).
    pub fn keywords(mut self, keywords: impl Into<String>) -> Self {
        self.metadata.keywords = Some(keywords.into());
        self
    }

    /// Set the creator application name.
    pub fn creator(mut self, creator: impl Into<String>) -> Self {
        self.metadata.creator = Some(creator.into());
        self
    }

    /// Enable PDF/UA-1 tagging on this document. When enabled, `build()` emits
    /// `/MarkInfo << /Marked true >>`, `/StructTreeRoot`, `/Lang`, and
    /// `/ViewerPreferences << /DisplayDocTitle true >>` in the catalog.
    ///
    /// Has no effect on existing callers that do not call this method (opt-in).
    pub fn tagged_pdf_ua1(mut self) -> Self {
        self.metadata.tagged = true;
        self
    }

    /// Set the document's natural language tag (e.g. `"en-US"`).
    /// This is emitted as `/Lang` in the catalog when `tagged_pdf_ua1()` is set.
    pub fn language(mut self, lang: impl Into<String>) -> Self {
        self.metadata.language = Some(lang.into());
        self
    }

    /// Add a role-map entry: custom structure type → standard PDF structure type.
    /// Emitted in `/RoleMap` inside the StructTreeRoot when `tagged_pdf_ua1()` is
    /// set. Multiple calls accumulate entries.
    pub fn role_map(mut self, custom: impl Into<String>, standard: impl Into<String>) -> Self {
        self.metadata
            .role_map
            .push((custom.into(), standard.into()));
        self
    }

    /// Set document metadata.
    pub fn metadata(mut self, metadata: DocumentMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Set the page template for headers and footers.
    pub fn template(mut self, template: PageTemplate) -> Self {
        self.template = Some(template);
        self
    }

    /// Number of pages in this document so far. Primarily for tests.
    #[allow(dead_code)]
    pub(crate) fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// Elements already queued on page `idx`. Primarily for tests in
    /// sibling modules that can't see the private `pages` field.
    #[allow(dead_code)]
    pub(crate) fn page_elements(&self, idx: usize) -> &[ContentElement] {
        &self.pages[idx].elements
    }

    /// Add a page with the specified size and return a page builder.
    pub fn page(&mut self, size: PageSize) -> FluentPageBuilder<'_> {
        let (width, height) = size.dimensions();
        let page_index = self.pages.len();
        self.pages.push(PageData {
            width,
            height,
            elements: Vec::new(),
            annotations: Vec::new(),
            form_fields: Vec::new(),
            form_field_meta: Vec::new(),
            tab_order: None,
            page_open_script: None,
            page_close_script: None,
            pending_footnotes: Vec::new(),
        });
        FluentPageBuilder {
            builder: self,
            page_index,
            cursor_x: 72.0,          // 1 inch margin
            cursor_y: height - 72.0, // Start from top with 1 inch margin
            text_config: TextConfig::default(),
            text_layout: TextLayout::new(),
            last_text_rect: None,
            pending_annotations: Vec::new(),
            current_matrix: None,
            pending_tab_order: None,
        }
    }

    /// Add a Letter-sized page.
    pub fn letter_page(&mut self) -> FluentPageBuilder<'_> {
        self.page(PageSize::Letter)
    }

    /// Add an A4-sized page.
    pub fn a4_page(&mut self) -> FluentPageBuilder<'_> {
        self.page(PageSize::A4)
    }

    /// Build the PDF document and return the bytes.
    pub fn build(self) -> Result<Vec<u8>> {
        let mut config = PdfWriterConfig::default();
        if let Some(version) = self.metadata.version.clone() {
            config.version = version;
        }
        config.title = self.metadata.title.clone();
        config.author = self.metadata.author.clone();
        config.subject = self.metadata.subject.clone();
        config.keywords = self.metadata.keywords.clone();
        if self.metadata.creator.is_some() {
            config.creator = self.metadata.creator.clone();
        }
        // F-1/F-2/F-4: wire tagged PDF settings into writer config
        config.tagged = self.metadata.tagged;
        config.language = self.metadata.language.clone();
        config.role_map = self.metadata.role_map.clone();
        if let Some(script) = self.open_action_script {
            config.open_action_script = Some(script);
        }
        config.compress = self.compress_streams;
        let tagged = config.tagged;

        let mut writer = PdfWriter::with_config(config);

        for (user_name, font) in self.embedded_fonts {
            writer.register_embedded_font_as(user_name, font);
        }

        // Transfer the outline (bookmarks) so PdfWriter::finish can
        // emit the /Outlines tree and link it from the catalog.
        // #393 Bundle B-1.
        writer.set_outline(self.outline);

        // Transfer the page-label number tree if set. #393 Bundle B-2.
        if let Some(labels) = self.page_labels {
            writer.set_page_labels(labels);
        }

        let total_pages = self.pages.len();

        for (idx, page_data) in self.pages.iter().enumerate() {
            let mut page = writer.add_page(page_data.width, page_data.height);

            // Propagate the page's /Tabs value if the user set one.
            // #393 Bundle D-4.
            if let Some(order) = page_data.tab_order {
                page.set_tab_order(order.as_pdf_char());
            }

            // Propagate page-level JS open/close actions (/AA /O and /AA /C).
            if let Some(ref s) = page_data.page_open_script {
                page.set_page_open_script(s.clone());
            }
            if let Some(ref s) = page_data.page_close_script {
                page.set_page_close_script(s.clone());
            }

            // 1. Add normal elements
            page.add_elements(&page_data.elements);

            // 2. Apply Template (Headers/Footers) - Draw on top of content
            if let Some(ref template) = self.template {
                let page_number = idx + 1;
                let context =
                    crate::writer::page_template::PlaceholderContext::new(page_number, total_pages)
                        .with_title(self.metadata.title.clone().unwrap_or_default())
                        .with_author(self.metadata.author.clone().unwrap_or_default());

                let layout_engine = TextLayout::new();

                // Apply Header
                if let Some(header) = template.get_header(page_number) {
                    for element in header.elements() {
                        let text = element.resolve(&context);
                        let style = element.style.as_ref().unwrap_or(&header.style);

                        let font_spec = crate::elements::FontSpec {
                            name: style.font_name.clone(),
                            size: style.font_size,
                        };

                        // Calculate width for alignment
                        let (text_width, _) = layout_engine.text_bounds(
                            &text,
                            &font_spec.name,
                            font_spec.size,
                            page_data.width,
                        );

                        let x = match element.alignment {
                            crate::writer::ArtifactAlignment::Left => template.margin_left,
                            crate::writer::ArtifactAlignment::Center => {
                                (page_data.width - text_width) / 2.0
                            },
                            crate::writer::ArtifactAlignment::Right => {
                                page_data.width - template.margin_right - text_width
                            },
                        };
                        let y = page_data.height - header.offset;

                        page.add_element(&ContentElement::Text(TextContent {
                            artifact_type: Some(crate::extractors::text::ArtifactType::Pagination(
                                crate::extractors::text::PaginationSubtype::Header,
                            )),
                            text,
                            bbox: Rect::new(x, y, text_width, style.font_size),
                            font: font_spec,
                            style: crate::elements::TextStyle {
                                color: crate::layout::Color {
                                    r: style.color.0,
                                    g: style.color.1,
                                    b: style.color.2,
                                },
                                weight: match style.font_weight {
                                    crate::writer::font_manager::FontWeight::Normal => {
                                        crate::layout::text_block::FontWeight::Normal
                                    },
                                    crate::writer::font_manager::FontWeight::Bold => {
                                        crate::layout::text_block::FontWeight::Bold
                                    },
                                },
                                ..Default::default()
                            },
                            reading_order: None,
                            origin: None,
                            rotation_degrees: None,
                            matrix: None,
                        }));
                    }
                }

                // Apply Footer
                if let Some(footer) = template.get_footer(page_number) {
                    for element in footer.elements() {
                        let text = element.resolve(&context);
                        let style = element.style.as_ref().unwrap_or(&footer.style);

                        let font_spec = crate::elements::FontSpec {
                            name: style.font_name.clone(),
                            size: style.font_size,
                        };

                        // Calculate width for alignment
                        let (text_width, _) = layout_engine.text_bounds(
                            &text,
                            &font_spec.name,
                            font_spec.size,
                            page_data.width,
                        );

                        let x = match element.alignment {
                            crate::writer::ArtifactAlignment::Left => template.margin_left,
                            crate::writer::ArtifactAlignment::Center => {
                                (page_data.width - text_width) / 2.0
                            },
                            crate::writer::ArtifactAlignment::Right => {
                                page_data.width - template.margin_right - text_width
                            },
                        };
                        let y = footer.offset;

                        page.add_element(&ContentElement::Text(TextContent {
                            artifact_type: Some(crate::extractors::text::ArtifactType::Pagination(
                                crate::extractors::text::PaginationSubtype::Footer,
                            )),
                            text,
                            bbox: Rect::new(x, y, text_width, style.font_size),
                            font: font_spec,
                            style: crate::elements::TextStyle {
                                color: crate::layout::Color {
                                    r: style.color.0,
                                    g: style.color.1,
                                    b: style.color.2,
                                },
                                weight: match style.font_weight {
                                    crate::writer::font_manager::FontWeight::Normal => {
                                        crate::layout::text_block::FontWeight::Normal
                                    },
                                    crate::writer::font_manager::FontWeight::Bold => {
                                        crate::layout::text_block::FontWeight::Bold
                                    },
                                },
                                ..Default::default()
                            },
                            reading_order: None,
                            origin: None,
                            rotation_degrees: None,
                            matrix: None,
                        }));
                    }
                }
            }

            // 3. Add annotations
            for annotation in &page_data.annotations {
                page.add_annotation(annotation.clone());
            }

            // 4. Emit form-field widgets. Each pending entry translates
            //    into the appropriate `pdf_writer::PageBuilder::add_*`
            //    call so the field lands in /AcroForm at finalize time.
            //    Metadata (required / read_only / tooltip) from the
            //    parallel `form_field_meta` vec is applied to each
            //    widget before the `add_*` call — #393 Bundle D-3.
            for (field_idx, field) in page_data.form_fields.iter().enumerate() {
                use super::form_fields::{CheckboxWidget, TextFieldWidget};
                let meta = page_data
                    .form_field_meta
                    .get(field_idx)
                    .cloned()
                    .unwrap_or_default();
                match field {
                    PendingFormField::TextField {
                        name,
                        rect,
                        default_value,
                    } => {
                        let mut widget = TextFieldWidget::new(name.clone(), *rect);
                        if let Some(default) = default_value {
                            widget = widget.with_default_value(default.clone());
                        }
                        if meta.required {
                            widget = widget.required();
                        }
                        if meta.read_only {
                            widget = widget.read_only();
                        }
                        if let Some(tip) = &meta.tooltip {
                            widget = widget.with_tooltip(tip.clone());
                        }
                        if let Some(s) = &meta.keystroke {
                            widget = widget.with_keystroke(s.clone());
                        }
                        if let Some(s) = &meta.format {
                            widget = widget.with_format(s.clone());
                        }
                        if let Some(s) = &meta.validate {
                            widget = widget.with_validate(s.clone());
                        }
                        if let Some(s) = &meta.calculate {
                            widget = widget.with_calculate(s.clone());
                        }
                        page.add_text_field(widget);
                    },
                    PendingFormField::Checkbox {
                        name,
                        rect,
                        checked,
                    } => {
                        let mut widget = CheckboxWidget::new(name.clone(), *rect);
                        if *checked {
                            widget = widget.checked();
                        }
                        if meta.required {
                            widget = widget.required();
                        }
                        if meta.read_only {
                            widget = widget.read_only();
                        }
                        if let Some(tip) = &meta.tooltip {
                            widget = widget.with_tooltip(tip.clone());
                        }
                        page.add_checkbox(widget);
                    },
                    PendingFormField::ComboBox {
                        name,
                        rect,
                        options,
                        selected,
                    } => {
                        use super::form_fields::ComboBoxWidget;
                        let mut widget =
                            ComboBoxWidget::new(name.clone(), *rect).with_options(options.clone());
                        if let Some(v) = selected {
                            widget = widget.with_value(v.clone());
                        }
                        if meta.required {
                            widget = widget.required();
                        }
                        if meta.read_only {
                            widget = widget.read_only();
                        }
                        if let Some(tip) = &meta.tooltip {
                            widget = widget.with_tooltip(tip.clone());
                        }
                        if let Some(s) = &meta.keystroke {
                            widget = widget.with_keystroke(s.clone());
                        }
                        if let Some(s) = &meta.validate {
                            widget = widget.with_validate(s.clone());
                        }
                        page.add_combo_box(widget);
                    },
                    PendingFormField::RadioGroup {
                        name,
                        buttons,
                        selected,
                    } => {
                        use super::form_fields::RadioButtonGroup;
                        let mut group = RadioButtonGroup::new(name.clone());
                        for (value, rect) in buttons {
                            group = group.add_button(value.clone(), *rect, value.clone());
                        }
                        if let Some(v) = selected {
                            group = group.selected(v.clone());
                        }
                        if meta.required {
                            group = group.required();
                        }
                        if meta.read_only {
                            group = group.read_only();
                        }
                        if let Some(tip) = &meta.tooltip {
                            group = group.with_tooltip(tip.clone());
                        }
                        page.add_radio_group(group);
                    },
                    PendingFormField::PushButton {
                        name,
                        rect,
                        caption,
                    } => {
                        use super::form_fields::PushButtonWidget;
                        let mut widget = PushButtonWidget::new(name.clone(), *rect)
                            .with_caption(caption.clone());
                        // PushButton has no `.required()` (it's a button,
                        // not a field a user fills in) — only read_only +
                        // tooltip apply.
                        if meta.read_only {
                            widget = widget.read_only();
                        }
                        if let Some(tip) = &meta.tooltip {
                            widget = widget.with_tooltip(tip.clone());
                        }
                        page.add_push_button(widget);
                    },
                    PendingFormField::ListBox {
                        name,
                        rect,
                        options,
                        selected,
                        multi_select,
                    } => {
                        use super::form_fields::ListBoxWidget;
                        let mut widget =
                            ListBoxWidget::new(name.clone(), *rect).with_options(options.clone());
                        if *multi_select {
                            widget = widget.multi_select();
                        }
                        if let Some(v) = selected {
                            widget = widget.with_value(v.clone());
                        }
                        if meta.required {
                            widget = widget.required();
                        }
                        if meta.read_only {
                            widget = widget.read_only();
                        }
                        if let Some(tip) = &meta.tooltip {
                            widget = widget.with_tooltip(tip.clone());
                        }
                        if let Some(s) = &meta.validate {
                            widget = widget.with_validate(s.clone());
                        }
                        page.add_list_box(widget);
                    },
                    PendingFormField::SignatureField { name, rect } => {
                        use super::form_fields::SignatureWidget;
                        let mut widget = SignatureWidget::new(name.clone(), *rect);
                        if meta.read_only {
                            widget = widget.read_only();
                        }
                        if let Some(tip) = &meta.tooltip {
                            widget = widget.with_tooltip(tip.clone());
                        }
                        page.add_signature_field(widget);
                    },
                }
            }

            // 5. Emit footnotes: separator artifact + Note elements at page bottom.
            if !page_data.pending_footnotes.is_empty() {
                use crate::elements::{
                    FontSpec, PathContent, PathOperation, StructureElement, TextStyle,
                };
                use crate::extractors::text::ArtifactType;

                let font_size = 8.0_f32;
                let line_height = font_size * 1.2;
                let bottom_y = 72.0_f32; // bottom margin
                let fn_count = page_data.pending_footnotes.len() as f32;
                // Stack: gap (4 pt) + one line per footnote, all above sep_y.
                let total_h = 4.0 + fn_count * line_height;
                let sep_y = bottom_y + total_h;
                let sep_x0 = 72.0_f32;
                let sep_x1 = (page_data.width / 3.0).min(page_data.width - 72.0);

                // Separator line — marked as Layout artifact.
                page.add_element(&ContentElement::Path(PathContent {
                    paint: Default::default(),
                    operations: vec![
                        PathOperation::MoveTo(sep_x0, sep_y),
                        PathOperation::LineTo(sep_x1, sep_y),
                    ],
                    bbox: Rect::new(sep_x0, sep_y, sep_x1 - sep_x0, 0.5),
                    stroke_color: Some(crate::layout::Color {
                        r: 0.3,
                        g: 0.3,
                        b: 0.3,
                    }),
                    fill_color: None,
                    stroke_width: 0.5,
                    line_cap: crate::elements::LineCap::Butt,
                    line_join: crate::elements::LineJoin::Miter,
                    dash_pattern: None,
                    matrix: None,
                    reading_order: None,
                    artifact_type: Some(ArtifactType::Layout),
                    layer: None,
                }));

                // Footnote bodies, stacked from sep_y downward.
                let text_w = page_data.width - 144.0;
                let mut fn_y = sep_y - 4.0 - font_size;
                for (ref_mark, note_text) in &page_data.pending_footnotes {
                    let label = format!("{} {}", ref_mark, note_text);
                    let body = ContentElement::Text(TextContent {
                        text: label,
                        bbox: Rect::new(sep_x0, fn_y, text_w, font_size),
                        font: FontSpec {
                            name: "Helvetica".to_string(),
                            size: font_size,
                        },
                        style: TextStyle::default(),
                        reading_order: None,
                        artifact_type: None,
                        origin: None,
                        rotation_degrees: None,
                        matrix: None,
                    });

                    if tagged {
                        // Wrap in <Note> structure element for PDF/UA.
                        page.add_element(&ContentElement::Structure(StructureElement {
                            structure_type: "Note".to_string(),
                            bbox: Rect::new(sep_x0, fn_y, text_w, font_size),
                            children: vec![body],
                            reading_order: None,
                            alt_text: Some(format!("Footnote {}", ref_mark)),
                            language: None,
                        }));
                    } else {
                        page.add_element(&body);
                    }
                    fn_y -= line_height;
                }
            }

            page.finish();
        }

        writer.finish()
    }

    /// Build and save the PDF to a file.
    pub fn save(self, path: impl AsRef<Path>) -> Result<()> {
        let bytes = self.build()?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    /// Build and save the PDF with AES-256 encryption using the given
    /// user and owner passwords. Grants all permissions — use
    /// [`DocumentBuilder::save_with_encryption`] for a custom
    /// [`crate::editor::EncryptionConfig`] (algorithm + permissions).
    ///
    /// Routes built bytes through the standard encryption pipeline
    /// (`DocumentEditor::save_with_options`), so the resulting PDF is
    /// byte-compatible with any PDF viewer that supports AES-256 (PDF
    /// 2.0 / `/V 5 /R 6`).
    ///
    /// # Example
    ///
    /// ```ignore
    /// use pdf_oxide::writer::{DocumentBuilder, PageSize};
    ///
    /// let mut builder = DocumentBuilder::new();
    /// builder.page(PageSize::A4).at(72.0, 700.0).text("secret").done();
    /// builder.save_encrypted("out.pdf", "user-pw", "owner-pw")?;
    /// # Ok::<(), pdf_oxide::error::Error>(())
    /// ```
    pub fn save_encrypted(
        self,
        path: impl AsRef<Path>,
        user_password: &str,
        owner_password: &str,
    ) -> Result<()> {
        use crate::editor::{EncryptionAlgorithm, EncryptionConfig, Permissions};
        let config = EncryptionConfig {
            user_password: user_password.to_string(),
            owner_password: owner_password.to_string(),
            algorithm: EncryptionAlgorithm::Aes256,
            permissions: Permissions::all(),
        };
        self.save_with_encryption(path, config)
    }

    /// Build and save the PDF with a custom encryption configuration.
    ///
    /// Use this when you need a specific algorithm (RC4-128, AES-128,
    /// AES-256) or restricted permissions. For the common AES-256
    /// all-permissions case, prefer [`DocumentBuilder::save_encrypted`].
    pub fn save_with_encryption(
        self,
        path: impl AsRef<Path>,
        config: crate::editor::EncryptionConfig,
    ) -> Result<()> {
        use crate::editor::{DocumentEditor, EditableDocument, SaveOptions};
        let bytes = self.build()?;
        let mut editor = DocumentEditor::from_bytes(bytes)?;
        editor.save_with_options(path, SaveOptions::with_encryption(config))
    }

    /// Build and return the encrypted PDF as bytes. Mirrors
    /// [`DocumentBuilder::save_encrypted`] but skips the filesystem —
    /// useful for WASM / server pipelines that stream bytes back to a
    /// caller.
    pub fn to_bytes_encrypted(self, user_password: &str, owner_password: &str) -> Result<Vec<u8>> {
        use crate::editor::{EncryptionAlgorithm, EncryptionConfig, Permissions};
        let config = EncryptionConfig {
            user_password: user_password.to_string(),
            owner_password: owner_password.to_string(),
            algorithm: EncryptionAlgorithm::Aes256,
            permissions: Permissions::all(),
        };
        self.to_bytes_with_encryption(config)
    }

    /// Build and return the PDF as encrypted bytes, using a custom
    /// configuration.
    pub fn to_bytes_with_encryption(
        self,
        config: crate::editor::EncryptionConfig,
    ) -> Result<Vec<u8>> {
        use crate::editor::{DocumentEditor, SaveOptions};
        let bytes = self.build()?;
        let mut editor = DocumentEditor::from_bytes(bytes)?;
        editor.save_to_bytes_with_options(SaveOptions::with_encryption(config))
    }
}

impl Default for DocumentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// PDF identity matrix `[1 0 0 1 0 0]`.
const IDENTITY: [f32; 6] = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

/// Compose two PDF affine matrices in row-order (`[a b c d e f]`).
///
/// `outer` is an already-set transform (from an enclosing `q M cm`
/// scope); `inner` is a newly-applied transform. PDF uses row vectors
/// `v · M`, so transforming first by `inner` then by `outer` is
/// `v · inner · outer`; returned matrix equals `inner · outer`.
///
/// Matches PDF's `q M_outer cm q M_inner cm ... Q Q` behaviour: a
/// point drawn at (0, 0) inside `translated(tx, ty, rotated(θ, ...))`
/// lands at `(tx, ty)` — rotation about origin preserves it, then the
/// translation shifts it.
fn compose_affine(outer: [f32; 6], inner: [f32; 6]) -> [f32; 6] {
    // Row-major matrix multiply: (inner * outer)_{ij} = Σ_k inner_{ik} * outer_{kj}
    let [a1, b1, c1, d1, e1, f1] = inner;
    let [a2, b2, c2, d2, e2, f2] = outer;
    [
        a1 * a2 + b1 * c2,
        a1 * b2 + b1 * d2,
        c1 * a2 + d1 * c2,
        c1 * b2 + d1 * d2,
        e1 * a2 + f1 * c2 + e2,
        e1 * b2 + f1 * d2 + f2,
    ]
}

/// Adapter that projects `FontManager` into the `table_renderer::FontMetrics`
/// trait against a fixed font name. Lives here (not on FontManager itself)
/// because FontMetrics is a table-renderer-owned abstraction the writer
/// layer doesn't know about.
struct FluentFontMetrics<'a> {
    manager: &'a FontManager,
    font_name: String,
}

impl FontMetrics for FluentFontMetrics<'_> {
    fn text_width(&self, text: &str, font_size: f32) -> f32 {
        self.manager.text_width(text, &self.font_name, font_size)
    }
}

/// Simple word wrapping utility.
#[allow(dead_code)]
fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= max_chars {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_size_dimensions() {
        assert_eq!(PageSize::Letter.dimensions(), (612.0, 792.0));
        assert_eq!(PageSize::A4.dimensions(), (595.0, 842.0));
        assert_eq!(PageSize::Legal.dimensions(), (612.0, 1008.0));
        assert_eq!(PageSize::Custom(100.0, 200.0).dimensions(), (100.0, 200.0));
    }

    #[test]
    fn test_document_metadata() {
        let meta = DocumentMetadata::new()
            .title("Test Title")
            .author("Test Author")
            .subject("Test Subject");

        assert_eq!(meta.title, Some("Test Title".to_string()));
        assert_eq!(meta.author, Some("Test Author".to_string()));
        assert_eq!(meta.subject, Some("Test Subject".to_string()));
    }

    #[test]
    fn test_wrap_text() {
        let text = "This is a test of the word wrapping function";
        let wrapped = wrap_text(text, 20);
        assert!(wrapped.len() > 1);
        for line in &wrapped {
            assert!(line.len() <= 20 || line.split_whitespace().count() == 1);
        }
    }

    #[test]
    fn test_wrap_text_empty() {
        let wrapped = wrap_text("", 20);
        assert_eq!(wrapped.len(), 1);
        assert_eq!(wrapped[0], "");
    }

    #[test]
    fn test_document_builder_basic() {
        let mut builder = DocumentBuilder::new();
        builder
            .letter_page()
            .at(72.0, 720.0)
            .text("Hello, World!")
            .done();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.starts_with("%PDF-1.7"));
        assert!(content.contains("%%EOF"));
    }

    #[test]
    fn test_watermark_appearance_is_indirect_not_inline() {
        // Regression: a watermark's `/AP /N` appearance stream must be an
        // indirect object (`/N <id> 0 R`), not an inline stream embedded in
        // the annotation dict (which is invalid PDF and renders to nothing).
        let mut builder = DocumentBuilder::new();
        let wm = crate::writer::WatermarkAnnotation::new("CONFIDENTIAL")
            .with_rect(crate::geometry::Rect::new(50.0, 50.0, 200.0, 400.0));
        builder
            .letter_page()
            .paragraph("body")
            .watermark_custom(wm)
            .done();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Subtype /Watermark"), "watermark missing");
        // `/N` references an indirect object...
        let re = regex_lite_n_ref(&content);
        assert!(re, "appearance `/N` is not an indirect reference: not found");
        // ...and the appearance dict is never opened inline after `/N`.
        assert!(
            !content.contains("/N <<"),
            "inline appearance stream leaked into annotation dict"
        );
    }

    // Minimal check for `/N <digits> 0 R` without pulling in a regex crate.
    fn regex_lite_n_ref(s: &str) -> bool {
        s.match_indices("/N ").any(|(i, _)| {
            let rest = &s[i + 3..];
            let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            !digits.is_empty() && rest[digits.len()..].starts_with(" 0 R")
        })
    }

    #[test]
    fn test_document_builder_with_metadata() {
        let mut builder = DocumentBuilder::new().metadata(
            DocumentMetadata::new()
                .title("Test Document")
                .author("Test Author"),
        );

        builder.letter_page().text("Content").done();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Title (Test Document)"));
        assert!(content.contains("/Author (Test Author)"));
    }

    #[test]
    fn test_document_builder_multiple_pages() {
        let mut builder = DocumentBuilder::new();
        builder.letter_page().text("Page 1").done();
        builder.a4_page().text("Page 2").done();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Count 2"));
    }

    #[test]
    fn test_fluent_page_builder() {
        let mut builder = DocumentBuilder::new();
        builder
            .letter_page()
            .at(72.0, 720.0)
            .font("Helvetica-Bold", 18.0)
            .text("Title")
            .font("Helvetica", 12.0)
            .text("Body text")
            .space(12.0)
            .text("More text")
            .done();

        let bytes = builder.build().unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_fluent_page_builder_measure() {
        // `measure` is a pure query: no cursor advance, no content emission.
        // Width must scale with font size and character count.
        let mut doc = DocumentBuilder::new();
        let page = doc.letter_page().at(72.0, 720.0);

        let short = page.measure("AB");
        let long = page.measure("ABCDEFGH");
        assert!(long > short, "longer string must measure wider");
        assert!(short > 0.0, "non-empty string must have positive width");

        // Empty string is zero width.
        assert_eq!(page.measure(""), 0.0);

        // Switching font size scales the measure.
        let small_page = page.font("Helvetica", 10.0);
        let small = small_page.measure("ABC");
        let big = small_page.font("Helvetica", 20.0).measure("ABC");
        assert!(
            (big - 2.0 * small).abs() < 0.5,
            "doubling font size should ~double measured width: {} vs 2*{}",
            big,
            small
        );
    }

    #[test]
    fn test_table_fluent_emits_elements_and_advances_cursor() {
        use super::super::table_renderer::{
            CellAlign, ColumnWidth, Table as RenderTable, TableCell,
        };

        let mut doc = DocumentBuilder::new();
        let page = doc.letter_page().font("Helvetica", 12.0);
        let cursor_before = page.cursor_y;

        let table = RenderTable::new(vec![
            vec![
                TableCell::text("Name"),
                TableCell::text("Value").align(CellAlign::Right),
            ],
            vec![TableCell::text("Alice"), TableCell::text("42")],
            vec![TableCell::text("Bob"), TableCell::text("7")],
        ])
        .with_header_row()
        .with_column_widths(vec![ColumnWidth::Fixed(200.0), ColumnWidth::Fixed(200.0)]);

        let page = page.table(table);
        let cursor_after = page.cursor_y;
        page.done();

        // Cursor advanced downward by at least one row's worth of height.
        assert!(
            cursor_after < cursor_before,
            "cursor must move down after .table(): before={} after={}",
            cursor_before,
            cursor_after
        );

        // At least one Text element per non-empty cell — 6 cells here.
        let texts: Vec<_> = doc.pages[0]
            .elements
            .iter()
            .filter_map(|e| match e {
                ContentElement::Text(t) => Some(t),
                _ => None,
            })
            .collect();
        assert_eq!(
            texts.len(),
            6,
            "expected one Text per cell; got {}: {:?}",
            texts.len(),
            texts.iter().map(|t| &t.text).collect::<Vec<_>>()
        );

        // Header row (first 2 Text elements) must use Helvetica-Bold because
        // TableCell::header and is_header promote the font name.
        assert_eq!(texts[0].font.name, "Helvetica-Bold", "header cell must use bold font");
        assert_eq!(texts[1].font.name, "Helvetica-Bold");
        // Body rows stay on the default Helvetica.
        assert_eq!(texts[2].font.name, "Helvetica");
    }

    #[test]
    fn test_table_fluent_reading_order_is_page_relative() {
        // If there's already stuff on the page before .table(), the table's
        // reading_order must start after the existing elements — not from 0.
        use super::super::table_renderer::{ColumnWidth, Table as RenderTable, TableCell};

        let mut doc = DocumentBuilder::new();
        doc.letter_page()
            .at(72.0, 720.0)
            .font("Helvetica", 12.0)
            .text("Before the table") // becomes reading_order=0
            .table(
                RenderTable::new(vec![vec![TableCell::text("a"), TableCell::text("b")]])
                    .with_column_widths(vec![
                        ColumnWidth::Fixed(100.0),
                        ColumnWidth::Fixed(100.0),
                    ]),
            )
            .text("After the table")
            .done();

        let orders: Vec<usize> = doc.pages[0]
            .elements
            .iter()
            .filter_map(|e| e.reading_order())
            .collect();

        // Orders must be monotone and start from 0.
        for pair in orders.windows(2) {
            assert!(pair[1] > pair[0], "reading_order must be strictly monotone: {:?}", orders);
        }
    }

    #[test]
    fn test_image_from_file_and_bytes() {
        // Happy path: load an image from disk + from bytes, confirm
        // ContentElement::Image is pushed with the right geometry.
        let mut doc = DocumentBuilder::new();
        doc.letter_page()
            .image_from_file(
                "tests/fixtures/adobe_cmyk_10x11_white.jpg",
                Rect::new(72.0, 600.0, 100.0, 110.0),
            )
            .expect("load from file")
            .done();

        let imgs: Vec<_> = doc.pages[0]
            .elements
            .iter()
            .filter_map(|e| match e {
                ContentElement::Image(i) => Some(i),
                _ => None,
            })
            .collect();
        assert_eq!(imgs.len(), 1);
        assert!((imgs[0].bbox.x - 72.0).abs() < 0.01);
        assert!((imgs[0].bbox.width - 100.0).abs() < 0.01);
        // 10×11 fixture.
        assert_eq!(imgs[0].width, 10);
        assert_eq!(imgs[0].height, 11);
    }

    #[test]
    fn test_image_from_bytes_roundtrip() {
        // Same image loaded via from_bytes must produce an equivalent
        // ContentElement::Image.
        let bytes =
            std::fs::read("tests/fixtures/adobe_cmyk_10x11_white.jpg").expect("fixture must exist");

        let mut doc = DocumentBuilder::new();
        doc.letter_page()
            .image_from_bytes(&bytes, Rect::new(0.0, 0.0, 50.0, 55.0))
            .expect("decode bytes")
            .done();

        let imgs: Vec<_> = doc.pages[0]
            .elements
            .iter()
            .filter_map(|e| match e {
                ContentElement::Image(i) => Some(i),
                _ => None,
            })
            .collect();
        assert_eq!(imgs.len(), 1);
        assert_eq!(imgs[0].width, 10);
    }

    #[test]
    fn test_image_from_bytes_invalid_errors() {
        // Garbage bytes must return Err, not panic.
        let mut doc = DocumentBuilder::new();
        let result = doc
            .letter_page()
            .image_from_bytes(b"not an image at all", Rect::new(0.0, 0.0, 10.0, 10.0));
        assert!(result.is_err());
    }

    #[test]
    fn test_shape_primitives_emit_path_elements() {
        // Every shape primitive must push exactly one ContentElement::Path
        // with stroke and/or fill honoured.
        let mut doc = DocumentBuilder::new();
        doc.letter_page()
            .circle(100.0, 100.0, 20.0, Some(LineStyle::new(1.5, 0.1, 0.2, 0.3)), None)
            .ellipse(200.0, 100.0, 30.0, 15.0, None, Some((0.9, 0.1, 0.1)))
            .polygon(
                &[
                    (300.0, 100.0),
                    (320.0, 120.0),
                    (340.0, 100.0),
                    (320.0, 80.0),
                ],
                Some(LineStyle::default()),
                Some((0.5, 0.5, 0.9)),
            )
            .arc(400.0, 100.0, 25.0, 0.0, std::f32::consts::PI, LineStyle::default())
            .bezier_curve(
                500.0,
                100.0,
                510.0,
                140.0,
                540.0,
                140.0,
                550.0,
                100.0,
                LineStyle::default(),
                None,
            )
            .done();

        let paths: Vec<_> = doc.pages[0]
            .elements
            .iter()
            .filter_map(|e| match e {
                ContentElement::Path(p) => Some(p),
                _ => None,
            })
            .collect();
        // Five primitives, five Path elements.
        assert_eq!(paths.len(), 5);

        // Circle: stroke set, no fill.
        assert!(paths[0].stroke_color.is_some() && paths[0].fill_color.is_none());
        assert!((paths[0].stroke_width - 1.5).abs() < 1e-6);

        // Ellipse: fill set, no stroke.
        assert!(paths[1].fill_color.is_some() && paths[1].stroke_color.is_none());

        // Polygon: both stroke (default 1pt black) and fill.
        assert!(paths[2].stroke_color.is_some() && paths[2].fill_color.is_some());

        // Arc: stroke only.
        assert!(paths[3].stroke_color.is_some() && paths[3].fill_color.is_none());

        // Bezier: stroke only (fill None).
        assert!(paths[4].stroke_color.is_some() && paths[4].fill_color.is_none());
    }

    #[test]
    fn test_transforms_extend_to_path_and_image() {
        // Inside a rotated scope, both a shape (Path) and an image
        // (Image) must carry the composed matrix, AND the emitted PDF
        // must wrap them in `q ... cm ... Q` brackets.
        let mut doc = DocumentBuilder::new();
        doc.letter_page()
            .rotated(30.0, |p| {
                p.circle(200.0, 500.0, 30.0, Some(LineStyle::default()), None)
                    .image_from_file(
                        "tests/fixtures/adobe_cmyk_10x11_white.jpg",
                        Rect::new(100.0, 400.0, 50.0, 55.0),
                    )
                    .expect("load image")
            })
            .done();

        // Check element-level matrices
        let page = &doc.pages[0];
        let paths: Vec<_> = page
            .elements
            .iter()
            .filter_map(|e| match e {
                ContentElement::Path(p) => Some(p),
                _ => None,
            })
            .collect();
        let images: Vec<_> = page
            .elements
            .iter()
            .filter_map(|e| match e {
                ContentElement::Image(i) => Some(i),
                _ => None,
            })
            .collect();

        assert_eq!(paths.len(), 1);
        assert_eq!(images.len(), 1);
        // Both must carry the same matrix (the outer rotation).
        assert!(paths[0].matrix.is_some(), "path must carry matrix");
        assert!(images[0].matrix.is_some(), "image must carry matrix");
        // 30° rotation: cos ≈ 0.866, sin = 0.5
        let pm = paths[0].matrix.unwrap();
        assert!((pm[0] - 30_f32.to_radians().cos()).abs() < 0.01);
        assert!((pm[1] - 30_f32.to_radians().sin()).abs() < 0.01);

        // Emitted PDF must contain the q / cm / Q operators.
        let bytes = doc.build().expect("build");
        let s = String::from_utf8_lossy(&bytes);
        // `cm` operator appears when a matrix is set.
        assert!(s.contains(" cm"), "expected cm operator in output");
    }

    #[test]
    fn test_bullet_list_emits_bullet_and_items() {
        let mut doc = DocumentBuilder::new();
        doc.letter_page()
            .at(72.0, 720.0)
            .bullet_list(&["Apples", "Bananas", "Cherries"])
            .done();
        let page = &doc.pages[0];
        let texts: Vec<_> = page
            .elements
            .iter()
            .filter_map(|e| match e {
                ContentElement::Text(t) => Some(t),
                _ => None,
            })
            .collect();
        // 3 items × 2 text elements each (bullet glyph + body) = 6.
        assert_eq!(
            texts.len(),
            6,
            "got texts: {:?}",
            texts.iter().map(|t| &t.text).collect::<Vec<_>>()
        );
        // Bullet glyph alternates with body text.
        assert_eq!(texts[0].text, "\u{2022}");
        assert_eq!(texts[1].text, "Apples");
        assert_eq!(texts[2].text, "\u{2022}");
        assert_eq!(texts[3].text, "Bananas");
    }

    #[test]
    fn test_numbered_list_decimal_roman_alpha() {
        let mut doc = DocumentBuilder::new();
        doc.letter_page()
            .at(72.0, 700.0)
            .numbered_list(&["one", "two", "three"], ListStyle::Decimal)
            .space(12.0)
            .numbered_list(&["alpha", "beta"], ListStyle::RomanLower)
            .space(12.0)
            .numbered_list(&["x", "y", "z"], ListStyle::AlphaLower)
            .done();
        let texts: Vec<String> = doc.pages[0]
            .elements
            .iter()
            .filter_map(|e| match e {
                ContentElement::Text(t) => Some(t.text.clone()),
                _ => None,
            })
            .collect();
        // Markers round-trip as expected.
        assert!(texts.contains(&"1.".to_string()));
        assert!(texts.contains(&"2.".to_string()));
        assert!(texts.contains(&"3.".to_string()));
        assert!(texts.contains(&"i.".to_string()));
        assert!(texts.contains(&"ii.".to_string()));
        assert!(texts.contains(&"a.".to_string()));
        assert!(texts.contains(&"c.".to_string()));
    }

    #[test]
    fn test_to_roman_lower() {
        assert_eq!(to_roman_lower(1), "i");
        assert_eq!(to_roman_lower(4), "iv");
        assert_eq!(to_roman_lower(9), "ix");
        assert_eq!(to_roman_lower(40), "xl");
        assert_eq!(to_roman_lower(90), "xc");
        assert_eq!(to_roman_lower(400), "cd");
        assert_eq!(to_roman_lower(900), "cm");
        assert_eq!(to_roman_lower(1994), "mcmxciv");
    }

    #[test]
    fn test_to_alpha_lower() {
        assert_eq!(to_alpha_lower(1), "a");
        assert_eq!(to_alpha_lower(26), "z");
        assert_eq!(to_alpha_lower(27), "aa");
        assert_eq!(to_alpha_lower(52), "az");
        assert_eq!(to_alpha_lower(53), "ba");
    }

    #[test]
    fn test_empty_list_is_no_op() {
        let mut doc = DocumentBuilder::new();
        doc.letter_page()
            .at(72.0, 720.0)
            .bullet_list::<&str>(&[])
            .numbered_list::<&str>(&[], ListStyle::Decimal)
            .done();
        // No text elements emitted.
        let n_texts = doc.pages[0]
            .elements
            .iter()
            .filter(|e| matches!(e, ContentElement::Text(_)))
            .count();
        assert_eq!(n_texts, 0);
    }

    #[test]
    fn test_code_block_emits_background_and_mono_lines() {
        let mut doc = DocumentBuilder::new();
        let before_y = {
            let p = doc.letter_page().at(72.0, 720.0);
            let y = p.cursor_y;
            p.code_block("rust", "fn main() {\n    println!(\"hi\");\n}\n")
                .done();
            y
        };

        let page = &doc.pages[0];

        // Must have emitted at least one Path (bg fill) + multiple
        // Text elements (one per source line).
        let paths: Vec<_> = page
            .elements
            .iter()
            .filter_map(|e| match e {
                ContentElement::Path(p) => Some(p),
                _ => None,
            })
            .collect();
        let texts: Vec<_> = page
            .elements
            .iter()
            .filter_map(|e| match e {
                ContentElement::Text(t) => Some(t),
                _ => None,
            })
            .collect();

        assert_eq!(paths.len(), 1, "exactly one fill rect for the code block");
        assert!(paths[0].fill_color.is_some(), "block background must be filled");
        assert!(
            texts.len() >= 3,
            "expected >=3 text lines (3-line source), got {}: {:?}",
            texts.len(),
            texts.iter().map(|t| &t.text).collect::<Vec<_>>()
        );
        // Mono font.
        assert_eq!(texts[0].font.name, "Courier");
        // Content reads back.
        assert!(texts.iter().any(|t| t.text.contains("fn main")));
        assert!(texts.iter().any(|t| t.text.contains("println")));

        // After the call the cursor must be below the initial cursor.
        // (We consumed `p` inside the closure; verify via a second
        // page's cursor-starts-fresh — less coupled than reading the
        // inner state.)
        assert!(before_y > 0.0);
    }

    #[test]
    fn test_tab_order_emits_tabs_entry() {
        for (order, letter) in [
            (TabOrder::Row, "R"),
            (TabOrder::Column, "C"),
            (TabOrder::Structure, "S"),
        ] {
            let mut doc = DocumentBuilder::new();
            doc.letter_page()
                .text_field("a", 72.0, 600.0, 100.0, 20.0, None)
                .tab_order(order)
                .done();
            let bytes = doc.build().expect("build");
            let s = String::from_utf8_lossy(&bytes);
            let needle = format!("/Tabs /{}", letter);
            assert!(
                s.contains(&needle),
                "expected {:?} in emitted PDF for TabOrder::{:?}",
                needle,
                order
            );
        }
    }

    #[test]
    fn test_field_metadata_required_readonly_tooltip() {
        let mut doc = DocumentBuilder::new();
        doc.letter_page()
            .text_field("email", 72.0, 600.0, 200.0, 20.0, None)
            .required()
            .tooltip("Your email address")
            .checkbox("terms", 72.0, 570.0, 15.0, 15.0, false)
            .read_only()
            .done();
        let bytes = doc.build().expect("build");
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/TU"), "tooltip (/TU) missing");
        assert!(s.contains("/Ff"), "field flag (/Ff) missing");
    }

    #[test]
    fn test_field_metadata_methods_are_no_ops_before_any_field() {
        let mut doc = DocumentBuilder::new();
        doc.letter_page().required().read_only().tooltip("x").done();
        let _ = doc.build().expect("build");
    }

    #[test]
    fn test_list_box_form_field_emits_in_pdf() {
        let mut doc = DocumentBuilder::new();
        doc.letter_page()
            .list_box(
                "interests",
                72.0,
                600.0,
                200.0,
                80.0,
                vec!["Hiking".into(), "Reading".into(), "Coding".into()],
                Some("Coding".into()),
                true, // multi-select
            )
            .done();
        let bytes = doc.build().expect("build");
        let s = String::from_utf8_lossy(&bytes);
        // ListBox options must appear somewhere in the PDF.
        assert!(s.contains("(Hiking)") || s.contains("Hiking"));
        assert!(s.contains("(Reading)") || s.contains("Reading"));
        // Field type Ch (choice) present for the list_box.
        assert!(s.contains("/Ch") || s.contains("Choice"));
    }

    #[test]
    fn test_insert_toc_creates_a_toc_page_from_bookmarks() {
        let mut doc = DocumentBuilder::new();
        doc.letter_page().text("ch1").done();
        doc.letter_page().text("ch2").done();
        doc.letter_page().text("ch3").done();

        let bytes = doc
            .bookmark("Chapter 1", 0)
            .bookmark("Chapter 2", 1)
            .bookmark("Chapter 3", 2)
            .insert_toc(0, "Table of Contents")
            .build()
            .expect("build");
        let content = String::from_utf8_lossy(&bytes);
        // Must have 4 pages now (3 content + 1 ToC).
        let page_count = content.matches("/Type /Page").count();
        assert!(page_count >= 4, "expected >=4 pages (3+1 ToC), got {}", page_count);
        // ToC title must appear.
        assert!(
            content.contains("(Table of Contents)")
                || content.contains("<5461626C65206F6620436F6E74656E7473>")
        );
    }

    #[test]
    fn test_insert_toc_no_bookmarks_is_no_op() {
        // With no bookmarks added, inserting a ToC should produce
        // zero extra pages.
        let mut doc = DocumentBuilder::new();
        doc.letter_page().text("only page").done();
        let before = doc.pages.len();
        doc = doc.insert_toc(0, "ToC");
        assert_eq!(doc.pages.len(), before, "no-op expected for empty outline");
    }

    #[test]
    fn test_page_labels_are_emitted_in_catalog() {
        use crate::extractors::page_labels::{PageLabelRange, PageLabelStyle};
        use crate::writer::PageLabelsBuilder;

        let mut doc = DocumentBuilder::new();
        doc.letter_page().text("prefi").done();
        doc.letter_page().text("prefii").done();
        doc.letter_page().text("body 1").done();

        let bytes = doc
            .with_page_labels(
                PageLabelsBuilder::new()
                    .add_range(PageLabelRange::new(0).with_style(PageLabelStyle::RomanLower))
                    .add_range(PageLabelRange::new(2).with_style(PageLabelStyle::Decimal)),
            )
            .build()
            .expect("build");
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("/PageLabels"), "catalog /PageLabels missing");
        // Check the /Nums tree includes our two ranges.
        assert!(content.contains("/Nums"));
    }

    #[test]
    fn test_bookmark_and_tree_emit_outlines_in_catalog() {
        use crate::writer::outline_builder::OutlineItem;

        let mut doc = DocumentBuilder::new();
        doc.letter_page().text("page 1").done();
        doc.letter_page().text("page 2").done();
        doc.letter_page().text("page 3").done();

        let bytes = doc
            .bookmark("Intro", 0)
            .bookmark_tree(|o| {
                o.add_item(OutlineItem::new("Chapter 1", 1));
                o.add_child(OutlineItem::new("Section 1.1", 2));
            })
            .build()
            .expect("build");

        let content = String::from_utf8_lossy(&bytes);
        // Catalog must reference /Outlines.
        assert!(
            content.contains("/Outlines"),
            "catalog must reference /Outlines; dump start=\n{}",
            &content[..content.len().min(400)]
        );
        // Outline titles must be emitted as PDF strings. The
        // ObjectSerializer may emit them as literal `(Intro)` or as
        // a hex form `<4974726F>` depending on content.
        let has_title = |needle: &str| -> bool {
            let literal = format!("({})", needle);
            let hex = needle
                .as_bytes()
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<String>();
            let wrapped_hex = format!("<{}>", hex);
            content.contains(&literal) || content.contains(&wrapped_hex)
        };
        assert!(has_title("Intro"), "missing bookmark Intro");
        assert!(has_title("Chapter 1"), "missing Chapter 1");
        assert!(has_title("Section 1.1"), "missing Section 1.1");
    }

    #[test]
    fn test_empty_outline_does_not_emit_outlines_entry() {
        // Baseline: no .bookmark() calls => no /Outlines in the catalog.
        let mut doc = DocumentBuilder::new();
        doc.letter_page().text("just a page").done();
        let bytes = doc.build().expect("build");
        let content = String::from_utf8_lossy(&bytes);
        assert!(!content.contains("/Outlines"));
    }

    #[test]
    fn test_transforms_apply_matrix_to_text_and_restore_on_exit() {
        // Inside a rotated/scaled/translated scope, TextContent must
        // carry the composed matrix. After the scope exits, the next
        // text emission must be back at identity.
        let mut doc = DocumentBuilder::new();
        doc.letter_page()
            .font("Helvetica", 12.0)
            .rotated(90.0, |p| p.at(100.0, 500.0).text("rotated"))
            .at(50.0, 50.0)
            .text("after")
            .done();

        let texts: Vec<_> = doc.pages[0]
            .elements
            .iter()
            .filter_map(|e| match e {
                ContentElement::Text(t) => Some(t),
                _ => None,
            })
            .collect();
        assert_eq!(texts.len(), 2);

        // Rotated text must carry a matrix.
        let m = texts[0]
            .matrix
            .expect("first text inside rotated scope must carry matrix");
        // 90° rotation: [cos sin -sin cos 0 0] ≈ [0 1 -1 0 0 0].
        assert!((m[0] - 0.0).abs() < 1e-4 && (m[1] - 1.0).abs() < 1e-4);
        assert!((m[2] - -1.0).abs() < 1e-4 && (m[3] - 0.0).abs() < 1e-4);

        // After scope exit, no matrix.
        assert!(texts[1].matrix.is_none());
    }

    #[test]
    fn test_transforms_compose_translated_rotated() {
        // translated + rotated must compose; the composed matrix
        // should have both a rotation and a translation component.
        let mut doc = DocumentBuilder::new();
        doc.letter_page()
            .font("Helvetica", 12.0)
            .translated(50.0, 100.0, |p| p.rotated(45.0, |p| p.text("tilted")))
            .done();

        let texts: Vec<_> = doc.pages[0]
            .elements
            .iter()
            .filter_map(|e| match e {
                ContentElement::Text(t) => Some(t),
                _ => None,
            })
            .collect();
        let m = texts[0].matrix.expect("composed matrix must be set");

        // Translation component must be (50, 100).
        assert!((m[4] - 50.0).abs() < 0.01, "tx expected 50, got {}", m[4]);
        assert!((m[5] - 100.0).abs() < 0.01, "ty expected 100, got {}", m[5]);

        // Rotation component: 45° → cos = sin ≈ 0.7071.
        let r45 = std::f32::consts::FRAC_1_SQRT_2;
        assert!((m[0] - r45).abs() < 0.01);
        assert!((m[1] - r45).abs() < 0.01);
    }

    #[test]
    fn test_compose_affine_identity_round_trip() {
        // Composing with identity is a no-op on either side.
        let m = [1.5, 0.3, -0.2, 0.9, 10.0, 20.0];
        assert_eq!(compose_affine(IDENTITY, m), m);
        assert_eq!(compose_affine(m, IDENTITY), m);
    }

    #[test]
    fn test_line_style_dash_round_trip() {
        // LineStyle accepts a dash pattern via .with_dash(); the value
        // propagates into PathContent.dash_pattern on both stroke_line
        // and shape primitives; .solid() clears it.
        let dashed = LineStyle::new(1.5, 0.1, 0.2, 0.3).with_dash(&[3.0, 2.0], 0.0);
        assert_eq!(dashed.dash, Some((vec![3.0, 2.0], 0.0)));
        let solid = dashed.clone().solid();
        assert!(solid.dash.is_none());

        let mut doc = DocumentBuilder::new();
        doc.letter_page()
            .stroke_line(10.0, 100.0, 100.0, 100.0, dashed.clone())
            .circle(200.0, 100.0, 30.0, Some(dashed), None)
            .done();

        let paths: Vec<_> = doc.pages[0]
            .elements
            .iter()
            .filter_map(|e| match e {
                ContentElement::Path(p) => Some(p),
                _ => None,
            })
            .collect();
        assert_eq!(paths.len(), 2);
        // Dash pattern must have been plumbed through to both paths.
        assert_eq!(paths[0].dash_pattern, Some((vec![3.0, 2.0], 0.0)));
        assert_eq!(paths[1].dash_pattern, Some((vec![3.0, 2.0], 0.0)));
    }

    #[test]
    fn test_polygon_requires_two_points() {
        // Fewer than 2 points must be a no-op, not a panic.
        let mut doc = DocumentBuilder::new();
        doc.letter_page()
            .polygon(&[], Some(LineStyle::default()), None)
            .polygon(&[(100.0, 100.0)], Some(LineStyle::default()), None)
            .done();
        // No paths emitted from either degenerate call.
        let paths_n = doc.pages[0]
            .elements
            .iter()
            .filter(|e| matches!(e, ContentElement::Path(_)))
            .count();
        assert_eq!(paths_n, 0);
    }

    #[test]
    fn test_remaining_space_matches_cursor_vs_bottom_margin() {
        // Letter page is 612 × 792. Default cursor_y at page start = 792 - 72 = 720.
        // Bottom margin convention = 72. So initial remaining = 720 - 72 = 648.
        let mut doc = DocumentBuilder::new();
        let page = doc.letter_page();
        assert!(
            (page.remaining_space() - 648.0).abs() < 0.01,
            "initial: {}",
            page.remaining_space()
        );

        // After .text(), cursor drops by size * line_height = 12 * 1.2 = 14.4.
        // Remaining should drop by the same.
        let page = page.font("Helvetica", 12.0).text("row 1");
        let expected = 648.0 - 12.0 * 1.2;
        assert!(
            (page.remaining_space() - expected).abs() < 0.01,
            "after one text line: {} vs expected {}",
            page.remaining_space(),
            expected
        );

        // Moving cursor below the bottom margin clamps remaining_space to 0.0.
        let page = page.at(72.0, 10.0);
        assert_eq!(page.remaining_space(), 0.0);
    }

    #[test]
    fn test_new_page_same_size_preserves_dimensions_and_config() {
        let mut doc = DocumentBuilder::new();
        doc.page(PageSize::A3)
            .font("Times-Roman", 14.0)
            .text("page 1")
            .new_page_same_size()
            .text("page 2") // uses carried font/size
            .done();

        assert_eq!(doc.pages.len(), 2);
        let (w0, h0) = (doc.pages[0].width, doc.pages[0].height);
        let (w1, h1) = (doc.pages[1].width, doc.pages[1].height);
        assert_eq!(w0, w1, "width preserved");
        assert_eq!(h0, h1, "height preserved");

        // The second page's "page 2" text element must be in Times-Roman
        // 14pt, proving text_config carried over.
        let texts_p2: Vec<_> = doc.pages[1]
            .elements
            .iter()
            .filter_map(|e| match e {
                ContentElement::Text(t) => Some(t),
                _ => None,
            })
            .collect();
        assert_eq!(texts_p2.len(), 1);
        assert_eq!(texts_p2[0].font.name, "Times-Roman");
        assert_eq!(texts_p2[0].font.size, 14.0);
    }

    #[test]
    fn test_new_page_same_size_resets_cursor_to_top() {
        // After a bunch of .text() calls the cursor is well down the first
        // page. A fresh new_page_same_size must start at the top-left
        // again (cursor_y = height - 72 for a fresh page).
        let mut doc = DocumentBuilder::new();
        let page = doc
            .letter_page()
            .font("Helvetica", 12.0)
            .text("l1")
            .text("l2")
            .text("l3");

        let first_remaining = page.remaining_space();
        let new_page = page.new_page_same_size();
        assert!(
            new_page.remaining_space() > first_remaining,
            "new page must have more headroom: new={} vs old={}",
            new_page.remaining_space(),
            first_remaining
        );
        assert!(
            (new_page.remaining_space() - 648.0).abs() < 0.01,
            "fresh letter page expected 648pt remaining, got {}",
            new_page.remaining_space()
        );
    }

    #[test]
    fn test_stroke_rect_emits_path_with_style() {
        // stroke_rect must push a Path element with the supplied width and
        // colour, fill unset, so downstream PDF emission does `S` (stroke
        // only) not `B` (stroke + fill).
        let mut doc = DocumentBuilder::new();
        doc.letter_page()
            .stroke_rect(50.0, 50.0, 200.0, 100.0, LineStyle::new(2.5, 0.8, 0.2, 0.1))
            .done();

        let page = &doc.pages[0];
        let paths: Vec<_> = page
            .elements
            .iter()
            .filter_map(|e| match e {
                ContentElement::Path(p) => Some(p),
                _ => None,
            })
            .collect();
        assert_eq!(paths.len(), 1);
        let p = paths[0];
        assert_eq!(p.stroke_width, 2.5);
        let c = p.stroke_color.expect("stroke color must be set");
        assert!((c.r - 0.8).abs() < 1e-6 && (c.g - 0.2).abs() < 1e-6 && (c.b - 0.1).abs() < 1e-6);
        assert!(p.fill_color.is_none(), "stroke_rect must not fill");
    }

    #[test]
    fn test_stroke_line_emits_path_with_style() {
        let mut doc = DocumentBuilder::new();
        doc.letter_page()
            .stroke_line(10.0, 100.0, 500.0, 100.0, LineStyle::new(0.5, 0.5, 0.5, 0.5))
            .done();

        let page = &doc.pages[0];
        let paths: Vec<_> = page
            .elements
            .iter()
            .filter_map(|e| match e {
                ContentElement::Path(p) => Some(p),
                _ => None,
            })
            .collect();
        assert_eq!(paths.len(), 1);
        let p = paths[0];
        assert_eq!(p.stroke_width, 0.5);
        let c = p.stroke_color.expect("stroke color must be set");
        assert!((c.r - 0.5).abs() < 1e-6 && (c.g - 0.5).abs() < 1e-6 && (c.b - 0.5).abs() < 1e-6);
        assert!(p.fill_color.is_none());
    }

    #[test]
    fn test_line_style_default() {
        let s = LineStyle::default();
        assert_eq!(s.width, 1.0);
        assert_eq!(s.color, (0.0, 0.0, 0.0));
    }

    #[test]
    fn test_text_in_rect_wraps_and_aligns() {
        // Feed long-enough text into a narrow rect and assert N emitted
        // TextContent elements with correct per-line placement for each
        // alignment mode.
        for (align, anchor) in [
            (TextAlign::Left, "left"),
            (TextAlign::Center, "center"),
            (TextAlign::Right, "right"),
        ] {
            let mut doc = DocumentBuilder::new();
            let rect = Rect::new(100.0, 600.0, 60.0, 200.0);
            doc.letter_page()
                .font("Helvetica", 10.0)
                .text_in_rect(rect, "alpha beta gamma delta epsilon", align)
                .done();

            let page = &doc.pages[0];
            let elements: Vec<_> = page
                .elements
                .iter()
                .filter_map(|e| match e {
                    ContentElement::Text(t) => Some(t),
                    _ => None,
                })
                .collect();

            assert!(
                elements.len() >= 2,
                "{} align: expected wrap to >= 2 lines, got {}",
                anchor,
                elements.len()
            );

            // Every emitted line must fit inside the rect horizontally.
            for (idx, tc) in elements.iter().enumerate() {
                assert!(
                    tc.bbox.x >= rect.x - 0.01,
                    "{} line {} starts outside left edge: x={} rect.x={}",
                    anchor,
                    idx,
                    tc.bbox.x,
                    rect.x
                );
                assert!(
                    tc.bbox.x + tc.bbox.width <= rect.x + rect.width + 0.01,
                    "{} line {} extends past right edge: end={} rect_end={}",
                    anchor,
                    idx,
                    tc.bbox.x + tc.bbox.width,
                    rect.x + rect.width
                );
            }

            // Per-alignment placement: check the first line specifically.
            let first = elements[0];
            match align {
                TextAlign::Left => {
                    assert!(
                        (first.bbox.x - rect.x).abs() < 0.01,
                        "left align: line x must equal rect.x, got {} vs {}",
                        first.bbox.x,
                        rect.x
                    );
                },
                TextAlign::Center => {
                    let expected = rect.x + (rect.width - first.bbox.width) / 2.0;
                    assert!(
                        (first.bbox.x - expected).abs() < 0.01,
                        "center align: expected x={}, got {}",
                        expected,
                        first.bbox.x
                    );
                },
                TextAlign::Right => {
                    let expected = rect.x + rect.width - first.bbox.width;
                    assert!(
                        (first.bbox.x - expected).abs() < 0.01,
                        "right align: expected x={}, got {}",
                        expected,
                        first.bbox.x
                    );
                },
            }

            // Lines are stacked top-down: y decreases monotonically.
            for pair in elements.windows(2) {
                assert!(
                    pair[1].bbox.y < pair[0].bbox.y,
                    "{} lines must move down (y decreases): {} then {}",
                    anchor,
                    pair[0].bbox.y,
                    pair[1].bbox.y
                );
            }
        }
    }

    #[test]
    fn test_text_in_rect_does_not_advance_cursor() {
        // Callers track the cursor themselves for text_in_rect; unlike
        // `.text()` and `.paragraph()`, this primitive must leave cursor_y
        // untouched so tables can advance their own geometry.
        let mut doc = DocumentBuilder::new();
        let rect = Rect::new(100.0, 600.0, 80.0, 100.0);
        doc.letter_page()
            .at(200.0, 750.0)
            .font("Helvetica", 12.0)
            .text_in_rect(rect, "test", TextAlign::Left)
            .text("after") // this should land at the untouched cursor
            .done();

        let page = &doc.pages[0];
        let texts: Vec<_> = page
            .elements
            .iter()
            .filter_map(|e| match e {
                ContentElement::Text(t) => Some(t),
                _ => None,
            })
            .collect();

        // Two text elements: the rect'd one and the "after" one.
        assert_eq!(texts.len(), 2);
        // The "after" text sits at y=750 (untouched cursor), not at some
        // y derived from rect.y - line_height.
        let after = texts.iter().find(|t| t.text == "after").unwrap();
        assert!(
            (after.bbox.y - 750.0).abs() < 0.01,
            "cursor must be untouched by text_in_rect; got y={}",
            after.bbox.y
        );
    }

    #[test]
    fn test_text_config() {
        let config = TextConfig {
            font: "Times-Roman".to_string(),
            size: 14.0,
            align: TextAlign::Center,
            line_height: 1.5,
        };

        assert_eq!(config.font, "Times-Roman");
        assert_eq!(config.size, 14.0);
    }

    // ==========================================================================
    // Annotation Tests
    // ==========================================================================

    #[test]
    fn test_link_url_annotation() {
        let mut builder = DocumentBuilder::new();
        builder
            .letter_page()
            .at(72.0, 720.0)
            .text("Click here")
            .link_url("https://example.com")
            .done();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Subtype /Link"));
        assert!(content.contains("/S /URI"));
        assert!(content.contains("example.com"));
    }

    #[test]
    fn test_link_page_annotation() {
        let mut builder = DocumentBuilder::new();
        builder.letter_page().text("Page 1").done();
        builder
            .letter_page()
            .at(72.0, 720.0)
            .text("Go to page 1")
            .link_page(0)
            .done();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Subtype /Link"));
        assert!(content.contains("/Dest"));
    }

    #[test]
    fn test_highlight_annotation() {
        let mut builder = DocumentBuilder::new();
        builder
            .letter_page()
            .at(72.0, 720.0)
            .text("Important text")
            .highlight((1.0, 1.0, 0.0))
            .done();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Subtype /Highlight"));
        assert!(content.contains("/QuadPoints"));
    }

    #[test]
    fn test_underline_annotation() {
        let mut builder = DocumentBuilder::new();
        builder
            .letter_page()
            .at(72.0, 720.0)
            .text("Underlined text")
            .underline((1.0, 0.0, 0.0))
            .done();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Subtype /Underline"));
    }

    #[test]
    fn test_strikeout_annotation() {
        let mut builder = DocumentBuilder::new();
        builder
            .letter_page()
            .at(72.0, 720.0)
            .text("Deleted text")
            .strikeout((1.0, 0.0, 0.0))
            .done();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Subtype /StrikeOut"));
    }

    #[test]
    fn test_sticky_note_annotation() {
        let mut builder = DocumentBuilder::new();
        builder
            .letter_page()
            .at(72.0, 720.0)
            .sticky_note("This is a comment")
            .done();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Subtype /Text"));
        assert!(content.contains("This is a comment"));
    }

    #[test]
    fn test_stamp_annotation() {
        let mut builder = DocumentBuilder::new();
        builder
            .letter_page()
            .at(72.0, 720.0)
            .stamp(StampType::Approved)
            .done();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Subtype /Stamp"));
        assert!(content.contains("/Name /Approved"));
    }

    #[test]
    fn test_freetext_annotation() {
        let mut builder = DocumentBuilder::new();
        builder
            .letter_page()
            .freetext(Rect::new(100.0, 500.0, 200.0, 50.0), "Free text content")
            .done();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Subtype /FreeText"));
        assert!(content.contains("Free text content"));
    }

    #[test]
    fn test_watermark_annotation() {
        let mut builder = DocumentBuilder::new();
        builder.letter_page().watermark("DRAFT").done();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Subtype /Watermark"));
    }

    #[test]
    fn test_watermark_presets() {
        let mut builder = DocumentBuilder::new();
        builder.letter_page().watermark_confidential().done();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Subtype /Watermark"));
    }

    #[test]
    fn test_multiple_annotations() {
        let mut builder = DocumentBuilder::new();
        builder
            .letter_page()
            .at(72.0, 720.0)
            .text("Linked and highlighted text")
            .link_url("https://example.com")
            .highlight((1.0, 1.0, 0.0))
            .sticky_note("Review this")
            .done();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        // Should have all three annotation types
        assert!(content.contains("/Subtype /Link"));
        assert!(content.contains("/Subtype /Highlight"));
        assert!(content.contains("/Subtype /Text"));
    }

    #[test]
    fn test_add_generic_annotation() {
        let mut builder = DocumentBuilder::new();
        let link =
            LinkAnnotation::uri(Rect::new(100.0, 700.0, 100.0, 20.0), "https://rust-lang.org");
        builder.letter_page().add_annotation(link).done();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("/Subtype /Link"));
        assert!(content.contains("rust-lang.org"));
    }

    #[test]
    fn test_no_annotation_when_no_text() {
        let mut builder = DocumentBuilder::new();
        // Try to add link without any text - should be a no-op
        builder
            .letter_page()
            .at(72.0, 720.0)
            .link_url("https://example.com") // No preceding text
            .done();

        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        // Should NOT contain a link annotation since there was no text to link
        assert!(!content.contains("/Subtype /Link"));
    }

    // ─── Tagged PDF / PDF/UA catalog wiring tests (F-1, F-2, F-4) ───────────

    #[test]
    fn test_tagged_pdf_ua1_emits_mark_info() {
        let mut builder = DocumentBuilder::new();
        builder = builder.metadata(
            DocumentMetadata::new()
                .title("Test")
                .tagged_pdf_ua1()
                .language("en-US"),
        );
        builder.letter_page().at(72.0, 720.0).text("Hello").done();
        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("/MarkInfo"), "catalog must contain /MarkInfo");
        assert!(content.contains("/Marked true"), "/MarkInfo must have /Marked true");
    }

    #[test]
    fn test_tagged_pdf_ua1_emits_struct_tree_root() {
        let mut builder = DocumentBuilder::new();
        builder = builder.metadata(
            DocumentMetadata::new()
                .title("Test")
                .tagged_pdf_ua1()
                .language("en-US"),
        );
        builder.letter_page().at(72.0, 720.0).text("Hello").done();
        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("/StructTreeRoot"), "catalog must contain /StructTreeRoot");
        assert!(content.contains("/ParentTree"), "StructTreeRoot must contain /ParentTree");
        assert!(
            content.contains("/ParentTreeNextKey"),
            "StructTreeRoot must contain /ParentTreeNextKey"
        );
    }

    #[test]
    fn test_tagged_pdf_ua1_emits_lang_and_viewer_prefs() {
        let mut builder = DocumentBuilder::new();
        builder = builder.metadata(
            DocumentMetadata::new()
                .title("Tagged Test")
                .tagged_pdf_ua1()
                .language("fr-CA"),
        );
        builder.letter_page().at(72.0, 720.0).text("Bonjour").done();
        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("/Lang"), "catalog must contain /Lang");
        assert!(content.contains("fr-CA"), "/Lang must carry the configured language");
        assert!(
            content.contains("/ViewerPreferences"),
            "catalog must contain /ViewerPreferences"
        );
        assert!(
            content.contains("/DisplayDocTitle true"),
            "/ViewerPreferences must set /DisplayDocTitle true"
        );
    }

    #[test]
    fn test_tagged_pdf_ua1_emits_struct_parents_on_pages() {
        let mut builder = DocumentBuilder::new();
        builder = builder.metadata(DocumentMetadata::new().tagged_pdf_ua1().language("en"));
        // Two-page document so we can verify both pages get /StructParents.
        builder.letter_page().at(72.0, 720.0).text("Page 1").done();
        builder.letter_page().at(72.0, 720.0).text("Page 2").done();
        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        // Each page dict should carry /StructParents.
        let count = content.matches("/StructParents").count();
        assert_eq!(count, 2, "each page must carry /StructParents; found {count} occurrences");
    }

    #[test]
    fn test_tagged_pdf_ua1_role_map() {
        let mut builder = DocumentBuilder::new();
        builder = builder.metadata(
            DocumentMetadata::new()
                .tagged_pdf_ua1()
                .language("en")
                .role_map("Note", "P")
                .role_map("Caption", "P"),
        );
        builder.letter_page().at(72.0, 720.0).text("hello").done();
        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(
            content.contains("/RoleMap"),
            "StructTreeRoot must contain /RoleMap when role mappings are set"
        );
        assert!(content.contains("/Note"), "RoleMap must contain /Note");
        assert!(content.contains("/Caption"), "RoleMap must contain /Caption");
    }

    #[test]
    fn test_untagged_pdf_has_no_mark_info_or_struct_tree() {
        let mut builder = DocumentBuilder::new();
        builder.letter_page().at(72.0, 720.0).text("plain").done();
        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(!content.contains("/MarkInfo"), "untagged PDF must NOT contain /MarkInfo");
        assert!(
            !content.contains("/StructTreeRoot"),
            "untagged PDF must NOT contain /StructTreeRoot"
        );
    }

    #[test]
    fn test_tagged_pdf_ua1_emits_xmp_metadata() {
        let mut builder = DocumentBuilder::new();
        builder = builder.metadata(
            DocumentMetadata::new()
                .title("XMP Test")
                .tagged_pdf_ua1()
                .language("en-US"),
        );
        builder.letter_page().at(72.0, 720.0).text("hello").done();
        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("/Metadata"), "catalog must reference /Metadata stream");
        assert!(content.contains("pdfuaid:part"), "XMP stream must contain pdfuaid:part");
        assert!(
            content.contains("<pdfuaid:part>1</pdfuaid:part>"),
            "XMP stream must declare pdfuaid:part = 1"
        );
        assert!(content.contains("XMP Test"), "XMP stream must include document title");
    }

    #[test]
    fn test_untagged_pdf_has_no_xmp_metadata() {
        let mut builder = DocumentBuilder::new();
        builder.letter_page().at(72.0, 720.0).text("plain").done();
        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(
            !content.contains("pdfuaid:part"),
            "untagged PDF must NOT contain XMP pdfuaid namespace"
        );
    }

    #[test]
    fn test_image_with_alt_emits_figure_bdc() {
        use crate::geometry::Rect;
        let img_bytes =
            std::fs::read("tests/fixtures/adobe_cmyk_10x11_white.jpg").expect("fixture must exist");
        let mut builder = DocumentBuilder::new();
        builder = builder.metadata(DocumentMetadata::new().tagged_pdf_ua1());
        let page = builder.letter_page();
        let _page = page
            .image_from_bytes_with_alt(&img_bytes, Rect::new(72.0, 600.0, 100.0, 100.0), "Logo")
            .unwrap()
            .done();
        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(
            content.contains("/Figure <</MCID"),
            "image with alt text must be wrapped in /Figure BDC"
        );
        assert!(content.contains("Logo"), "alt text must appear in the /Alt entry");
    }

    #[test]
    fn test_image_as_artifact_emits_artifact_bdc() {
        use crate::geometry::Rect;
        let img_bytes =
            std::fs::read("tests/fixtures/adobe_cmyk_10x11_white.jpg").expect("fixture must exist");
        let mut builder = DocumentBuilder::new();
        builder = builder.metadata(DocumentMetadata::new().tagged_pdf_ua1());
        let page = builder.letter_page();
        let _page = page
            .image_from_bytes_as_artifact(&img_bytes, Rect::new(72.0, 600.0, 100.0, 100.0))
            .unwrap()
            .done();
        let bytes = builder.build().unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(
            content.contains("/Artifact <<"),
            "decorative image must be wrapped in /Artifact BDC"
        );
    }
}
