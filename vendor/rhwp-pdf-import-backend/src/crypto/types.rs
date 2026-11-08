//! Algorithm enums and value types shared by every [`CryptoProvider`]
//! implementation.
//!
//! These types are intentionally small and `Copy` where possible so the
//! trait surface stays cheap to call. Anything that needs heap data
//! (RSA modulus, X.509 cert bytes) is passed by reference.

/// Hash algorithms PDF and CMS care about.
///
/// PDF Standard Security R≤4 hard-requires MD5 (ISO 32000-1 §7.6.3
/// Algorithms 2/3/4/5). PKCS#7 / CMS signatures use SHA-1, SHA-256,
/// SHA-384, SHA-512 (ISO 32000-1 §12.8.3 Table 252). RIPEMD-160 is
/// listed in the spec but pdf_oxide does not currently support it; if
/// a downstream provider implements it, add a variant here behind a
/// minor-version bump.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HashAlgorithm {
    /// MD5 — legacy. Required for PDF R≤4 password derivation.
    /// FIPS 140-3 forbids MD5 for any use; FIPS providers reject this.
    Md5,
    /// SHA-1 — legacy. Required for `adbe.pkcs7.sha1` and historical
    /// signatures. NIST SP 800-131A allows SHA-1 for verification of
    /// historical signatures but disallows it for new generation.
    Sha1,
    /// SHA-256 — FIPS 140-3 approved.
    Sha256,
    /// SHA-384 — FIPS 140-3 approved.
    Sha384,
    /// SHA-512 — FIPS 140-3 approved.
    Sha512,
}

impl HashAlgorithm {
    /// Output size in bytes (matches `hash`-style crates' `OutputSize`).
    pub const fn output_size(self) -> usize {
        match self {
            HashAlgorithm::Md5 => 16,
            HashAlgorithm::Sha1 => 20,
            HashAlgorithm::Sha256 => 32,
            HashAlgorithm::Sha384 => 48,
            HashAlgorithm::Sha512 => 64,
        }
    }

    /// Human-readable name used in error messages and audit logs.
    pub const fn name(self) -> &'static str {
        match self {
            HashAlgorithm::Md5 => "MD5",
            HashAlgorithm::Sha1 => "SHA-1",
            HashAlgorithm::Sha256 => "SHA-256",
            HashAlgorithm::Sha384 => "SHA-384",
            HashAlgorithm::Sha512 => "SHA-512",
        }
    }

    /// Whether this hash is FIPS 140-3 approved for new use.
    /// SHA-1 is allowed for verify-only by some FIPS deployments
    /// (NIST SP 800-131A) but not for signing — that policy decision
    /// is made by the provider, not the algorithm enum.
    pub const fn is_fips_approved(self) -> bool {
        matches!(self, HashAlgorithm::Sha256 | HashAlgorithm::Sha384 | HashAlgorithm::Sha512)
    }
}

/// Padding mode for AES-CBC.
///
/// PDF stream/string encryption (V≥4) uses PKCS#7 padding. Algorithm
/// 2.B inner encryption and the V=5 R=5/6 key-wrap (UE/OE entries)
/// require **no padding** — the caller pre-pads to a 16-byte multiple
/// or supplies exactly two blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Padding {
    /// PKCS#7 padding (PDF V≥4 stream/string encryption).
    Pkcs7,
    /// No padding — input must be a 16-byte multiple. PDF Algorithm
    /// 2.B and V=5 R=5/6 UE/OE key wrap.
    None,
}

/// AES key size in bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AesKeySize {
    /// 128-bit key (16 bytes). PDF V=4, R=4 (`/CFM AESV2`).
    Aes128,
    /// 256-bit key (32 bytes). PDF V=5, R=5/6 (`/CFM AESV3`).
    Aes256,
}

impl AesKeySize {
    /// Key length in bytes (16 for AES-128, 32 for AES-256).
    pub const fn key_bytes(self) -> usize {
        match self {
            AesKeySize::Aes128 => 16,
            AesKeySize::Aes256 => 32,
        }
    }
    /// Human-readable algorithm name (`"AES-128"` / `"AES-256"`).
    pub const fn name(self) -> &'static str {
        match self {
            AesKeySize::Aes128 => "AES-128",
            AesKeySize::Aes256 => "AES-256",
        }
    }
}

/// Elliptic curve identifier.
///
/// PDF / CMS signing in the wild uses NIST P-256 (secp256r1) and
/// P-384 (secp384r1) overwhelmingly; P-521 and secp256k1 exist but
/// pdf_oxide doesn't currently consume them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EcCurve {
    /// NIST P-256 / secp256r1 / prime256v1.
    P256,
    /// NIST P-384 / secp384r1.
    P384,
}

