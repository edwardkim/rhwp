//! Digital signature types and data structures.
//!
//! This module defines the core types used for PDF digital signatures.

use crate::error::{Error, Result};
use crate::geometry::Rect;

/// Digest algorithm used for signing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DigestAlgorithm {
    /// SHA-1 (deprecated, but still common in legacy PDFs)
    Sha1,
    /// SHA-256 (recommended)
    #[default]
    Sha256,
    /// SHA-384
    Sha384,
    /// SHA-512
    Sha512,
}

impl DigestAlgorithm {
    /// Get the OID for this digest algorithm.
    pub fn oid(&self) -> &'static [u8] {
        match self {
            DigestAlgorithm::Sha1 => &[0x2B, 0x0E, 0x03, 0x02, 0x1A], // 1.3.14.3.2.26
            DigestAlgorithm::Sha256 => &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x01], // 2.16.840.1.101.3.4.2.1
            DigestAlgorithm::Sha384 => &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x02], // 2.16.840.1.101.3.4.2.2
            DigestAlgorithm::Sha512 => &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x03], // 2.16.840.1.101.3.4.2.3
        }
    }

    /// Get the name of this algorithm.
    pub fn name(&self) -> &'static str {
        match self {
            DigestAlgorithm::Sha1 => "SHA-1",
            DigestAlgorithm::Sha256 => "SHA-256",
            DigestAlgorithm::Sha384 => "SHA-384",
            DigestAlgorithm::Sha512 => "SHA-512",
        }
    }
}

/// Signature sub-filter type (signature format).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SignatureSubFilter {
    /// adbe.pkcs7.detached - PKCS#7 detached signature
    #[default]
    Pkcs7Detached,
    /// adbe.pkcs7.sha1 - PKCS#7 with SHA-1 digest
    Pkcs7Sha1,
    /// ETSI.CAdES.detached - PAdES CAdES signature
    CadesDetached,
    /// ETSI.RFC3161 - Timestamp token
    Rfc3161,
}

impl SignatureSubFilter {
    /// Get the PDF name for this sub-filter.
    pub fn as_pdf_name(&self) -> &'static str {
        match self {
            SignatureSubFilter::Pkcs7Detached => "adbe.pkcs7.detached",
            SignatureSubFilter::Pkcs7Sha1 => "adbe.pkcs7.sha1",
            SignatureSubFilter::CadesDetached => "ETSI.CAdES.detached",
            SignatureSubFilter::Rfc3161 => "ETSI.RFC3161",
        }
    }

    /// Parse a PDF name into a sub-filter type.
    pub fn from_pdf_name(name: &str) -> Option<Self> {
        match name {
            "adbe.pkcs7.detached" => Some(SignatureSubFilter::Pkcs7Detached),
            "adbe.pkcs7.sha1" => Some(SignatureSubFilter::Pkcs7Sha1),
            "ETSI.CAdES.detached" => Some(SignatureSubFilter::CadesDetached),
            "ETSI.RFC3161" => Some(SignatureSubFilter::Rfc3161),
            _ => None,
        }
    }
}

/// Signing credentials containing certificate and private key.
#[derive(Clone)]
pub struct SigningCredentials {
    /// DER-encoded X.509 certificate
    pub certificate: Vec<u8>,
    /// DER-encoded private key (PKCS#8 format)
    pub private_key: Vec<u8>,
    /// Certificate chain (intermediate certificates, DER-encoded)
    pub chain: Vec<Vec<u8>>,
}

impl SigningCredentials {
    /// Create new signing credentials from raw components.
    pub fn new(certificate: Vec<u8>, private_key: Vec<u8>) -> Self {
        Self {
            certificate,
            private_key,
            chain: Vec::new(),
        }
    }

    /// Create credentials with a certificate chain.
    pub fn with_chain(mut self, chain: Vec<Vec<u8>>) -> Self {
        self.chain = chain;
        self
    }

