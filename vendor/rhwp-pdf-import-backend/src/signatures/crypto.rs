//! Shared crypto helpers for the signatures module.
//!
//! These exist so that `cms_verify`, `timestamp`, and `tsa_client` don't
//! each carry their own copy of the digest-OID ⇄ hash mapping. All three
//! subsystems need to pick a SHA variant based on an ObjectIdentifier
//! carried inside a signed blob, and all three need to reach for the
//! same PKCS#1 v1.5 `DigestInfo` prefix tables when they're building or
//! checking an RSA signature.
//!
//! Nothing here is PDF-specific — it's the narrowest possible
//! intersection of what the three consumers needed. Keeping it internal
//! (`pub(super)`) lets us keep churning the API shape without a
//! backcompat concern.

use der::oid::db::rfc5912::{ID_SHA_1, ID_SHA_256, ID_SHA_384, ID_SHA_512};
use der::oid::ObjectIdentifier;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha384, Sha512};

use super::timestamp::HashAlgorithm;

// ─── Hashing dispatch ────────────────────────────────────────────────

/// Hash `msg` with the algorithm identified by `oid` (one of the
/// `ID_SHA_*` constants). Returns `None` for an unrecognised OID — the
/// caller is expected to treat that as "I don't know how to verify
/// this" rather than a hard failure.
pub(super) fn hash_with_oid(oid: ObjectIdentifier, msg: &[u8]) -> Option<Vec<u8>> {
    if oid == ID_SHA_1 {
        Some(Sha1::digest(msg).to_vec())
    } else if oid == ID_SHA_256 {
        Some(Sha256::digest(msg).to_vec())
    } else if oid == ID_SHA_384 {
        Some(Sha384::digest(msg).to_vec())
    } else if oid == ID_SHA_512 {
        Some(Sha512::digest(msg).to_vec())
    } else {
        None
    }
}

/// Hash `data` using a [`HashAlgorithm`] variant. `Unknown` falls back
/// to SHA-256 so the TSA client still produces a deterministic imprint
/// rather than panicking — mirrors the pre-refactor behaviour.
///
/// Used by the TSA client path; gated on the same feature so that
/// builds without `tsa-client` don't emit a dead-code warning.
#[cfg(feature = "tsa-client")]
pub(super) fn hash_with_algorithm(algo: HashAlgorithm, data: &[u8]) -> Vec<u8> {
    match algo {
        HashAlgorithm::Sha1 => Sha1::digest(data).to_vec(),
        HashAlgorithm::Sha256 | HashAlgorithm::Unknown => Sha256::digest(data).to_vec(),
        HashAlgorithm::Sha384 => Sha384::digest(data).to_vec(),
        HashAlgorithm::Sha512 => Sha512::digest(data).to_vec(),
    }
}

/// Map a DER-level digest OID to our [`HashAlgorithm`] enum.
pub(super) fn hash_algorithm_from_oid(oid: ObjectIdentifier) -> HashAlgorithm {
    if oid == ID_SHA_256 {
        HashAlgorithm::Sha256
    } else if oid == ID_SHA_384 {
        HashAlgorithm::Sha384
    } else if oid == ID_SHA_512 {
        HashAlgorithm::Sha512
    } else if oid == ID_SHA_1 {
        HashAlgorithm::Sha1
    } else {
        HashAlgorithm::Unknown
    }
}

/// Map a [`HashAlgorithm`] back to its DER-level OID. Returns `None`
/// for `Unknown` — the caller is expected to reject rather than pick
/// a default.
///
/// Used by the TSA client path; gated on the same feature so that
/// builds without `tsa-client` don't emit a dead-code warning.
#[cfg(feature = "tsa-client")]
pub(super) fn oid_for_algorithm(algo: HashAlgorithm) -> Option<ObjectIdentifier> {
    match algo {
        HashAlgorithm::Sha1 => Some(ID_SHA_1),
        HashAlgorithm::Sha256 => Some(ID_SHA_256),
        HashAlgorithm::Sha384 => Some(ID_SHA_384),
        HashAlgorithm::Sha512 => Some(ID_SHA_512),
        HashAlgorithm::Unknown => None,
    }
}

// ─── PKCS#1 v1.5 DigestInfo prefixes (RFC 8017 §9.2) ────────────────
//
// These are the DER encodings of `DigestInfo { digestAlgorithm,
// OCTET STRING }` with an empty OCTET STRING; the hash bytes are
// appended at the end. Using a fixed byte prefix + raw hash lets us
// call `rsa::Pkcs1v15Sign::new_unprefixed()` and sidestep the
// `Digest + AssociatedOid` trait-bound mismatch between rsa 0.9
// (digest 0.10) and our sha2 0.11.

