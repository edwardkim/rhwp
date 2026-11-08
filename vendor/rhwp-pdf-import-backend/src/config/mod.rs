//! Configuration module for PDF text extraction.
//!
//! Provides extraction profiles, document-type classification, and threshold configuration
//! for customizing text extraction behavior to different document types.

pub mod extraction_profiles;

pub use extraction_profiles::{DocumentType, ExtractionProfile};
