//! RFC 3161 timestamp parsing.
//!
//! A "timestamp" in PDF-signature context is a CMS SignedData whose
//! `eContentInfo.eContent` holds a DER-encoded `TSTInfo` structure
//! (RFC 3161 §2.4.2). This module parses either the full TimeStampToken
//! (CMS-wrapped) or the inner bare TSTInfo — callers get the same
//! [`Timestamp`] back either way.
//!
//! Backed by the `x509-tsp` crate from the RustCrypto formats family.
//! All accessors surface through the `pdf_timestamp_*` FFI and, in
//! turn, every binding's idiomatic `Timestamp` type.

use crate::error::{Error, Result};
use crate::signatures::tsp::TstInfo;
use cms::cert::x509::ext::pkix::name::GeneralName;
use cms::signed_data::SignedData;
use der::{Decode, Encode};

/// Hash algorithm used for a message imprint (matches the enum values
/// the FFI uses for `pdf_timestamp_get_hash_algorithm`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    /// SHA-1 (RFC 3174). Legacy; weak for new timestamps but still
    /// valid when a TSA signed its token with it.
    Sha1 = 1,
    /// SHA-256 (FIPS 180-4) — the modern default.
    Sha256 = 2,
    /// SHA-384 (FIPS 180-4).
    Sha384 = 3,
    /// SHA-512 (FIPS 180-4).
    Sha512 = 4,
    /// Any other OID — treat as opaque.
    Unknown = 0,
}

/// Parsed RFC 3161 timestamp. Backed by the original DER bytes so
/// callers can still retrieve them for verification / re-embedding.
#[derive(Debug)]
pub struct Timestamp {
    token_bytes: Vec<u8>,
    tst: TstInfo,
}

impl Timestamp {
    /// Parse a DER blob that may be either a full TimeStampToken
    /// (CMS-wrapped) or the bare TSTInfo SEQUENCE. Tries the CMS path
    /// first and falls back to bare TSTInfo on failure — that matches
    /// what callers get from either `pdf_signature_get_timestamp`
    /// (CMS-wrapped) or a hand-constructed fixture (bare).
    pub fn from_der(token: &[u8]) -> Result<Self> {
        let token_bytes = token.to_vec();

        if let Some(tst) = decode_cms_wrapped(token) {
            return Ok(Self { token_bytes, tst });
        }

        let tst = TstInfo::from_der(token).map_err(|e| {
            Error::InvalidPdf(format!("not a valid TimeStampToken or TSTInfo: {e}"))
        })?;
        Ok(Self { token_bytes, tst })
    }

    /// Raw DER bytes of the original TimeStampToken / TSTInfo.
    pub fn token_bytes(&self) -> &[u8] {
        &self.token_bytes
    }

    /// Generation time as a Unix epoch (seconds).
    pub fn time(&self) -> i64 {
        self.tst.gen_time.to_unix_duration().as_secs() as i64
    }

    /// Serial number as a hex string (no `0x` prefix).
    pub fn serial(&self) -> String {
        hex_upper(self.tst.serial_number.as_bytes())
    }

    /// Policy OID in dotted-decimal form.
    pub fn policy_oid(&self) -> String {
        self.tst.policy.to_string()
    }

    /// TSA name when present in the token (RFC 3161 § 2.4.2 "tsa"
    /// GeneralName field), or empty when the TSA didn't include its
    /// name. For directory-name GeneralNames the result is the
    /// distinguished-name string; for URI / DNS forms the raw value.
    pub fn tsa_name(&self) -> String {
        match &self.tst.tsa {
            Some(GeneralName::DirectoryName(dn)) => dn.to_string(),
            Some(GeneralName::UniformResourceIdentifier(s)) => s.to_string(),
            Some(GeneralName::DnsName(s)) => s.to_string(),
            Some(GeneralName::Rfc822Name(s)) => s.to_string(),
            _ => String::new(),
        }
    }

    /// Hash algorithm of the message imprint.
    pub fn hash_algorithm(&self) -> HashAlgorithm {
        super::crypto::hash_algorithm_from_oid(self.tst.message_imprint.hash_algorithm.oid)
    }

    /// The raw message-imprint hash bytes (cloned).
    pub fn message_imprint(&self) -> Vec<u8> {
        self.tst.message_imprint.hashed_message.as_bytes().to_vec()
    }

    /// Borrowed view of the message-imprint hash bytes — valid for
    /// the lifetime of the `Timestamp`. Used by the FFI layer so it
    /// doesn't have to hand out caller-freed buffers for every call.
    pub fn message_imprint_ref(&self) -> &[u8] {
        self.tst.message_imprint.hashed_message.as_bytes()
    }

