//! Default [`CryptoProvider`] backed by the existing RustCrypto
//! crate stack — `sha2`, `sha1`, `md-5`, `aes`, `cbc`, `rsa`,
//! `p256`, `p384`, `getrandom`.
//!
//! This provider exists primarily so the rest of the crate can route
//! all crypto operations through the trait without changing
//! observable behaviour. Phases 3 and 4 use this to keep the existing
//! 86-PDF cross-build sweep at 888 / 888 byte-equal during the
//! migration.
//!
//! [`is_legacy_allowed`] returns `true`: every algorithm PDF specs
//! reference is permitted, including MD5, SHA-1 (sign and verify),
//! and RC4. Use [`super::AwsLcProvider`] (Phase 6) for FIPS-validated
//! deployments.
//!
//! [`is_legacy_allowed`]: super::CryptoProvider::is_legacy_allowed
//! [`super::AwsLcProvider`]: super::AwsLcProvider

use super::error::{Error, Result};
use super::provider::{
    CryptoProvider, Hasher, SignatureVerifier, Signer, SigningKeyMaterial, SymmetricCipher,
};
use super::types::{AesKeySize, EcCurve, HashAlgorithm, Padding, RsaPublicKey};
#[cfg(feature = "signatures")]
use super::types::{AsymmetricAlgorithm, RsaScheme, SigningAlgorithm};

/// The default Rust-only crypto provider.
///
/// Constructed via [`Self::new`]. Has no fields — providers are
/// stateless apart from any backend-specific initialization (which
/// the FIPS provider needs but this one doesn't).
#[derive(Debug, Default, Clone, Copy)]
pub struct RustCryptoProvider;

impl RustCryptoProvider {
    /// Create a new default-policy provider. Accepts every PDF-spec
    /// algorithm including the legacy MD5 / SHA-1 / RC4 paths.
    pub const fn new() -> Self {
        Self
    }
}

impl CryptoProvider for RustCryptoProvider {
    fn name(&self) -> &'static str {
        "rust-crypto"
    }

    fn is_legacy_allowed(&self) -> bool {
        cfg!(feature = "legacy-crypto")
    }

    fn hasher(&self, algo: HashAlgorithm) -> Result<Box<dyn Hasher>> {
        Ok(match algo {
            HashAlgorithm::Md5 => {
                #[cfg(feature = "legacy-crypto")]
                {
                    Box::new(Md5Hasher::new())
                }
                #[cfg(not(feature = "legacy-crypto"))]
                {
                    return Err(Error::AlgorithmNotPermitted {
                        kind: crate::crypto::error::AlgorithmKind::Hash,
                        name: "MD5",
                        reason: "legacy-crypto feature disabled at compile time",
                    });
                }
            },
            HashAlgorithm::Sha1 => {
                #[cfg(feature = "signatures")]
                {
                    Box::new(Sha1Hasher::new())
                }
                #[cfg(not(feature = "signatures"))]
                {
                    return Err(Error::Backend("SHA-1 requires the 'signatures' cargo feature"));
                }
            },
            HashAlgorithm::Sha256 => Box::new(Sha256Hasher::new()),
            HashAlgorithm::Sha384 => Box::new(Sha384Hasher::new()),
            HashAlgorithm::Sha512 => Box::new(Sha512Hasher::new()),
        })
    }

    fn symmetric(&self) -> &dyn SymmetricCipher {
        &RustSymmetric
    }

    fn verifier(&self) -> &dyn SignatureVerifier {
        &RustVerifier
    }

    fn random_bytes(&self, out: &mut [u8]) -> Result<()> {
        getrandom::fill(out).map_err(|_| Error::Backend("getrandom failed"))
    }

    fn signer(&self, _key: &SigningKeyMaterial<'_>) -> Result<Box<dyn Signer>> {
        #[cfg(feature = "signatures")]
        {
            signing::build_signer(_key)
        }
        #[cfg(not(feature = "signatures"))]
        {
            Err(Error::Backend("signing requires the 'signatures' cargo feature"))
        }
    }
}

// ---------------------------------------------------------------------------
// Hashers — one impl per algorithm. Boxed dispatch keeps the Hasher trait
// object-safe and avoids leaking generic digest::Update bounds out of the
// crypto module.
// ---------------------------------------------------------------------------

