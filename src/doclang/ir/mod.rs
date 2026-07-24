//! Semantic Intermediate Representation (SIR).
//!
//! This module is **rhwp-agnostic**: no rhwp types appear here.  Only
//! `src/parser_adapter/` is allowed to depend on rhwp; it produces a
//! `SirDocument` that the writer and loss modules consume.

pub mod block;
pub mod formula;
pub mod geometry;
pub mod inline;
pub mod prov;
pub mod style;
pub mod table;

pub use block::{Block, HeaderFooterApply, ListItem};
pub use formula::Formula;
pub use geometry::Geometry;
pub use inline::Inline;
pub use prov::{Location, Prov};
pub use style::{LostProperties, StyleFlags};
pub use table::{Table, TableCell};

/// The top-level Semantic IR document produced by the parser adapter.
///
/// Sections correspond to HWP `Section` objects (each with its own page
/// layout); most single-section documents produce exactly one entry.
#[derive(Debug, Clone, PartialEq)]
pub struct SirDocument {
    /// Ordered list of document sections.
    pub sections: Vec<Section>,
    /// DocLang version string to embed in the output root element.
    /// Typically `"0.6"` (from `ConvertOptions::doclang_version`).
    pub doclang_version: &'static str,
}

/// A single HWP section, corresponding to one continuous page-layout region.
///
/// Blocks are in document order; the writer emits them sequentially.
#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    /// Block-level content of this section.
    pub blocks: Vec<Block>,
}
