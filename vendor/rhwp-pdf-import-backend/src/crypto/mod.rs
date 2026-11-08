//! Cryptographic provider abstraction.
//!
//! This module decouples PDF encryption and signature paths from any
//! one cryptography crate so deployments can choose between:
//!
//! - **`RustCryptoProvider`** (default) — built on `sha2`, `sha1`,
//!   `md-5`, `aes`, `rsa`, `p256`, `p384`, `getrandom`. Permits all
//!   PDF-spec-required algorithms including the legacy MD5+RC4 path
//!   needed for ISO 32000-1 R≤4 documents.
//! - **`AwsLcProvider`** (Phase 6, behind `--features fips`)
//!   — built on `aws-lc-rs` with the `fips` feature. FIPS 140-3
//!   validated since 2024. Refuses MD5, RC4, and SHA-1-for-signing.
//!
//! Downstream consumers can also implement [`CryptoProvider`] for
//! HSM/PKCS#11 backends, sovereign-jurisdiction algorithms (GOST,
//! SM2/3/4), or hardware-rooted Cloud KMS providers.
//!
//! Tracking issue: <https://github.com/yfedoseev/pdf_oxide/issues/236>.

mod active;
#[cfg(feature = "fips")]
mod aws_lc_provider;
mod cbom;
mod error;
mod policy;
mod provider;
mod rust_provider;
mod types;

pub use active::{
    active, active_policy, inventory, is_policy_set, is_set, record_algorithm_use, set_policy,
    set_provider, SetPolicyError, SetProviderError,
};
#[cfg(feature = "fips")]
pub use aws_lc_provider::AwsLcProvider;
pub use cbom::cbom_json;
pub use error::{not_permitted, AlgorithmKind, Error, Result};
pub use policy::{
    AlgorithmId, AlgorithmUse, AuditEvent, AuditSink, Decision, LogAuditSink, NoopAuditSink,
    PolicyMode, PolicyParseError, SecurityPolicy, SecurityPolicyBuilder,
};
pub use provider::{
    CryptoProvider, Hasher, SignatureVerifier, Signer, SigningKeyMaterial, SymmetricCipher,
};
pub use rust_provider::RustCryptoProvider;
pub use types::{
    AesKeySize, AsymmetricAlgorithm, EcCurve, HashAlgorithm, Padding, RsaPublicKey, RsaScheme,
    SigningAlgorithm,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// Trait-object compile check: `CryptoProvider` must be usable
    /// behind `Box<dyn CryptoProvider>` (so we can store the active
    /// provider in a `OnceLock<Arc<dyn CryptoProvider>>` later).
    #[test]
    fn crypto_provider_is_object_safe() {
        fn _assert_object_safe(_: Box<dyn CryptoProvider>) {}
    }

    /// Sub-traits must be object-safe too.
    #[test]
    fn sub_traits_object_safe() {
        fn _hasher(_: Box<dyn Hasher>) {}
        fn _sym(_: &dyn SymmetricCipher) {}
        fn _ver(_: &dyn SignatureVerifier) {}
        fn _signer(_: Box<dyn Signer>) {}
    }

    #[test]
    fn hash_algorithm_output_sizes_match_spec() {
        assert_eq!(HashAlgorithm::Md5.output_size(), 16);
        assert_eq!(HashAlgorithm::Sha1.output_size(), 20);
        assert_eq!(HashAlgorithm::Sha256.output_size(), 32);
        assert_eq!(HashAlgorithm::Sha384.output_size(), 48);
        assert_eq!(HashAlgorithm::Sha512.output_size(), 64);
    }

    #[test]
    fn fips_approved_set_excludes_legacy() {
        // SHA-2 family is approved.
        assert!(HashAlgorithm::Sha256.is_fips_approved());
        assert!(HashAlgorithm::Sha384.is_fips_approved());
        assert!(HashAlgorithm::Sha512.is_fips_approved());
        // MD5 and SHA-1 are not.
        assert!(!HashAlgorithm::Md5.is_fips_approved());
        assert!(!HashAlgorithm::Sha1.is_fips_approved());
    }

    #[test]
    fn aes_key_size_bytes() {
        assert_eq!(AesKeySize::Aes128.key_bytes(), 16);
        assert_eq!(AesKeySize::Aes256.key_bytes(), 32);
    }

    #[test]
    fn error_display_includes_kind_and_name() {
        let err = not_permitted(AlgorithmKind::Hash, "MD5", "FIPS 140-3 forbids MD5");
        let s = err.to_string();
        assert!(s.contains("hash"));
        assert!(s.contains("MD5"));
        assert!(s.contains("FIPS"));
    }

    #[test]
    fn signing_algorithm_names_round_trip() {
        let alg = SigningAlgorithm {
            asym: AsymmetricAlgorithm::Rsa(RsaScheme::Pkcs1v15),
            hash: HashAlgorithm::Sha256,
        };
        assert_eq!(alg.name(), "RSA-PKCS1v15-SHA256");

        let alg = SigningAlgorithm {
            asym: AsymmetricAlgorithm::Ecdsa(EcCurve::P256),
            hash: HashAlgorithm::Sha256,
        };
        assert_eq!(alg.name(), "ECDSA-P256-SHA256");
    }
}