    /// Load credentials from a PKCS#12 (.p12/.pfx) file.
    ///
    /// Parses the DER-encoded PFX container using `p12-keystore`. Both the
    /// end-entity certificate and the PKCS#8 private key are extracted; any
    /// additional certificates in the bag become the chain.
    #[cfg(feature = "signatures")]
    pub fn from_pkcs12(data: &[u8], password: &str) -> Result<Self> {
        // p12-keystore 0.3 requires an explicit import policy; `Strict` is the
        // crate default and matches the prior 0.2 behaviour — pair each private
        // key with its matched certificate chain, ignoring unmatched keys and
        // untrusted certificates.
        let ks = p12_keystore::KeyStore::from_pkcs12(
            data,
            password,
            p12_keystore::Pkcs12ImportPolicy::Strict,
        )
        .map_err(|e| Error::InvalidPdf(format!("PKCS#12 parse error: {e}")))?;

        let (_, pkc) = ks
            .private_key_chain()
            .ok_or_else(|| Error::InvalidPdf("PKCS#12 contains no private key".into()))?;

        // 0.3: key() returns &PrivateKey; as_der() yields the PKCS#8 DER bytes.
        let private_key = pkc.key().as_der().to_vec();

        // certs(): first entry is the entity cert, subsequent are intermediates/root
        // (0.3 renamed the 0.2 `chain()` accessor and returns a &[Certificate]).
        let mut cert_iter = pkc.certs().iter();
        let certificate = cert_iter
            .next()
            .ok_or_else(|| Error::InvalidPdf("PKCS#12 contains no certificate".into()))?
            .as_der()
            .to_vec();
        let chain: Vec<Vec<u8>> = cert_iter.map(|c| c.as_der().to_vec()).collect();

        Ok(Self {
            certificate,
            private_key,
            chain,
        })
    }

    /// Load credentials from separate PEM-encoded certificate and private key.
    ///
    /// Both PEM blocks are decoded with `x509-parser`'s PEM reader which
    /// accepts `BEGIN CERTIFICATE`, `BEGIN PRIVATE KEY` (PKCS#8), and
    /// `BEGIN RSA PRIVATE KEY` (PKCS#1) labels. The certificate is validated
    /// by parsing it as X.509. The key is stored as raw DER.
    #[cfg(feature = "signatures")]
    pub fn from_pem(cert_pem: &str, key_pem: &str) -> Result<Self> {
        use x509_parser::pem::parse_x509_pem;
        use x509_parser::prelude::*;

        // Parse the certificate PEM block and validate it.
        let (_, cert_block) = parse_x509_pem(cert_pem.as_bytes())
            .map_err(|e| Error::InvalidPdf(format!("invalid certificate PEM: {e}")))?;
        let cert_der = cert_block.contents;
        let (_, _) = X509Certificate::from_der(&cert_der).map_err(|e| {
            Error::InvalidPdf(format!("certificate PEM contains invalid X.509 DER: {e}"))
        })?;

        // Parse the private key PEM block — any `BEGIN ... KEY` label is fine.
        let (_, key_block) = parse_x509_pem(key_pem.as_bytes())
            .map_err(|e| Error::InvalidPdf(format!("invalid private key PEM: {e}")))?;
        let private_key = key_block.contents;
        if private_key.is_empty() {
            return Err(Error::InvalidPdf("private key PEM decoded to empty bytes".into()));
        }

        Ok(Self {
            certificate: cert_der,
            private_key,
            chain: Vec::new(),
        })
    }

    /// Load credentials from a raw DER-encoded X.509 certificate. No
    /// private key is attached — the resulting value is only useful
    /// for inspection (subject / issuer / serial / validity / is_valid)
    /// and not for signing. The signing-path dep chain (PKCS#12 parsing)
    /// is still upstream, but every binding can surface Certificate
    /// metadata by feeding the raw cert DER.
    #[cfg(feature = "signatures")]
    pub fn from_der(cert_der: Vec<u8>) -> Result<Self> {
        use x509_parser::prelude::*;
        // Validate the DER parses before handing out credentials.
        let (_, _parsed) = X509Certificate::from_der(&cert_der)
            .map_err(|e| Error::InvalidPdf(format!("invalid X.509 DER: {e}")))?;
        Ok(Self {
            certificate: cert_der,
            private_key: Vec::new(),
            chain: Vec::new(),
        })
    }

    /// Certificate subject Distinguished Name (e.g. `CN=pdfoxide-test, O=...`).
    #[cfg(feature = "signatures")]
    pub fn subject(&self) -> Result<String> {
        use x509_parser::prelude::*;
        let (_, cert) = X509Certificate::from_der(&self.certificate)
            .map_err(|e| Error::InvalidPdf(format!("invalid X.509 DER: {e}")))?;
        Ok(cert.subject().to_string())
    }

    /// Certificate issuer Distinguished Name.
    #[cfg(feature = "signatures")]
    pub fn issuer(&self) -> Result<String> {
        use x509_parser::prelude::*;
        let (_, cert) = X509Certificate::from_der(&self.certificate)
            .map_err(|e| Error::InvalidPdf(format!("invalid X.509 DER: {e}")))?;
        Ok(cert.issuer().to_string())
    }

