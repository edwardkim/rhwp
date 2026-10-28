//! Text processing utilities for the extraction pipeline.
//!
//! This module provides various text post-processing operations that improve
//! extraction quality by normalizing whitespace, detecting citations, and
//! enhancing encoding fallback chains.

pub mod citations;
pub mod whitespace;

pub use citations::{Citation, CitationDetector, CitationType};
pub use whitespace::WhitespaceNormalizer;
