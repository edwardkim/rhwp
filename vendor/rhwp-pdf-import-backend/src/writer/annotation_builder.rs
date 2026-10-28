//! Annotation builder for PDF generation.
//!
//! This module provides support for creating PDF annotations per PDF spec Section 12.5.
//!
//! # Supported Annotation Types
//!
//! - **Link**: Hyperlinks (URI, GoTo page, named destinations)
//! - More types coming in Phase 2+ (Highlight, Underline, StrikeOut, Text, FreeText, etc.)
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::writer::{LinkAnnotation, AnnotationBuilder, Annotation};
//! use pdf_oxide::geometry::Rect;
//!
//! // Create a link annotation
//! let link = LinkAnnotation::uri(
//!     Rect::new(72.0, 720.0, 100.0, 12.0),
//!     "https://example.com",
//! );
//!
//! // Add via generic annotation interface
//! let mut builder = AnnotationBuilder::new();
//! builder.add_annotation(link);
//! ```

use super::freetext::FreeTextAnnotation;
use super::ink::InkAnnotation;
use super::movie::MovieAnnotation;
use super::richmedia::RichMediaAnnotation;
use super::screen::ScreenAnnotation;
use super::shape_annotations::{LineAnnotation, PolygonAnnotation, ShapeAnnotation};
use super::sound::SoundAnnotation;
use super::special_annotations::{
    CaretAnnotation, FileAttachmentAnnotation, PopupAnnotation, RedactAnnotation,
};
use super::stamp::StampAnnotation;
use super::text_annotations::TextAnnotation;
use super::text_markup::TextMarkupAnnotation;
use super::threed::ThreeDAnnotation;
use super::watermark::WatermarkAnnotation;
use crate::geometry::Rect;
use crate::object::{Object, ObjectRef};
use std::collections::HashMap;

/// Border style for annotations.
#[derive(Debug, Clone, Copy)]
pub struct BorderStyle {
    /// Horizontal corner radius
    pub horizontal_radius: f32,
    /// Vertical corner radius
    pub vertical_radius: f32,
    /// Border width
    pub width: f32,
    /// Dash pattern (if dashed)
    pub dash: Option<(f32, f32)>,
}

impl Default for BorderStyle {
    fn default() -> Self {
        Self {
            horizontal_radius: 0.0,
            vertical_radius: 0.0,
            width: 0.0, // No visible border by default
            dash: None,
        }
    }
}

impl BorderStyle {
    /// Create a border style with no visible border.
    pub fn none() -> Self {
        Self::default()
    }

    /// Create a solid border with specified width.
    pub fn solid(width: f32) -> Self {
        Self {
            width,
            ..Default::default()
        }
    }

    /// Create a dashed border.
    pub fn dashed(width: f32, dash_length: f32, gap_length: f32) -> Self {
        Self {
            width,
            dash: Some((dash_length, gap_length)),
            ..Default::default()
        }
    }

    /// Set corner radius.
    pub fn with_radius(mut self, radius: f32) -> Self {
        self.horizontal_radius = radius;
        self.vertical_radius = radius;
        self
    }

    /// Convert to PDF Border array.
    pub fn to_border_array(&self) -> Object {
        let mut arr = vec![
            Object::Real(self.horizontal_radius as f64),
            Object::Real(self.vertical_radius as f64),
            Object::Real(self.width as f64),
        ];

        if let Some((dash, gap)) = self.dash {
            arr.push(Object::Array(vec![Object::Real(dash as f64), Object::Real(gap as f64)]));
        }

        Object::Array(arr)
    }
}

/// Highlight mode for link annotations.
#[derive(Debug, Clone, Copy, Default)]
pub enum HighlightMode {
    /// No highlighting (N)
    None,
    /// Invert the contents of the annotation rectangle (I)
    #[default]
    Invert,
    /// Invert the annotation's border (O)
    Outline,
    /// Display the annotation as if it were being pushed (P)
    Push,
}

impl HighlightMode {
    /// Get the PDF name for this highlight mode.
    pub fn pdf_name(&self) -> &'static str {
        match self {
            HighlightMode::None => "N",
            HighlightMode::Invert => "I",
            HighlightMode::Outline => "O",
            HighlightMode::Push => "P",
        }
    }
}

/// Action for link annotations.
#[derive(Debug, Clone)]
pub enum LinkAction {
    /// Open a URI (external link)
    Uri(String),
    /// Go to a page destination
    GoTo {
        /// Page index (0-indexed)
        page: usize,
        /// Optional fit mode
        fit: Option<super::outline_builder::FitMode>,
    },
    /// Go to a named destination
    GoToNamed(String),
    /// Go to a destination in another PDF file
    GoToRemote {
        /// File path
        file: String,
        /// Page index
        page: usize,
    },
    /// Launch an application
    Launch(String),
    /// JavaScript action
    JavaScript(String),
}

/// A link annotation for hyperlinks.
#[derive(Debug, Clone)]
pub struct LinkAnnotation {
    /// Bounding rectangle (in page coordinates)
    pub rect: Rect,
    /// The action to perform when clicked
    pub action: LinkAction,
    /// Border style
    pub border: BorderStyle,
    /// Highlight mode
    pub highlight: HighlightMode,
    /// Optional color for the border/highlight
    pub color: Option<(f32, f32, f32)>,
    /// Optional quad points for precise link area
    pub quad_points: Option<Vec<f32>>,
}

impl LinkAnnotation {
    /// Create a new link annotation with a URI action.
    pub fn uri(rect: Rect, uri: impl Into<String>) -> Self {
        Self {
            rect,
            action: LinkAction::Uri(uri.into()),
            border: BorderStyle::none(),
            highlight: HighlightMode::default(),
            color: None,
            quad_points: None,
        }
    }

    /// Create a new link annotation that goes to a page.
    pub fn goto_page(rect: Rect, page: usize) -> Self {
        Self {
            rect,
            action: LinkAction::GoTo { page, fit: None },
            border: BorderStyle::none(),
            highlight: HighlightMode::default(),
            color: None,
            quad_points: None,
        }
    }