    /// Certificate serial number as a hex string.
    #[cfg(feature = "signatures")]
    pub fn serial(&self) -> Result<String> {
        use x509_parser::prelude::*;
        let (_, cert) = X509Certificate::from_der(&self.certificate)
            .map_err(|e| Error::InvalidPdf(format!("invalid X.509 DER: {e}")))?;
        Ok(cert.serial.to_str_radix(16))
    }

    /// Validity window as Unix timestamps `(not_before, not_after)`.
    #[cfg(feature = "signatures")]
    pub fn validity(&self) -> Result<(i64, i64)> {
        use x509_parser::prelude::*;
        let (_, cert) = X509Certificate::from_der(&self.certificate)
            .map_err(|e| Error::InvalidPdf(format!("invalid X.509 DER: {e}")))?;
        let nb = cert.validity().not_before.timestamp();
        let na = cert.validity().not_after.timestamp();
        Ok((nb, na))
    }

    /// Whether the certificate is within its validity window right now.
    #[cfg(feature = "signatures")]
    pub fn is_valid(&self) -> Result<bool> {
        use x509_parser::prelude::*;
        let (_, cert) = X509Certificate::from_der(&self.certificate)
            .map_err(|e| Error::InvalidPdf(format!("invalid X.509 DER: {e}")))?;
        // WASM note: if signatures are ever enabled for wasm32, SystemTime::now()
        // here will also need cfg-gating (currently masked because the `signatures`
        // feature is not enabled in the wasm build).
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let nb = cert.validity().not_before.timestamp();
        let na = cert.validity().not_after.timestamp();
        Ok(now >= nb && now <= na)
    }
}

impl std::fmt::Debug for SigningCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SigningCredentials")
            .field("certificate", &format!("{} bytes", self.certificate.len()))
            .field("private_key", &"[REDACTED]")
            .field("chain", &format!("{} certificates", self.chain.len()))
            .finish()
    }
}

/// Options for signing a PDF.
#[derive(Debug, Clone)]
pub struct SignOptions {
    /// Digest algorithm to use
    pub digest_algorithm: DigestAlgorithm,
    /// Signature sub-filter (format)
    pub sub_filter: SignatureSubFilter,
    /// Reason for signing
    pub reason: Option<String>,
    /// Location where the document was signed
    pub location: Option<String>,
    /// Contact information
    pub contact_info: Option<String>,
    /// Name of the signer (if different from certificate CN)
    pub name: Option<String>,
    /// Signature appearance (for visible signatures)
    pub appearance: Option<SignatureAppearance>,
    /// Whether to embed a timestamp
    pub embed_timestamp: bool,
    /// Timestamp server URL (for embedded timestamps)
    pub timestamp_url: Option<String>,
    /// Estimated signature size in bytes (for ByteRange calculation)
    pub estimated_size: usize,
}

impl Default for SignOptions {
    fn default() -> Self {
        Self {
            digest_algorithm: DigestAlgorithm::Sha256,
            sub_filter: SignatureSubFilter::Pkcs7Detached,
            reason: None,
            location: None,
            contact_info: None,
            name: None,
            appearance: None,
            embed_timestamp: false,
            timestamp_url: None,
            estimated_size: 8192, // Conservative default for signature size
        }
    }
}

impl SignOptions {
    /// Create sign options with a visible signature appearance.
    pub fn with_appearance(mut self, appearance: SignatureAppearance) -> Self {
        self.appearance = Some(appearance);
        self
    }

    /// Set the reason for signing.
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Set the signing location.
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    /// Enable timestamping with the specified TSA URL.
    pub fn with_timestamp(mut self, tsa_url: impl Into<String>) -> Self {
        self.embed_timestamp = true;
        self.timestamp_url = Some(tsa_url.into());
        self
    }
}

/// Visible signature appearance configuration.
#[derive(Debug, Clone)]
pub struct SignatureAppearance {
    /// Page number (0-indexed)
    pub page: usize,
    /// Rectangle for the signature appearance
    pub rect: Rect,
    /// Whether to show signer name
    pub show_name: bool,
    /// Whether to show signing date
    pub show_date: bool,
    /// Whether to show signing reason
    pub show_reason: bool,
    /// Whether to show signing location
    pub show_location: bool,
    /// Custom background image (PNG data)
    pub background_image: Option<Vec<u8>>,
    /// Custom font size
    pub font_size: f32,
}

