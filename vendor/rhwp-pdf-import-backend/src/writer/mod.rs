//! PDF writing module for generating PDF files.
//!
//! This module provides components for creating PDF files from ContentElements.
//!
//! ## Architecture
//!
//! ```text
//! ContentElement[]
//!     ↓
//! [DocumentBuilder] (high-level fluent API)
//!     ↓
//! [ContentStreamBuilder] (elements → content stream bytes)
//!     ↓
//! [PdfWriter] (assembles complete PDF structure)
//!     ↓
//! [ObjectSerializer] (serializes PDF objects)
//!     ↓
//! PDF bytes
//! ```
//!
//! ## High-Level API (DocumentBuilder)
//!
//! ```ignore
//! use pdf_oxide::writer::{DocumentBuilder, PageSize, DocumentMetadata};
//!
//! let bytes = DocumentBuilder::new()
//!     .metadata(DocumentMetadata::new().title("My Document"))
//!     .page(PageSize::Letter)
//!         .at(72.0, 720.0)
//!         .heading(1, "Hello, World!")
//!         .paragraph("This is a PDF document.")
//!         .done()
//!     .build()?;
//! ```
//!
//! ## Low-Level API (PdfWriter)
//!
//! ```ignore
//! use pdf_oxide::writer::{PdfWriter, ContentStreamBuilder};
//!
//! let mut writer = PdfWriter::new();
//! let page = writer.add_page(612.0, 792.0);
//! page.add_text("Hello, World!", 72.0, 720.0);
//! let bytes = writer.finish()?;
//! ```

mod acroform;
mod annotation_builder;
mod appearance_stream;
pub mod barcode;
mod content_stream;
mod document_builder;
mod embedded_files;
#[cfg(feature = "system-fonts")]
mod font_discovery;
mod font_manager;
mod font_pdf_objects;
#[cfg(feature = "system-fonts")]
mod font_shaping;
pub mod form_fields;
mod freetext;
mod graphics_state;
mod image_handler;
mod ink;
pub mod layers;
pub mod linearization;
mod movie;
mod object_serializer;
pub mod outline_builder;
mod page_labels;
mod page_template;
mod pattern;
mod pdf_writer;
mod richmedia;
mod screen;
mod shading;
mod shape_annotations;
mod sound;
mod special_annotations;
mod stamp;
mod streaming_table;
mod table_renderer;
mod text_annotations;
mod text_markup;
mod threed;
mod watermark;
mod xmp_metadata;

pub use acroform::AcroFormBuilder;
pub use annotation_builder::{
    Annotation, AnnotationBuilder, BorderStyle, HighlightMode, LinkAction, LinkAnnotation,
};
pub use appearance_stream::AppearanceStreamBuilder;
pub use barcode::{
    BarcodeGenerator, BarcodeOptions, BarcodeType, QrCodeOptions, QrErrorCorrection,
};
pub use content_stream::{
    BlendMode, ContentStreamBuilder, ContentStreamOp, LineCap, LineJoin, PendingImage,
    TextArrayItem,
};
// Crate-internal: shared Standard-14 name mapping so the editor's overlay-font
// registration keys off the exact name the content-stream writer emits.
pub(crate) use content_stream::map_base14_font_name;
pub use document_builder::{
    DocumentBuilder, DocumentMetadata, FluentPageBuilder, LineStyle, ListStyle, PageSize,
    TextAlign, TextConfig, TextRun, TextRunStyle,
};
pub use embedded_files::{AFRelationship, EmbeddedFile, EmbeddedFilesBuilder};
pub use font_manager::{
    EmbeddedFont, EmbeddedFontManager, FontFamily, FontInfo, FontManager, FontWeight, TextLayout,
};
pub use font_pdf_objects::{build_embedded_font_objects, EmbeddedFontIds, FontResourceName};