    /// Create a link to a named destination.
    pub fn goto_named(rect: Rect, name: impl Into<String>) -> Self {
        Self {
            rect,
            action: LinkAction::GoToNamed(name.into()),
            border: BorderStyle::none(),
            highlight: HighlightMode::default(),
            color: None,
            quad_points: None,
        }
    }

    /// Set the border style.
    pub fn with_border(mut self, border: BorderStyle) -> Self {
        self.border = border;
        self
    }

    /// Set the highlight mode.
    pub fn with_highlight(mut self, highlight: HighlightMode) -> Self {
        self.highlight = highlight;
        self
    }

    /// Set the color.
    pub fn with_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.color = Some((r, g, b));
        self
    }

    /// Set quad points for precise link area.
    ///
    /// Quad points define a quadrilateral that more precisely
    /// describes the link area. Each quad is 8 numbers:
    /// x1,y1 (bottom-left), x2,y2 (bottom-right),
    /// x3,y3 (top-right), x4,y4 (top-left)
    pub fn with_quad_points(mut self, points: Vec<f32>) -> Self {
        self.quad_points = Some(points);
        self
    }

    /// Build the annotation dictionary.
    pub fn build(&self, page_refs: &[ObjectRef]) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        dict.insert("Type".to_string(), Object::Name("Annot".to_string()));
        dict.insert("Subtype".to_string(), Object::Name("Link".to_string()));

        // Rectangle
        dict.insert(
            "Rect".to_string(),
            Object::Array(vec![
                Object::Real(self.rect.x as f64),
                Object::Real(self.rect.y as f64),
                Object::Real((self.rect.x + self.rect.width) as f64),
                Object::Real((self.rect.y + self.rect.height) as f64),
            ]),
        );

        // Border
        dict.insert("Border".to_string(), self.border.to_border_array());

        // Highlight mode
        dict.insert("H".to_string(), Object::Name(self.highlight.pdf_name().to_string()));

        // Color
        if let Some((r, g, b)) = self.color {
            dict.insert(
                "C".to_string(),
                Object::Array(vec![
                    Object::Real(r as f64),
                    Object::Real(g as f64),
                    Object::Real(b as f64),
                ]),
            );
        }

        // Quad points
        if let Some(ref points) = self.quad_points {
            dict.insert(
                "QuadPoints".to_string(),
                Object::Array(points.iter().map(|&p| Object::Real(p as f64)).collect()),
            );
        }

        // Action or Destination
        match &self.action {
            LinkAction::Uri(uri) => {
                let mut action = HashMap::new();
                action.insert("S".to_string(), Object::Name("URI".to_string()));
                action.insert("URI".to_string(), Object::text_string(uri));
                dict.insert("A".to_string(), Object::Dictionary(action));
            },
            LinkAction::GoTo { page, fit } => {
                if let Some(page_ref) = page_refs.get(*page) {
                    let dest = if let Some(fit_mode) = fit {
                        self.build_destination(*page_ref, fit_mode)
                    } else {
                        Object::Array(vec![
                            Object::Reference(*page_ref),
                            Object::Name("Fit".to_string()),
                        ])
                    };
                    dict.insert("Dest".to_string(), dest);
                }
            },
            LinkAction::GoToNamed(name) => {
                dict.insert("Dest".to_string(), Object::text_string(name));
            },
            LinkAction::GoToRemote { file, page } => {
                let mut action = HashMap::new();
                action.insert("S".to_string(), Object::Name("GoToR".to_string()));
                action.insert("F".to_string(), Object::text_string(file));
                action.insert(
                    "D".to_string(),
                    Object::Array(vec![
                        Object::Integer(*page as i64),
                        Object::Name("Fit".to_string()),
                    ]),
                );
                dict.insert("A".to_string(), Object::Dictionary(action));
            },
            LinkAction::Launch(app) => {
                let mut action = HashMap::new();
                action.insert("S".to_string(), Object::Name("Launch".to_string()));
                action.insert("F".to_string(), Object::text_string(app));
                dict.insert("A".to_string(), Object::Dictionary(action));
            },
            LinkAction::JavaScript(script) => {
                let mut action = HashMap::new();
                action.insert("S".to_string(), Object::Name("JavaScript".to_string()));
                action.insert("JS".to_string(), Object::text_string(script));
                dict.insert("A".to_string(), Object::Dictionary(action));
            },
        }

        dict
    }

    /// Build a destination array.
    fn build_destination(
        &self,
        page_ref: ObjectRef,
        fit: &super::outline_builder::FitMode,
    ) -> Object {
        use super::outline_builder::FitMode;

        let mut arr = vec![Object::Reference(page_ref)];

        match fit {
            FitMode::Fit => {
                arr.push(Object::Name("Fit".to_string()));
            },
            FitMode::FitH(top) => {
                arr.push(Object::Name("FitH".to_string()));
                arr.push(top.map(|t| Object::Real(t as f64)).unwrap_or(Object::Null));
            },
            FitMode::FitV(left) => {
                arr.push(Object::Name("FitV".to_string()));
                arr.push(left.map(|l| Object::Real(l as f64)).unwrap_or(Object::Null));
            },
            FitMode::FitR {
                left,
                bottom,
                right,
                top,
            } => {
                arr.push(Object::Name("FitR".to_string()));
                arr.push(Object::Real(*left as f64));
                arr.push(Object::Real(*bottom as f64));
                arr.push(Object::Real(*right as f64));
                arr.push(Object::Real(*top as f64));
            },
            FitMode::FitB => {
                arr.push(Object::Name("FitB".to_string()));
            },
            FitMode::FitBH(top) => {
                arr.push(Object::Name("FitBH".to_string()));
                arr.push(top.map(|t| Object::Real(t as f64)).unwrap_or(Object::Null));
            },
            FitMode::FitBV(left) => {
                arr.push(Object::Name("FitBV".to_string()));
                arr.push(left.map(|l| Object::Real(l as f64)).unwrap_or(Object::Null));
            },
            FitMode::XYZ { left, top, zoom } => {
                arr.push(Object::Name("XYZ".to_string()));
                arr.push(left.map(|l| Object::Real(l as f64)).unwrap_or(Object::Null));
                arr.push(top.map(|t| Object::Real(t as f64)).unwrap_or(Object::Null));
                arr.push(zoom.map(|z| Object::Real(z as f64)).unwrap_or(Object::Null));
            },
        }

        Object::Array(arr)
    }
}

