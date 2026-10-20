//! PDF Digital Signatures module.
//!
//! This module provides functionality for creating and verifying digital signatures
//! in PDF documents according to the PDF specification and PAdES (PDF Advanced
//! Electronic Signatures) standards.
//!
//! ## Features
//!
//! - **Signature Creation**: Sign PDFs with X.509 certificates
//! - **Signature Verification**: Verify existing PDF signatures
//! - **Certificate Handling**: Parse and validate X.509 certificate chains
//! - **ByteRange Calculation**: Proper handling of PDF byte ranges for signing
//!
//! ## Signature Types Supported
//!
//! - PKCS#7 detached signatures (adbe.pkcs7.detached)
//! - PKCS#7 SHA-1 signatures (adbe.pkcs7.sha1)
//! - PAdES signatures (ETSI.CAdES.detached)
//!
//! ## Example
//!
//! ```ignore
//! use pdf_oxide::api::Pdf;
//! use pdf_oxide::signatures::{SigningCredentials, SignOptions};
//!
//! let mut pdf = Pdf::open("document.pdf")?;
//!
//! // Load signing credentials
//! let credentials = SigningCredentials::from_pkcs12("cert.p12", "password")?;
//!
//! // Sign the document
//! pdf.sign(&credentials, SignOptions::default())?;
//! pdf.save("signed_document.pdf")?;
//! ```
//!
//! ## PDF Specification Reference
//!
//! - ISO 32000-1:2008 Section 12.8 - Digital Signatures
//! - ISO 32000-2:2020 Section 12.8 - Digital Signatures
//! - ETSI TS 102 778 - PAdES
//!
//! Requires the `signatures` feature to be enabled.

mod byterange;
#[cfg(feature = "signatures")]
mod cms;
#[cfg(feature = "signatures")]
mod cms_verify;
#[cfg(feature = "signatures")]
pub(crate) mod der_util;
// `pub(crate)` so `RustCryptoProvider::verify_rsa_pkcs1v15` in
// `src/crypto/rust_provider.rs` can re-use `digest_info_prefix` (the
// OID → PKCS#1 DigestInfo prefix table) — same source of truth used
// by `cms_verify.rs:306-316` and `signer.rs:220-224`.
#[cfg(feature = "signatures")]
pub(crate) mod crypto;
mod enumerate;
#[cfg(feature = "signatures")]
pub mod pades;
mod pdf_date;
#[cfg(feature = "signatures")]
mod sign_bytes;
mod signer;
#[cfg(feature = "signatures")]
mod timestamp;
#[cfg(all(feature = "signatures", feature = "tsa-client"))]
mod tsa_client;
pub(crate) mod tsp;
mod types;
mod verifier;

pub use byterange::ByteRangeCalculator;
#[cfg(feature = "signatures")]
pub use cms::extract_signer_certificate_der;
#[cfg(feature = "signatures")]
pub use cms_verify::{verify_signer, verify_signer_detached, SignerVerify};
pub use enumerate::{count_signatures, enumerate_signatures};
#[cfg(feature = "signatures")]
pub use pades::{
    classify_pades_level, has_document_timestamp, read_dss, DocumentSecurityStore, PadesLevel,
    RevocationMaterial, VriEntry,
};
pub use pdf_date::parse_pdf_date_to_epoch;
#[cfg(feature = "signatures")]
pub use sign_bytes::{sign_pdf_bytes, sign_pdf_bytes_pades};
pub use signer::PdfSigner;
#[cfg(feature = "signatures")]
pub use timestamp::{HashAlgorithm, Timestamp};
#[cfg(all(feature = "signatures", feature = "tsa-client"))]
pub use tsa_client::{TsaClient, TsaClientConfig};
pub use types::{
    DigestAlgorithm, SignOptions, SignatureAppearance, SignatureInfo, SignatureSubFilter,
    SigningCredentials, VerificationResult, VerificationStatus,
};
pub use verifier::SignatureVerifier;
