//! PDF compliance validation and conversion module.
//!
//! This module provides functionality for validating PDF documents against
//! PDF/A (archival), PDF/UA (accessibility), and PDF/X (print production)
//! standards, as well as converting documents to these standards.
//!
//! ## PDF/A Conformance Levels (Archival)
//!
//! - **PDF/A-1b**: Basic conformance, visual appearance preservation
//! - **PDF/A-1a**: Full conformance, includes logical structure (Tagged PDF)
//! - **PDF/A-2b**: Based on PDF 1.7, allows JPEG2000, transparency
//! - **PDF/A-2a**: PDF/A-2b plus logical structure
//! - **PDF/A-2u**: PDF/A-2b plus Unicode mapping
//! - **PDF/A-3b**: PDF/A-2b plus embedded files of any type
//! - **PDF/A-3a**: PDF/A-3b plus logical structure
//! - **PDF/A-3u**: PDF/A-3b plus Unicode mapping
//!
//! ## PDF/X Conformance Levels (Print Production)
//!
//! - **PDF/X-1a**: CMYK and spot colors only, no transparency
//! - **PDF/X-3**: Allows ICC-based color management
//! - **PDF/X-4**: Allows transparency and layers
//! - **PDF/X-5**: Allows external graphics and ICC profiles
//! - **PDF/X-6**: Based on PDF 2.0
//!
//! ## PDF/UA Requirements (Accessibility)
//!
//! - Document must be a Tagged PDF
//! - All content must be part of the structure tree
//! - Language must be specified
//! - Images must have alternative text
//! - Tables must have proper headers
//! - Form fields must have accessible names
//!
//! ## Example
//!
//! ```ignore
//! use pdf_oxide::api::Pdf;
//! use pdf_oxide::compliance::{
//!     PdfAValidator, PdfALevel, PdfUaValidator, PdfUaLevel,
//!     PdfXValidator, PdfXLevel, validate_pdf_x,
//! };
//!
//! let mut pdf = Pdf::open("document.pdf")?;
//!
//! // Validate against PDF/A-2b
//! let pdf_a_result = validate_pdf_a(&mut pdf.document()?, PdfALevel::A2b)?;
//! if pdf_a_result.is_compliant {
//!     println!("Document is PDF/A-2b compliant");
//! }
//!
//! // Validate against PDF/X-1a:2003 (print production)
//! let pdf_x_result = validate_pdf_x(&mut pdf.document()?, PdfXLevel::X1a2003)?;
//! if pdf_x_result.is_compliant {
//!     println!("Document is PDF/X-1a:2003 compliant");
//! }
//!
//! // Convert to PDF/A-2b
//! let conversion_result = convert_to_pdf_a(&mut pdf.document()?, PdfALevel::A2b)?;
//! if conversion_result.success {
//!     println!("Document converted to PDF/A-2b");
//! }
//!
//! // Validate against PDF/UA-1
//! let pdf_ua_result = validate_pdf_ua(&mut pdf.document()?, PdfUaLevel::Ua1)?;
//! if pdf_ua_result.is_compliant {
//!     println!("Document is PDF/UA-1 compliant");
//! }
//! ```
//!
//! ## Standards Reference
//!
//! - ISO 19005-1:2005 (PDF/A-1)
//! - ISO 19005-2:2011 (PDF/A-2)
//! - ISO 19005-3:2012 (PDF/A-3)
//! - ISO 15930-1:2001 (PDF/X-1a:2001)
//! - ISO 15930-4:2003 (PDF/X-1a:2003)
//! - ISO 15930-6:2003 (PDF/X-3:2003)
//! - ISO 15930-7:2010 (PDF/X-4)
//! - ISO 15930-8:2010 (PDF/X-5)
//! - ISO 15930-9:2020 (PDF/X-6)
//! - ISO 14289-1:2014 (PDF/UA-1)

mod converter;
pub(crate) mod pdf_a;
pub(crate) mod pdf_ua;
pub(crate) mod pdf_x;
pub(crate) mod types;
mod validators;

pub use converter::{
    convert_to_pdf_a, ActionType, ConversionAction, ConversionConfig, ConversionError,
    ConversionResult, PdfAConverter,
};
pub use pdf_a::{validate_pdf_a, PdfAValidator};
pub use pdf_ua::{
    validate_pdf_ua, PdfUaLevel, PdfUaValidator, UaComplianceError, UaErrorCode,
    UaValidationResult, UaValidationStats,
};
pub use pdf_x::{
    validate_pdf_x, PdfXLevel, PdfXValidator, XComplianceError, XErrorCode, XSeverity,
    XValidationResult, XValidationStats,
};
pub use types::{
    ComplianceError, ComplianceWarning, ErrorCode, PdfALevel, PdfAPart, ValidationResult,
    ValidationStats, WarningCode,
};
