//! PAdES-B-T `signature-time-stamp` **unsigned** attribute (#235,
//! feature plan §3.1 / §5.2, TODO #6).
//!
//! B-T = B-B + an RFC 3161 timestamp over the signature value, carried
//! as a CMS *unsigned* attribute in the `SignerInfo`
//! (ETSI EN 319 142-1 §5):
//!
//! ```text
//! Attribute ::= SEQUENCE {
//!   attrType   OBJECT IDENTIFIER  -- id-aa-signatureTimeStampToken
//!                                 --   1.2.840.113549.1.9.16.2.14
//!   attrValues SET OF TimeStampToken }
//! ```
//!
//! The `TimeStampToken` is exactly the DER a TSA returns (a CMS
//! `ContentInfo` carrying the `TstInfo`) — what
//! [`crate::signatures::Timestamp::token_bytes`] already provides.
//! Because it is *unsigned*, adding it does **not** change the signed
//! hash, so a B-T file is byte-identical to its B-B form up to the
//! `SignerInfo`'s unsigned-attrs slot (feature plan §4 invariant I7).
//!
//! This module owns only the pure `token_der → Attribute DER` step
//! (round-trip-verifiable here via the real RustCrypto decoder). The
//! TSA fetch, computing the imprint over `SignerInfo.signature`, and
//! splicing the attribute into the `SignerInfo` are the sign-path
//! wiring (TODO #7) — and the end-to-end CMS conformance is gated by
//! the EU-DSS validator (feature plan §5.5).

use crate::error::{Error, Result};
use crate::signatures::der_util::{der_oid, der_sequence, der_set};

/// OID `id-aa-signatureTimeStampToken` = 1.2.840.113549.1.9.16.2.14
/// (content octets, no tag/length).
const OID_SIGNATURE_TIME_STAMP: &[u8] = &[
    0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x09, 0x10, 0x02, 0x0E,
];

/// Total length (header + content) of the single leading
/// definite-length DER element in `der`. Rejects indefinite-length
/// encoding (BER, not DER) and any header that runs past the slice.
/// Used to assert a TSA token is exactly one self-delimiting element.
fn der_single_element_len(der: &[u8]) -> Result<usize> {
    let invalid =
        || Error::InvalidPdf("signature-time-stamp: malformed DER token header".to_string());
    if der.len() < 2 {
        return Err(invalid());
    }
    let l0 = der[1] as usize;
    if l0 < 0x80 {
        // Short form: one length octet.
        return 2usize.checked_add(l0).ok_or_else(invalid);
    }
    if l0 == 0x80 {
        // Indefinite length is BER, never valid DER.
        return Err(invalid());
    }
    let n = l0 & 0x7f;
    // 0xFF is a reserved initial length octet; cap at usize width.
    if n == 0x7f || n > core::mem::size_of::<usize>() || der.len() < 2 + n {
        return Err(invalid());
    }
    let mut len: usize = 0;
    for &b in &der[2..2 + n] {
        len = len.checked_shl(8).ok_or_else(invalid)? | b as usize;
    }
    2usize
        .checked_add(n)
        .and_then(|h| h.checked_add(len))
        .ok_or_else(invalid)
}