impl EcCurve {
    /// Human-readable curve name (`"P-256"` / `"P-384"`).
    pub const fn name(self) -> &'static str {
        match self {
            EcCurve::P256 => "P-256",
            EcCurve::P384 => "P-384",
        }
    }
}

/// RSA signing scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RsaScheme {
    /// PKCS#1 v1.5 — most common for `adbe.pkcs7.detached` PDF
    /// signatures. FIPS approved when paired with SHA-256+.
    Pkcs1v15,
    /// PKCS#1 v2.1 PSS with `salt_len = digest_size`. FIPS 186-5
    /// §A.5.1 / RFC 8017 §9.1 default; matches both `rsa::pss` and
    /// `aws-lc-rs::signature::RSA_PSS_*` defaults.
    Pss,
}

impl RsaScheme {
    /// Human-readable scheme name.
    pub const fn name(self) -> &'static str {
        match self {
            RsaScheme::Pkcs1v15 => "RSA-PKCS#1-v1.5",
            RsaScheme::Pss => "RSA-PSS",
        }
    }
}

/// Wraps a public RSA key as the `(modulus, exponent)` big-endian
/// byte pair. Both providers (RustCrypto, aws-lc-rs) reconstruct the
/// internal key type from these bytes — keeping the wire format
/// abstract avoids leaking either crate's `RsaPublicKey` into our API.
#[derive(Debug, Clone)]
pub struct RsaPublicKey<'a> {
    /// Modulus `n` as a big-endian unsigned integer (no leading sign
    /// byte). Length defines the key strength (256 bytes = RSA-2048).
    pub modulus_be: &'a [u8],
    /// Public exponent `e` as a big-endian unsigned integer
    /// (typically `65537`, encoded as `[0x01, 0x00, 0x01]`).
    pub exponent_be: &'a [u8],
}

/// Identifies which signing algorithm + hash a [`Signer`] produces.
///
/// [`Signer`]: super::Signer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SigningAlgorithm {
    /// Asymmetric algorithm + parameters (RSA scheme or ECDSA curve).
    pub asym: AsymmetricAlgorithm,
    /// Message digest algorithm.
    pub hash: HashAlgorithm,
}

/// Asymmetric signing primitive choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsymmetricAlgorithm {
    /// RSA with the given padding scheme.
    Rsa(RsaScheme),
    /// ECDSA on the given curve.
    Ecdsa(EcCurve),
}

impl SigningAlgorithm {
    /// Human-readable algorithm name suitable for audit logs and
    /// error messages. Returns the canonical RFC name for the common
    /// `(asym, hash)` pairs PDF / CMS use; less common pairings fall
    /// back to a generic `"signing-algorithm"` and the caller is
    /// expected to render the asym + hash separately.
    pub const fn name(self) -> &'static str {
        // Static names for the common combinations; less common ones
        // can be reported as "<asym> with <hash>" in error formatting
        // by the caller.
        match (self.asym, self.hash) {
            (AsymmetricAlgorithm::Rsa(RsaScheme::Pkcs1v15), HashAlgorithm::Sha256) => {
                "RSA-PKCS1v15-SHA256"
            },
            (AsymmetricAlgorithm::Rsa(RsaScheme::Pkcs1v15), HashAlgorithm::Sha384) => {
                "RSA-PKCS1v15-SHA384"
            },
            (AsymmetricAlgorithm::Rsa(RsaScheme::Pkcs1v15), HashAlgorithm::Sha512) => {
                "RSA-PKCS1v15-SHA512"
            },
            (AsymmetricAlgorithm::Rsa(RsaScheme::Pss), HashAlgorithm::Sha256) => "RSA-PSS-SHA256",
            (AsymmetricAlgorithm::Rsa(RsaScheme::Pss), HashAlgorithm::Sha384) => "RSA-PSS-SHA384",
            (AsymmetricAlgorithm::Rsa(RsaScheme::Pss), HashAlgorithm::Sha512) => "RSA-PSS-SHA512",
            (AsymmetricAlgorithm::Ecdsa(EcCurve::P256), HashAlgorithm::Sha256) => {
                "ECDSA-P256-SHA256"
            },
            (AsymmetricAlgorithm::Ecdsa(EcCurve::P384), HashAlgorithm::Sha384) => {
                "ECDSA-P384-SHA384"
            },
            _ => "signing-algorithm",
        }
    }
}
