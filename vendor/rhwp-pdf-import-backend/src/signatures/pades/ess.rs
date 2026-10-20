//! RFC 5035 ESS `signing-certificate-v2` signed attribute (#235,
//! feature plan §4.4, TODO #3).
//!
//! CAdES-B-B / PAdES-B-B *mandate* this signed attribute (the one true
//! gap in the crate's prior B-B claim — feature plan §2.2). It binds the
//! signature to the exact signer certificate via a hash + issuer/serial.
//!
//! ```text
//! Attribute ::= SEQUENCE {
//!   attrType   OBJECT IDENTIFIER  -- id-aa-signingCertificateV2
//!                                 --   1.2.840.113549.1.9.16.2.47
//!   attrValues SET OF SigningCertificateV2 }
//! SigningCertificateV2 ::= SEQUENCE { certs SEQUENCE OF ESSCertIDv2 }
//! ESSCertIDv2 ::= SEQUENCE {
//!   hashAlgorithm  AlgorithmIdentifier DEFAULT {id-sha256},
//!   certHash       OCTET STRING,
//!   issuerSerial   IssuerSerial OPTIONAL }
//! IssuerSerial ::= SEQUENCE {
//!   issuer        GeneralNames,            -- directoryName [4] EXPLICIT
//!   serialNumber  CertificateSerialNumber }
//! ```
//!
//! Strict DER: a `hashAlgorithm` equal to its DEFAULT (`id-sha256`)
//! **must be omitted** (RFC 5035 + X.690 §11.5); for SHA-384/512 the
//! `AlgorithmIdentifier` is encoded with *absent* parameters (RFC 5754).
//! The cert hash is taken via the active [`crypto`] provider so #230 /
//! FIPS policy is honoured (no direct `sha2::`). This module only
//! *builds* the attribute (a pure `cert_der + digest → DER` step,
//! round-trip-verifiable here); splicing it into the signed attributes
//! and the EU-DSS conformance check are the wiring step (TODO #4).

use crate::crypto::{self, HashAlgorithm};
use crate::error::{Error, Result};
use crate::signatures::der_util::{der_octet_string, der_oid, der_sequence, der_set, der_tag};
use crate::signatures::types::DigestAlgorithm;
use cms::cert::x509::Certificate as X509Certificate;
use der::{Decode, Encode};

/// OID `id-aa-signingCertificateV2` = 1.2.840.113549.1.9.16.2.47
/// (content octets, no tag/length).
const OID_SIGNING_CERT_V2: &[u8] = &[
    0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x09, 0x10, 0x02, 0x2F,
];
/// OID content for the SHA hash AlgorithmIdentifier (non-default cases).
const OID_SHA384: &[u8] = &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x02];
const OID_SHA512: &[u8] = &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x03];
const OID_SHA1: &[u8] = &[0x2B, 0x0E, 0x03, 0x02, 0x1A];

fn hash_algorithm(d: DigestAlgorithm) -> HashAlgorithm {
    match d {
        DigestAlgorithm::Sha1 => HashAlgorithm::Sha1,
        DigestAlgorithm::Sha256 => HashAlgorithm::Sha256,
        DigestAlgorithm::Sha384 => HashAlgorithm::Sha384,
        DigestAlgorithm::Sha512 => HashAlgorithm::Sha512,
    }
}

/// The `hashAlgorithm` `AlgorithmIdentifier` DER, or `None` when it
/// equals the RFC 5035 DEFAULT (`id-sha256`) and so must be omitted in
/// strict DER. SHA-2 identifiers carry *absent* parameters (RFC 5754).
fn hash_alg_id(d: DigestAlgorithm) -> Option<Vec<u8>> {
    let oid = match d {
        DigestAlgorithm::Sha256 => return None, // DEFAULT → omit
        DigestAlgorithm::Sha384 => OID_SHA384,
        DigestAlgorithm::Sha512 => OID_SHA512,
        DigestAlgorithm::Sha1 => OID_SHA1,
    };
    Some(der_sequence(&der_oid(oid)))
}

