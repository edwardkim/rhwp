//! Text and content extraction from PDF documents.
//!
//! Provides high-performance extraction of text, images, paths, and layout analysis.

pub mod auto;
pub mod ccitt_bilevel;
pub mod forms;
pub mod gap_statistics;
pub mod geometric_spacing;
pub mod hierarchical;
pub mod image_clip;
mod image_clip_path;
pub mod images;
mod jpeg_input;
mod jpeg_limited;
pub mod page_labels;
pub mod paths;
pub mod pattern_detector;
mod png_limited;
pub mod status;
pub mod structured;
pub mod synthetic_structure;
pub mod text;
pub mod warnings;
pub mod xmp;

#[cfg(feature = "debug-span-merging")]
pub mod debug_span_merging;

pub use auto::{
    AutoExtractOptions, AutoExtractOptionsBuilder, AutoExtractor, DocumentClassification,
    DocumentExtraction, DocumentSummary, ExtractMode, ExtractSource, ExtractionStatus,
    ImageCodecClass, OcrLanguage, OcrModelSpec, PageClassification, PageExtraction, PageKind,
    PageSignals, ProducerPrior, Quad, ReasonCode, Region, RegionKind, TableData,
};
pub use forms::{FieldType, FieldValue, FormExtractor, FormField};
pub use gap_statistics::{
    analyze_document_gaps, calculate_statistics, determine_adaptive_threshold, extract_gaps,
    AdaptiveThresholdConfig, AdaptiveThresholdResult, GapStatistics,
};
pub use geometric_spacing::{should_insert_space, SpaceInsertion, SpacingConfig};
pub use hierarchical::HierarchicalExtractor;
pub use images::{
    expand_inline_image_dict, extract_image_from_xobject, ColorSpace, ImageData, PdfImage,
    PixelFormat,
};
pub use page_labels::{PageLabelExtractor, PageLabelRange, PageLabelStyle};
pub use paths::{FillRule, PathExtractor};
pub use pattern_detector::{PatternDetector, PatternPreservationConfig};
pub use status::OcrUnavailableReason;
pub use structured::{
    BoundingBox, DocumentElement, DocumentMetadata, ExtractorConfig, ListItem, StructuredDocument,
    StructuredExtractor, TextAlignment, TextStyle,
};
pub use synthetic_structure::{SyntheticStructureConfig, SyntheticStructureGenerator};
pub use text::{SpanMergingConfig, TextExtractionConfig, TextExtractor};
pub use warnings::{Warning, WarningCategory, WarningSink};
pub use xmp::{XmpExtractor, XmpMetadata};

mod jpeg_cmyk_limited;