impl Default for SignatureAppearance {
    fn default() -> Self {
        Self {
            page: 0,
            rect: Rect::new(72.0, 72.0, 200.0, 50.0),
            show_name: true,
            show_date: true,
            show_reason: true,
            show_location: true,
            background_image: None,
            font_size: 10.0,
        }
    }
}

/// Information about an existing signature in a PDF.
#[derive(Debug, Clone, Default)]
pub struct SignatureInfo {
    /// Name of the signer
    pub signer_name: Option<String>,
    /// Signing time
    pub signing_time: Option<String>,
    /// Reason for signing
    pub reason: Option<String>,
    /// Signing location
    pub location: Option<String>,
    /// Contact information
    pub contact_info: Option<String>,
    /// Signature sub-filter type
    pub sub_filter: Option<SignatureSubFilter>,
    /// Whether the signature covers the whole document
    pub covers_whole_document: bool,
    /// Byte range of the signed data
    pub byte_range: Vec<i64>,
    /// Certificate subject common name
    pub certificate_cn: Option<String>,
    /// Certificate issuer
    pub certificate_issuer: Option<String>,
    /// Certificate validity start
    pub valid_from: Option<String>,
    /// Certificate validity end
    pub valid_to: Option<String>,
    /// Raw DER-encoded PKCS#7/CMS SignedData blob from `/Contents`,
    /// retained so that later accessors (signer certificate, verify)
    /// can parse it on demand. `None` when the signature dictionary
    /// had no `/Contents` entry (blank signature field).
    ///
    /// Prefer [`SignatureInfo::contents`] (the accessor) over touching
    /// this field directly — the field layout is not a stable part of
    /// the public API.
    pub contents: Option<Vec<u8>>,
}

impl SignatureInfo {
    /// Borrowed view of the raw PKCS#7/CMS SignedData blob from the
    /// signature dictionary's `/Contents` entry. Returns `None` when
    /// the dictionary had no `/Contents`.
    ///
    /// This is the FFI-stable way to get at the signed bytes — use it
    /// instead of reaching for the `contents` field, whose backing
    /// storage may change.
    pub fn contents(&self) -> Option<&[u8]> {
        self.contents.as_deref()
    }

    /// Borrowed view of the signature's `/ByteRange` array. Empty when
    /// the signature is a blank field.
    pub fn byte_range(&self) -> &[i64] {
        &self.byte_range
    }
}

/// Result of signature verification.
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Overall verification status
    pub status: VerificationStatus,
    /// Signature information
    pub signature_info: SignatureInfo,
    /// Verification messages (errors, warnings)
    pub messages: Vec<String>,
    /// Whether the document was modified after signing
    pub document_modified: bool,
    /// Whether the certificate is trusted
    pub certificate_trusted: bool,
    /// Whether the certificate chain is valid
    pub chain_valid: bool,
    /// Whether the certificate has expired
    pub certificate_expired: bool,
    /// Whether the signature timestamp is valid (if present)
    pub timestamp_valid: Option<bool>,
}

impl Default for VerificationResult {
    fn default() -> Self {
        Self {
            status: VerificationStatus::Unknown,
            signature_info: SignatureInfo::default(),
            messages: Vec::new(),
            document_modified: false,
            certificate_trusted: false,
            chain_valid: false,
            certificate_expired: false,
            timestamp_valid: None,
        }
    }
}

/// Verification status of a signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationStatus {
    /// Signature is valid
    Valid,
    /// Signature is invalid (cryptographically)
    Invalid,
    /// Signature validity is unknown (e.g., untrusted certificate)
    Unknown,
    /// Signature is valid but the document was modified
    ValidWithWarnings,
}

impl VerificationStatus {
    /// Check if the status indicates a valid signature.
    pub fn is_valid(&self) -> bool {
        matches!(self, VerificationStatus::Valid)
    }

