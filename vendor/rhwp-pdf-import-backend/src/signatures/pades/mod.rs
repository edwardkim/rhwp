//! PAdES Advanced Electronic Signatures — long-term-validation types
//! (#235, feature plan §3.2).
//!
//! This module owns the **stable public vocabulary** for PAdES baseline
//! levels and the read-side Document Security Store, per
//! ETSI EN 319 142-1 §5 and ISO 32000-2:2020 §12.8.4.3 (DSS/VRI — *not*
//! in the bundled ISO 32000-1 `docs/spec/pdf.md`; implemented from the
//! ETSI/ISO text).
//!
//! The signature-correctness-critical pieces — the ESS
//! `signing-certificate-v2` signed attribute, the B-T
//! `signature-time-stamp` unsigned attribute, and the
//! Catalog-overriding DSS incremental-update appender — are
//! **implemented and shipped as of v0.3.50 (#235)** and re-exported
//! from this module (see `pub use` below). The **EU DSS demonstration
//! validator** is the release *conformance gate* (a manual release
//! verification step — a single ASN.1/byte-range error silently
//! produces an invalid signature), not a reason these pieces are
//! absent.
//!
//! [`PadesLevel`] is `#[non_exhaustive]` and its integer mapping is
//! frozen now (BB=0, BT=1, BLt=2, BLta=3) so every binding's enum
//! mapping is stable from v0.3.50 onward (OCP — adding behaviour later
//! is non-breaking).

mod dss;
mod dss_read;
mod ess;
mod level;
mod ts_attr;

pub use dss::append_dss;
pub use dss_read::{parse_dss, read_dss};
pub use ess::build_signing_certificate_v2;
pub use level::{classify_pades_level, has_document_timestamp, vri_key};
pub use ts_attr::build_signature_timestamp_attr;

/// PAdES baseline level. Each level is a strict superset of the one
/// below (ETSI EN 319 142-1 §5). Ordered `BB < BT < BLt < BLta`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum PadesLevel {
    /// CAdES-B-B baseline embedded as `ETSI.CAdES.detached` (signed
    /// attrs incl. ESS `signing-certificate-v2`, RFC 5035).
    BB,
    /// B-B + an RFC 3161 `signature-time-stamp` **unsigned** attribute
    /// (OID `1.2.840.113549.1.9.16.2.14`) over the signature value.
    BT,
    /// B-T + a Document Security Store (certs/CRLs/OCSPs + per-signature
    /// VRI) added by a separate incremental update.
    BLt,
    /// B-LT + a document timestamp. Produced by appending a document
    /// `/DocTimeStamp` (ETSI.RFC3161) over the B-LT file via
    /// [`sign_pdf_bytes_pades`](crate::signatures::sign_pdf_bytes_pades)
    /// (shipped v0.3.50, #235).
    BLta,
}

impl PadesLevel {
    /// Frozen integer code for the C ABI / all bindings
    /// (BB=0, BT=1, BLt=2, BLta=3). **Never renumber** — three FFI
    /// consumers depend on it (feature plan §7.1).
    pub fn code(self) -> i32 {
        match self {
            PadesLevel::BB => 0,
            PadesLevel::BT => 1,
            PadesLevel::BLt => 2,
            PadesLevel::BLta => 3,
        }
    }

    /// Inverse of [`code`](PadesLevel::code); unknown codes ⇒ `None`.
    pub fn from_code(code: i32) -> Option<PadesLevel> {
        match code {
            0 => Some(PadesLevel::BB),
            1 => Some(PadesLevel::BT),
            2 => Some(PadesLevel::BLt),
            3 => Some(PadesLevel::BLta),
            _ => None,
        }
    }
}

/// All validation material for one signature's chain plus its
/// timestamp's TSA chain — supplied offline by the caller or gathered
/// by the (future, feature-gated) revocation client. Each entry is a
/// raw DER blob (RFC 5280 cert / CRL, RFC 6960 OCSPResponse); carried
/// opaquely into DSS streams (feature plan §3.2, R4).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct RevocationMaterial {
    /// DER X.509 certificates (full chain: signer + TSA).
    pub certificates: Vec<Vec<u8>>,
    /// DER CRLs.
    pub crls: Vec<Vec<u8>>,
    /// DER `OCSPResponse` (RFC 6960).
    pub ocsp_responses: Vec<Vec<u8>>,
}

