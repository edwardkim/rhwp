//! PAdES baseline-level classification (#235, feature plan §3.3 / §5.2,
//! TODO #13).
//!
//! Read-side only: inspects a signature's `/SubFilter`, its CMS
//! `SignerInfo` **unsigned** attributes, and the document's DSS/VRI to
//! decide whether it is `BB`, `BT`, or `BLt`
//! (ETSI EN 319 142-1 §5). Touches no signed bytes — zero risk to
//! existing signatures and independently verifiable without the EU DSS
//! validator (the validator is the conformance oracle for the *writer*;
//! the reader is proven by round-trip + a checked-in reference sample
//! per the feature plan §5.5).

use super::{DocumentSecurityStore, PadesLevel};
use crate::crypto::{self, HashAlgorithm};
use crate::signatures::types::SignatureInfo;
use cms::content_info::ContentInfo;
use cms::signed_data::SignedData;
use der::oid::ObjectIdentifier;
use der::{Decode, Encode, SliceReader};

/// `id-aa-signatureTimeStampToken` — the RFC 3161 timestamp carried as a
/// CMS **unsigned** attribute that distinguishes PAdES-B-T from B-B
/// (ETSI EN 319 142-1 §5; OID `1.2.840.113549.1.9.16.2.14`).
const OID_SIGNATURE_TIME_STAMP: ObjectIdentifier =
    ObjectIdentifier::new_unwrap("1.2.840.113549.1.9.16.2.14");

/// Whether the CMS blob's first `SignerInfo` carries the B-T signature
/// timestamp unsigned attribute. Malformed CMS ⇒ `false` (conservative:
/// never over-claim a level).
fn has_bt_timestamp(cms: &[u8]) -> bool {
    // A PDF `/Contents` value is the CMS DER followed by zero-padding to
    // the fixed-width placeholder, so it almost always has trailing
    // bytes. Decode one self-delimiting element via a reader (no
    // `finish()`), tolerating that padding — strict `from_der` would
    // reject it and silently down-classify every real signature to B-B.
    let Ok(mut reader) = SliceReader::new(cms) else {
        return false;
    };
    let Ok(ci) = ContentInfo::decode(&mut reader) else {
        return false;
    };
    let Ok(sd_bytes) = ci.content.to_der() else {
        return false;
    };
    let Ok(sd) = SignedData::from_der(&sd_bytes) else {
        return false;
    };
    let Some(signer) = sd.signer_infos.0.iter().next() else {
        return false;
    };
    match signer.unsigned_attrs.as_ref() {
        Some(attrs) => attrs.iter().any(|a| a.oid == OID_SIGNATURE_TIME_STAMP),
        None => false,
    }
}

/// The VRI key for a signature: uppercase hex of SHA-1 over the raw
/// `/Contents` byte string (ISO 32000-2:2020 §12.8.4.3 /
/// ETSI EN 319 142-1). SHA-1 here is a *spec-fixed naming* digest, not a
/// security signature; obtained via the active [`crypto`] provider so a
/// FIPS provider's policy is respected (it fails loud rather than
/// substituting — feature plan §8). `None` if the provider refuses SHA-1
/// (caller then cannot upgrade to B-LT, which is correct fail-safe).
pub fn vri_key(contents: &[u8]) -> Option<String> {
    let mut hasher = crypto::active().hasher(HashAlgorithm::Sha1).ok()?;
    hasher.update(contents);
    let digest = hasher.finalize();
    let mut s = String::with_capacity(digest.len() * 2);
    for b in digest {
        s.push_str(&format!("{b:02X}"));
    }
    Some(s)
}