#[cfg(feature = "system-fonts")]
pub use font_discovery::{FontResolveError, FontStyle, ResolvedFont, SystemFontDb};
#[cfg(feature = "system-fonts")]
pub use font_shaping::{shape as shape_text, Direction as ShapeDirection, ShapedGlyph, ShapedRun};
pub use form_fields::{
    ButtonFieldFlags, CheckboxWidget, ChoiceFieldFlags, ChoiceOption, ComboBoxWidget, FieldFlags,
    FormAction, FormAppearanceGenerator, FormFieldEntry, FormFieldWidget, ListBoxWidget,
    PushButtonWidget, RadioButtonGroup, RadioButtonWidget, SignatureWidget, SubmitFormFlags,
    TextAlignment, TextFieldFlags, TextFieldWidget,
};
pub use freetext::FreeTextAnnotation;
pub use graphics_state::{ExtGStateBuilder, SoftMask, SoftMaskSubtype};
pub use image_handler::{ColorSpace, ImageData, ImageFormat, ImageManager, ImagePlacement};
pub use ink::InkAnnotation;
pub use layers::{
    Layer, LayerBuilder, LayerIntent, LayerMembership, LayerVisibility, VisibilityPolicy,
};
pub use linearization::{
    HintTables, LinearizationAnalyzer, LinearizationConfig, LinearizationParams,
    LinearizedPdfBuilder, ObjectInfo, PageOffsetEntry, PageOffsetHeader, SharedObjectEntry,
    SharedObjectHeader,
};
pub use movie::{MovieActivation, MovieAnnotation, MovieData, MoviePlayMode};
pub(crate) use object_serializer::hoist_appearance_streams;
pub use object_serializer::ObjectSerializer;
pub use outline_builder::{
    FitMode, OutlineBuildResult, OutlineBuilder, OutlineDestination, OutlineItem, OutlineStyle,
};
pub use page_labels::PageLabelsBuilder;
pub use page_template::{
    Artifact, ArtifactAlignment, ArtifactElement, ArtifactStyle, Footer, Header, PageNumberFormat,
    PageTemplate, Placeholder, PlaceholderContext,
};
pub use pattern::{
    PatternPaintType, PatternPresets, PatternTilingType, ShadingPatternBuilder,
    TilingPatternBuilder,
};
pub use pdf_writer::{PageBuilder, PdfWriter, PdfWriterConfig};
pub use richmedia::{
    RichMediaActivation, RichMediaAnnotation, RichMediaAsset, RichMediaContent,
    RichMediaDeactivation, RichMediaSettings, RichMediaWindow,
};
pub use screen::{
    MediaClip, MediaPermissions, MediaPlayParams, MediaRendition, RenditionOperation,
    ScreenAnnotation, TemporalAccess, WindowType,
};
pub use shading::{
    ColorSpace as ShadingColorSpace, GradientPresets, GradientStop, LinearGradientBuilder,
    RadialGradientBuilder,
};
pub use shape_annotations::{
    CaptionPosition, LineAnnotation, PolygonAnnotation, PolygonType, ShapeAnnotation, ShapeType,
};
pub use sound::{SoundAnnotation, SoundData, SoundEncoding, SoundIcon};
pub use special_annotations::{
    CaretAnnotation, CaretSymbol, FileAttachmentAnnotation, FileAttachmentIcon, PopupAnnotation,
    RedactAnnotation,
};
pub use stamp::{StampAnnotation, StampType};
pub use streaming_table::{StreamingColumn, StreamingRow, StreamingTable, StreamingTableConfig};
pub use table_renderer::{
    Borders, CellAlign, CellPadding, CellPosition, CellVAlign, ColumnWidth, FontMetrics,
    SimpleFontMetrics, Table, TableBorderStyle, TableCell, TableLayout, TableRow, TableStyle,
};
pub use text_annotations::TextAnnotation;
pub use text_markup::TextMarkupAnnotation;
pub use threed::{
    ThreeDActivation, ThreeDAnnotation, ThreeDBackground, ThreeDCamera, ThreeDDeactivation,
    ThreeDFormat, ThreeDLighting, ThreeDProjection, ThreeDRenderMode, ThreeDStream, ThreeDView,
};
pub use watermark::{FixedPrintSettings, WatermarkAnnotation};
pub use xmp_metadata::{iso_timestamp, XmpWriter};

use crate::elements::ContentElement;
use crate::error::Result;

/// Trait for building content streams from elements.
///
/// Content streams contain the PDF operators that render content on a page.
pub trait ContentBuilder: Send + Sync {
    /// Build a content stream from elements.
    fn build(&self, elements: &[ContentElement]) -> Result<Vec<u8>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify key types are exported
        let _serializer = ObjectSerializer::new();
        let _builder = ContentStreamBuilder::new();
    }
}
