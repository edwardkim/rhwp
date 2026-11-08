//! PDF signing implementation.
//!
//! This module handles the creation of digital signatures for PDF documents.

use super::byterange::ByteRangeCalculator;
use super::types::{DigestAlgorithm, SignOptions, SigningCredentials};
use crate::error::{Error, Result};

#[cfg(feature = "signatures")]
use sha2::{Digest, Sha256, Sha384, Sha512};

#[cfg(feature = "signatures")]
use sha1::Sha1;

/// PDF signer that creates digital signatures.
pub struct PdfSigner {
    credentials: SigningCredentials,
    options: SignOptions,
    byte_range_calc: ByteRangeCalculator,
}

impl PdfSigner {
    /// Create a new PDF signer with the given credentials and options.
    pub fn new(credentials: SigningCredentials, options: SignOptions) -> Self {
        let byte_range_calc = ByteRangeCalculator::new(options.estimated_size);
        Self {
            credentials,
            options,
            byte_range_calc,
        }
    }

    /// Get the placeholder size for the signature.
    pub fn placeholder_size(&self) -> usize {
        self.byte_range_calc.placeholder_size()
    }

    /// Generate the placeholder for the /Contents value.
    pub fn generate_contents_placeholder(&self) -> String {
        self.byte_range_calc.generate_placeholder()
    }

    /// Build the signature dictionary content (without /Contents value).
    ///
    /// This returns the dictionary entries that should appear in the signature
    /// dictionary. The actual /Contents value should be set to the placeholder.
    pub fn build_signature_dictionary(&self) -> String {
        let mut dict = String::new();

        // Required fields
        dict.push_str("/Type /Sig\n");
        dict.push_str("/Filter /Adobe.PPKLite\n");
        dict.push_str(&format!("/SubFilter /{}\n", self.options.sub_filter.as_pdf_name()));

        // ByteRange placeholder - will be filled in after file is assembled
        dict.push_str("/ByteRange [0 0 0 0]\n");

        // Optional fields
        if let Some(ref name) = self.options.name {
            dict.push_str(&format!("/Name ({})\n", escape_pdf_string(name)));
        }

        if let Some(ref reason) = self.options.reason {
            dict.push_str(&format!("/Reason ({})\n", escape_pdf_string(reason)));
        }

        if let Some(ref location) = self.options.location {
            dict.push_str(&format!("/Location ({})\n", escape_pdf_string(location)));
        }

        if let Some(ref contact) = self.options.contact_info {
            dict.push_str(&format!("/ContactInfo ({})\n", escape_pdf_string(contact)));
        }

        // Signing time (M field)
        let signing_time = format_pdf_date();
        dict.push_str(&format!("/M ({})\n", signing_time));

        dict
    }

