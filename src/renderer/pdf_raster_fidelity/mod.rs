//! #4764 PDF raster fidelity leftovers after #3820 page-count lock.
//!
//! #3820 / PR #4763 fixed physical page ownership. This module tracks the
//! leftover visual residuals — glyph, paint, wrap flow, font width/weight,
//! table placement — and isolates one page's raster failure from the rest of
//! the document. Runtime decisions use geometry and ink, not document names.
//!
//! #3772 ExtraLight bold and #3773 svg2pdf `SubsetError` isolation stay on
//! their own PRs. This path does not rewrite those font-chain or subsetter
//! loops.

mod catalog;
mod classify;
mod contract;
mod fingerprint;
mod isolate;
mod pdf_page;
mod report;
mod wrap;

pub use catalog::{
    load_page_catalog, load_page_catalog_from_path, CatalogLoadError, CorpusId, IsolationPolicy,
    PageRecord, SourceKind,
};
pub use classify::{classify_page_residual, ClassificationInput, ClassificationLimits};
pub use contract::{
    locked_page_count, page_count_matches_lock, ADMIN_HANDBOOK_PAGE_COUNT, ISSUE2006_PAGE_COUNT,
    ISSUE4090_PAGE_COUNT, ISSUE4490_PAGE_COUNT, ISSUE4491_PAGE_COUNT, NOTE_TAIL_PAGE_COUNT,
    REGULATORY_76076_PAGE_COUNT,
};
pub use fingerprint::{
    compare_fingerprints, render_synthetic_page, FingerprintDelta, RasterFingerprint,
    RasterPageSpec, RasterPrimitive, RgbaPage,
};
pub use isolate::{compare_document_pages, page_input, IsolatedPageInput, IsolatedRasterReport};
pub use pdf_page::{
    build_multipage_pdf, extract_isolated_page, parse_pdf_page_tree, rect_content, IsolatedPdfPage,
    PdfBuildPage, PdfPageTree, PdfParseError,
};
pub use report::{
    IsolatedPageOutcome, IsolatedPageStatus, ResidualDocumentReport, ResidualPageReport,
};
pub use wrap::{
    left_strip_text_deficit, WrapGeometry, WrapStripSample, LEFT_STRIP_PDF_INK_MIN,
    LEFT_STRIP_RHWP_RATIO_MAX,
};

/// Residual class for one isolated page. `FontEnv` is not a layout defect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResidualClass {
    None,
    Glyph,
    Paint,
    WrapFlow,
    FontWidth,
    FontWeight,
    TablePlace,
    FontEnv,
}

impl ResidualClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Glyph => "glyph",
            Self::Paint => "paint",
            Self::WrapFlow => "wrap_flow",
            Self::FontWidth => "font_width",
            Self::FontWeight => "font_weight",
            Self::TablePlace => "table_place",
            Self::FontEnv => "font_env",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "none" | "clean" => Some(Self::None),
            "glyph" => Some(Self::Glyph),
            "paint" => Some(Self::Paint),
            "wrap_flow" | "wrap" => Some(Self::WrapFlow),
            "font_width" => Some(Self::FontWidth),
            "font_weight" => Some(Self::FontWeight),
            "table_place" | "table" => Some(Self::TablePlace),
            "font_env" | "font-environment" => Some(Self::FontEnv),
            _ => None,
        }
    }

    /// Font-environment-only residuals must not trip layout or page-count gates.
    pub fn is_layout_defect(self) -> bool {
        matches!(
            self,
            Self::WrapFlow | Self::TablePlace | Self::Paint | Self::Glyph
        )
    }

    pub fn is_font_surface(self) -> bool {
        matches!(self, Self::FontWidth | Self::FontWeight | Self::FontEnv)
    }
}

impl core::fmt::Display for ResidualClass {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}
