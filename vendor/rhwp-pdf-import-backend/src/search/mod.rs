//! Text search functionality for PDF documents.
//!
//! This module provides regex-based text search with position tracking,
//! returning bounding boxes for each match. Supports:
//! - Single page and multi-page search
//! - Regular expression patterns
//! - Case-insensitive search
//! - Match highlighting
//!
//! ## Example
//!
//! ```ignore
//! use pdf_oxide::api::Pdf;
//! use pdf_oxide::search::SearchOptions;
//!
//! let mut pdf = Pdf::open("document.pdf")?;
//!
//! // Simple text search
//! let results = pdf.search("hello")?;
//! for result in results {
//!     println!("Found '{}' on page {} at {:?}", result.text, result.page, result.bbox);
//! }
//!
//! // Regex search with options
//! let results = pdf.search_with_options(r"\d+", SearchOptions::case_insensitive())?;
//!
//! // Highlight matches
//! pdf.highlight_matches(&results, [1.0, 1.0, 0.0])?;  // Yellow
//! pdf.save("highlighted.pdf")?;
//! ```

mod text_search;

pub(crate) use text_search::SearchPageIndex;
pub use text_search::{SearchOptions, SearchResult, TextSearcher};