    /// Compute the digest of the signed bytes.
    #[cfg(feature = "signatures")]
    pub fn compute_digest(&self, signed_bytes: &[u8]) -> Vec<u8> {
        match self.options.digest_algorithm {
            DigestAlgorithm::Sha1 => {
                let mut hasher = Sha1::new();
                hasher.update(signed_bytes);
                hasher.finalize().to_vec()
            },
            DigestAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(signed_bytes);
                hasher.finalize().to_vec()
            },
            DigestAlgorithm::Sha384 => {
                let mut hasher = Sha384::new();
                hasher.update(signed_bytes);
                hasher.finalize().to_vec()
            },
            DigestAlgorithm::Sha512 => {
                let mut hasher = Sha512::new();
                hasher.update(signed_bytes);
                hasher.finalize().to_vec()
            },
        }
    }

    /// Sign the document and return a DER-encoded CMS/PKCS#7 SignedData blob.
    ///
    /// The returned bytes should be hex-encoded and written to the PDF
    /// `/Contents` placeholder using [`PdfSigner::insert_signature`].
    #[cfg(feature = "signatures")]
    pub fn sign(&self, signed_bytes: &[u8]) -> Result<Vec<u8>> {
        // Legacy / adbe.pkcs7.detached path — no ESS attr, byte-identical
        // to prior releases (#235 plan Q3: ESS is PAdES-only in v0.3.50).
        self.create_pkcs7_signature_inner(signed_bytes, None, None)
    }

    /// Sign producing a CAdES/PAdES-B-B-conformant CMS: adds the RFC 5035
    /// ESS `signing-certificate-v2` signed attribute (#235 TODO #4).
    ///
    /// The ESS attribute is *signed* (it changes the hashed
    /// `signedAttrs`), so it is built and inserted **before** the RSA
    /// sign step, in canonical SET-OF order. End-to-end CMS conformance
    /// is gated by the EU-DSS validator (feature plan §5.5); this path
    /// is self-checked by sign→`verify_signer_detached` round-trip and
    /// the attribute's presence in the parsed CMS.
    #[cfg(feature = "signatures")]
    pub fn sign_pades(&self, signed_bytes: &[u8]) -> Result<Vec<u8>> {
        let ess = crate::signatures::pades::build_signing_certificate_v2(
            &self.credentials.certificate,
            self.options.digest_algorithm,
        )?;
        self.create_pkcs7_signature_inner(signed_bytes, Some(&ess), None)
    }

    /// Sign producing PAdES-**B-T**: B-B (with ESS) + an RFC 3161
    /// `signature-time-stamp` *unsigned* attribute over the signature
    /// value (#235 TODO #7). `timestamper` receives the raw signature
    /// value (`SignerInfo.signature` content) and returns the DER
    /// `TimeStampToken` — in production this calls a TSA over the
    /// imprint; offline callers pass a pre-fetched token. Because the
    /// attribute is *unsigned*, the signed bytes are byte-identical to
    /// the B-B form ([`Self::sign_pades`]) — invariant I7.
    #[cfg(feature = "signatures")]
    pub fn sign_pades_t(
        &self,
        signed_bytes: &[u8],
        timestamper: &dyn Fn(&[u8]) -> Result<Vec<u8>>,
    ) -> Result<Vec<u8>> {
        let ess = crate::signatures::pades::build_signing_certificate_v2(
            &self.credentials.certificate,
            self.options.digest_algorithm,
        )?;
        self.create_pkcs7_signature_inner(signed_bytes, Some(&ess), Some(timestamper))
    }

    /// Build a detached CMS SignedData (RFC 5652) over `signed_bytes`:
    /// SHA-256 (or `options.digest_algorithm`), RSA-PKCS#1 v1.5, signed
    /// attrs (content-type + message-digest [+ ESS when `ess_attr`]),
    /// optional B-T `signature-time-stamp` unsigned attr via
    /// `timestamper`. Compatible with `verify_signer_detached`.
    #[cfg(feature = "signatures")]
    fn create_pkcs7_signature_inner(
        &self,
        signed_bytes: &[u8],
        ess_attr: Option<&[u8]>,
        timestamper: Option<&dyn Fn(&[u8]) -> Result<Vec<u8>>>,
    ) -> Result<Vec<u8>> {
        use super::crypto::digest_info_prefix;
        use cms::cert::x509::Certificate as X509Certificate;
        use der::oid::db::rfc5912::{ID_SHA_1, ID_SHA_256, ID_SHA_384, ID_SHA_512};
        use der::{Decode, Encode};
        use rsa::pkcs8::DecodePrivateKey;
        use rsa::{Pkcs1v15Sign, RsaPrivateKey};
        use sha1::Sha1;
        use sha2::{Digest, Sha256, Sha384, Sha512};

        // ── OID byte arrays (pre-encoded, without tag/length) ──────────
        const OID_SIGNED_DATA: &[u8] = &[0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x07, 0x02];
        const OID_DATA: &[u8] = &[0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x07, 0x01];
        const OID_SHA1: &[u8] = &[0x2B, 0x0E, 0x03, 0x02, 0x1A];
        const OID_SHA256: &[u8] = &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x01];
        const OID_SHA384: &[u8] = &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x02];
        const OID_SHA512: &[u8] = &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x03];
        const OID_RSA_ENC: &[u8] = &[0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x01, 0x01];
        const OID_CONTENT_TYPE: &[u8] = &[0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x09, 0x03];
        const OID_MSG_DIGEST: &[u8] = &[0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x09, 0x04];

        // ── Pick digest algorithm ───────────────────────────────────────
        let (digest_oid_bytes, digest_oid, message_digest): (&[u8], _, Vec<u8>) =
            match self.options.digest_algorithm {
                DigestAlgorithm::Sha1 => (OID_SHA1, ID_SHA_1, Sha1::digest(signed_bytes).to_vec()),
                DigestAlgorithm::Sha256 => {
                    (OID_SHA256, ID_SHA_256, Sha256::digest(signed_bytes).to_vec())
                },
                DigestAlgorithm::Sha384 => {
                    (OID_SHA384, ID_SHA_384, Sha384::digest(signed_bytes).to_vec())
                },
                DigestAlgorithm::Sha512 => {
                    (OID_SHA512, ID_SHA_512, Sha512::digest(signed_bytes).to_vec())
                },
            };

        // ── Parse signer certificate (need issuer + serial for SignerInfo) ──
        let cert = X509Certificate::from_der(&self.credentials.certificate)
            .map_err(|e| Error::InvalidPdf(format!("cannot parse signer certificate: {e}")))?;
        let issuer_der = cert
            .tbs_certificate()
            .issuer()
            .to_der()
            .map_err(|e| Error::InvalidPdf(format!("cannot DER-encode issuer: {e}")))?;
        let serial_der = cert
            .tbs_certificate()
            .serial_number()
            .to_der()
            .map_err(|e| Error::InvalidPdf(format!("cannot DER-encode serial: {e}")))?;

        // ── Parse RSA private key (PKCS#8, fall back to PKCS#1) ────────
        let rsa_key = RsaPrivateKey::from_pkcs8_der(&self.credentials.private_key)
            .or_else(|_| {
                use pkcs1::DecodeRsaPrivateKey;
                RsaPrivateKey::from_pkcs1_der(&self.credentials.private_key)
            })
            .map_err(|_| {
                Error::InvalidPdf("private key is not valid PKCS#8 or PKCS#1 RSA DER".into())
            })?;

        // ── #230 Phase D: governed RSA modulus-size floor ─────────────
        // A `strict`/`fips-strict`/`cnsa2` policy refuses to *sign* with
        // a weak RSA key (NIST SP 800-131A ≥2048; CNSA 2.0 ≥3072) — the
        // strength gate that `min_security_bits` (algorithm-level)
        // cannot see, since key size is a property of the key, not the
        // algorithm id. Default `compat` keeps the floor at 0 (no
        // behaviour change).
        let modulus_bits = {
            use rsa::traits::PublicKeyParts;
            rsa_key.n().bits()
        };
        if crate::crypto::active_policy().rsa_modulus_allowed(modulus_bits)
            == crate::crypto::Decision::Deny
        {
            return Err(Error::Unsupported(format!(
                "RSA signing key modulus is {modulus_bits} bits; the active crypto \
                 SecurityPolicy requires at least {} (#230 Phase D)",
                crate::crypto::active_policy().min_rsa_modulus_bits()
            )));
        }

        // ── Signed attributes ──────────────────────────────────────────
        // Attribute 1: id-contentType = id-data
        let attr_ct = {
            let mut c = Vec::new();
            c.extend(der_oid(OID_CONTENT_TYPE));
            c.extend(der_set(&der_oid(OID_DATA)));
            der_sequence(&c)
        };
        // Attribute 2: id-messageDigest = hash of signed_bytes
        let attr_md = {
            let mut c = Vec::new();
            c.extend(der_oid(OID_MSG_DIGEST));
            c.extend(der_set(&der_octet_string(&message_digest)));
            der_sequence(&c)
        };
        // Canonical SET-OF order (X.690 §11.6 / RFC 5652 §5.4): compare
        // element encodings as octet strings. All three are `30 LL 06
        // <oidlen> …`; content-type/message-digest OIDs are 9 bytes
        // (`06 09 … 09 03` / `… 09 04`) so ct < md; the ESS
        // signing-certificate-v2 OID is 11 bytes (`06 0B …`) and `0B`
        // > `09`, so ESS sorts strictly LAST. Appending it (only on the
        // PAdES path) therefore keeps the legacy ct‖md bytes
        // byte-identical (#235 plan Q3) while being canonically correct.
        let mut attrs_content = Vec::new();
        attrs_content.extend(&attr_ct);
        attrs_content.extend(&attr_md);
        if let Some(ess) = ess_attr {
            attrs_content.extend_from_slice(ess);
        }

        // For hashing: SET tag (RFC 5652 §5.4)
        let attrs_for_hashing = der_set(&attrs_content);
        // For SignerInfo storage: [0] IMPLICIT replaces the SET tag
        let attrs_for_storage = der_tag(0xA0, &attrs_content);

        // ── Sign: hash(signed_attrs SET) → DigestInfo → RSA sign ───────
        let attrs_hash: Vec<u8> = match self.options.digest_algorithm {
            DigestAlgorithm::Sha1 => Sha1::digest(&attrs_for_hashing).to_vec(),
            DigestAlgorithm::Sha256 => Sha256::digest(&attrs_for_hashing).to_vec(),
            DigestAlgorithm::Sha384 => Sha384::digest(&attrs_for_hashing).to_vec(),
            DigestAlgorithm::Sha512 => Sha512::digest(&attrs_for_hashing).to_vec(),
        };
        let di_prefix = digest_info_prefix(digest_oid)
            .ok_or_else(|| Error::InvalidPdf("no DigestInfo prefix for digest OID".into()))?;
        let mut digest_info_bytes = Vec::with_capacity(di_prefix.len() + attrs_hash.len());
        digest_info_bytes.extend_from_slice(di_prefix);
        digest_info_bytes.extend_from_slice(&attrs_hash);
        let sig_bytes = rsa_key
            .sign(Pkcs1v15Sign::new_unprefixed(), &digest_info_bytes)
            .map_err(|e| Error::InvalidPdf(format!("RSA signing failed: {e}")))?;

        // ── B-T: unsigned signature-time-stamp attribute ────────────────
        // RFC 3161 token over the signature value (SignerInfo.signature
        // content). UNSIGNED — does not change the hashed signedAttrs,
        // so the signature stays valid and the signed bytes are
        // byte-identical to the B-B form (#235 plan §4 / I7).
        // SignerInfo.unsignedAttrs ::= [1] IMPLICIT SET OF Attribute, so
        // the [1] (0xA1) tag wraps the (single) attribute directly.
        let unsigned_attrs: Option<Vec<u8>> = match timestamper {
            Some(ts) => {
                let token = ts(&sig_bytes)?;
                let attr = crate::signatures::pades::build_signature_timestamp_attr(&token)?;
                Some(der_tag(0xA1, &attr))
            },
            None => None,
        };

        // ── Build SignerInfo ────────────────────────────────────────────
        let signer_info = {
            // IssuerAndSerialNumber SEQUENCE
            let mut isn = Vec::new();
            isn.extend(&issuer_der);
            isn.extend(&serial_der);
            let isn = der_sequence(&isn);

            // digestAlgorithm (no parameters for SHA-*)
            let digest_alg = der_sequence(&der_oid(digest_oid_bytes));

            // signatureAlgorithm: rsaEncryption with NULL params
            let sig_alg = {
                let mut c = Vec::new();
                c.extend(der_oid(OID_RSA_ENC));
                c.extend_from_slice(&[0x05, 0x00]); // NULL
                der_sequence(&c)
            };

            let mut si = Vec::new();
            si.extend(der_integer(1));
            si.extend(isn);
            si.extend(digest_alg);
            si.extend(attrs_for_storage);
            si.extend(sig_alg);
            si.extend(der_octet_string(&sig_bytes));
            if let Some(ref ua) = unsigned_attrs {
                si.extend_from_slice(ua);
            }
            der_sequence(&si)
        };

        // ── Build SignedData SEQUENCE ───────────────────────────────────
        let signed_data = {
            // digestAlgorithms SET { SHA-* }
            let digest_algs = der_set(&der_sequence(&der_oid(digest_oid_bytes)));

            // encapContentInfo: id-data, no eContent (detached)
            let encap_ci = der_sequence(&der_oid(OID_DATA));

            // certificates [0] IMPLICIT: the signer cert DER
            let certs = der_tag(0xA0, &self.credentials.certificate);

            // signerInfos SET { signer_info }
            let signer_infos = der_set(&signer_info);

            let mut sd = Vec::new();
            sd.extend(der_integer(1)); // version
            sd.extend(digest_algs);
            sd.extend(encap_ci);
            sd.extend(certs);
            sd.extend(signer_infos);
            der_sequence(&sd)
        };

        // ── Build ContentInfo ───────────────────────────────────────────
        let mut ci = Vec::new();
        ci.extend(der_oid(OID_SIGNED_DATA));
        ci.extend(der_tag(0xA0, &signed_data)); // [0] EXPLICIT wraps SignedData
        Ok(der_sequence(&ci))
    }

    /// Calculate the ByteRange for a prepared PDF.
    pub fn calculate_byte_range(&self, file_size: usize, contents_offset: usize) -> [i64; 4] {
        self.byte_range_calc
            .calculate_byte_range(file_size, contents_offset)
    }

    /// Extract the bytes to be signed from the PDF.
    pub fn extract_signed_bytes(pdf_data: &[u8], byte_range: &[i64; 4]) -> Result<Vec<u8>> {
        ByteRangeCalculator::extract_signed_bytes(pdf_data, byte_range)
    }

    /// Insert the signature into the prepared PDF.
    pub fn insert_signature(
        &self,
        pdf_data: &mut [u8],
        contents_offset: usize,
        signature: &[u8],
    ) -> Result<()> {
        // Convert signature to hex
        let signature_hex = bytes_to_hex(signature);
        self.byte_range_calc
            .insert_signature(pdf_data, contents_offset, &signature_hex)
    }

    /// Get the signing options.
    pub fn options(&self) -> &SignOptions {
        &self.options
    }

    /// Get the signing credentials (certificate info only).
    pub fn credentials(&self) -> &SigningCredentials {
        &self.credentials
    }
}