const DIGEST_INFO_SHA1: &[u8] = &[
    0x30, 0x21, 0x30, 0x09, 0x06, 0x05, 0x2b, 0x0e, 0x03, 0x02, 0x1a, 0x05, 0x00, 0x04, 0x14,
];
const DIGEST_INFO_SHA256: &[u8] = &[
    0x30, 0x31, 0x30, 0x0d, 0x06, 0x09, 0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x01, 0x05,
    0x00, 0x04, 0x20,
];
const DIGEST_INFO_SHA384: &[u8] = &[
    0x30, 0x41, 0x30, 0x0d, 0x06, 0x09, 0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x02, 0x05,
    0x00, 0x04, 0x30,
];
const DIGEST_INFO_SHA512: &[u8] = &[
    0x30, 0x51, 0x30, 0x0d, 0x06, 0x09, 0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x03, 0x05,
    0x00, 0x04, 0x40,
];

/// `pub(crate)` so the default `crypto::RustCryptoProvider`
/// implementation in `src/crypto/rust_provider.rs` can re-use the
/// same OID-to-DigestInfo-prefix table as `cms_verify.rs` and
/// `signer.rs`. Single source of truth for PKCS#1 v1.5 wrappers.
pub(crate) fn digest_info_prefix(oid: ObjectIdentifier) -> Option<&'static [u8]> {
    if oid == ID_SHA_1 {
        Some(DIGEST_INFO_SHA1)
    } else if oid == ID_SHA_256 {
        Some(DIGEST_INFO_SHA256)
    } else if oid == ID_SHA_384 {
        Some(DIGEST_INFO_SHA384)
    } else if oid == ID_SHA_512 {
        Some(DIGEST_INFO_SHA512)
    } else {
        None
    }
}

// ─── RSA-PKCS#1 v1.5 signature algorithm OIDs ───────────────────────
//
// These appear on `SignerInfo.signature_algorithm`. The digest half is
// redundantly named by `signer.digest_alg`, so we use this set only to
// recognise "this is RSA + PKCS#1 v1.5 padding" vs. PSS / ECDSA / etc.

/// `rsaEncryption` — shows up as both a SubjectPublicKeyInfo
/// algorithm and (occasionally) as a signer's signatureAlgorithm when
/// the signer wants to defer the digest to `digest_alg`.
pub(super) const OID_RSA_ENCRYPTION: ObjectIdentifier =
    ObjectIdentifier::new_unwrap("1.2.840.113549.1.1.1");

const OID_SHA1_RSA: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113549.1.1.5");
const OID_SHA256_RSA: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113549.1.1.11");
const OID_SHA384_RSA: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113549.1.1.12");
const OID_SHA512_RSA: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113549.1.1.13");

/// Does `oid` name an RSA-PKCS#1 v1.5 signature algorithm?
pub(super) fn is_rsa_pkcs1v15_sig_oid(oid: ObjectIdentifier) -> bool {
    oid == OID_SHA1_RSA
        || oid == OID_SHA256_RSA
        || oid == OID_SHA384_RSA
        || oid == OID_SHA512_RSA
        || oid == OID_RSA_ENCRYPTION
}

// ─── RSA-PSS signature algorithm OID ────────────────────────────────

/// `id-RSASSA-PSS` (RFC 4055 §3.1). Appears in SignerInfo.signature_algorithm
/// when the signer uses RSA with PSS padding.
pub(super) const OID_RSASSA_PSS: ObjectIdentifier =
    ObjectIdentifier::new_unwrap("1.2.840.113549.1.1.10");

// ─── ECDSA signature algorithm OIDs ─────────────────────────────────
//
// The digest algorithm is implied by the signature OID for ECDSA.

/// ecdsa-with-SHA256 (RFC 5480 / ANSI X9.62)
pub(super) const OID_ECDSA_SHA256: ObjectIdentifier =
    ObjectIdentifier::new_unwrap("1.2.840.10045.4.3.2");

/// ecdsa-with-SHA384 (RFC 5480 / ANSI X9.62)
pub(super) const OID_ECDSA_SHA384: ObjectIdentifier =
    ObjectIdentifier::new_unwrap("1.2.840.10045.4.3.3");

/// ecdsa-with-SHA512 (RFC 5480 / ANSI X9.62)
pub(super) const OID_ECDSA_SHA512: ObjectIdentifier =
    ObjectIdentifier::new_unwrap("1.2.840.10045.4.3.4");

/// `id-ecPublicKey` — SubjectPublicKeyInfo algorithm OID for EC keys.
pub(super) const OID_EC_PUBLIC_KEY: ObjectIdentifier =
    ObjectIdentifier::new_unwrap("1.2.840.10045.2.1");

/// P-256 named curve OID (secp256r1 / prime256v1).
pub(super) const OID_P256: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.10045.3.1.7");

/// P-384 named curve OID (secp384r1).
pub(super) const OID_P384: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.3.132.0.34");