/// Build the `signature-time-stamp` unsigned `Attribute` (DER) wrapping
/// an RFC 3161 `TimeStampToken` (`token_der`, a full CMS `ContentInfo`
/// as returned by a TSA / [`crate::signatures::Timestamp::token_bytes`]).
///
/// # Errors
/// [`Error::InvalidPdf`] if `token_der` is not *exactly one* well-formed
/// definite-length DER element (a valid token is always a `SEQUENCE`):
/// the tag must be `0x30` and the encoded length must span the slice
/// with no trailing bytes. Rejecting a malformed or non-self-delimiting
/// token here keeps a broken timestamp out of the CMS rather than
/// producing an unparseable `SignerInfo` (fail-closed, as documented).
pub fn build_signature_timestamp_attr(token_der: &[u8]) -> Result<Vec<u8>> {
    // RFC 3161 TimeStampToken is a ContentInfo SEQUENCE (tag 0x30).
    // Validate it is a single, self-delimiting, definite-length DER
    // element whose total length is exactly `token_der.len()` — a bare
    // `[0] == 0x30` check would accept truncated or trailing-garbage
    // input and silently emit an invalid attribute.
    if token_der.first() != Some(&0x30) {
        return Err(Error::InvalidPdf(
            "signature-time-stamp: token is not a DER SEQUENCE".to_string(),
        ));
    }
    if der_single_element_len(token_der)? != token_der.len() {
        return Err(Error::InvalidPdf(
            "signature-time-stamp: token is not a single self-delimiting \
             DER element (truncated or has trailing bytes)"
                .to_string(),
        ));
    }
    let mut attr = Vec::with_capacity(16 + token_der.len());
    attr.extend_from_slice(&der_oid(OID_SIGNATURE_TIME_STAMP));
    // attrValues ::= SET OF TimeStampToken — the token is one value.
    attr.extend_from_slice(&der_set(token_der));
    Ok(der_sequence(&attr))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cms::cert::x509::attr::Attribute;
    use der::oid::ObjectIdentifier;
    use der::Decode;
    use der::Encode;

    const ID_AA_SIG_TS: ObjectIdentifier =
        ObjectIdentifier::new_unwrap("1.2.840.113549.1.9.16.2.14");

    /// A minimal well-formed DER SEQUENCE standing in for a token
    /// (round-trip exercises the wrapping; real tokens are larger but
    /// structurally just a SEQUENCE here).
    fn fake_token() -> Vec<u8> {
        // SEQUENCE { INTEGER 1, OCTET STRING "ts" }
        vec![0x30, 0x07, 0x02, 0x01, 0x01, 0x04, 0x02, b't', b's']
    }

    #[test]
    fn attribute_round_trips_with_correct_oid() {
        let token = fake_token();
        let attr_der = build_signature_timestamp_attr(&token).unwrap();
        let parsed = Attribute::from_der(&attr_der).expect("valid DER Attribute");
        assert_eq!(parsed.oid, ID_AA_SIG_TS);
        assert_eq!(parsed.values.len(), 1, "one TimeStampToken value");
        // Deterministic DER: re-encode is byte-identical.
        assert_eq!(parsed.to_der().unwrap(), attr_der);
    }

    #[test]
    fn token_bytes_are_embedded_verbatim() {
        let token = fake_token();
        let attr = build_signature_timestamp_attr(&token).unwrap();
        assert!(
            attr.windows(token.len()).any(|w| w == token.as_slice()),
            "the token DER must be embedded unchanged in the attribute"
        );
    }

    #[test]
    fn rejects_non_sequence_token_fail_closed() {
        // An INTEGER, an OCTET STRING, empty — none is a valid token.
        assert!(build_signature_timestamp_attr(&[0x02, 0x01, 0x00]).is_err());
        assert!(build_signature_timestamp_attr(&[0x04, 0x01, 0xAA]).is_err());
        assert!(build_signature_timestamp_attr(&[]).is_err());
    }

    #[test]
    fn rejects_non_self_delimiting_token_fail_closed() {
        // Starts with 0x30 but has trailing garbage after the element:
        // a bare first-byte check would wrongly accept this.
        let mut trailing = fake_token();
        trailing.extend_from_slice(&[0xDE, 0xAD]);
        assert!(
            build_signature_timestamp_attr(&trailing).is_err(),
            "token with trailing bytes must be rejected"
        );
        // Truncated: header claims 7 content bytes, only 3 present.
        assert!(
            build_signature_timestamp_attr(&[0x30, 0x07, 0x02, 0x01, 0x01]).is_err(),
            "truncated token must be rejected"
        );
        // Indefinite-length (BER, not DER) must be rejected.
        assert!(
            build_signature_timestamp_attr(&[0x30, 0x80, 0x00, 0x00]).is_err(),
            "indefinite-length encoding must be rejected"
        );
        // Exactly one element with no trailing bytes still works.
        assert!(build_signature_timestamp_attr(&fake_token()).is_ok());
    }
}
