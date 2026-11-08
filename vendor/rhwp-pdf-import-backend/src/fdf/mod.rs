//! Forms Data Format (FDF) support for exporting/importing form field values.
//!
//! This module provides functionality to export form field data to:
//! - **FDF** (Forms Data Format): Binary format per ISO 32000-1:2008 Section 12.7.7
//! - **XFDF** (XML Forms Data Format): XML representation of FDF
//!
//! ## Use Cases
//!
//! - Exporting form data for batch processing
//! - Pre-filling forms with external data
//! - Data exchange between PDF viewers/tools
//! - Web-based form data handling
//!
//! ## Example
//!
//! ```ignore
//! use pdf_oxide::fdf::{FdfWriter, XfdfWriter};
//! use pdf_oxide::extractors::forms::FormExtractor;
//!
//! // Export form data to FDF
//! let mut doc = PdfDocument::open("form.pdf")?;
//! let fields = FormExtractor::extract_fields(&mut doc)?;
//! let fdf_writer = FdfWriter::from_fields(fields);
//! fdf_writer.write_to_file("form_data.fdf")?;
//!
//! // Export form data to XFDF
//! let xfdf_writer = XfdfWriter::from_fields(fields);
//! xfdf_writer.write_to_file("form_data.xfdf")?;
//! ```

mod fdf_writer;
mod xfdf_writer;

pub use fdf_writer::{FdfField, FdfValue, FdfWriter};
pub use xfdf_writer::XfdfWriter;