/// Build the complete RFC 5035 `signing-certificate-v2` `Attribute`
/// (DER) for `cert_der` (the signer certificate), hashing it with the
/// same digest the signature uses.
///
/// # Errors
/// - [`Error::InvalidPdf`] if the certificate cannot be parsed or its
///   issuer/serial cannot be re-encoded.
/// - the crypto provider's error if it refuses the digest (fail-loud —
///   never substitute a different hash; the ESS hash must match the
///   signature digest, feature plan §8).
pub fn build_signing_certificate_v2(
    cert_der: &[u8],
    digest_alg: DigestAlgorithm,
) -> Result<Vec<u8>> {
    // 1. certHash over the signer cert DER, via the active provider.
    //    The crypto module has its own error type; map to ours (and
    //    fail-loud — never substitute a different digest).
    let mut hasher = crypto::active()
        .hasher(hash_algorithm(digest_alg))
        .map_err(|e| {
            Error::Unsupported(format!("ESS: digest {digest_alg:?} unavailable: {e:?}"))
        })?;
    hasher.update(cert_der);
    let cert_hash = hasher.finalize();

    // 2. issuer + serial from the certificate (reuse the exact path
    //    signer.rs uses for the CMS SignerInfo issuerAndSerialNumber).
    let cert = X509Certificate::from_der(cert_der)
        .map_err(|e| Error::InvalidPdf(format!("ESS: cannot parse signer certificate: {e}")))?;
    let issuer_der = cert
        .tbs_certificate()
        .issuer()
        .to_der()
        .map_err(|e| Error::InvalidPdf(format!("ESS: cannot DER-encode issuer: {e}")))?;
    let serial_der = cert
        .tbs_certificate()
        .serial_number()
        .to_der()
        .map_err(|e| Error::InvalidPdf(format!("ESS: cannot DER-encode serial: {e}")))?;

    // 3. IssuerSerial ::= SEQUENCE { issuer GeneralNames, serial INTEGER }
    //    GeneralNames ::= SEQUENCE OF GeneralName; the single entry is a
    //    directoryName, GeneralName CHOICE tag [4]. A CHOICE cannot be
    //    IMPLICIT-tagged, so [4] is EXPLICIT (constructed, 0xA4) and
    //    wraps the issuer Name DER unchanged.
    let general_name = der_tag(0xA4, &issuer_der);
    let general_names = der_sequence(&general_name);
    let mut issuer_serial = Vec::new();
    issuer_serial.extend_from_slice(&general_names);
    issuer_serial.extend_from_slice(&serial_der);
    let issuer_serial = der_sequence(&issuer_serial);

    // 4. ESSCertIDv2 ::= SEQUENCE { [hashAlgorithm] certHash issuerSerial }
    let mut esscertid = Vec::new();
    if let Some(alg) = hash_alg_id(digest_alg) {
        esscertid.extend_from_slice(&alg);
    }
    esscertid.extend_from_slice(&der_octet_string(&cert_hash));
    esscertid.extend_from_slice(&issuer_serial);
    let esscertid = der_sequence(&esscertid);

    // 5. SigningCertificateV2 ::= SEQUENCE { certs SEQUENCE OF ESSCertIDv2 }
    let certs = der_sequence(&esscertid);
    let signing_cert_v2 = der_sequence(&certs);

    // 6. Attribute ::= SEQUENCE { attrType OID, attrValues SET OF … }
    let mut attr = Vec::new();
    attr.extend_from_slice(&der_oid(OID_SIGNING_CERT_V2));
    attr.extend_from_slice(&der_set(&signing_cert_v2));
    Ok(der_sequence(&attr))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cms::cert::x509::attr::Attribute;
    use der::oid::ObjectIdentifier;

    const ID_AA_SIGNING_CERT_V2: ObjectIdentifier =
        ObjectIdentifier::new_unwrap("1.2.840.113549.1.9.16.2.47");

    /// Load the checked-in deterministic test signer certificate (DER).
    fn test_cert_der() -> Vec<u8> {
        let pem = include_str!("../../../tests/fixtures/test_signing_cert.pem");
        let der = pem
            .lines()
            .skip_while(|l| !l.contains("BEGIN CERTIFICATE"))
            .skip(1)
            .take_while(|l| !l.contains("END CERTIFICATE"))
            .collect::<String>();
        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode(der.trim())
            .expect("decode test cert b64")
    }

    #[test]
    fn attribute_round_trips_through_cms_decoder() {
        let cert = test_cert_der();
        let attr_der = build_signing_certificate_v2(&cert, DigestAlgorithm::Sha256).unwrap();
        // The real RustCrypto `cms` Attribute decoder must accept it and
        // see the correct attrType (structural conformance, short of the
        // EU-DSS validator which gates end-to-end CMS conformance).
        let parsed = Attribute::from_der(&attr_der).expect("valid DER Attribute");
        assert_eq!(parsed.oid, ID_AA_SIGNING_CERT_V2);
        assert_eq!(parsed.values.len(), 1, "exactly one SigningCertificateV2");
        // Re-encode is byte-identical (deterministic DER).
        assert_eq!(parsed.to_der().unwrap(), attr_der);
    }

    #[test]
    fn cert_hash_is_the_signature_digest_of_the_cert() {
        let cert = test_cert_der();
        let attr = build_signing_certificate_v2(&cert, DigestAlgorithm::Sha256).unwrap();
        // SHA-256(cert) must appear verbatim inside the attribute.
        let mut h = crypto::active().hasher(HashAlgorithm::Sha256).unwrap();
        h.update(&cert);
        let expect = h.finalize();
        assert!(
            attr.windows(expect.len()).any(|w| w == expect.as_slice()),
            "certHash (SHA-256 of cert) not embedded in the attribute"
        );
    }

    #[test]
    fn sha256_omits_hash_algorithm_but_sha512_includes_it() {
        let cert = test_cert_der();
        let a256 = build_signing_certificate_v2(&cert, DigestAlgorithm::Sha256).unwrap();
        let a512 = build_signing_certificate_v2(&cert, DigestAlgorithm::Sha512).unwrap();
        // Both must be valid DER Attributes.
        assert!(Attribute::from_der(&a256).is_ok());
        assert!(Attribute::from_der(&a512).is_ok());
        // The SHA-512 OID appears only when not the DEFAULT.
        assert!(!a256.windows(OID_SHA512.len()).any(|w| w == OID_SHA512));
        assert!(a512.windows(OID_SHA512.len()).any(|w| w == OID_SHA512));
        // SHA-512 form is longer (carries the AlgorithmIdentifier).
        assert!(a512.len() > a256.len());
    }

    #[test]
    fn rejects_garbage_certificate() {
        let err = build_signing_certificate_v2(b"not a certificate", DigestAlgorithm::Sha256)
            .unwrap_err();
        assert!(matches!(err, Error::InvalidPdf(_)), "got {err:?}");
    }
}