// Minimal DER/ASN.1 encoders for RFC 5652 SignedData construction now
// live in `super::der_util` (#235 TODO #2 — single source of truth,
// shared with the PAdES ESS/ts-attr/DSS writers).
#[cfg(feature = "signatures")]
use super::der_util::{der_integer, der_octet_string, der_oid, der_sequence, der_set, der_tag};

// ────────────────────────────────────────────────────────────────────────────

/// Convert bytes to uppercase hex string.
fn bytes_to_hex(bytes: &[u8]) -> String {
    const HEX_CHARS: &[u8] = b"0123456789ABCDEF";
    let mut hex = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        hex.push(HEX_CHARS[(byte >> 4) as usize] as char);
        hex.push(HEX_CHARS[(byte & 0x0F) as usize] as char);
    }
    hex
}

/// Escape special characters in a PDF string.
fn escape_pdf_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 10);
    for c in s.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '(' => result.push_str("\\("),
            ')' => result.push_str("\\)"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            _ => result.push(c),
        }
    }
    result
}

/// Current time as a PDF date string. Delegates to the single
/// leap-year-correct implementation; the prior local copy hard-coded
/// month/day to "0101" and approximated the year as 1970 + days/365,
/// corrupting every signature /M date (README latent bug). WASM note:
/// SystemTime::now() in the shared helper still needs cfg-gating if
/// signatures are ever enabled for wasm32 (currently masked).
fn format_pdf_date() -> String {
    super::pdf_date::format_pdf_date_utc()
}