/// Classify a signature's PAdES baseline level.
///
/// - `BB`  — a CMS signature with no B-T timestamp (the baseline; an
///   `ETSI.CAdES.detached` `/SubFilter` is the PAdES marker but a
///   missing/other SubFilter is still reported as `BB`, not a separate
///   "not PAdES" state — the v0.3.50 enum has no such variant).
/// - `BT`  — `BB` + the `id-aa-signatureTimeStampToken` unsigned attr.
/// - `BLt` — `BT` + a DSS `/VRI` entry keyed by this signature's
///   `uppercase-hex SHA-1(/Contents)`.
///
/// Never returns `BLta` (not produced in v0.3.50). Conservative: any
/// uncertainty (malformed CMS, provider-refused SHA-1, no matching VRI)
/// keeps the lower level — it never over-claims.
pub fn classify_pades_level(
    info: &SignatureInfo,
    dss: Option<&DocumentSecurityStore>,
) -> PadesLevel {
    let Some(contents) = info.contents.as_deref() else {
        return PadesLevel::BB;
    };

    if !has_bt_timestamp(contents) {
        return PadesLevel::BB;
    }

    // B-T present; upgrade to B-LT only if the DSS has a VRI entry whose
    // key matches this signature's Contents digest.
    if let (Some(dss), Some(key)) = (dss, vri_key(contents)) {
        if dss.vri_for(&key).is_some() {
            return PadesLevel::BLt;
        }
    }
    PadesLevel::BT
}

/// Whether the document carries a PAdES-B-LTA archival document
/// timestamp: a `/Type /DocTimeStamp` object with
/// `/SubFilter /ETSI.RFC3161` (ETSI EN 319 142-1 §5 / ISO 32000-2
/// §12.8.5).
///
/// B-LTA is inherently *document*-scoped — the timestamp is a separate
/// object covering the whole file (signature **and** its DSS), not a
/// property of any one signature — so it cannot be derived from the
/// signature-scoped [`classify_pades_level`] (whose `(info, dss)`
/// inputs and the frozen `pdf_signature_get_pades_level` C ABI have no
/// document handle). A reader concludes **B-LTA** when
/// `classify_pades_level(sig, dss) == BLt` **and**
/// `has_document_timestamp(file) == true`. Byte-scan (AcroForm-
/// independent, matching the rest of this module).
pub fn has_document_timestamp(pdf: &[u8]) -> bool {
    fn contains(hay: &[u8], needle: &[u8]) -> bool {
        needle.len() <= hay.len() && hay.windows(needle.len()).any(|w| w == needle)
    }
    contains(pdf, b"/DocTimeStamp") && contains(pdf, b"/ETSI.RFC3161")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signatures::pades::VriEntry;

    fn sig_with(contents: Option<Vec<u8>>) -> SignatureInfo {
        SignatureInfo {
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
            contents,
        }
    }

    #[test]
    fn no_contents_is_bb() {
        assert_eq!(classify_pades_level(&sig_with(None), None), PadesLevel::BB);
    }

    #[test]
    fn malformed_cms_is_bb_not_a_panic() {
        let s = sig_with(Some(b"definitely not a CMS blob".to_vec()));
        assert_eq!(classify_pades_level(&s, None), PadesLevel::BB);
        // Even with a DSS present, garbage stays BB (no over-claim).
        let dss = DocumentSecurityStore::default();
        assert_eq!(classify_pades_level(&s, Some(&dss)), PadesLevel::BB);
    }

    #[test]
    fn vri_key_is_uppercase_hex_sha1_of_contents() {
        // SHA-1("") = da39a3ee5e6b4b0d3255bfef95601890afd80709.
        let key = vri_key(b"").expect("provider supports SHA-1");
        assert_eq!(key, "DA39A3EE5E6B4B0D3255BFEF95601890AFD80709");
        // SHA-1("abc") = a9993e364706816aba3e25717850c26c9cd0d89d.
        assert_eq!(vri_key(b"abc").unwrap(), "A9993E364706816ABA3E25717850C26C9CD0D89D");
    }

    #[test]
    fn bt_requires_real_timestamp_attr_not_just_dss() {
        // A bogus CMS never reaches BT, so a DSS/VRI match cannot
        // spuriously upgrade it to B-LT (BT is a hard prerequisite).
        let contents = b"\x30\x03\x02\x01\x00".to_vec(); // trivial DER, not CMS
        let s = sig_with(Some(contents.clone()));
        let mut dss = DocumentSecurityStore::default();
        if let Some(k) = vri_key(&contents) {
            dss.vri.push(VriEntry {
                signature_digest: k,
                ..VriEntry::default()
            });
        }
        // No valid B-T attr ⇒ BB even though a VRI entry "matches".
        assert_eq!(classify_pades_level(&s, Some(&dss)), PadesLevel::BB);
    }
}