    /// Cryptographically verify this TimeStampToken.
    ///
    /// Parses the outer CMS SignedData, extracts the encapsulated TSTInfo
    /// bytes, then calls [`super::cms_verify::verify_signer_detached`] — the same
    /// path used for PDF signatures, covering RSA-PKCS#1 v1.5, RSA-PSS,
    /// and ECDSA P-256/P-384.
    ///
    /// Returns `Ok(true)` when the TSA's signer crypto and the
    /// `messageDigest` attribute both pass. Returns `Ok(false)` when a
    /// crypto check fails (tampered token or wrong key). Returns `Err` when
    /// the token is not CMS-wrapped or uses an unsupported algorithm.
    pub fn verify(&self) -> Result<bool> {
        use cms::content_info::ContentInfo;
        use cms::signed_data::SignedData;

        // Only CMS-wrapped tokens can be verified — bare TSTInfo carries no
        // outer SignedData and therefore no TSA signature to check.
        let content = ContentInfo::from_der(&self.token_bytes).map_err(|_| {
            Error::InvalidPdf("timestamp token is not CMS-wrapped; cannot verify signature".into())
        })?;
        let sd_bytes = content.content.to_der().map_err(|e| {
            Error::InvalidPdf(format!("failed to re-encode timestamp ContentInfo: {e}"))
        })?;
        let sd = SignedData::from_der(&sd_bytes).map_err(|e| {
            Error::InvalidPdf(format!("timestamp token is not valid SignedData: {e}"))
        })?;
        let econtent = sd.encap_content_info.econtent.as_ref().ok_or_else(|| {
            Error::InvalidPdf("timestamp SignedData has no encapsulated TSTInfo content".into())
        })?;
        // econtent.value() returns the raw TSTInfo DER bytes — exactly what
        // the TSA hashed when building its messageDigest signed attribute.
        let tst_bytes = econtent.value().to_vec();

        match super::cms_verify::verify_signer_detached(&self.token_bytes, &tst_bytes) {
            Ok(super::cms_verify::SignerVerify::Valid) => Ok(true),
            Ok(super::cms_verify::SignerVerify::Invalid) => Ok(false),
            Ok(super::cms_verify::SignerVerify::Unknown) => Err(Error::InvalidPdf(
                "timestamp TSA uses an algorithm not yet supported by the verifier".into(),
            )),
            Err(e) => Err(e),
        }
    }
}

/// Try to decode `bytes` as a full CMS-wrapped TimeStampToken. Returns
/// `None` on any failure — the caller falls back to bare TSTInfo.
fn decode_cms_wrapped(bytes: &[u8]) -> Option<TstInfo> {
    let content = cms::content_info::ContentInfo::from_der(bytes).ok()?;
    let sd = SignedData::from_der(&content.content.to_der().ok()?).ok()?;
    let econtent = sd.encap_content_info.econtent?;
    TstInfo::from_der(econtent.value()).ok()
}

fn hex_upper(bytes: &[u8]) -> String {
    static HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0F) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    /// The reference TimeStampResp from the x509-tsp crate's own
    /// tests — openssl-generated, 2023-06-07 gen_time.
    const TSP_RESPONSE: &[u8] = &hex!(
        "3082028403030201003082027B06092A864886F70D010702A082026C30820268020103310F300D06096086480165030402010500"
    );

    #[test]
    fn timestamp_from_bare_tstinfo() {
        // Extracted from the crate's reference response — this is the
        // TSTInfo SEQUENCE hand-unwrapped from TSP_RESPONSE above. The
        // bytes below were produced by running the x509-tsp
        // `response_test` and calling `.to_der()` on the TstInfo.
        let bare_tstinfo: &[u8] = &hex!(
            "3081B302010106042A0304013031300D060960864801650304020105000420BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD020104180F32303233303630373131323632365A300A020101800201F48101640101FF0208314CFCE4E0651827A048A4463044310B30090603550406130255533113301106035504080C0A536F6D652D5374617465310D300B060355040A0C04546573743111300F06035504030C085465737420545341"
        );
        let ts = Timestamp::from_der(bare_tstinfo).expect("parse bare TSTInfo");
        assert_eq!(ts.time(), 1_686_137_186); // 2023-06-07T11:26:26Z
        assert_eq!(ts.serial(), "04");
        assert_eq!(ts.policy_oid(), "1.2.3.4.1");
        assert_eq!(ts.hash_algorithm(), HashAlgorithm::Sha256);
        assert_eq!(
            ts.message_imprint(),
            hex!("BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD")
        );
        assert_eq!(ts.tsa_name(), "CN=Test TSA,O=Test,ST=Some-State,C=US");
        assert_eq!(ts.token_bytes(), bare_tstinfo);
    }

    #[test]
    fn timestamp_rejects_garbage() {
        let err = Timestamp::from_der(b"not a timestamp").unwrap_err();
        assert!(matches!(err, Error::InvalidPdf(_)), "expected InvalidPdf, got {err:?}");
    }

    #[test]
    fn verify_bare_tstinfo_returns_err() {
        // A Timestamp parsed from bare TSTInfo bytes has no outer CMS
        // SignedData and therefore no TSA signature to verify.
        let bare_tstinfo: &[u8] = &hex!(
            "3081B302010106042A0304013031300D060960864801650304020105000420BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD020104180F32303233303630373131323632365A300A020101800201F48101640101FF0208314CFCE4E0651827A048A4463044310B30090603550406130255533113301106035504080C0A536F6D652D5374617465310D300B060355040A0C04546573743111300F06035504030C085465737420545341"
        );
        let ts = Timestamp::from_der(bare_tstinfo).expect("parse bare TSTInfo");
        let err = ts.verify().unwrap_err();
        assert!(matches!(err, Error::InvalidPdf(_)), "expected InvalidPdf, got {err:?}");
    }

    #[test]
    fn hash_algorithm_variants_match_ffi_enum() {
        // The FFI `pdf_timestamp_get_hash_algorithm` returns an i32
        // that every binding's Timestamp class decodes — pin the
        // numeric contract so we can't silently renumber.
        assert_eq!(HashAlgorithm::Sha1 as i32, 1);
        assert_eq!(HashAlgorithm::Sha256 as i32, 2);
        assert_eq!(HashAlgorithm::Sha384 as i32, 3);
        assert_eq!(HashAlgorithm::Sha512 as i32, 4);
        assert_eq!(HashAlgorithm::Unknown as i32, 0);
    }
}
