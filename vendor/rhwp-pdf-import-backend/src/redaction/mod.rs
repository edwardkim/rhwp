//! True / destructive redaction and document sanitization (#231).
//!
//! Replaces the prior *cosmetic* redaction (a filled rectangle drawn over
//! content whose underlying bytes survived) with physical content removal
//! and a document-wide sanitization pass, per ISO 32000-1:2008 §12.5.6.23:
//! *"shall remove all traces of the specified content … clipping or image
//! masks shall not be used to hide that data."*
//!
//! The capability is built incrementally per the feature plan tracked in
//! <https://github.com/yfedoseev/pdf_oxide/issues/231>. One responsibility
//! per submodule (SRP); the geometric region model lands first and is the
//! shared input to every pruner. The
//! pruners (text/image/path/xobject), the font scrubber, the sanitizer and
//! the orchestrating engine follow as subsequent submodules.

#![forbid(unsafe_code)]

pub mod classify;
pub mod engine;
pub mod font_scrub;
pub mod image_prune;
pub mod options;
pub mod overlay;
pub mod path_prune;
pub mod region;
pub mod sanitize;
pub mod serialize;
pub mod text_engine;
pub mod text_prune;

pub use classify::Classification;
pub use engine::{redact_content_stream, FontInfoMetrics};
pub use options::{OcgPolicy, RedactionOptions, RedactionReport};
pub use region::{RedactionRegion, RegionSet, DEFAULT_EDGE_PADDING};
pub use sanitize::{sanitize_catalog, CatalogScrub, SanitizeCounts};
