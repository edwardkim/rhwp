//! Debug visualization module for PDF analysis.
//!
//! This module provides tools for visualizing PDF structure and elements,
//! useful for debugging layout analysis and content extraction.
//!
//! ## Features
//!
//! - Render pages with element bounding boxes overlaid
//! - Color-code elements by type (text, images, paths, tables)
//! - Export element tree to JSON/SVG
//!
//! ## Example
//!
//! ```ignore
//! use pdf_oxide::api::Pdf;
//! use pdf_oxide::debug::{DebugVisualizer, DebugOptions};
//!
//! let mut pdf = Pdf::open("document.pdf")?;
//! let visualizer = DebugVisualizer::new(DebugOptions::default());
//! visualizer.render_debug_page(&mut pdf, 0, "debug_page1.png")?;
//! ```

mod visualizer;

pub use visualizer::{DebugOptions, DebugVisualizer, ElementColors};