/// Generic annotation wrapper for all annotation types.
///
/// This enum provides a unified interface for all PDF annotation types.
#[derive(Debug, Clone)]
pub enum Annotation {
    /// Link annotation (hyperlinks, page navigation)
    Link(LinkAnnotation),
    /// Text markup annotation (Highlight, Underline, StrikeOut, Squiggly)
    TextMarkup(TextMarkupAnnotation),
    /// Text annotation (sticky notes)
    Text(TextAnnotation),
    /// FreeText annotation (text boxes displayed on page)
    FreeText(FreeTextAnnotation),
    /// Line annotation
    Line(LineAnnotation),
    /// Shape annotation (Square or Circle)
    Shape(ShapeAnnotation),
    /// Polygon or PolyLine annotation
    Polygon(PolygonAnnotation),
    /// Ink annotation (freehand drawing)
    Ink(InkAnnotation),
    /// Stamp annotation (approval stamps, etc.)
    Stamp(StampAnnotation),
    /// Popup annotation (pop-up window for other annotations)
    Popup(PopupAnnotation),
    /// Caret annotation (text insertion markers)
    Caret(CaretAnnotation),
    /// File attachment annotation
    FileAttachment(FileAttachmentAnnotation),
    /// Redact annotation (marks content for removal)
    Redact(RedactAnnotation),
    /// Watermark annotation (transparent text overlay)
    Watermark(WatermarkAnnotation),
    /// Sound annotation (embedded audio)
    Sound(SoundAnnotation),
    /// Movie annotation (embedded video, legacy)
    Movie(MovieAnnotation),
    /// Screen annotation (modern multimedia with renditions)
    Screen(ScreenAnnotation),
    /// 3D annotation (embedded 3D models)
    ThreeD(ThreeDAnnotation),
    /// RichMedia annotation (Flash, video players)
    RichMedia(RichMediaAnnotation),
}

impl Annotation {
    /// Build the annotation dictionary.
    pub fn build(&self, page_refs: &[ObjectRef]) -> HashMap<String, Object> {
        match self {
            Annotation::Link(link) => link.build(page_refs),
            Annotation::TextMarkup(markup) => markup.build(page_refs),
            Annotation::Text(text) => text.build(page_refs),
            Annotation::FreeText(freetext) => freetext.build(page_refs),
            Annotation::Line(line) => line.build(page_refs),
            Annotation::Shape(shape) => shape.build(page_refs),
            Annotation::Polygon(polygon) => polygon.build(page_refs),
            Annotation::Ink(ink) => ink.build(page_refs),
            Annotation::Stamp(stamp) => stamp.build(page_refs),
            Annotation::Popup(popup) => popup.build(page_refs),
            Annotation::Caret(caret) => caret.build(page_refs),
            Annotation::FileAttachment(file) => file.build(page_refs),
            Annotation::Redact(redact) => redact.build(page_refs),
            Annotation::Watermark(watermark) => {
                // Watermark build needs a page ref, use first page if available
                let page_ref = page_refs.first().copied().unwrap_or(ObjectRef::new(0, 0));
                watermark.build(page_ref)
            },
            Annotation::Sound(sound) => sound.build(page_refs),
            Annotation::Movie(movie) => movie.build(page_refs),
            Annotation::Screen(screen) => screen.build(page_refs),
            Annotation::ThreeD(threed) => threed.build(page_refs),
            Annotation::RichMedia(richmedia) => richmedia.build(page_refs),
        }
    }

    /// Get the bounding rectangle of this annotation.
    pub fn rect(&self) -> Rect {
        match self {
            Annotation::Link(link) => link.rect,
            Annotation::TextMarkup(markup) => markup.rect,
            Annotation::Text(text) => text.rect,
            Annotation::FreeText(freetext) => freetext.rect,
            Annotation::Line(line) => line.calculate_rect(),
            Annotation::Shape(shape) => shape.rect,
            Annotation::Polygon(polygon) => polygon.calculate_rect(),
            Annotation::Ink(ink) => ink.calculate_rect(),
            Annotation::Stamp(stamp) => stamp.rect(),
            Annotation::Popup(popup) => popup.rect(),
            Annotation::Caret(caret) => caret.rect(),
            Annotation::FileAttachment(file) => file.rect(),
            Annotation::Redact(redact) => redact.rect(),
            Annotation::Watermark(watermark) => watermark.rect(),
            Annotation::Sound(sound) => sound.rect,
            Annotation::Movie(movie) => movie.rect,
            Annotation::Screen(screen) => screen.rect,
            Annotation::ThreeD(threed) => threed.rect,
            Annotation::RichMedia(richmedia) => richmedia.rect,
        }
    }
}

impl From<LinkAnnotation> for Annotation {
    fn from(link: LinkAnnotation) -> Self {
        Annotation::Link(link)
    }
}

impl From<TextMarkupAnnotation> for Annotation {
    fn from(markup: TextMarkupAnnotation) -> Self {
        Annotation::TextMarkup(markup)
    }
}

impl From<TextAnnotation> for Annotation {
    fn from(text: TextAnnotation) -> Self {
        Annotation::Text(text)
    }
}

impl From<FreeTextAnnotation> for Annotation {
    fn from(freetext: FreeTextAnnotation) -> Self {
        Annotation::FreeText(freetext)
    }
}

impl From<LineAnnotation> for Annotation {
    fn from(line: LineAnnotation) -> Self {
        Annotation::Line(line)
    }
}

impl From<ShapeAnnotation> for Annotation {
    fn from(shape: ShapeAnnotation) -> Self {
        Annotation::Shape(shape)
    }
}

impl From<PolygonAnnotation> for Annotation {
    fn from(polygon: PolygonAnnotation) -> Self {
        Annotation::Polygon(polygon)
    }
}

