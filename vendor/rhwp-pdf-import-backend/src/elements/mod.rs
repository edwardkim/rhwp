//! Unified content elements for read/write operations.
//!
//! This module provides a unified intermediate representation for PDF content
//! that works for both reading (extraction) and writing (generation).
//!
//! ## Design
//!
//! The `ContentElement` enum represents all content types that can be:
//! - Extracted from existing PDFs (read path)
//! - Generated into new PDFs (write path)
//!
//! This symmetric design enables round-trip operations (read → transform → write)
//! and provides a consistent API for both directions.
//!
//! ## Example
//!
//! ```ignore
//! use pdf_oxide::elements::{ContentElement, TextContent, FontSpec};
//! use pdf_oxide::geometry::Rect;
//!
//! // Create a text element (for writing)
//! let text = TextContent { artifact_type: None,
//!     text: "Hello, World!".to_string(),
//!     bbox: Rect::new(72.0, 720.0, 100.0, 12.0),
//!     font: FontSpec::default(),
//!     style: Default::default(),
//!     reading_order: Some(0),
//! };
//!
//! let element = ContentElement::Text(text);
//! ```

mod image;
mod path;
mod table;
mod text;

pub use image::{ColorSpace, ImageContent, ImageFormat};
pub use path::{
    LineCap, LineJoin, PathClip, PathClipObservation, PathClipObservations, PathContent,
    PathOperation, PathPaint,
};
pub use table::{
    TableCellAlign, TableCellContent, TableCellVAlign, TableContent, TableContentStyle,
    TableDetectionInfo, TableRowContent, TableSource,
};
pub use text::{FontSpec, FontStyle, TextContent, TextStyle};

// Note: StructureElement is defined inline in this module, not in a submodule

use crate::geometry::Rect;

/// A content element that can be extracted from or written to a PDF.
///
/// This is the unified intermediate representation used by both the
/// read pipeline (PDF → ContentElement) and write pipeline (ContentElement → PDF).
#[derive(Debug, Clone)]
pub enum ContentElement {
    /// Text content with positioning and styling
    Text(TextContent),
    /// Image content with position and format information
    Image(ImageContent),
    /// Vector path/graphics content
    Path(PathContent),
    /// Structural element (for Tagged PDF support)
    Structure(StructureElement),
    /// Table content with rows and cells
    Table(TableContent),
}

impl ContentElement {
    /// Get the bounding box of this element.
    pub fn bbox(&self) -> Rect {
        match self {
            ContentElement::Text(t) => t.bbox,
            ContentElement::Image(i) => i.bbox,
            ContentElement::Path(p) => p.bbox,
            ContentElement::Structure(s) => s.bbox,
            ContentElement::Table(t) => t.bbox,
        }
    }

    /// Get the reading order of this element, if assigned.
    pub fn reading_order(&self) -> Option<usize> {
        match self {
            ContentElement::Text(t) => t.reading_order,
            ContentElement::Image(i) => i.reading_order,
            ContentElement::Path(p) => p.reading_order,
            ContentElement::Structure(s) => s.reading_order,
            ContentElement::Table(t) => t.reading_order,
        }
    }

    /// Check if this is a text element.
    pub fn is_text(&self) -> bool {
        matches!(self, ContentElement::Text(_))
    }

    /// Check if this is an image element.
    pub fn is_image(&self) -> bool {
        matches!(self, ContentElement::Image(_))
    }

    /// Check if this is a path element.
    pub fn is_path(&self) -> bool {
        matches!(self, ContentElement::Path(_))
    }

    /// Check if this is a table element.
    pub fn is_table(&self) -> bool {
        matches!(self, ContentElement::Table(_))
    }

    /// Get as text content if this is a text element.
    pub fn as_text(&self) -> Option<&TextContent> {
        match self {
            ContentElement::Text(t) => Some(t),
            _ => None,
        }
    }

    /// Get as image content if this is an image element.
    pub fn as_image(&self) -> Option<&ImageContent> {
        match self {
            ContentElement::Image(i) => Some(i),
            _ => None,
        }
    }

    /// Get as path content if this is a path element.
    pub fn as_path(&self) -> Option<&PathContent> {
        match self {
            ContentElement::Path(p) => Some(p),
            _ => None,
        }
    }

    /// Get as table content if this is a table element.
    pub fn as_table(&self) -> Option<&TableContent> {
        match self {
            ContentElement::Table(t) => Some(t),
            _ => None,
        }
    }
}

/// A structural element for Tagged PDF support.
///
/// Represents a node in the PDF structure tree that groups
/// other content elements semantically.
#[derive(Debug, Clone)]
pub struct StructureElement {
    /// Structure type (e.g., "P", "H1", "Table", "Figure")
    pub structure_type: String,
    /// Bounding box encompassing all child elements
    pub bbox: Rect,
    /// Children content elements
    pub children: Vec<ContentElement>,
    /// Reading order index
    pub reading_order: Option<usize>,
    /// Alternate text (for accessibility)
    pub alt_text: Option<String>,
    /// Language tag (e.g., "en-US")
    pub language: Option<String>,
}

impl Default for StructureElement {
    fn default() -> Self {
        Self {
            structure_type: String::new(),
            bbox: Rect::new(0.0, 0.0, 0.0, 0.0),
            children: Vec::new(),
            reading_order: None,
            alt_text: None,
            language: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_element_bbox() {
        let text = TextContent {
            artifact_type: None,
            text: "Test".to_string(),
            bbox: Rect::new(10.0, 20.0, 50.0, 12.0),
            font: FontSpec::default(),
            style: TextStyle::default(),
            reading_order: Some(0),
            origin: None,
            rotation_degrees: None,
            matrix: None,
        };

        let element = ContentElement::Text(text);
        let bbox = element.bbox();

        assert_eq!(bbox.x, 10.0);
        assert_eq!(bbox.y, 20.0);
        assert_eq!(bbox.width, 50.0);
        assert_eq!(bbox.height, 12.0);
    }

    #[test]
    fn test_content_element_type_checks() {
        let text = ContentElement::Text(TextContent {
            artifact_type: None,
            text: "Test".to_string(),
            bbox: Rect::new(0.0, 0.0, 10.0, 10.0),
            font: FontSpec::default(),
            style: TextStyle::default(),
            reading_order: None,
            origin: None,
            rotation_degrees: None,
            matrix: None,
        });

        assert!(text.is_text());
        assert!(!text.is_image());
        assert!(!text.is_path());
        assert!(text.as_text().is_some());
        assert!(text.as_image().is_none());
    }

    #[test]
    fn test_reading_order() {
        let text = ContentElement::Text(TextContent {
            artifact_type: None,
            text: "First".to_string(),
            bbox: Rect::new(0.0, 0.0, 10.0, 10.0),
            font: FontSpec::default(),
            style: TextStyle::default(),
            reading_order: Some(5),
            origin: None,
            rotation_degrees: None,
            matrix: None,
        });

        assert_eq!(text.reading_order(), Some(5));
    }
}
