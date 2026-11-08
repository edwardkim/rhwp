//! The [`CryptoProvider`] trait family.
//!
//! These traits decouple PDF encryption and signature paths from any
//! one cryptography crate, so deployments that need a FIPS 140-3
//! validated module (`aws-lc-rs` with the `fips` feature) or a
//! sovereign-jurisdiction provider (GOST R 34.11/34.10, Chinese
//! SM2/SM3/SM4) can swap in a different backend without touching the
//! parsing or signature-construction code.
//!
//! See `docs/CRYPTO_PROVIDERS.md` (added in Phase 8) for the
//! end-to-end story; tracking issue #236.
//!
//! # Trait shape
//!
//! Three sub-traits handle independent concerns:
//!
//! - [`Hasher`] — incremental hashing (`update` / `finalize`).
//! - [`SymmetricCipher`] — AES-CBC (PKCS#7 and no-padding) and RC4.
//! - [`SignatureVerifier`] — RSA-PKCS#1-v1.5 / RSA-PSS / ECDSA verify.
//! - [`Signer`] — opaque signing handle (decouples PEM/DER loading
//!   from the call site so HSM / PKCS#11 providers can plug in).
//!
//! [`CryptoProvider`] composes them and adds policy
//! ([`is_legacy_allowed`]) plus secure RNG.
//!
//! # FIPS posture
//!
//! Every provider documents what it permits via
//! [`CryptoProvider::is_legacy_allowed`]. When `false`, MD5, SHA-1
//! signing, RC4, and RSA-PKCS#1-v1.5 with SHA-1 return
//! [`Error::AlgorithmNotPermitted`]. SHA-1 *verification* of
//! historical signatures is permitted (NIST SP 800-131A) — the policy
//! split happens in [`SignatureVerifier`] vs [`Signer`].

use super::error::Result;
use super::types::{
    AesKeySize, EcCurve, HashAlgorithm, Padding, RsaPublicKey, RsaScheme, SigningAlgorithm,
};

/// Incremental hashing.
///
/// Modeled after the `digest` crate's `DynDigest` so providers can
/// trivially adapt — but stripped to just the operations PDF needs
/// (no XOF, no variable-output, no reset).
pub trait Hasher: Send {
    /// Feed input into the hash state. May be called any number of
    /// times before [`Self::finalize`].
    fn update(&mut self, data: &[u8]);

    /// Finalize the hash, consuming `self`. The returned `Vec` is
    /// exactly [`HashAlgorithm::output_size`] bytes long.
    ///
    /// Boxed receiver lets implementors live behind `Box<dyn Hasher>`
    /// without paying for `Sized` constraints up the call stack.
    fn finalize(self: Box<Self>) -> Vec<u8>;

    /// Reports the algorithm so callers can sanity-check the output
    /// size or feed the right OID into a CMS construction.
    fn algorithm(&self) -> HashAlgorithm;
}

/// Symmetric encryption operations PDF needs.
///
/// All methods return owned `Vec<u8>` to match the existing
/// `src/encryption/aes.rs` / `src/encryption/rc4.rs` shape so Phase 3
/// migration is mechanical. Performance-critical callers can be
/// converted to streaming later (in-place CBC, etc.) without breaking
/// the trait — adding methods is non-breaking.
pub trait SymmetricCipher: Send + Sync {
    /// AES-CBC encrypt.
    ///
    /// `key.len()` must equal `key_size.key_bytes()`; `iv.len()` must
    /// be 16. With [`Padding::None`], `data.len()` must be a multiple
    /// of 16.
    fn aes_cbc_encrypt(
        &self,
        key_size: AesKeySize,
        key: &[u8],
        iv: &[u8],
        data: &[u8],
        padding: Padding,
    ) -> Result<Vec<u8>>;

    /// AES-CBC decrypt. Same argument constraints as
    /// [`Self::aes_cbc_encrypt`].
    fn aes_cbc_decrypt(
        &self,
        key_size: AesKeySize,
        key: &[u8],
        iv: &[u8],
        data: &[u8],
        padding: Padding,
    ) -> Result<Vec<u8>>;

    /// RC4 encrypt/decrypt (the operation is symmetric so one method
    /// covers both directions).
    ///
    /// Required for PDF Standard Security R≤4 (ISO 32000-1 §7.6.3).
    /// Returns [`super::error::Error::AlgorithmNotPermitted`] under FIPS providers.
    fn rc4(&self, key: &[u8], data: &[u8]) -> Result<Vec<u8>>;
}

/// Verify a digital signature.
///
/// SHA-1 is permitted here per NIST SP 800-131A (verification of
/// historical signatures). Use [`Signer`] for generation — that path
/// rejects SHA-1 under FIPS.
pub trait SignatureVerifier: Send + Sync {
    /// Verify an RSA-PKCS#1-v1.5 signature over the raw *message* bytes.
    ///
    /// The implementation hashes `message` with `hash` internally — the
    /// same convention as `verify_rsa_pss` and `verify_ecdsa`. Passing
    /// the message (rather than a pre-computed digest) allows providers
    /// that only expose a message-level API (e.g., aws-lc-rs 1.x) to
    /// implement this without a lower-level primitive.
    fn verify_rsa_pkcs1v15(
        &self,
        pubkey: &RsaPublicKey<'_>,
        hash: HashAlgorithm,
        message: &[u8],
        signature: &[u8],
    ) -> Result<()>;

    /// Verify an RSA-PSS signature over the *message* bytes (PSS
    /// internally applies the hash; salt length defaults to digest
    /// size per RFC 8017 §9.1).
    fn verify_rsa_pss(
        &self,
        pubkey: &RsaPublicKey<'_>,
        hash: HashAlgorithm,
        message: &[u8],
        signature: &[u8],
    ) -> Result<()>;