impl From<InkAnnotation> for Annotation {
    fn from(ink: InkAnnotation) -> Self {
        Annotation::Ink(ink)
    }
}

impl From<StampAnnotation> for Annotation {
    fn from(stamp: StampAnnotation) -> Self {
        Annotation::Stamp(stamp)
    }
}

impl From<PopupAnnotation> for Annotation {
    fn from(popup: PopupAnnotation) -> Self {
        Annotation::Popup(popup)
    }
}

impl From<CaretAnnotation> for Annotation {
    fn from(caret: CaretAnnotation) -> Self {
        Annotation::Caret(caret)
    }
}

impl From<FileAttachmentAnnotation> for Annotation {
    fn from(file: FileAttachmentAnnotation) -> Self {
        Annotation::FileAttachment(file)
    }
}

impl From<RedactAnnotation> for Annotation {
    fn from(redact: RedactAnnotation) -> Self {
        Annotation::Redact(redact)
    }
}

impl From<WatermarkAnnotation> for Annotation {
    fn from(watermark: WatermarkAnnotation) -> Self {
        Annotation::Watermark(watermark)
    }
}

impl From<SoundAnnotation> for Annotation {
    fn from(sound: SoundAnnotation) -> Self {
        Annotation::Sound(sound)
    }
}

impl From<MovieAnnotation> for Annotation {
    fn from(movie: MovieAnnotation) -> Self {
        Annotation::Movie(movie)
    }
}

impl From<ScreenAnnotation> for Annotation {
    fn from(screen: ScreenAnnotation) -> Self {
        Annotation::Screen(screen)
    }
}

impl From<ThreeDAnnotation> for Annotation {
    fn from(threed: ThreeDAnnotation) -> Self {
        Annotation::ThreeD(threed)
    }
}

impl From<RichMediaAnnotation> for Annotation {
    fn from(richmedia: RichMediaAnnotation) -> Self {
        Annotation::RichMedia(richmedia)
    }
}

/// Builder for page annotations.
///
/// Collects annotations for a single page and builds them into PDF objects.
#[derive(Debug, Default, Clone)]
pub struct AnnotationBuilder {
    /// All annotations for this page
    annotations: Vec<Annotation>,
}

impl AnnotationBuilder {
    /// Create a new annotation builder.
    pub fn new() -> Self {
        Self {
            annotations: Vec::new(),
        }
    }

    /// Add any annotation type.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use pdf_oxide::writer::{AnnotationBuilder, LinkAnnotation};
    /// use pdf_oxide::geometry::Rect;
    ///
    /// let mut builder = AnnotationBuilder::new();
    /// builder.add_annotation(LinkAnnotation::uri(
    ///     Rect::new(0.0, 0.0, 100.0, 20.0),
    ///     "https://example.com"
    /// ));
    /// ```
    pub fn add_annotation(&mut self, annotation: impl Into<Annotation>) -> &mut Self {
        self.annotations.push(annotation.into());
        self
    }

    /// Add a link annotation.
    pub fn add_link(&mut self, link: LinkAnnotation) -> &mut Self {
        self.add_annotation(link)
    }

    /// Add a URI link.
    pub fn uri(&mut self, rect: Rect, uri: impl Into<String>) -> &mut Self {
        self.add_link(LinkAnnotation::uri(rect, uri))
    }

    /// Add an internal page link.
    pub fn goto(&mut self, rect: Rect, page: usize) -> &mut Self {
        self.add_link(LinkAnnotation::goto_page(rect, page))
    }

    /// Get all annotations.
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    /// Get all link annotations.
    pub fn links(&self) -> Vec<&LinkAnnotation> {
        self.annotations
            .iter()
            .filter_map(|a| match a {
                Annotation::Link(link) => Some(link),
                _ => None,
            })
            .collect()
    }

    /// Get all text markup annotations.
    pub fn text_markups(&self) -> Vec<&TextMarkupAnnotation> {
        self.annotations
            .iter()
            .filter_map(|a| match a {
                Annotation::TextMarkup(markup) => Some(markup),
                _ => None,
            })
            .collect()
    }

    /// Get all text annotations (sticky notes).
    pub fn text_notes(&self) -> Vec<&TextAnnotation> {
        self.annotations
            .iter()
            .filter_map(|a| match a {
                Annotation::Text(text) => Some(text),
                _ => None,
            })
            .collect()
    }

    /// Add a text markup annotation.
    pub fn add_text_markup(&mut self, markup: TextMarkupAnnotation) -> &mut Self {
        self.add_annotation(markup)
    }

    /// Add a text annotation (sticky note).
    pub fn add_text_note(&mut self, note: TextAnnotation) -> &mut Self {
        self.add_annotation(note)
    }

    /// Get all FreeText annotations.
    pub fn free_texts(&self) -> Vec<&FreeTextAnnotation> {
        self.annotations
            .iter()
            .filter_map(|a| match a {
                Annotation::FreeText(ft) => Some(ft),
                _ => None,
            })
            .collect()
    }

    /// Add a FreeText annotation.
    pub fn add_freetext(&mut self, freetext: FreeTextAnnotation) -> &mut Self {
        self.add_annotation(freetext)
    }

    /// Get all Line annotations.
    pub fn lines(&self) -> Vec<&LineAnnotation> {
        self.annotations
            .iter()
            .filter_map(|a| match a {
                Annotation::Line(line) => Some(line),
                _ => None,
            })
            .collect()
    }

    /// Add a Line annotation.
    pub fn add_line(&mut self, line: LineAnnotation) -> &mut Self {
        self.add_annotation(line)
    }

    /// Get all Shape annotations.
    pub fn shapes(&self) -> Vec<&ShapeAnnotation> {
        self.annotations
            .iter()
            .filter_map(|a| match a {
                Annotation::Shape(shape) => Some(shape),
                _ => None,
            })
            .collect()
    }

    /// Add a Shape annotation.
    pub fn add_shape(&mut self, shape: ShapeAnnotation) -> &mut Self {
        self.add_annotation(shape)
    }