#[cfg(feature = "legacy-crypto")]
struct Md5Hasher(md5::Md5);
#[cfg(feature = "legacy-crypto")]
impl Md5Hasher {
    fn new() -> Self {
        use md5::Digest;
        Self(md5::Md5::new())
    }
}
#[cfg(feature = "legacy-crypto")]
impl Hasher for Md5Hasher {
    fn update(&mut self, data: &[u8]) {
        use md5::Digest;
        self.0.update(data);
    }
    fn finalize(self: Box<Self>) -> Vec<u8> {
        use md5::Digest;
        self.0.finalize().to_vec()
    }
    fn algorithm(&self) -> HashAlgorithm {
        HashAlgorithm::Md5
    }
}

#[cfg(feature = "signatures")]
struct Sha1Hasher(sha1::Sha1);

#[cfg(feature = "signatures")]
impl Sha1Hasher {
    fn new() -> Self {
        use sha1::Digest;
        Self(sha1::Sha1::new())
    }
}

#[cfg(feature = "signatures")]
impl Hasher for Sha1Hasher {
    fn update(&mut self, data: &[u8]) {
        use sha1::Digest;
        self.0.update(data);
    }
    fn finalize(self: Box<Self>) -> Vec<u8> {
        use sha1::Digest;
        self.0.finalize().to_vec()
    }
    fn algorithm(&self) -> HashAlgorithm {
        HashAlgorithm::Sha1
    }
}

macro_rules! sha2_hasher {
    ($name:ident, $inner:ty, $algo:expr) => {
        struct $name($inner);
        impl $name {
            fn new() -> Self {
                use sha2::Digest;
                Self(<$inner>::new())
            }
        }
        impl Hasher for $name {
            fn update(&mut self, data: &[u8]) {
                use sha2::Digest;
                self.0.update(data);
            }
            fn finalize(self: Box<Self>) -> Vec<u8> {
                use sha2::Digest;
                self.0.finalize().to_vec()
            }
            fn algorithm(&self) -> HashAlgorithm {
                $algo
            }
        }
    };
}

sha2_hasher!(Sha256Hasher, sha2::Sha256, HashAlgorithm::Sha256);
sha2_hasher!(Sha384Hasher, sha2::Sha384, HashAlgorithm::Sha384);
sha2_hasher!(Sha512Hasher, sha2::Sha512, HashAlgorithm::Sha512);

// ---------------------------------------------------------------------------
// Symmetric — AES-128/256-CBC (PKCS#7 + no-padding) and RC4.
// ---------------------------------------------------------------------------

struct RustSymmetric;

impl SymmetricCipher for RustSymmetric {
    fn aes_cbc_encrypt(
        &self,
        key_size: AesKeySize,
        key: &[u8],
        iv: &[u8],
        data: &[u8],
        padding: Padding,
    ) -> Result<Vec<u8>> {
        check_key_iv(key_size, key, iv)?;
        if matches!(padding, Padding::None) && !data.len().is_multiple_of(16) {
            return Err(Error::InvalidInput(
                "no-padding AES-CBC requires data length to be a 16-byte multiple",
            ));
        }
        match (key_size, padding) {
            (AesKeySize::Aes128, Padding::Pkcs7) => {
                aes_cbc_encrypt_pkcs7::<aes::Aes128>(key, iv, data)
            },
            (AesKeySize::Aes128, Padding::None) => {
                aes_cbc_encrypt_no_pad::<aes::Aes128>(key, iv, data)
            },
            (AesKeySize::Aes256, Padding::Pkcs7) => {
                aes_cbc_encrypt_pkcs7::<aes::Aes256>(key, iv, data)
            },
            (AesKeySize::Aes256, Padding::None) => {
                aes_cbc_encrypt_no_pad::<aes::Aes256>(key, iv, data)
            },
        }
    }

    fn aes_cbc_decrypt(
        &self,
        key_size: AesKeySize,
        key: &[u8],
        iv: &[u8],
        data: &[u8],
        padding: Padding,
    ) -> Result<Vec<u8>> {
        check_key_iv(key_size, key, iv)?;
        if !data.len().is_multiple_of(16) {
            return Err(Error::InvalidInput("AES-CBC ciphertext must be a 16-byte multiple"));
        }
        match (key_size, padding) {
            (AesKeySize::Aes128, Padding::Pkcs7) => {
                aes_cbc_decrypt_pkcs7::<aes::Aes128>(key, iv, data)
            },
            (AesKeySize::Aes128, Padding::None) => {
                aes_cbc_decrypt_no_pad::<aes::Aes128>(key, iv, data)
            },
            (AesKeySize::Aes256, Padding::Pkcs7) => {
                aes_cbc_decrypt_pkcs7::<aes::Aes256>(key, iv, data)
            },
            (AesKeySize::Aes256, Padding::None) => {
                aes_cbc_decrypt_no_pad::<aes::Aes256>(key, iv, data)
            },
        }
    }

