//! Reading order strategies for text extraction.
//!
//! This module provides pluggable strategies for determining the reading order
//! of text spans extracted from PDF pages.
//!
//! # Available Strategies
//!
//! - [`StructureTreeStrategy`]: Uses PDF structure tree MCIDs for Tagged PDFs
//! - [`GeometricStrategy`]: Column-aware geometric analysis
//! - [`XYCutStrategy`]: Recursive XY-Cut spatial partitioning (newspapers, academic papers)
//! - [`SimpleStrategy`]: Simple top-to-bottom, left-to-right ordering

pub mod article_thread;
pub mod detectors;
pub mod geometric;
pub mod simple;
pub mod structure_tree;
pub mod tategaki;
pub mod xycut;

pub use article_thread::ArticleThreadStrategy;
pub use detectors::{
    classify_region, detect_dense_single_line, detect_dramatic_script, detect_narrow_tracked,
    detect_sub_super_glyphs, DetectorGlyph, ReadingOrderClass,
};
pub use geometric::GeometricStrategy;
pub use simple::SimpleStrategy;
pub use structure_tree::StructureTreeStrategy;
pub use tategaki::TategakiStrategy;
pub use xycut::XYCutStrategy;

use crate::error::Result;
use crate::geometry::Rect;
use crate::layout::TextSpan;
use crate::pipeline::config::ReadingOrderStrategyType;
use crate::pipeline::OrderedTextSpan;

/// Trait for determining reading order of text spans.
///
/// Implementations decide how to order spans for reading. This is a key
/// abstraction point as different PDF types require different strategies:
///
/// - Tagged PDFs: Use structure tree MCIDs (PDF-spec-compliant)
/// - Multi-column: Use geometric column detection
/// - Simple: Top-to-bottom, left-to-right
pub trait ReadingOrderStrategy: Send + Sync {
    /// Apply reading order to a collection of spans.
    ///
    /// # Arguments
    ///
    /// * `spans` - Unordered text spans extracted from the page
    /// * `context` - Optional context information (structure tree, page info)
    ///
    /// # Returns
    ///
    /// Spans with assigned reading order indices.
    fn apply(
        &self,
        spans: Vec<TextSpan>,
        context: &ReadingOrderContext,
    ) -> Result<Vec<OrderedTextSpan>>;

    /// Return the name of this strategy for debugging.
    fn name(&self) -> &'static str;
}

/// Context information for reading order determination.
#[derive(Debug, Default)]
pub struct ReadingOrderContext {
    /// Current page number (0-indexed).
    pub page_number: u32,

    /// Page bounding box (if available).
    pub page_bbox: Option<Rect>,

    /// Whether the document has a structure tree (Tagged PDF).
    pub has_structure_tree: bool,

    /// Marked-content reading order from the structure tree, if available.
    ///
    /// Each entry is `(scope, mcid)`: ISO 32000-1:2008 §14.7.4.2 makes an
    /// `/MCID` unique only "within its content stream", so a page and a Form
    /// XObject may each number theirs from 0 and the bare id does not identify
    /// a piece of marked content. This carried bare ids, and the scope — which
    /// the structure traversal computes correctly — was projected away at the
    /// boundary, so a form's MCID 0 was ordered into the page's slot.
    pub mcid_order: Option<Vec<crate::pipeline::reading_order::structure_tree::McidKey>>,

    /// Ordered article-thread bead rectangles for this page, in `/N` order
    /// (ISO 32000-1:2008 §12.4.3). Set only when the document declares
    /// `/Threads`, no trustworthy structure tree governs the page, and the
    /// beads cover enough of the page text to be trusted. When present, the
    /// reading-order strategy threads spans through these regions instead of
    /// using pure geometric inference. `None` ⇒ the geometric path is unchanged
    /// (fails closed).
    pub bead_rects: Option<Vec<Rect>>,

    /// Whether the structure tree contains suspect (unreliable) content.
    ///
    /// Per ISO 32000-1:2008 Section 14.7.1, when this is true, the structure
    /// tree may be unreliable and reading order strategies should consider
    /// falling back to geometric ordering.
    pub suspects: bool,

    /// Preserve the caller-supplied span order verbatim instead of running a
    /// reading-order strategy. Set when the converter has already established a
    /// non-geometric order the strategies cannot reproduce — currently the
    /// two-column-prose column-major reorder (#734), whose ordering the
    /// geometric XY-cut fallback would otherwise re-derive as row-major and
    /// interleave the columns. Ignored when `mcid_order` is present (a tagged
    /// PDF's structure order wins).
    pub preserve_input_order: bool,
}

impl ReadingOrderContext {
    /// Create a new empty context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the page number.
    pub fn with_page(mut self, page_number: u32) -> Self {
        self.page_number = page_number;
        self
    }

    /// Set the page bounding box.
    pub fn with_bbox(mut self, bbox: Rect) -> Self {
        self.page_bbox = Some(bbox);
        self
    }

    /// Set MCID order from structure tree traversal.
    pub fn with_mcid_order(
        mut self,
        mcid_order: Vec<crate::pipeline::reading_order::structure_tree::McidKey>,
    ) -> Self {
        self.has_structure_tree = true;
        self.mcid_order = Some(mcid_order);
        self
    }

    /// Set whether the structure tree contains suspect content.
    ///
    /// When true, the StructureTreeStrategy will fall back to geometric
    /// ordering instead of trusting the potentially unreliable structure tree.
    pub fn with_suspects(mut self, suspects: bool) -> Self {
        self.suspects = suspects;
        self
    }

    /// Preserve the caller-supplied span order (see `preserve_input_order`).
    pub fn with_preserve_input_order(mut self, preserve: bool) -> Self {
        self.preserve_input_order = preserve;
        self
    }

    /// Set the ordered article-thread bead rectangles for this page.
    pub fn with_bead_rects(mut self, bead_rects: Vec<Rect>) -> Self {
        self.bead_rects = Some(bead_rects);
        self
    }
}

/// Create a reading order strategy based on configuration.
pub fn create_strategy(
    config: &crate::pipeline::ReadingOrderConfig,
) -> Box<dyn ReadingOrderStrategy> {
    match config.strategy {
        ReadingOrderStrategyType::StructureTreeFirst => Box::new(StructureTreeStrategy::new()),
        ReadingOrderStrategyType::Geometric => Box::new(GeometricStrategy::new()),
        ReadingOrderStrategyType::XYCut => Box::new(XYCutStrategy::new()),
        ReadingOrderStrategyType::Simple => Box::new(SimpleStrategy),
    }
}