impl RevocationMaterial {
    /// `true` if no certificate, CRL, or OCSP response is present.
    pub fn is_empty(&self) -> bool {
        self.certificates.is_empty() && self.crls.is_empty() && self.ocsp_responses.is_empty()
    }
}

/// One `/VRI` (Validation-Related Information) entry — the validation
/// material for a single signature, keyed by the uppercase-hex SHA-1 of
/// that signature's `/Contents` (ISO 32000-2:2020 §12.8.4.3 /
/// ETSI EN 319 142-1).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct VriEntry {
    /// Uppercase hex SHA-1 of the signature's `/Contents` (the VRI key).
    pub signature_digest: String,
    /// DER certificates scoped to this signature.
    pub certificates: Vec<Vec<u8>>,
    /// DER CRLs scoped to this signature.
    pub crls: Vec<Vec<u8>>,
    /// DER OCSP responses scoped to this signature.
    pub ocsp_responses: Vec<Vec<u8>>,
    /// `/TU` validation time as a PDF date string, if present.
    pub timestamp: Option<String>,
}

/// A parsed Document Security Store (read side) — Catalog `/DSS`
/// (ISO 32000-2:2020 §12.8.4.3).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct DocumentSecurityStore {
    /// Document-level DER certificates (`/Certs`).
    pub certificates: Vec<Vec<u8>>,
    /// Document-level DER CRLs (`/CRLs`).
    pub crls: Vec<Vec<u8>>,
    /// Document-level DER OCSP responses (`/OCSPs`).
    pub ocsp_responses: Vec<Vec<u8>>,
    /// Per-signature `/VRI` entries.
    pub vri: Vec<VriEntry>,
}

impl DocumentSecurityStore {
    /// Whether the store carries no material at all.
    pub fn is_empty(&self) -> bool {
        self.certificates.is_empty()
            && self.crls.is_empty()
            && self.ocsp_responses.is_empty()
            && self.vri.is_empty()
    }

    /// The `/VRI` entry whose key matches `signature_digest` (uppercase
    /// hex SHA-1 of a signature's `/Contents`), if any.
    pub fn vri_for(&self, signature_digest: &str) -> Option<&VriEntry> {
        self.vri
            .iter()
            .find(|e| e.signature_digest == signature_digest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_codes_are_frozen_and_round_trip() {
        // The C ABI / every binding depends on these exact values.
        assert_eq!(PadesLevel::BB.code(), 0);
        assert_eq!(PadesLevel::BT.code(), 1);
        assert_eq!(PadesLevel::BLt.code(), 2);
        assert_eq!(PadesLevel::BLta.code(), 3);
        for lvl in [
            PadesLevel::BB,
            PadesLevel::BT,
            PadesLevel::BLt,
            PadesLevel::BLta,
        ] {
            assert_eq!(PadesLevel::from_code(lvl.code()), Some(lvl));
        }
        assert_eq!(PadesLevel::from_code(4), None);
        assert_eq!(PadesLevel::from_code(-1), None);
    }

    #[test]
    fn level_ordering_is_superset_chain() {
        assert!(PadesLevel::BB < PadesLevel::BT);
        assert!(PadesLevel::BT < PadesLevel::BLt);
        assert!(PadesLevel::BLt < PadesLevel::BLta);
    }

    #[test]
    fn revocation_material_is_empty() {
        assert!(RevocationMaterial::default().is_empty());
        let m = RevocationMaterial {
            certificates: vec![vec![0x30, 0x82]],
            ..RevocationMaterial::default()
        };
        assert!(!m.is_empty());
    }

    #[test]
    fn dss_empty_and_vri_lookup() {
        let mut dss = DocumentSecurityStore::default();
        assert!(dss.is_empty());
        assert!(dss.vri_for("ABCD").is_none());
        dss.vri.push(VriEntry {
            signature_digest: "ABCD1234".to_string(),
            ..VriEntry::default()
        });
        assert!(!dss.is_empty());
        assert_eq!(dss.vri_for("ABCD1234").map(|e| e.signature_digest.as_str()), Some("ABCD1234"));
        assert!(dss.vri_for("nope").is_none());
    }
}
