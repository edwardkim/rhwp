//! PDF signature verification.
//!
//! This module handles verification of existing digital signatures in PDF documents.

use super::byterange::ByteRangeCalculator;
use super::types::{SignatureInfo, SignatureSubFilter, VerificationResult, VerificationStatus};
use crate::error::{Error, Result};
use crate::object::Object;

/// Verifier for PDF digital signatures.
pub struct SignatureVerifier {
    /// Trusted root certificates (DER-encoded)
    trusted_roots: Vec<Vec<u8>>,
}

impl SignatureVerifier {
    /// Create a new signature verifier.
    pub fn new() -> Self {
        Self {
            trusted_roots: Vec::new(),
        }
    }

    /// Add a trusted root certificate.
    pub fn add_trusted_root(&mut self, cert_der: Vec<u8>) {
        self.trusted_roots.push(cert_der);
    }

    /// Add multiple trusted root certificates.
    pub fn add_trusted_roots(&mut self, certs: Vec<Vec<u8>>) {
        self.trusted_roots.extend(certs);
    }

    /// Load system root certificates.
    #[cfg(feature = "signatures")]
    pub fn load_system_roots(&mut self) -> Result<usize> {
        // This would use webpki-roots or native-tls to load system certificates
        // For now, return an error indicating this is not yet implemented
        Err(Error::InvalidPdf("System root loading not yet implemented".to_string()))
    }

    /// Extract signature information from a signature dictionary.
    pub fn extract_signature_info(&self, sig_dict: &Object) -> Result<SignatureInfo> {
        let dict = match sig_dict {
            Object::Dictionary(d) => d,
            _ => return Err(Error::InvalidPdf("Signature must be a dictionary".to_string())),
        };

        let mut info = SignatureInfo::default();

        // Extract /Name
        if let Some(Object::String(name)) = dict.get("Name") {
            info.signer_name = Some(String::from_utf8_lossy(name).to_string());
        }

        // Extract /M (signing time)
        if let Some(Object::String(time)) = dict.get("M") {
            info.signing_time = Some(String::from_utf8_lossy(time).to_string());
        }

        // Extract /Reason
        if let Some(Object::String(reason)) = dict.get("Reason") {
            info.reason = Some(String::from_utf8_lossy(reason).to_string());
        }

        // Extract /Location
        if let Some(Object::String(location)) = dict.get("Location") {
            info.location = Some(String::from_utf8_lossy(location).to_string());
        }

        // Extract /ContactInfo
        if let Some(Object::String(contact)) = dict.get("ContactInfo") {
            info.contact_info = Some(String::from_utf8_lossy(contact).to_string());
        }

        // Extract /SubFilter
        if let Some(Object::Name(sub_filter)) = dict.get("SubFilter") {
            info.sub_filter = SignatureSubFilter::from_pdf_name(sub_filter);
        }

        // Extract /ByteRange
        if let Some(Object::Array(byte_range)) = dict.get("ByteRange") {
            info.byte_range = byte_range
                .iter()
                .filter_map(|obj| match obj {
                    Object::Integer(i) => Some(*i),
                    _ => None,
                })
                .collect();
        }

        // Check if signature covers whole document (ByteRange should be 4 elements)
        info.covers_whole_document = info.byte_range.len() == 4;

        // Retain /Contents (the PKCS#7/CMS SignedData blob) so higher
        // layers can parse the signer certificate / verify the
        // signature without needing a second pass through the dict.
        if let Some(Object::String(bytes)) = dict.get("Contents") {
            info.contents = Some(bytes.clone());
        }

        Ok(info)
    }

    /// Verify a signature end-to-end.
    ///
    /// Runs the RSA-PKCS#1 v1.5 signer-attributes check and the
    /// `messageDigest` content-integrity check (via
    /// [`crate::signatures::verify_signer_detached`]), then enriches
    /// the result with certificate trust, expiration, and DN metadata
    /// extracted from the embedded signer certificate.
    ///
    /// Digest algorithm is driven by the CMS blob's `signer.digest_alg`
    /// (SHA-1 / SHA-256 / SHA-384 / SHA-512), so PDFs that don't use
    /// SHA-256 — still common with PAdES — are covered.
    #[cfg(feature = "signatures")]
    pub fn verify(
        &self,
        pdf_data: &[u8],
        sig_dict: &Object,
        contents: &[u8],
    ) -> Result<VerificationResult> {
        let mut result = VerificationResult {
            signature_info: self.extract_signature_info(sig_dict)?,
            ..VerificationResult::default()
        };

        if result.signature_info.byte_range.len() != 4 {
            result.status = VerificationStatus::Invalid;
            result
                .messages
                .push("Invalid ByteRange: expected 4 elements".to_string());
            return Ok(result);
        }

        let byte_range: [i64; 4] = [
            result.signature_info.byte_range[0],
            result.signature_info.byte_range[1],
            result.signature_info.byte_range[2],
            result.signature_info.byte_range[3],
        ];

        if let Err(e) = ByteRangeCalculator::validate_byte_range(&byte_range, pdf_data.len()) {
            result.status = VerificationStatus::Invalid;
            result.document_modified = true;
            result
                .messages
                .push(format!("ByteRange validation failed: {}", e));
            return Ok(result);
        }

        let signed_bytes = ByteRangeCalculator::extract_signed_bytes(pdf_data, &byte_range)?;

        match super::cms_verify::verify_signer_detached(contents, &signed_bytes) {
            Ok(super::cms_verify::SignerVerify::Valid) => {
                result.status = VerificationStatus::Valid;
                self.enrich_with_cert_metadata(contents, &mut result);
            },
            Ok(super::cms_verify::SignerVerify::Invalid) => {
                result.status = VerificationStatus::Invalid;
                result.messages.push(
                    "Signature verification failed: signer crypto or messageDigest did not \
                     match (wrong key or tampered content)"
                        .to_string(),
                );
            },
            Ok(super::cms_verify::SignerVerify::Unknown) => {
                result.status = VerificationStatus::Unknown;
                result.messages.push(
                    "Signature could not be verified: signer uses an unsupported algorithm or \
                     the CMS blob lacks signed attributes"
                        .to_string(),
                );
            },
            Err(e) => {
                result.status = VerificationStatus::Invalid;
                result
                    .messages
                    .push(format!("Signature verification failed: {e}"));
            },
        }

        Ok(result)
    }

