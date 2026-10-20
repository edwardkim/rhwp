//! Errors emitted by [`CryptoProvider`] implementations.
//!
//! These are deliberately separate from `crate::error::PdfError` so a
//! `CryptoProvider` can be implemented and tested without dragging in
//! the parser/document error surface — and so downstream callers can
//! match on the FIPS / sovereign-compliance failure modes without
//! string-matching error messages.

use std::fmt;

/// Result alias used throughout the crypto module.
pub type Result<T> = std::result::Result<T, Error>;

/// Identifies the broad family of an algorithm so error messages and
/// FIPS audit logs can group rejections without parsing strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlgorithmKind {
    /// Cryptographic hash function (e.g. MD5, SHA-256).
    Hash,
    /// Symmetric block / stream cipher (e.g. AES-CBC, RC4).
    SymmetricCipher,
    /// Digital signature *creation*. Distinct from `SignatureVerify`
    /// because FIPS providers usually permit verification of legacy
    /// signatures while forbidding generation under the same algos.
    SignatureSign,
    /// Digital signature *verification*.
    SignatureVerify,
    /// Key-derivation function or password-based key construction.
    KeyDerivation,
    /// Cryptographically strong random byte source.
    RandomBytes,
}

impl fmt::Display for AlgorithmKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            AlgorithmKind::Hash => "hash",
            AlgorithmKind::SymmetricCipher => "symmetric cipher",
            AlgorithmKind::SignatureSign => "signature (sign)",
            AlgorithmKind::SignatureVerify => "signature (verify)",
            AlgorithmKind::KeyDerivation => "key derivation",
            AlgorithmKind::RandomBytes => "random bytes",
        };
        f.write_str(s)
    }
}

/// Errors emitted by a [`super::provider::CryptoProvider`] or any of its sub-traits.
#[derive(Debug)]
pub enum Error {
    /// The provider does not permit this algorithm under its current
    /// policy (e.g. MD5, SHA-1 sign, RC4 under a FIPS provider).
    /// `name` is the human-readable algorithm name; `kind` groups the
    /// rejection for audit logs.
    ///
    /// PDF Standard Security R≤4 documents fundamentally require MD5
    /// and RC4 (ISO 32000-1 §7.6.3), so opening one under a FIPS
    /// provider returns this error with a suggested workaround.
    AlgorithmNotPermitted {
        /// Operation family that triggered the rejection.
        kind: AlgorithmKind,
        /// Human-readable algorithm name (e.g. `"MD5"`, `"RC4"`).
        name: &'static str,
        /// One-line policy citation, e.g. `"FIPS 140-3 forbids MD5"`.
        reason: &'static str,
    },

    /// Input shape was wrong — wrong key length for AES, wrong IV
    /// size, malformed signature bytes. Distinct from
    /// [`Error::Verification`] which is specifically "math says no".
    InvalidInput(&'static str),

    /// Signature verification ran to completion and concluded the
    /// signature does not match. Callers usually want to surface this
    /// distinctly from `InvalidInput` (auditable, not a programmer
    /// bug).
    Verification(&'static str),

    /// The provider hit an internal error neither the input nor the
    /// policy can be blamed for — e.g. the FIPS module failed
    /// self-test, the OS RNG returned `EAGAIN`, the HSM session was
    /// dropped. Carries a static reason to avoid heap allocation in
    /// the hot path.
    Backend(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::AlgorithmNotPermitted { kind, name, reason } => {
                write!(
                    f,
                    "{kind} algorithm '{name}' not permitted by active CryptoProvider: {reason}"
                )
            },
            Error::InvalidInput(s) => write!(f, "crypto: invalid input — {s}"),
            Error::Verification(s) => write!(f, "crypto: verification failed — {s}"),
            Error::Backend(s) => write!(f, "crypto: backend error — {s}"),
        }
    }
}

impl std::error::Error for Error {}

/// Convenience helper for providers to construct a uniform
/// `AlgorithmNotPermitted` error.
#[inline]
pub fn not_permitted(kind: AlgorithmKind, name: &'static str, reason: &'static str) -> Error {
    Error::AlgorithmNotPermitted { kind, name, reason }
}