    /// Get all Polygon/PolyLine annotations.
    pub fn polygons(&self) -> Vec<&PolygonAnnotation> {
        self.annotations
            .iter()
            .filter_map(|a| match a {
                Annotation::Polygon(polygon) => Some(polygon),
                _ => None,
            })
            .collect()
    }

    /// Add a Polygon or PolyLine annotation.
    pub fn add_polygon(&mut self, polygon: PolygonAnnotation) -> &mut Self {
        self.add_annotation(polygon)
    }

    /// Get all Ink annotations.
    pub fn inks(&self) -> Vec<&InkAnnotation> {
        self.annotations
            .iter()
            .filter_map(|a| match a {
                Annotation::Ink(ink) => Some(ink),
                _ => None,
            })
            .collect()
    }

    /// Add an Ink annotation.
    pub fn add_ink(&mut self, ink: InkAnnotation) -> &mut Self {
        self.add_annotation(ink)
    }

    /// Get all Stamp annotations.
    pub fn stamps(&self) -> Vec<&StampAnnotation> {
        self.annotations
            .iter()
            .filter_map(|a| match a {
                Annotation::Stamp(stamp) => Some(stamp),
                _ => None,
            })
            .collect()
    }

    /// Add a Stamp annotation.
    pub fn add_stamp(&mut self, stamp: StampAnnotation) -> &mut Self {
        self.add_annotation(stamp)
    }

    /// Add a Popup annotation.
    pub fn add_popup(&mut self, popup: PopupAnnotation) -> &mut Self {
        self.add_annotation(popup)
    }

    /// Add a Caret annotation.
    pub fn add_caret(&mut self, caret: CaretAnnotation) -> &mut Self {
        self.add_annotation(caret)
    }

    /// Add a FileAttachment annotation.
    pub fn add_file_attachment(&mut self, file: FileAttachmentAnnotation) -> &mut Self {
        self.add_annotation(file)
    }

    /// Add a Redact annotation.
    pub fn add_redact(&mut self, redact: RedactAnnotation) -> &mut Self {
        self.add_annotation(redact)
    }

    /// Check if there are any annotations.
    pub fn is_empty(&self) -> bool {
        self.annotations.is_empty()
    }

    /// Get the number of annotations.
    pub fn len(&self) -> usize {
        self.annotations.len()
    }