    #[cfg_attr(not(feature = "legacy-crypto"), allow(unused_variables))]
    fn rc4(&self, key: &[u8], data: &[u8]) -> Result<Vec<u8>> {
        #[cfg(not(feature = "legacy-crypto"))]
        {
            Err(Error::AlgorithmNotPermitted {
                kind: crate::crypto::error::AlgorithmKind::SymmetricCipher,
                name: "RC4",
                reason: "legacy-crypto feature disabled at compile time",
            })
        }
        #[cfg(feature = "legacy-crypto")]
        {
            if key.is_empty() || key.len() > 256 {
                return Err(Error::InvalidInput("RC4 key must be 1..=256 bytes"));
            }
            // Calls the in-tree pure cipher impl directly (not the
            // `pub fn rc4_crypt` wrapper, which itself routes through us
            // — that would loop). Byte-equal to pre-Phase-3 output.
            Ok(crate::encryption::rc4::rc4_crypt_impl(key, data))
        }
    }
}

fn check_key_iv(key_size: AesKeySize, key: &[u8], iv: &[u8]) -> Result<()> {
    if key.len() != key_size.key_bytes() {
        return Err(Error::InvalidInput(match key_size {
            AesKeySize::Aes128 => "AES-128 requires a 16-byte key",
            AesKeySize::Aes256 => "AES-256 requires a 32-byte key",
        }));
    }
    if iv.len() != 16 {
        return Err(Error::InvalidInput("AES-CBC requires a 16-byte IV"));
    }
    Ok(())
}

// Generic over the block cipher so AES-128 and AES-256 share the body.