    /// Check if the status indicates any form of validity (including warnings).
    pub fn is_ok(&self) -> bool {
        matches!(self, VerificationStatus::Valid | VerificationStatus::ValidWithWarnings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digest_algorithm_names() {
        assert_eq!(DigestAlgorithm::Sha256.name(), "SHA-256");
        assert_eq!(DigestAlgorithm::Sha1.name(), "SHA-1");
    }

    #[test]
    fn test_sub_filter_names() {
        assert_eq!(SignatureSubFilter::Pkcs7Detached.as_pdf_name(), "adbe.pkcs7.detached");
        assert_eq!(
            SignatureSubFilter::from_pdf_name("adbe.pkcs7.detached"),
            Some(SignatureSubFilter::Pkcs7Detached)
        );
    }

    #[test]
    fn test_sign_options_default() {
        let opts = SignOptions::default();
        assert_eq!(opts.digest_algorithm, DigestAlgorithm::Sha256);
        assert_eq!(opts.sub_filter, SignatureSubFilter::Pkcs7Detached);
        assert!(!opts.embed_timestamp);
    }

    #[test]
    fn test_sign_options_builder() {
        let opts = SignOptions::default()
            .with_reason("Test signing")
            .with_location("Test City");
        assert_eq!(opts.reason, Some("Test signing".to_string()));
        assert_eq!(opts.location, Some("Test City".to_string()));
    }

    #[test]
    fn test_verification_status() {
        assert!(VerificationStatus::Valid.is_valid());
        assert!(!VerificationStatus::Invalid.is_valid());
        assert!(VerificationStatus::ValidWithWarnings.is_ok());
        assert!(!VerificationStatus::Unknown.is_valid());
    }

    #[test]
    fn test_signing_credentials_debug() {
        let creds = SigningCredentials::new(vec![1, 2, 3], vec![4, 5, 6]);
        let debug = format!("{:?}", creds);
        assert!(debug.contains("[REDACTED]"));
        assert!(debug.contains("3 bytes"));
    }

    #[test]
    #[cfg(feature = "signatures")]
    fn test_from_pem_loads_cert_and_key() {
        let cert_pem = std::fs::read_to_string("tests/fixtures/test_signing_cert.pem")
            .expect("test fixture must exist");
        let key_pem = std::fs::read_to_string("tests/fixtures/test_signing_key.pem")
            .expect("test fixture must exist");
        let creds =
            SigningCredentials::from_pem(&cert_pem, &key_pem).expect("from_pem should succeed");
        assert!(!creds.certificate.is_empty(), "certificate must be non-empty");
        assert!(!creds.private_key.is_empty(), "private key must be non-empty");
        let subj = creds.subject().expect("subject must parse");
        assert!(subj.contains("pdfoxide-test"), "subject must include CN: got {subj}");
    }

    #[test]
    #[cfg(feature = "signatures")]
    fn test_from_pem_rejects_invalid_cert() {
        let result = SigningCredentials::from_pem("not a pem at all", "also bad");
        assert!(result.is_err(), "should reject invalid PEM cert");
    }

    #[test]
    #[cfg(feature = "signatures")]
    fn test_from_pkcs12_loads_cert_and_key() {
        let data =
            std::fs::read("tests/fixtures/test_signing.p12").expect("test fixture must exist");
        let creds = SigningCredentials::from_pkcs12(&data, "testpass")
            .expect("from_pkcs12 should succeed with correct password");
        assert!(!creds.certificate.is_empty(), "certificate must be non-empty");
        assert!(!creds.private_key.is_empty(), "private key must be non-empty");
        let subj = creds.subject().expect("subject must parse");
        assert!(subj.contains("pdfoxide-test"), "subject must include CN: got {subj}");
    }

    #[test]
    #[cfg(feature = "signatures")]
    fn test_from_pkcs12_rejects_wrong_password() {
        let data =
            std::fs::read("tests/fixtures/test_signing.p12").expect("test fixture must exist");
        // Wrong password leads to either parse error or empty key/cert bags.
        // Either outcome is acceptable — we just must not panic.
        let result = SigningCredentials::from_pkcs12(&data, "wrongpassword");
        // The p12 crate may or may not fail at parse time; it may return Ok
        // but with empty bags — treat both as "not usable".
        match result {
            Err(_) => { /* correct: explicit parse error */ },
            Ok(c) => {
                // p12 decryption with wrong password silently gives garbage
                // bytes; the cert DER will fail to parse as X.509.
                assert!(
                    c.subject().is_err() || c.certificate.is_empty() || c.private_key.is_empty(),
                    "wrong-password PKCS#12 must not yield valid usable credentials"
                );
            },
        }
    }

    #[test]
    #[cfg(feature = "signatures")]
    fn test_from_pkcs12_rejects_garbage() {
        let result = SigningCredentials::from_pkcs12(b"not pkcs12 data", "password");
        assert!(result.is_err(), "should reject non-PKCS#12 data");
    }
}