    /// Extract signer-cert metadata (CN, issuer, trust flag, expiry) from
    /// the CMS blob and stamp it into the result. Non-fatal: a cert that
    /// can't be parsed just leaves the fields at default and does not
    /// demote a `Valid` verdict.
    #[cfg(feature = "signatures")]
    fn enrich_with_cert_metadata(&self, contents: &[u8], result: &mut VerificationResult) {
        let Ok(cert_der) = super::cms::extract_signer_certificate_der(contents) else {
            return;
        };
        use x509_parser::prelude::*;
        let Ok((_, cert)) = X509Certificate::from_der(&cert_der) else {
            return;
        };

        result.signature_info.certificate_cn = Some(cert.subject().to_string());
        result.signature_info.certificate_issuer = Some(cert.issuer().to_string());

        result.certificate_trusted = self.is_certificate_trusted(&cert_der);
        if !result.certificate_trusted {
            result.status = VerificationStatus::Unknown;
            result
                .messages
                .push("Certificate is not trusted".to_string());
        }

        // WASM note: if signatures are ever enabled for wasm32, SystemTime::now()
        // here will also need cfg-gating (currently masked because the `signatures`
        // feature is not enabled in the wasm build).
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let na = cert.validity().not_after.timestamp();
        let nb = cert.validity().not_before.timestamp();
        result.certificate_expired = now > na || now < nb;
        if result.certificate_expired && result.status == VerificationStatus::Valid {
            result.status = VerificationStatus::ValidWithWarnings;
            result.messages.push("Certificate has expired".to_string());
        }
    }

    /// Check if a certificate is in the trusted roots.
    fn is_certificate_trusted(&self, cert_der: &[u8]) -> bool {
        // Simple check: is the certificate in our trusted roots?
        // A full implementation would verify the chain
        self.trusted_roots.iter().any(|root| root == cert_der)
    }

    /// Quick check if a signature appears valid (without full cryptographic verification).
    pub fn quick_check(&self, sig_dict: &Object) -> Result<bool> {
        let info = self.extract_signature_info(sig_dict)?;

        // Basic sanity checks
        let has_valid_byte_range = info.byte_range.len() == 4;
        let has_sub_filter = info.sub_filter.is_some();

        Ok(has_valid_byte_range && has_sub_filter)
    }
}

impl Default for SignatureVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_sig_dict() -> Object {
        let mut dict = HashMap::new();
        dict.insert("Type".to_string(), Object::Name("Sig".to_string()));
        dict.insert("Filter".to_string(), Object::Name("Adobe.PPKLite".to_string()));
        dict.insert("SubFilter".to_string(), Object::Name("adbe.pkcs7.detached".to_string()));
        dict.insert("Name".to_string(), Object::String(b"Test Signer".to_vec()));
        dict.insert("Reason".to_string(), Object::String(b"Testing".to_vec()));
        dict.insert("Location".to_string(), Object::String(b"Test City".to_vec()));
        dict.insert("M".to_string(), Object::String(b"D:20240101120000Z".to_vec()));
        dict.insert(
            "ByteRange".to_string(),
            Object::Array(vec![
                Object::Integer(0),
                Object::Integer(100),
                Object::Integer(200),
                Object::Integer(50),
            ]),
        );
        Object::Dictionary(dict)
    }

    #[test]
    fn test_extract_signature_info() {
        let verifier = SignatureVerifier::new();
        let sig_dict = make_sig_dict();

        let info = verifier.extract_signature_info(&sig_dict).unwrap();

        assert_eq!(info.signer_name, Some("Test Signer".to_string()));
        assert_eq!(info.reason, Some("Testing".to_string()));
        assert_eq!(info.location, Some("Test City".to_string()));
        assert_eq!(info.sub_filter, Some(SignatureSubFilter::Pkcs7Detached));
        assert_eq!(info.byte_range, vec![0, 100, 200, 50]);
        assert!(info.covers_whole_document);
    }

    #[test]
    fn test_quick_check_valid() {
        let verifier = SignatureVerifier::new();
        let sig_dict = make_sig_dict();

        let result = verifier.quick_check(&sig_dict).unwrap();
        assert!(result);
    }

    #[test]
    fn test_quick_check_missing_byte_range() {
        let verifier = SignatureVerifier::new();
        let mut dict = HashMap::new();
        dict.insert("Type".to_string(), Object::Name("Sig".to_string()));
        dict.insert("SubFilter".to_string(), Object::Name("adbe.pkcs7.detached".to_string()));
        let sig_dict = Object::Dictionary(dict);

        let result = verifier.quick_check(&sig_dict).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_trusted_roots() {
        let mut verifier = SignatureVerifier::new();
        let test_cert = vec![1, 2, 3, 4];

        assert!(!verifier.is_certificate_trusted(&test_cert));

        verifier.add_trusted_root(test_cert.clone());
        assert!(verifier.is_certificate_trusted(&test_cert));
    }
}