fn aes_cbc_encrypt_pkcs7<C>(key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>>
where
    C: aes::cipher::BlockCipherEncrypt + aes::cipher::KeyInit,
{
    use aes::cipher::{block_padding::Pkcs7, BlockModeEncrypt, KeyIvInit};
    type Enc<C> = cbc::Encryptor<C>;
    let cipher = <Enc<C> as KeyIvInit>::new_from_slices(key, iv)
        .map_err(|_| Error::InvalidInput("AES-CBC key/iv length mismatch"))?;
    // `encrypt_padded` writes from `buf[..msg_len]` and adds padding
    // up to one extra block; size the buffer accordingly and copy the
    // plaintext into the prefix region first.
    let mut buf = vec![0u8; data.len() + 16];
    buf[..data.len()].copy_from_slice(data);
    let n = cipher
        .encrypt_padded::<Pkcs7>(&mut buf, data.len())
        .map_err(|_| Error::Backend("AES-CBC PKCS#7 encryption failed"))?
        .len();
    buf.truncate(n);
    Ok(buf)
}

fn aes_cbc_decrypt_pkcs7<C>(key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>>
where
    C: aes::cipher::BlockCipherDecrypt + aes::cipher::KeyInit,
{
    use aes::cipher::{block_padding::Pkcs7, BlockModeDecrypt, KeyIvInit};
    type Dec<C> = cbc::Decryptor<C>;
    let cipher = <Dec<C> as KeyIvInit>::new_from_slices(key, iv)
        .map_err(|_| Error::InvalidInput("AES-CBC key/iv length mismatch"))?;
    let mut buf = data.to_vec();
    let n = cipher
        .decrypt_padded::<Pkcs7>(&mut buf)
        .map_err(|_| Error::Verification("AES-CBC PKCS#7 padding invalid"))?
        .len();
    buf.truncate(n);
    Ok(buf)
}

fn aes_cbc_encrypt_no_pad<C>(key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>>
where
    C: aes::cipher::BlockCipherEncrypt + aes::cipher::KeyInit,
{
    use aes::cipher::{block_padding::NoPadding, BlockModeEncrypt, KeyIvInit};
    type Enc<C> = cbc::Encryptor<C>;
    let cipher = <Enc<C> as KeyIvInit>::new_from_slices(key, iv)
        .map_err(|_| Error::InvalidInput("AES-CBC key/iv length mismatch"))?;
    let mut buf = data.to_vec();
    let len = data.len();
    cipher
        .encrypt_padded::<NoPadding>(&mut buf, len)
        .map_err(|_| Error::Backend("AES-CBC no-padding encryption failed"))?;
    Ok(buf)
}

fn aes_cbc_decrypt_no_pad<C>(key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>>
where
    C: aes::cipher::BlockCipherDecrypt + aes::cipher::KeyInit,
{
    use aes::cipher::{block_padding::NoPadding, BlockModeDecrypt, KeyIvInit};
    type Dec<C> = cbc::Decryptor<C>;
    let cipher = <Dec<C> as KeyIvInit>::new_from_slices(key, iv)
        .map_err(|_| Error::InvalidInput("AES-CBC key/iv length mismatch"))?;
    let mut buf = data.to_vec();
    cipher
        .decrypt_padded::<NoPadding>(&mut buf)
        .map_err(|_| Error::Backend("AES-CBC no-padding decryption failed"))?;
    Ok(buf)
}

// ---------------------------------------------------------------------------
// Signature verification — gated on the `signatures` feature.
// ---------------------------------------------------------------------------

struct RustVerifier;

#[cfg(feature = "signatures")]
impl SignatureVerifier for RustVerifier {
    /// Verifies a PKCS#1 v1.5 signature.
    ///
    /// `message` is the raw bytes to verify — the same convention as
    /// `verify_rsa_pss` / `verify_ecdsa`. We hash it with `hash` here
    /// and build the DigestInfo before calling
    /// `rsa::Pkcs1v15Sign::new_unprefixed()`.
    fn verify_rsa_pkcs1v15(
        &self,
        pubkey: &RsaPublicKey<'_>,
        hash: HashAlgorithm,
        message: &[u8],
        signature: &[u8],
    ) -> Result<()> {
        use crate::signatures::crypto::digest_info_prefix;
        use rsa::pkcs1v15::Pkcs1v15Sign;
        use rsa::RsaPublicKey as RcRsa;

        let (oid, digest) = match hash {
            HashAlgorithm::Sha1 => {
                use sha1::Digest as _;
                (der::oid::db::rfc5912::ID_SHA_1, sha1::Sha1::digest(message).to_vec())
            },
            HashAlgorithm::Sha256 => {
                use sha2::Digest as _;
                (der::oid::db::rfc5912::ID_SHA_256, sha2::Sha256::digest(message).to_vec())
            },
            HashAlgorithm::Sha384 => {
                use sha2::Digest as _;
                (der::oid::db::rfc5912::ID_SHA_384, sha2::Sha384::digest(message).to_vec())
            },
            HashAlgorithm::Sha512 => {
                use sha2::Digest as _;
                (der::oid::db::rfc5912::ID_SHA_512, sha2::Sha512::digest(message).to_vec())
            },
            HashAlgorithm::Md5 => {
                return Err(Error::Verification(
                    "MD5 not supported for RSA-PKCS#1-v1.5 signature verification",
                ));
            },
        };

        let prefix = digest_info_prefix(oid)
            .ok_or(Error::Backend("no DigestInfo prefix table entry for selected hash"))?;
        let mut digest_info = Vec::with_capacity(prefix.len() + digest.len());
        digest_info.extend_from_slice(prefix);
        digest_info.extend_from_slice(&digest);

        let n = rsa::BoxedUint::from_be_slice_vartime(pubkey.modulus_be);
        let e = rsa::BoxedUint::from_be_slice_vartime(pubkey.exponent_be);
        let key =
            RcRsa::new(n, e).map_err(|_| Error::InvalidInput("invalid RSA modulus/exponent"))?;
        key.verify(Pkcs1v15Sign::new_unprefixed(), &digest_info, signature)
            .map_err(|_| Error::Verification("RSA-PKCS#1-v1.5 signature did not verify"))
    }

    fn verify_rsa_pss(
        &self,
        pubkey: &RsaPublicKey<'_>,
        hash: HashAlgorithm,
        message: &[u8],
        signature: &[u8],
    ) -> Result<()> {
        use rsa::pss::{Signature as PssSignature, VerifyingKey};
        use rsa::signature::Verifier;
        use rsa::RsaPublicKey as RcRsa;

        let n = rsa::BoxedUint::from_be_slice_vartime(pubkey.modulus_be);
        let e = rsa::BoxedUint::from_be_slice_vartime(pubkey.exponent_be);
        let key =
            RcRsa::new(n, e).map_err(|_| Error::InvalidInput("invalid RSA modulus/exponent"))?;
        let sig = PssSignature::try_from(signature)
            .map_err(|_| Error::InvalidInput("malformed RSA-PSS signature bytes"))?;
        let ok = match hash {
            HashAlgorithm::Sha256 => VerifyingKey::<sha2::Sha256>::new(key)
                .verify(message, &sig)
                .is_ok(),
            HashAlgorithm::Sha384 => VerifyingKey::<sha2::Sha384>::new(key)
                .verify(message, &sig)
                .is_ok(),
            HashAlgorithm::Sha512 => VerifyingKey::<sha2::Sha512>::new(key)
                .verify(message, &sig)
                .is_ok(),
            HashAlgorithm::Sha1 | HashAlgorithm::Md5 => {
                // Mirrors `cms_verify.rs:160-162` — the rsa 0.9 +
                // sha1 0.11 / md-5 0.11 trait-bound mismatch makes
                // these unreachable in this provider; FIPS provider
                // also rejects them.
                return Err(Error::Backend(
                    "SHA-1 / MD5 RSA-PSS not supported by RustCryptoProvider \
                     (rsa 0.9 ↔ sha1 0.11 / md-5 0.11 digest-trait mismatch)",
                ));
            },
        };
        if ok {
            Ok(())
        } else {
            Err(Error::Verification("RSA-PSS signature did not verify"))
        }
    }

    fn verify_ecdsa(
        &self,
        curve: EcCurve,
        pubkey_sec1: &[u8],
        message: &[u8],
        signature_der: &[u8],
    ) -> Result<()> {
        match curve {
            EcCurve::P256 => {
                use p256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
                let vk = VerifyingKey::from_sec1_bytes(pubkey_sec1)
                    .map_err(|_| Error::InvalidInput("invalid SEC1 P-256 public key"))?;
                let sig = Signature::from_der(signature_der)
                    .map_err(|_| Error::InvalidInput("malformed P-256 ECDSA signature DER"))?;
                vk.verify(message, &sig)
                    .map_err(|_| Error::Verification("P-256 ECDSA signature did not verify"))
            },
            EcCurve::P384 => {
                use p384::ecdsa::{signature::Verifier, Signature, VerifyingKey};
                let vk = VerifyingKey::from_sec1_bytes(pubkey_sec1)
                    .map_err(|_| Error::InvalidInput("invalid SEC1 P-384 public key"))?;
                let sig = Signature::from_der(signature_der)
                    .map_err(|_| Error::InvalidInput("malformed P-384 ECDSA signature DER"))?;
                vk.verify(message, &sig)
                    .map_err(|_| Error::Verification("P-384 ECDSA signature did not verify"))
            },
        }
    }
}

#[cfg(not(feature = "signatures"))]
impl SignatureVerifier for RustVerifier {
    fn verify_rsa_pkcs1v15(
        &self,
        _pubkey: &RsaPublicKey<'_>,
        _hash: HashAlgorithm,
        _message: &[u8],
        _signature: &[u8],
    ) -> Result<()> {
        Err(Error::Backend("RSA verification requires the 'signatures' cargo feature"))
    }
    fn verify_rsa_pss(
        &self,
        _pubkey: &RsaPublicKey<'_>,
        _hash: HashAlgorithm,
        _message: &[u8],
        _signature: &[u8],
    ) -> Result<()> {
        Err(Error::Backend("RSA-PSS verification requires the 'signatures' cargo feature"))
    }
    fn verify_ecdsa(
        &self,
        _curve: EcCurve,
        _pubkey_sec1: &[u8],
        _digest: &[u8],
        _signature_der: &[u8],
    ) -> Result<()> {
        Err(Error::Backend("ECDSA verification requires the 'signatures' cargo feature"))
    }
}

// ---------------------------------------------------------------------------
// Signing — gated on the `signatures` feature.
// ---------------------------------------------------------------------------

/// PKCS#1 v1.5 signing only — matches the existing
/// `src/signatures/signer.rs::create_pkcs7_signature` path (which
/// passes a pre-built DigestInfo to `Pkcs1v15Sign::new_unprefixed()`).
/// PSS signing isn't implemented here because no in-tree caller
/// produces it today; FIPS deployments that need PSS sign should use
/// [`super::AwsLcProvider`] in Phase 6.
#[cfg(feature = "signatures")]
mod signing {
    use super::*;
    use rsa::pkcs1v15::Pkcs1v15Sign;
    use rsa::pkcs8::DecodePrivateKey;
    use rsa::RsaPrivateKey;

    pub(super) fn build_signer(key: &SigningKeyMaterial<'_>) -> Result<Box<dyn Signer>> {
        let (priv_key, algo, scheme) = match key {
            SigningKeyMaterial::Pkcs8Der { algo, bytes } => match algo.asym {
                AsymmetricAlgorithm::Rsa(scheme) => {
                    let pk = RsaPrivateKey::from_pkcs8_der(bytes)
                        .map_err(|_| Error::InvalidInput("invalid PKCS#8 RSA private key"))?;
                    (pk, *algo, scheme)
                },
                AsymmetricAlgorithm::Ecdsa(_) => {
                    return Err(Error::Backend(
                        "ECDSA signing not yet implemented in RustCryptoProvider",
                    ));
                },
            },
            SigningKeyMaterial::Pkcs1Der {
                scheme,
                hash,
                bytes,
            } => {
                use rsa::pkcs1::DecodeRsaPrivateKey;
                let pk = RsaPrivateKey::from_pkcs1_der(bytes)
                    .map_err(|_| Error::InvalidInput("invalid PKCS#1 RSA private key"))?;
                let algo = SigningAlgorithm {
                    asym: AsymmetricAlgorithm::Rsa(*scheme),
                    hash: *hash,
                };
                (pk, algo, *scheme)
            },
        };

        if !matches!(scheme, RsaScheme::Pkcs1v15) {
            return Err(Error::Backend(
                "RustCryptoProvider only signs RSA-PKCS#1-v1.5; use AwsLcProvider for PSS sign",
            ));
        }

        Ok(Box::new(RsaSigner {
            key: priv_key,
            algo,
        }))
    }

    struct RsaSigner {
        key: RsaPrivateKey,
        algo: SigningAlgorithm,
    }

    impl Signer for RsaSigner {
        fn algorithm(&self) -> SigningAlgorithm {
            self.algo
        }

        /// `message` is a pre-built DigestInfo (algo OID prefix +
        /// hashed bytes); raw RSA is applied via
        /// `Pkcs1v15Sign::new_unprefixed`. Mirrors the call shape used
        /// at `src/signatures/signer.rs:226` so Phase 4's switchover
        /// is mechanical.
        fn sign(&self, message: &[u8]) -> Result<Vec<u8>> {
            self.key
                .sign(Pkcs1v15Sign::new_unprefixed(), message)
                .map_err(|_| Error::Backend("RSA-PKCS#1-v1.5 signing failed"))
        }
    }
}

// ---------------------------------------------------------------------------
// Tests — provider behaviour against fixed vectors.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn provider() -> RustCryptoProvider {
        RustCryptoProvider::new()
    }

    #[test]
    fn name_and_legacy_policy() {
        let p = provider();
        assert_eq!(p.name(), "rust-crypto");
        #[cfg(feature = "legacy-crypto")]
        assert!(p.is_legacy_allowed());
        #[cfg(not(feature = "legacy-crypto"))]
        assert!(!p.is_legacy_allowed());
    }

    // --- Hash vectors (RFCs / NIST FIPS 180-4) ---

    use hex_literal::hex;

    #[cfg(feature = "legacy-crypto")]
    #[test]
    fn md5_empty() {
        let p = provider();
        let mut h = p.hasher(HashAlgorithm::Md5).unwrap();
        h.update(b"");
        assert_eq!(h.finalize(), hex!("d41d8cd98f00b204e9800998ecf8427e"));
    }

    #[cfg(feature = "legacy-crypto")]
    #[test]
    fn md5_abc() {
        let p = provider();
        let mut h = p.hasher(HashAlgorithm::Md5).unwrap();
        h.update(b"abc");
        assert_eq!(h.finalize(), hex!("900150983cd24fb0d6963f7d28e17f72"));
    }

    #[test]
    fn sha256_abc() {
        let p = provider();
        let mut h = p.hasher(HashAlgorithm::Sha256).unwrap();
        h.update(b"abc");
        assert_eq!(
            h.finalize(),
            hex!("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
        );
    }

    #[test]
    fn sha384_abc() {
        let p = provider();
        let mut h = p.hasher(HashAlgorithm::Sha384).unwrap();
        h.update(b"abc");
        assert_eq!(
            h.finalize(),
            hex!(
                "cb00753f45a35e8bb5a03d699ac65007"
                "272c32ab0eded1631a8b605a43ff5bed"
                "8086072ba1e7cc2358baeca134c825a7"
            )
        );
    }

    #[test]
    fn sha512_abc() {
        let p = provider();
        let mut h = p.hasher(HashAlgorithm::Sha512).unwrap();
        h.update(b"abc");
        assert_eq!(
            h.finalize(),
            hex!(
                "ddaf35a193617abacc417349ae204131"
                "12e6fa4e89a97ea20a9eeee64b55d39a"
                "2192992a274fc1a836ba3c23a3feebbd"
                "454d4423643ce80e2a9ac94fa54ca49f"
            )
        );
    }

    #[cfg(feature = "signatures")]
    #[test]
    fn sha1_abc() {
        let p = provider();
        let mut h = p.hasher(HashAlgorithm::Sha1).unwrap();
        h.update(b"abc");
        assert_eq!(h.finalize(), hex!("a9993e364706816aba3e25717850c26c9cd0d89d"));
    }

    // --- AES round-trip (NIST CAVP would need fixed inputs — round-trip
    // is sufficient to prove the provider plumbs both directions correctly;
    // byte-equal tests against the existing aes::aes128_encrypt come in
    // Phase 3 once that module routes through the trait).

    #[test]
    fn aes128_cbc_pkcs7_round_trip() {
        let p = provider();
        let key = [0x42u8; 16];
        let iv = [0x13u8; 16];
        let plaintext = b"PDF Standard Security Handler V=4 stream content.";
        let ct = p
            .symmetric()
            .aes_cbc_encrypt(AesKeySize::Aes128, &key, &iv, plaintext, Padding::Pkcs7)
            .unwrap();
        let pt = p
            .symmetric()
            .aes_cbc_decrypt(AesKeySize::Aes128, &key, &iv, &ct, Padding::Pkcs7)
            .unwrap();
        assert_eq!(pt, plaintext);
    }

    #[test]
    fn aes256_cbc_no_pad_round_trip() {
        let p = provider();
        let key = [0x07u8; 32];
        let iv = [0u8; 16]; // matches V=5 key-wrap zero IV
        let plaintext = [0xa5u8; 32]; // exactly 2 blocks (UE/OE shape)
        let ct = p
            .symmetric()
            .aes_cbc_encrypt(AesKeySize::Aes256, &key, &iv, &plaintext, Padding::None)
            .unwrap();
        let pt = p
            .symmetric()
            .aes_cbc_decrypt(AesKeySize::Aes256, &key, &iv, &ct, Padding::None)
            .unwrap();
        assert_eq!(pt, plaintext);
    }

    #[test]
    fn aes_cbc_no_pad_rejects_unaligned_input() {
        let p = provider();
        let key = [0u8; 16];
        let iv = [0u8; 16];
        let result = p.symmetric().aes_cbc_encrypt(
            AesKeySize::Aes128,
            &key,
            &iv,
            b"15 bytes only..",
            Padding::None,
        );
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }

    #[test]
    fn aes_rejects_wrong_key_length() {
        let p = provider();
        let result = p.symmetric().aes_cbc_encrypt(
            AesKeySize::Aes128,
            &[0u8; 8], // too short
            &[0u8; 16],
            b"data1234data1234",
            Padding::None,
        );
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }

    // --- RC4 round-trip (PDF R≤4 path).

    #[cfg(feature = "legacy-crypto")]
    #[test]
    fn rc4_round_trip() {
        let p = provider();
        let key = b"PDF-encrypt-key";
        let plaintext = b"Sensitive PDF content stream.";
        let ct = p.symmetric().rc4(key, plaintext).unwrap();
        let pt = p.symmetric().rc4(key, &ct).unwrap();
        assert_eq!(pt, plaintext);
    }

    // --- Random bytes — non-zero output, non-determinism.

    #[test]
    fn random_bytes_are_non_constant() {
        let p = provider();
        let mut a = [0u8; 32];
        let mut b = [0u8; 32];
        p.random_bytes(&mut a).unwrap();
        p.random_bytes(&mut b).unwrap();
        assert_ne!(a, b, "random_bytes must not return constant data");
    }
}