    /// Verify an ECDSA signature over the *message* bytes. The
    /// implementation applies the standard hash for the curve
    /// (SHA-256 for P-256, SHA-384 for P-384) — `aws-lc-rs` and the
    /// `p256` / `p384` crates' `Verifier::verify` already hash
    /// internally.
    ///
    /// `pubkey_sec1` is the SEC1-encoded uncompressed public point
    /// (`0x04 || X || Y`); `signature_der` is the ASN.1 DER-encoded
    /// signature (the form CMS / X.509 carry).
    fn verify_ecdsa(
        &self,
        curve: EcCurve,
        pubkey_sec1: &[u8],
        message: &[u8],
        signature_der: &[u8],
    ) -> Result<()>;
}

/// Opaque signing handle.
///
/// Separating the handle type from the trait lets a provider back
/// `Signer` with anything: a software RSA key parsed from PKCS#8, an
/// HSM session, a Cloud KMS reference, a smart-card PKCS#11 slot. The
/// handle just has to remember its [`SigningAlgorithm`] and
/// `sign(message)`.
///
/// # PDF context
///
/// `signer.rs::create_pkcs7_signature` only needs the final signing
/// step to be opaque — DER/PEM key loading + CMS construction stay in
/// non-trait code. The trait is therefore intentionally narrow.
pub trait Signer: Send {
    /// Reports which `(asymmetric algo, hash)` pair this signer
    /// produces. Callers use this to populate the CMS
    /// `digestAlgorithm` and `signatureAlgorithm` fields.
    fn algorithm(&self) -> SigningAlgorithm;

    /// Sign `message`. For RSA-PKCS#1-v1.5 the caller passes a
    /// pre-built `DigestInfo` (algorithm OID + hashed bytes) and the
    /// signer applies raw RSA. For RSA-PSS and ECDSA the caller
    /// passes either the raw message (PSS internally hashes) or the
    /// pre-computed digest (ECDSA), as documented per scheme.
    ///
    /// The exact "what does `message` mean" contract follows what the
    /// existing CMS construction expects, so Phase 4 can keep
    /// `signer.rs` byte-equal.
    fn sign(&self, message: &[u8]) -> Result<Vec<u8>>;
}

/// The root trait that ties everything together.
///
/// Implementations live in:
///
/// - [`super::rust_provider::RustCryptoProvider`] (Phase 2) — default,
///   uses `sha2`/`sha1`/`md-5`/`aes`/`rsa`/`p256`/`p384`. Permits
///   legacy.
/// - `super::aws_lc_provider::AwsLcProvider` (Phase 6, behind
///   `--features fips`) — FIPS 140-3 validated. Refuses
///   legacy.
pub trait CryptoProvider: Send + Sync + 'static {
    /// Human-readable provider name for logs / SBOM annotations.
    fn name(&self) -> &'static str;

    /// Whether legacy algorithms (MD5, SHA-1 sign, RC4) are allowed.
    /// FIPS providers return `false`.
    ///
    /// Note: SHA-1 *verification* is allowed regardless — a separate
    /// policy via [`SignatureVerifier`].
    fn is_legacy_allowed(&self) -> bool;

    /// Construct a hasher for `algo`. Returns
    /// [`super::error::Error::AlgorithmNotPermitted`] if the
    /// algorithm is forbidden under this provider's policy.
    fn hasher(&self, algo: HashAlgorithm) -> Result<Box<dyn Hasher>>;

    /// Returns the symmetric cipher implementation. Always Some — the
    /// provider trait guarantees AES support; only `rc4()` may fail
    /// at call time under FIPS.
    fn symmetric(&self) -> &dyn SymmetricCipher;

    /// Returns the signature verification implementation.
    fn verifier(&self) -> &dyn SignatureVerifier;

    /// Fill `out` with cryptographically strong random bytes. Both
    /// providers source this from the OS RNG; `RustCryptoProvider`
    /// uses `getrandom`, `AwsLcProvider` uses
    /// `aws_lc_rs::rand::SystemRandom`.
    fn random_bytes(&self, out: &mut [u8]) -> Result<()>;

    /// Build a [`Signer`] from the provided [`SigningKeyMaterial`].
    /// Software providers parse the PEM/DER bytes; HSM/PKCS#11
    /// providers ignore the bytes path and instead consume their
    /// own handle variant (the enum is `#[non_exhaustive]` so adding
    /// `Pkcs11Slot` later isn't a breaking change).
    fn signer(&self, key: &SigningKeyMaterial<'_>) -> Result<Box<dyn Signer>>;
}

/// Material for constructing a [`Signer`].
///
/// `#[non_exhaustive]` so HSM/Cloud-KMS providers can add their own
/// variants in a follow-up release.
#[non_exhaustive]
#[derive(Debug)]
pub enum SigningKeyMaterial<'a> {
    /// PKCS#8 DER-encoded private key bytes. Software providers
    /// (`RustCryptoProvider`, `AwsLcProvider`) parse this directly.
    Pkcs8Der {
        /// Algorithm hint so the provider knows which scheme + hash
        /// to bind into the resulting signer.
        algo: SigningAlgorithm,
        /// PKCS#8 DER bytes.
        bytes: &'a [u8],
    },
    /// PKCS#1 DER-encoded RSA private key (legacy software keys).
    Pkcs1Der {
        /// RSA padding scheme the resulting signer should apply.
        scheme: RsaScheme,
        /// Message digest the signer should use.
        hash: HashAlgorithm,
        /// PKCS#1 DER bytes.
        bytes: &'a [u8],
    },
}