    /// Build annotation objects for a page.
    ///
    /// Returns a vector of annotation dictionaries.
    pub fn build(&self, page_refs: &[ObjectRef]) -> Vec<HashMap<String, Object>> {
        self.annotations
            .iter()
            .map(|annot| annot.build(page_refs))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_border_style_none() {
        let border = BorderStyle::none();
        assert_eq!(border.width, 0.0);
    }

    #[test]
    fn test_border_style_solid() {
        let border = BorderStyle::solid(1.0);
        assert_eq!(border.width, 1.0);
        assert!(border.dash.is_none());
    }

    #[test]
    fn test_border_style_dashed() {
        let border = BorderStyle::dashed(2.0, 3.0, 1.0);
        assert_eq!(border.width, 2.0);
        assert_eq!(border.dash, Some((3.0, 1.0)));
    }

    #[test]
    fn test_highlight_modes() {
        assert_eq!(HighlightMode::None.pdf_name(), "N");
        assert_eq!(HighlightMode::Invert.pdf_name(), "I");
        assert_eq!(HighlightMode::Outline.pdf_name(), "O");
        assert_eq!(HighlightMode::Push.pdf_name(), "P");
    }

    #[test]
    fn test_link_annotation_uri() {
        let link = LinkAnnotation::uri(Rect::new(72.0, 720.0, 100.0, 12.0), "https://example.com");

        assert!(matches!(link.action, LinkAction::Uri(_)));
        assert_eq!(link.rect.x, 72.0);
    }

    #[test]
    fn test_link_annotation_goto() {
        let link = LinkAnnotation::goto_page(Rect::new(72.0, 720.0, 50.0, 12.0), 5);

        assert!(matches!(link.action, LinkAction::GoTo { page: 5, .. }));
    }

    #[test]
    fn test_link_annotation_build_uri() {
        let link = LinkAnnotation::uri(Rect::new(0.0, 0.0, 100.0, 20.0), "https://rust-lang.org");

        let page_refs = vec![ObjectRef::new(10, 0)];
        let dict = link.build(&page_refs);

        assert_eq!(dict.get("Type"), Some(&Object::Name("Annot".to_string())));
        assert_eq!(dict.get("Subtype"), Some(&Object::Name("Link".to_string())));
        assert!(dict.contains_key("A")); // Action
    }

    #[test]
    fn test_link_annotation_build_goto() {
        let link = LinkAnnotation::goto_page(Rect::new(0.0, 0.0, 100.0, 20.0), 0);

        let page_refs = vec![ObjectRef::new(10, 0), ObjectRef::new(11, 0)];
        let dict = link.build(&page_refs);

        assert!(dict.contains_key("Dest")); // Destination
    }

    #[test]
    fn test_annotation_builder() {
        let mut builder = AnnotationBuilder::new();
        assert!(builder.is_empty());

        builder.uri(Rect::new(0.0, 0.0, 50.0, 10.0), "https://example.com");
        builder.goto(Rect::new(100.0, 0.0, 50.0, 10.0), 1);

        assert_eq!(builder.len(), 2);
        assert!(!builder.is_empty());
    }

    #[test]
    fn test_link_with_styling() {
        let link = LinkAnnotation::uri(Rect::new(0.0, 0.0, 100.0, 20.0), "https://example.com")
            .with_border(BorderStyle::solid(1.0))
            .with_color(0.0, 0.0, 1.0)
            .with_highlight(HighlightMode::Push);

        assert_eq!(link.border.width, 1.0);
        assert_eq!(link.color, Some((0.0, 0.0, 1.0)));
        assert!(matches!(link.highlight, HighlightMode::Push));
    }

    // ========== Additional Coverage Tests ==========

    #[test]
    fn test_border_style_with_radius() {
        let border = BorderStyle::solid(2.0).with_radius(5.0);
        assert_eq!(border.horizontal_radius, 5.0);
        assert_eq!(border.vertical_radius, 5.0);
        assert_eq!(border.width, 2.0);
    }

    #[test]
    fn test_border_style_default() {
        let border = BorderStyle::default();
        assert_eq!(border.horizontal_radius, 0.0);
        assert_eq!(border.vertical_radius, 0.0);
        assert_eq!(border.width, 0.0);
        assert!(border.dash.is_none());
    }

    #[test]
    fn test_border_style_to_border_array_solid() {
        let border = BorderStyle::solid(1.5);
        let obj = border.to_border_array();
        match obj {
            Object::Array(arr) => {
                assert_eq!(arr.len(), 3);
            },
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_border_style_to_border_array_dashed() {
        let border = BorderStyle::dashed(2.0, 3.0, 1.0);
        let obj = border.to_border_array();
        match obj {
            Object::Array(arr) => {
                assert_eq!(arr.len(), 4); // 3 values + dash array
                                          // Last element should be an array
                match &arr[3] {
                    Object::Array(dash) => assert_eq!(dash.len(), 2),
                    _ => panic!("Expected dash array"),
                }
            },
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_border_style_to_border_array_with_radius() {
        let border = BorderStyle::solid(1.0).with_radius(3.0);
        let obj = border.to_border_array();
        match obj {
            Object::Array(arr) => {
                assert_eq!(arr.len(), 3);
                // First two should be the radius values
                assert_eq!(arr[0], Object::Real(3.0));
                assert_eq!(arr[1], Object::Real(3.0));
            },
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_highlight_mode_default() {
        let mode = HighlightMode::default();
        assert_eq!(mode.pdf_name(), "I"); // Invert is default
    }

    #[test]
    fn test_link_annotation_goto_named() {
        let link = LinkAnnotation::goto_named(Rect::new(0.0, 0.0, 50.0, 10.0), "chapter1");
        assert!(matches!(link.action, LinkAction::GoToNamed(_)));
    }

    #[test]
    fn test_link_annotation_build_goto_named() {
        let link = LinkAnnotation::goto_named(Rect::new(0.0, 0.0, 50.0, 10.0), "chapter1");
        let page_refs = vec![ObjectRef::new(10, 0)];
        let dict = link.build(&page_refs);
        assert!(dict.contains_key("Dest"));
        match dict.get("Dest") {
            Some(Object::String(s)) => {
                assert_eq!(s, b"chapter1");
            },
            _ => panic!("Expected string Dest"),
        }
    }

    #[test]
    fn test_link_annotation_build_goto_remote() {
        let link = LinkAnnotation {
            rect: Rect::new(0.0, 0.0, 100.0, 20.0),
            action: LinkAction::GoToRemote {
                file: "other.pdf".to_string(),
                page: 3,
            },
            border: BorderStyle::none(),
            highlight: HighlightMode::default(),
            color: None,
            quad_points: None,
        };
        let page_refs = vec![ObjectRef::new(10, 0)];
        let dict = link.build(&page_refs);
        assert!(dict.contains_key("A"));
        match dict.get("A") {
            Some(Object::Dictionary(action)) => {
                assert_eq!(action.get("S"), Some(&Object::Name("GoToR".to_string())));
            },
            _ => panic!("Expected action dict"),
        }
    }

    #[test]
    fn test_link_annotation_build_launch() {
        let link = LinkAnnotation {
            rect: Rect::new(0.0, 0.0, 100.0, 20.0),
            action: LinkAction::Launch("/usr/bin/app".to_string()),
            border: BorderStyle::none(),
            highlight: HighlightMode::default(),
            color: None,
            quad_points: None,
        };
        let page_refs = vec![];
        let dict = link.build(&page_refs);
        match dict.get("A") {
            Some(Object::Dictionary(action)) => {
                assert_eq!(action.get("S"), Some(&Object::Name("Launch".to_string())));
            },
            _ => panic!("Expected action dict"),
        }
    }

    #[test]
    fn test_link_annotation_build_javascript() {
        let link = LinkAnnotation {
            rect: Rect::new(0.0, 0.0, 100.0, 20.0),
            action: LinkAction::JavaScript("alert('hello')".to_string()),
            border: BorderStyle::none(),
            highlight: HighlightMode::default(),
            color: None,
            quad_points: None,
        };
        let page_refs = vec![];
        let dict = link.build(&page_refs);
        match dict.get("A") {
            Some(Object::Dictionary(action)) => {
                assert_eq!(action.get("S"), Some(&Object::Name("JavaScript".to_string())));
            },
            _ => panic!("Expected action dict"),
        }
    }

    #[test]
    fn test_link_annotation_build_with_color() {
        let link = LinkAnnotation::uri(Rect::new(0.0, 0.0, 100.0, 20.0), "https://example.com")
            .with_color(1.0, 0.0, 0.0);
        let dict = link.build(&[]);
        assert!(dict.contains_key("C"));
        match dict.get("C") {
            Some(Object::Array(arr)) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], Object::Real(1.0));
            },
            _ => panic!("Expected color array"),
        }
    }

    #[test]
    fn test_link_annotation_build_no_color() {
        let link = LinkAnnotation::uri(Rect::new(0.0, 0.0, 100.0, 20.0), "https://example.com");
        let dict = link.build(&[]);
        assert!(!dict.contains_key("C"));
    }

    #[test]
    fn test_link_annotation_with_quad_points() {
        let points = vec![0.0, 0.0, 100.0, 0.0, 100.0, 20.0, 0.0, 20.0];
        let link = LinkAnnotation::uri(Rect::new(0.0, 0.0, 100.0, 20.0), "https://example.com")
            .with_quad_points(points.clone());

        assert!(link.quad_points.is_some());
        let dict = link.build(&[]);
        assert!(dict.contains_key("QuadPoints"));
        match dict.get("QuadPoints") {
            Some(Object::Array(arr)) => assert_eq!(arr.len(), 8),
            _ => panic!("Expected quad points array"),
        }
    }

    #[test]
    fn test_link_annotation_build_goto_out_of_range() {
        // GoTo with a page index that doesn't exist
        let link = LinkAnnotation::goto_page(Rect::new(0.0, 0.0, 100.0, 20.0), 99);
        let page_refs = vec![ObjectRef::new(10, 0)]; // Only 1 page
        let dict = link.build(&page_refs);
        // Should not contain Dest since page 99 doesn't exist
        assert!(!dict.contains_key("Dest"));
    }

    #[test]
    fn test_link_build_goto_with_fit_mode() {
        use super::super::outline_builder::FitMode;
        let link = LinkAnnotation {
            rect: Rect::new(0.0, 0.0, 100.0, 20.0),
            action: LinkAction::GoTo {
                page: 0,
                fit: Some(FitMode::FitH(Some(500.0))),
            },
            border: BorderStyle::none(),
            highlight: HighlightMode::default(),
            color: None,
            quad_points: None,
        };
        let page_refs = vec![ObjectRef::new(10, 0)];
        let dict = link.build(&page_refs);
        assert!(dict.contains_key("Dest"));
    }

    #[test]
    fn test_link_build_goto_with_fit_mode_fit() {
        use super::super::outline_builder::FitMode;
        let link = LinkAnnotation {
            rect: Rect::new(0.0, 0.0, 100.0, 20.0),
            action: LinkAction::GoTo {
                page: 0,
                fit: Some(FitMode::Fit),
            },
            border: BorderStyle::none(),
            highlight: HighlightMode::default(),
            color: None,
            quad_points: None,
        };
        let page_refs = vec![ObjectRef::new(10, 0)];
        let dict = link.build(&page_refs);
        assert!(dict.contains_key("Dest"));
    }

    #[test]
    fn test_link_build_goto_with_fit_mode_fitv() {
        use super::super::outline_builder::FitMode;
        let link = LinkAnnotation {
            rect: Rect::new(0.0, 0.0, 100.0, 20.0),
            action: LinkAction::GoTo {
                page: 0,
                fit: Some(FitMode::FitV(Some(100.0))),
            },
            border: BorderStyle::none(),
            highlight: HighlightMode::default(),
            color: None,
            quad_points: None,
        };
        let page_refs = vec![ObjectRef::new(10, 0)];
        let dict = link.build(&page_refs);
        assert!(dict.contains_key("Dest"));
    }

    #[test]
    fn test_link_build_goto_with_fit_mode_fitr() {
        use super::super::outline_builder::FitMode;
        let link = LinkAnnotation {
            rect: Rect::new(0.0, 0.0, 100.0, 20.0),
            action: LinkAction::GoTo {
                page: 0,
                fit: Some(FitMode::FitR {
                    left: 0.0,
                    bottom: 0.0,
                    right: 612.0,
                    top: 792.0,
                }),
            },
            border: BorderStyle::none(),
            highlight: HighlightMode::default(),
            color: None,
            quad_points: None,
        };
        let page_refs = vec![ObjectRef::new(10, 0)];
        let dict = link.build(&page_refs);
        assert!(dict.contains_key("Dest"));
    }

    #[test]
    fn test_link_build_goto_with_fit_mode_fitb() {
        use super::super::outline_builder::FitMode;
        let link = LinkAnnotation {
            rect: Rect::new(0.0, 0.0, 100.0, 20.0),
            action: LinkAction::GoTo {
                page: 0,
                fit: Some(FitMode::FitB),
            },
            border: BorderStyle::none(),
            highlight: HighlightMode::default(),
            color: None,
            quad_points: None,
        };
        let page_refs = vec![ObjectRef::new(10, 0)];
        let dict = link.build(&page_refs);
        assert!(dict.contains_key("Dest"));
    }

    #[test]
    fn test_link_build_goto_with_fit_mode_fitbh() {
        use super::super::outline_builder::FitMode;
        let link = LinkAnnotation {
            rect: Rect::new(0.0, 0.0, 100.0, 20.0),
            action: LinkAction::GoTo {
                page: 0,
                fit: Some(FitMode::FitBH(Some(700.0))),
            },
            border: BorderStyle::none(),
            highlight: HighlightMode::default(),
            color: None,
            quad_points: None,
        };
        let page_refs = vec![ObjectRef::new(10, 0)];
        let dict = link.build(&page_refs);
        assert!(dict.contains_key("Dest"));
    }

    #[test]
    fn test_link_build_goto_with_fit_mode_fitbv() {
        use super::super::outline_builder::FitMode;
        let link = LinkAnnotation {
            rect: Rect::new(0.0, 0.0, 100.0, 20.0),
            action: LinkAction::GoTo {
                page: 0,
                fit: Some(FitMode::FitBV(None)),
            },
            border: BorderStyle::none(),
            highlight: HighlightMode::default(),
            color: None,
            quad_points: None,
        };
        let page_refs = vec![ObjectRef::new(10, 0)];
        let dict = link.build(&page_refs);
        assert!(dict.contains_key("Dest"));
    }

    #[test]
    fn test_link_build_goto_with_xyz() {
        use super::super::outline_builder::FitMode;
        let link = LinkAnnotation {
            rect: Rect::new(0.0, 0.0, 100.0, 20.0),
            action: LinkAction::GoTo {
                page: 0,
                fit: Some(FitMode::XYZ {
                    left: Some(72.0),
                    top: Some(720.0),
                    zoom: Some(1.5),
                }),
            },
            border: BorderStyle::none(),
            highlight: HighlightMode::default(),
            color: None,
            quad_points: None,
        };
        let page_refs = vec![ObjectRef::new(10, 0)];
        let dict = link.build(&page_refs);
        assert!(dict.contains_key("Dest"));
    }

    #[test]
    fn test_link_build_goto_with_fith_none() {
        use super::super::outline_builder::FitMode;
        let link = LinkAnnotation {
            rect: Rect::new(0.0, 0.0, 100.0, 20.0),
            action: LinkAction::GoTo {
                page: 0,
                fit: Some(FitMode::FitH(None)),
            },
            border: BorderStyle::none(),
            highlight: HighlightMode::default(),
            color: None,
            quad_points: None,
        };
        let page_refs = vec![ObjectRef::new(10, 0)];
        let dict = link.build(&page_refs);
        assert!(dict.contains_key("Dest"));
    }

    #[test]
    fn test_annotation_builder_default() {
        let builder = AnnotationBuilder::default();
        assert!(builder.is_empty());
        assert_eq!(builder.len(), 0);
    }

    #[test]
    fn test_annotation_builder_add_annotation() {
        let mut builder = AnnotationBuilder::new();
        let link = LinkAnnotation::uri(Rect::new(0.0, 0.0, 50.0, 10.0), "https://example.com");
        builder.add_annotation(link);
        assert_eq!(builder.len(), 1);
        assert!(!builder.is_empty());
    }

    #[test]
    fn test_annotation_builder_add_link() {
        let mut builder = AnnotationBuilder::new();
        let link = LinkAnnotation::uri(Rect::new(0.0, 0.0, 50.0, 10.0), "https://example.com");
        builder.add_link(link);
        assert_eq!(builder.len(), 1);
        assert_eq!(builder.links().len(), 1);
    }

    #[test]
    fn test_annotation_builder_links_filter() {
        let mut builder = AnnotationBuilder::new();
        builder.uri(Rect::new(0.0, 0.0, 50.0, 10.0), "https://example.com");
        builder.goto(Rect::new(100.0, 0.0, 50.0, 10.0), 1);

        let links = builder.links();
        assert_eq!(links.len(), 2);
    }

    #[test]
    fn test_annotation_builder_text_markups_filter() {
        let builder = AnnotationBuilder::new();
        assert!(builder.text_markups().is_empty());
    }

    #[test]
    fn test_annotation_builder_text_notes_filter() {
        let builder = AnnotationBuilder::new();
        assert!(builder.text_notes().is_empty());
    }

    #[test]
    fn test_annotation_builder_free_texts_filter() {
        let builder = AnnotationBuilder::new();
        assert!(builder.free_texts().is_empty());
    }

    #[test]
    fn test_annotation_builder_lines_filter() {
        let builder = AnnotationBuilder::new();
        assert!(builder.lines().is_empty());
    }

    #[test]
    fn test_annotation_builder_shapes_filter() {
        let builder = AnnotationBuilder::new();
        assert!(builder.shapes().is_empty());
    }

    #[test]
    fn test_annotation_builder_polygons_filter() {
        let builder = AnnotationBuilder::new();
        assert!(builder.polygons().is_empty());
    }

    #[test]
    fn test_annotation_builder_inks_filter() {
        let builder = AnnotationBuilder::new();
        assert!(builder.inks().is_empty());
    }

    #[test]
    fn test_annotation_builder_stamps_filter() {
        let builder = AnnotationBuilder::new();
        assert!(builder.stamps().is_empty());
    }

    #[test]
    fn test_annotation_builder_annotations_accessor() {
        let mut builder = AnnotationBuilder::new();
        builder.uri(Rect::new(0.0, 0.0, 50.0, 10.0), "https://example.com");
        assert_eq!(builder.annotations().len(), 1);
    }

    #[test]
    fn test_annotation_builder_build() {
        let mut builder = AnnotationBuilder::new();
        builder.uri(Rect::new(0.0, 0.0, 100.0, 20.0), "https://example.com");
        builder.goto(Rect::new(0.0, 0.0, 100.0, 20.0), 0);

        let page_refs = vec![ObjectRef::new(10, 0)];
        let dicts = builder.build(&page_refs);
        assert_eq!(dicts.len(), 2);
        for dict in &dicts {
            assert_eq!(dict.get("Type"), Some(&Object::Name("Annot".to_string())));
        }
    }

    #[test]
    fn test_annotation_from_link() {
        let link = LinkAnnotation::uri(Rect::new(0.0, 0.0, 50.0, 10.0), "https://example.com");
        let annot: Annotation = link.into();
        assert!(matches!(annot, Annotation::Link(_)));
    }

    #[test]
    fn test_annotation_rect_for_link() {
        let link = LinkAnnotation::uri(Rect::new(10.0, 20.0, 50.0, 10.0), "https://example.com");
        let annot: Annotation = link.into();
        let r = annot.rect();
        assert_eq!(r.x, 10.0);
        assert_eq!(r.y, 20.0);
    }

    #[test]
    fn test_link_build_rect_values() {
        let link = LinkAnnotation::uri(Rect::new(72.0, 720.0, 100.0, 12.0), "https://example.com");
        let dict = link.build(&[]);
        match dict.get("Rect") {
            Some(Object::Array(arr)) => {
                assert_eq!(arr.len(), 4);
                assert_eq!(arr[0], Object::Real(72.0));
                assert_eq!(arr[1], Object::Real(720.0));
                assert_eq!(arr[2], Object::Real(172.0)); // x + width
                assert_eq!(arr[3], Object::Real(732.0)); // y + height
            },
            _ => panic!("Expected Rect array"),
        }
    }

    #[test]
    fn test_link_build_highlight_mode_in_dict() {
        let link = LinkAnnotation::uri(Rect::new(0.0, 0.0, 100.0, 20.0), "https://example.com")
            .with_highlight(HighlightMode::Outline);
        let dict = link.build(&[]);
        assert_eq!(dict.get("H"), Some(&Object::Name("O".to_string())));
    }

    #[test]
    fn test_link_build_highlight_mode_none_in_dict() {
        let link = LinkAnnotation::uri(Rect::new(0.0, 0.0, 100.0, 20.0), "https://example.com")
            .with_highlight(HighlightMode::None);
        let dict = link.build(&[]);
        assert_eq!(dict.get("H"), Some(&Object::Name("N".to_string())));
    }

    #[test]
    fn test_link_build_border_in_dict() {
        let link = LinkAnnotation::uri(Rect::new(0.0, 0.0, 100.0, 20.0), "https://example.com")
            .with_border(BorderStyle::dashed(2.0, 3.0, 1.0));
        let dict = link.build(&[]);
        assert!(dict.contains_key("Border"));
    }
}
