//! PDF/X compliance validation module.
//!
//! This module provides functionality for validating PDF documents against
//! PDF/X standards (ISO 15930) for print production workflows.
//!
//! ## PDF/X Standards
//!
//! - **PDF/X-1a**: CMYK and spot colors only, no transparency (ISO 15930-1, 15930-4)
//! - **PDF/X-3**: Allows ICC-based color management (ISO 15930-3, 15930-6)
//! - **PDF/X-4**: Allows transparency and layers (ISO 15930-7)
//! - **PDF/X-5**: Allows external graphics and ICC profiles (ISO 15930-8)
//! - **PDF/X-6**: Based on PDF 2.0 (ISO 15930-9)
//!
//! ## Example
//!
//! ```ignore
//! use pdf_oxide::api::Pdf;
//! use pdf_oxide::compliance::{PdfXValidator, PdfXLevel, validate_pdf_x};
//!
//! let mut pdf = Pdf::open("document.pdf")?;
//!
//! // Quick validation
//! let result = validate_pdf_x(&mut pdf.document()?, PdfXLevel::X1a2003)?;
//! if result.is_compliant {
//!     println!("Document is PDF/X-1a:2003 compliant");
//! }
//!
//! // Detailed validation with custom options
//! let validator = PdfXValidator::new(PdfXLevel::X4)
//!     .stop_on_first_error(false)
//!     .include_warnings(true);
//! let result = validator.validate(&mut pdf.document()?)?;
//!
//! for error in &result.errors {
//!     println!("Error: {}", error);
//! }
//! for warning in &result.warnings {
//!     println!("Warning: {}", warning);
//! }
//! ```
//!
//! ## Key Requirements by Level
//!
//! | Requirement | X-1a | X-3 | X-4 | X-5 |
//! |-------------|------|-----|-----|-----|
//! | Output Intent | Required | Required | Required | Required |
//! | CMYK Only | Yes | No | No | No |
//! | Transparency | No | No | Yes | Yes |
//! | Layers (OCG) | No | No | Yes | Yes |
//! | Embedded Fonts | Yes | Yes | Yes | Yes |
//! | External ICC | No | No | X-4p | X-5n |
//! | External Graphics | No | No | No | X-5g |

pub(crate) mod types;
pub(crate) mod validator;

pub use types::{
    PdfXLevel, XComplianceError, XErrorCode, XSeverity, XValidationResult, XValidationStats,
};
pub use validator::{validate_pdf_x, PdfXValidator};
