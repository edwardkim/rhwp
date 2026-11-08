//! XFA (XML Forms Architecture) support.
//!
//! This module provides limited support for XFA forms in PDF documents.
//! XFA is an XML-based form specification used in some PDFs, particularly
//! government and financial forms.
//!
//! # Features
//!
//! - Parse XFA template and datasets
//! - Extract field definitions and values
//! - Convert static XFA to AcroForm
//!
//! # Limitations
//!
//! This implementation supports **static conversion only**:
//! - Extracts field definitions and current values
//! - Converts fields to equivalent AcroForm types
//! - Uses simple vertical stacking layout
//!
//! **NOT supported:**
//! - Dynamic XFA features (scripts, calculations, conditional logic)
//! - Complex layouts (tables, grids, repeating sections)
//! - XFA-specific UI elements (subforms with special behavior)
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::xfa::{XfaExtractor, XfaParser, XfaConverter};
//! use pdf_oxide::PdfDocument;
//!
//! let mut doc = PdfDocument::open("form.pdf")?;
//!
//! // Check if document has XFA form
//! if XfaExtractor::has_xfa(&mut doc)? {
//!     // Extract and parse XFA data
//!     let xfa_data = XfaExtractor::extract_xfa(&mut doc)?;
//!
//!     let mut parser = XfaParser::new();
//!     let form = parser.parse(&xfa_data)?;
//!
//!     // Convert to AcroForm
//!     let converter = XfaConverter::new();
//!     let result = converter.convert(&form)?;
//!     println!("Converted {} fields", result.field_count);
//! }
//! ```

mod converter;
mod extractor;
mod integration;
mod parser;

pub use converter::{
    ConvertedField, ConvertedPage, XfaConversionOptions, XfaConversionResult, XfaConverter,
};
pub use extractor::XfaExtractor;
pub use integration::{
    add_converted_field, add_converted_page, analyze_xfa_document, convert_xfa_document,
    XfaAnalysis,
};
pub use parser::{is_xfa_data, XfaField, XfaFieldType, XfaForm, XfaOption, XfaPage, XfaParser};