// SignOptions is re-exported from super::types

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_pdf_string() {
        assert_eq!(escape_pdf_string("Hello"), "Hello");
        assert_eq!(escape_pdf_string("Hello (World)"), "Hello \\(World\\)");
        assert_eq!(escape_pdf_string("Line1\nLine2"), "Line1\\nLine2");
        assert_eq!(escape_pdf_string("Path\\to\\file"), "Path\\\\to\\\\file");
    }

    #[test]
    fn test_format_pdf_date() {
        let date = format_pdf_date();
        assert!(date.starts_with("D:"));
        assert!(date.ends_with("Z"));
    }

    #[test]
    fn test_signer_placeholder() {
        let creds = SigningCredentials::new(vec![], vec![]);
        let opts = SignOptions {
            estimated_size: 1024,
            ..Default::default()
        };
        let signer = PdfSigner::new(creds, opts);

        let placeholder = signer.generate_contents_placeholder();
        // 1024 * 2 + 2 = 2050 characters
        assert_eq!(placeholder.len(), 2050);
        assert!(placeholder.starts_with('<'));
        assert!(placeholder.ends_with('>'));
    }

    #[test]
    fn test_build_signature_dictionary() {
        let creds = SigningCredentials::new(vec![], vec![]);
        let opts = SignOptions {
            reason: Some("Test signing".to_string()),
            location: Some("Test City".to_string()),
            ..Default::default()
        };
        let signer = PdfSigner::new(creds, opts);

        let dict = signer.build_signature_dictionary();
        assert!(dict.contains("/Type /Sig"));
        assert!(dict.contains("/Filter /Adobe.PPKLite"));
        assert!(dict.contains("/SubFilter /adbe.pkcs7.detached"));
        assert!(dict.contains("/Reason (Test signing)"));
        assert!(dict.contains("/Location (Test City)"));
        assert!(dict.contains("/ByteRange"));
        assert!(dict.contains("/M (D:"));
    }

    #[test]
    fn test_calculate_byte_range() {
        let creds = SigningCredentials::new(vec![], vec![]);
        let opts = SignOptions {
            estimated_size: 50, // 50 bytes = 102 char placeholder
            ..Default::default()
        };
        let signer = PdfSigner::new(creds, opts);

        let byte_range = signer.calculate_byte_range(1000, 400);
        assert_eq!(byte_range[0], 0);
        assert_eq!(byte_range[1], 400);
        assert_eq!(byte_range[2], 502); // 400 + 102
        assert_eq!(byte_range[3], 498); // 1000 - 502
    }

    #[test]
    #[cfg(feature = "signatures")]
    fn test_sign_produces_valid_cms_blob() {
        use super::super::cms_verify::SignerVerify;
        use super::super::types::SignOptions;
        use super::super::{verify_signer_detached, SigningCredentials};

        let cert_pem = std::fs::read_to_string("tests/fixtures/test_signing_cert.pem")
            .expect("test fixture must exist");
        let key_pem = std::fs::read_to_string("tests/fixtures/test_signing_key.pem")
            .expect("test fixture must exist");
        let creds =
            SigningCredentials::from_pem(&cert_pem, &key_pem).expect("credentials must load");

        let content = b"hello world this is the signed PDF content";
        let signer = PdfSigner::new(creds, SignOptions::default());
        let cms_blob = signer.sign(content).expect("sign must succeed");

        // The produced blob must be parseable and verifiable
        let result = verify_signer_detached(&cms_blob, content)
            .expect("verify_signer_detached must not error");
        assert_eq!(
            result,
            SignerVerify::Valid,
            "signature must verify as Valid with the same content"
        );
    }

    #[test]
    #[cfg(feature = "signatures")]
    fn test_sign_pades_adds_ess_and_still_verifies() {
        use super::super::cms_verify::SignerVerify;
        use super::super::types::SignOptions;
        use super::super::{verify_signer_detached, SigningCredentials};

        let cert_pem = std::fs::read_to_string("tests/fixtures/test_signing_cert.pem").unwrap();
        let key_pem = std::fs::read_to_string("tests/fixtures/test_signing_key.pem").unwrap();
        let creds = SigningCredentials::from_pem(&cert_pem, &key_pem).unwrap();
        let content = b"PAdES-B-B content under signature";
        let signer = PdfSigner::new(creds, SignOptions::default());

        // id-aa-signingCertificateV2 OID content octets.
        const ESS_OID: &[u8] = &[
            0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x09, 0x10, 0x02, 0x2F,
        ];

        // Legacy path: still Valid, and carries NO ESS attribute
        // (byte-compat for existing adbe.pkcs7.detached users — plan Q3).
        let legacy = signer.sign(content).unwrap();
        assert_eq!(verify_signer_detached(&legacy, content).unwrap(), SignerVerify::Valid);
        assert!(
            !legacy.windows(ESS_OID.len()).any(|w| w == ESS_OID),
            "legacy sign() must not add the ESS attribute"
        );

        // PAdES path: the ESS signing-certificate-v2 attr is present AND
        // the signature still verifies (the signed-attrs hash + RSA sign
        // correctly account for the extra signed attribute — the core
        // TODO #4 correctness check, short of the EU-DSS validator).
        let pades = signer.sign_pades(content).unwrap();
        assert!(
            pades.windows(ESS_OID.len()).any(|w| w == ESS_OID),
            "sign_pades() must embed the ESS signing-certificate-v2 attr"
        );
        assert_eq!(
            verify_signer_detached(&pades, content).unwrap(),
            SignerVerify::Valid,
            "PAdES signature with ESS must still verify as Valid"
        );
        // Tampered content must fail for the PAdES blob too.
        assert_ne!(
            verify_signer_detached(&pades, b"different content").unwrap(),
            SignerVerify::Valid
        );
    }

    #[test]
    #[cfg(feature = "signatures")]
    fn test_sign_pades_t_embeds_timestamp_and_classifies_bt() {
        use super::super::cms_verify::SignerVerify;
        use super::super::types::{SignOptions, SignatureInfo};
        use super::super::{classify_pades_level, verify_signer_detached, SigningCredentials};
        use crate::signatures::PadesLevel;

        let cert_pem = std::fs::read_to_string("tests/fixtures/test_signing_cert.pem").unwrap();
        let key_pem = std::fs::read_to_string("tests/fixtures/test_signing_key.pem").unwrap();
        let creds = SigningCredentials::from_pem(&cert_pem, &key_pem).unwrap();
        let content = b"PAdES-B-T content under signature";
        let signer = PdfSigner::new(creds, SignOptions::default());

        // Offline stub TSA: returns a minimal well-formed DER SEQUENCE
        // standing in for an RFC 3161 TimeStampToken (no network in unit
        // tests — feature plan §5.1). It must receive the signature
        // value to timestamp.
        let seen = std::cell::RefCell::new(Vec::new());
        let token: &dyn Fn(&[u8]) -> Result<Vec<u8>> = &|sig: &[u8]| {
            *seen.borrow_mut() = sig.to_vec();
            Ok(vec![0x30, 0x07, 0x02, 0x01, 0x01, 0x04, 0x02, b't', b's'])
        };

        let b_b = signer.sign_pades(content).unwrap();
        let b_t = signer.sign_pades_t(content, token).unwrap();

        // The timestamper was invoked over the (non-empty) signature value.
        assert!(!seen.borrow().is_empty(), "timestamper must see the sig value");

        // I7: B-T does not change the signed bytes — the RSA signature
        // value is deterministic and identical to the B-B form (only an
        // UNSIGNED attribute was added). The 256-byte sig appears in both.
        let sig = &seen.borrow().clone();
        assert!(b_b.windows(sig.len()).any(|w| w == sig.as_slice()));
        assert!(b_t.windows(sig.len()).any(|w| w == sig.as_slice()));

        // The B-T CMS still verifies (unsigned attr is outside the
        // signed data).
        assert_eq!(verify_signer_detached(&b_t, content).unwrap(), SignerVerify::Valid);

        // id-aa-signatureTimeStampToken OID present, and the real cms
        // decoder (via classify) sees it as an unsigned attr ⇒ B-T.
        const TS_OID: &[u8] = &[
            0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x09, 0x10, 0x02, 0x0E,
        ];
        assert!(b_t.windows(TS_OID.len()).any(|w| w == TS_OID));
        let info = SignatureInfo {
            signer_name: None,
            signing_time: None,
            reason: None,
            location: None,
            contact_info: None,
            sub_filter: None,
            covers_whole_document: false,
            byte_range: vec![],
            certificate_cn: None,
            certificate_issuer: None,
            valid_from: None,
            valid_to: None,
            contents: Some(b_t.clone()),
        };
        assert_eq!(classify_pades_level(&info, None), PadesLevel::BT);
        // The plain B-B blob (no ts attr) classifies as BB.
        let info_bb = SignatureInfo {
            contents: Some(b_b),
            ..info
        };
        assert_eq!(classify_pades_level(&info_bb, None), PadesLevel::BB);
    }

    #[test]
    #[cfg(feature = "signatures")]
    fn test_sign_detects_tampered_content() {
        use super::super::cms_verify::SignerVerify;
        use super::super::types::SignOptions;
        use super::super::{verify_signer_detached, SigningCredentials};

        let cert_pem = std::fs::read_to_string("tests/fixtures/test_signing_cert.pem")
            .expect("test fixture must exist");
        let key_pem = std::fs::read_to_string("tests/fixtures/test_signing_key.pem")
            .expect("test fixture must exist");
        let creds =
            SigningCredentials::from_pem(&cert_pem, &key_pem).expect("credentials must load");

        let content = b"original content";
        let tampered = b"tampered content!";
        let signer = PdfSigner::new(creds, SignOptions::default());
        let cms_blob = signer.sign(content).expect("sign must succeed");

        let result = verify_signer_detached(&cms_blob, tampered)
            .expect("verify must not error on tampered content");
        assert_eq!(result, SignerVerify::Invalid, "tampered content must verify as Invalid");
    }

    #[test]
    #[cfg(feature = "signatures")]
    fn test_sign_via_pkcs12() {
        use super::super::cms_verify::SignerVerify;
        use super::super::types::SignOptions;
        use super::super::{verify_signer_detached, SigningCredentials};

        let p12_data =
            std::fs::read("tests/fixtures/test_signing.p12").expect("test fixture must exist");
        let creds =
            SigningCredentials::from_pkcs12(&p12_data, "testpass").expect("PKCS#12 must load");

        let content = b"PDF content for pkcs12 signing test";
        let signer = PdfSigner::new(creds, SignOptions::default());
        let cms_blob = signer.sign(content).expect("sign must succeed");

        let result = verify_signer_detached(&cms_blob, content).expect("verify must not error");
        assert_eq!(result, SignerVerify::Valid, "PKCS#12-signed blob must verify as Valid");
    }
}
