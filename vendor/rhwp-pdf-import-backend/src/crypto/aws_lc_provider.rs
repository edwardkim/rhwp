//! FIPS 140-3 validated [`CryptoProvider`] backed by `aws-lc-rs`.
//!
//! Built behind the `fips` cargo feature (off by default).
//! When enabled and installed at runtime via [`super::set_provider`],
//! every PDF crypto operation routes to AWS-LC's FIPS-validated
//! module. Algorithms outside the NIST 140-3 approved set
//! (MD5, RC4, SHA-1 sign, RSA-PKCS#1-v1.5+SHA-1) return
//! [`super::Error::AlgorithmNotPermitted`] — opening a PDF Standard
//! Security R≤4 document, for example, fails with a clear
//! remediation message at [`crate::encryption::handler::EncryptionHandler::new`].
//!
//! # Algorithm coverage
//!
//! | Operation | FIPS-approved? | This provider |
//! |---|---|---|
//! | SHA-256 / 384 / 512 | ✅ | implemented |
//! | MD5 | ❌ | rejected |
//! | SHA-1 hash | (verify-only) | implemented (callers gate sign vs verify) |
//! | AES-128 / 256 CBC PKCS#7 | ✅ | implemented |
//! | AES-128 / 256 CBC no-padding | ✅ | implemented (`unstable::cipher`) |
//! | RC4 | ❌ | rejected |
//! | RSA-PSS verify (SHA-256/384/512) | ✅ | implemented |
//! | RSA-PSS sign | ✅ | not implemented yet (no in-tree caller) |
//! | RSA-PKCS#1-v1.5 verify (SHA-256+) | ✅ | implemented |
//! | RSA-PKCS#1-v1.5 verify (SHA-1) | (verify-only) | implemented |
//! | RSA-PKCS#1-v1.5 sign | ✅ when paired with SHA-256+ | not yet wired (Phase 4 stays on rust-crypto for sign) |
//! | ECDSA P-256 / P-384 verify | ✅ | implemented |
//! | OS RNG | ✅ | `aws_lc_rs::rand::SystemRandom` |
//!
//! Issue #236.

use aws_lc_rs::cipher::{
    DecryptingKey, EncryptingKey, EncryptionContext, PaddedBlockDecryptingKey,
    PaddedBlockEncryptingKey, UnboundCipherKey, AES_128, AES_256,
};
use aws_lc_rs::digest::{Context, SHA1_FOR_LEGACY_USE_ONLY, SHA256, SHA384, SHA512};
use aws_lc_rs::iv::FixedLength;
use aws_lc_rs::rand::{SecureRandom, SystemRandom};
use aws_lc_rs::signature::{
    self, UnparsedPublicKey, ECDSA_P256_SHA256_ASN1, ECDSA_P384_SHA384_ASN1,
    RSA_PKCS1_2048_8192_SHA1_FOR_LEGACY_USE_ONLY, RSA_PKCS1_2048_8192_SHA256,
    RSA_PKCS1_2048_8192_SHA384, RSA_PKCS1_2048_8192_SHA512, RSA_PSS_2048_8192_SHA256,
    RSA_PSS_2048_8192_SHA384, RSA_PSS_2048_8192_SHA512,
};

use super::error::{not_permitted, AlgorithmKind, Error, Result};
use super::provider::{
    CryptoProvider, Hasher, SignatureVerifier, Signer, SigningKeyMaterial, SymmetricCipher,
};
use super::types::{AesKeySize, EcCurve, HashAlgorithm, Padding, RsaPublicKey};

/// FIPS-validated provider backed by `aws-lc-rs`.
#[derive(Debug, Default, Clone, Copy)]
pub struct AwsLcProvider;

impl AwsLcProvider {
    /// Create a new FIPS-validated provider. The AWS-LC FIPS module
    /// runs its required power-on self-test on first crypto call;
    /// failures surface as [`Error::Backend`].
    pub const fn new() -> Self {
        Self
    }
}

impl CryptoProvider for AwsLcProvider {
    fn name(&self) -> &'static str {
        "aws-lc-rs"
    }

    fn is_legacy_allowed(&self) -> bool {
        false
    }

    fn hasher(&self, algo: HashAlgorithm) -> Result<Box<dyn Hasher>> {
        let alg = match algo {
            HashAlgorithm::Md5 => {
                return Err(not_permitted(
                    AlgorithmKind::Hash,
                    "MD5",
                    "FIPS 140-3 forbids MD5 for any use (NIST SP 800-131A)",
                ));
            },
            HashAlgorithm::Sha1 => &SHA1_FOR_LEGACY_USE_ONLY,
            HashAlgorithm::Sha256 => &SHA256,
            HashAlgorithm::Sha384 => &SHA384,
            HashAlgorithm::Sha512 => &SHA512,
        };
        Ok(Box::new(AwsLcHasher {
            ctx: Context::new(alg),
            algo,
        }))
    }

    fn symmetric(&self) -> &dyn SymmetricCipher {
        &AwsLcSymmetric
    }

    fn verifier(&self) -> &dyn SignatureVerifier {
        &AwsLcVerifier
    }

    fn random_bytes(&self, out: &mut [u8]) -> Result<()> {
        SystemRandom::new()
            .fill(out)
            .map_err(|_| Error::Backend("aws_lc_rs SystemRandom failed"))
    }

    fn signer(&self, _key: &SigningKeyMaterial<'_>) -> Result<Box<dyn Signer>> {
        // Signing is not yet wired through this provider — current
        // in-tree caller (`signatures/signer.rs`) builds CMS via
        // `RustCryptoProvider`'s software RSA sign. A follow-up
        // commit will route signing here once the FIPS sign API
        // (`aws_lc_rs::rsa::KeyPair::sign`) integration is tested
        // against the existing CMS test fixtures.
        Err(Error::Backend(
            "AwsLcProvider signing not yet implemented — \
             use RustCryptoProvider for sign or pre-sign with an HSM",
        ))
    }
}

// ---------------------------------------------------------------------------
// Hasher
// ---------------------------------------------------------------------------

struct AwsLcHasher {
    ctx: Context,
    algo: HashAlgorithm,
}

impl Hasher for AwsLcHasher {
    fn update(&mut self, data: &[u8]) {
        self.ctx.update(data);
    }
    fn finalize(self: Box<Self>) -> Vec<u8> {
        self.ctx.finish().as_ref().to_vec()
    }
    fn algorithm(&self) -> HashAlgorithm {
        self.algo
    }
}

// ---------------------------------------------------------------------------
// Symmetric — AES-CBC; RC4 rejected.
// ---------------------------------------------------------------------------

fn aes_alg(key_size: AesKeySize) -> &'static aws_lc_rs::cipher::Algorithm {
    match key_size {
        AesKeySize::Aes128 => &AES_128,
        AesKeySize::Aes256 => &AES_256,
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

struct AwsLcSymmetric;

impl SymmetricCipher for AwsLcSymmetric {
    fn aes_cbc_encrypt(
        &self,
        key_size: AesKeySize,
        key: &[u8],
        iv: &[u8],
        data: &[u8],
        padding: Padding,
    ) -> Result<Vec<u8>> {
        check_key_iv(key_size, key, iv)?;
        let iv_array: [u8; 16] = iv.try_into().expect("checked above");
        let ec = EncryptionContext::Iv128(FixedLength::from(iv_array));
        let alg = aes_alg(key_size);

        match padding {
            Padding::Pkcs7 => {
                let unbound = UnboundCipherKey::new(alg, key)
                    .map_err(|_| Error::Backend("aws_lc_rs UnboundCipherKey::new failed"))?;
                let ek = PaddedBlockEncryptingKey::cbc_pkcs7(unbound).map_err(|_| {
                    Error::Backend("aws_lc_rs PaddedBlockEncryptingKey::cbc_pkcs7 failed")
                })?;
                // PKCS#7 expands input by up to one full block.
                let mut buf: Vec<u8> = Vec::with_capacity(data.len() + 16);
                buf.extend_from_slice(data);
                ek.less_safe_encrypt(&mut buf, ec)
                    .map_err(|_| Error::Backend("aws_lc_rs AES-CBC PKCS#7 encrypt failed"))?;
                Ok(buf)
            },
            Padding::None => {
                if !data.len().is_multiple_of(16) {
                    return Err(Error::InvalidInput(
                        "no-padding AES-CBC requires data length to be a 16-byte multiple",
                    ));
                }
                let unbound = UnboundCipherKey::new(alg, key)
                    .map_err(|_| Error::Backend("aws_lc_rs UnboundCipherKey::new failed"))?;
                let ek = EncryptingKey::cbc(unbound)
                    .map_err(|_| Error::Backend("aws_lc_rs EncryptingKey::cbc failed"))?;
                let mut buf = data.to_vec();
                ek.less_safe_encrypt(&mut buf, ec)
                    .map_err(|_| Error::Backend("aws_lc_rs AES-CBC encrypt failed"))?;
                Ok(buf)
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
        let iv_array: [u8; 16] = iv.try_into().expect("checked above");
        let dc = aws_lc_rs::cipher::DecryptionContext::Iv128(FixedLength::from(iv_array));
        let alg = aes_alg(key_size);

        match padding {
            Padding::Pkcs7 => {
                let unbound = UnboundCipherKey::new(alg, key)
                    .map_err(|_| Error::Backend("aws_lc_rs UnboundCipherKey::new failed"))?;
                let dk = PaddedBlockDecryptingKey::cbc_pkcs7(unbound).map_err(|_| {
                    Error::Backend("aws_lc_rs PaddedBlockDecryptingKey::cbc_pkcs7 failed")
                })?;
                let mut buf = data.to_vec();
                let pt = dk
                    .decrypt(&mut buf, dc)
                    .map_err(|_| Error::Backend("aws_lc_rs AES-CBC PKCS#7 decrypt failed"))?;
                Ok(pt.to_vec())
            },
            Padding::None => {
                let unbound = UnboundCipherKey::new(alg, key)
                    .map_err(|_| Error::Backend("aws_lc_rs UnboundCipherKey::new failed"))?;
                let dk = DecryptingKey::cbc(unbound)
                    .map_err(|_| Error::Backend("aws_lc_rs DecryptingKey::cbc failed"))?;
                let mut buf = data.to_vec();
                let pt = dk
                    .decrypt(&mut buf, dc)
                    .map_err(|_| Error::Backend("aws_lc_rs AES-CBC decrypt failed"))?;
                Ok(pt.to_vec())
            },
        }
    }

    fn rc4(&self, _key: &[u8], _data: &[u8]) -> Result<Vec<u8>> {
        Err(not_permitted(
            AlgorithmKind::SymmetricCipher,
            "RC4",
            "FIPS 140-3 forbids RC4 — re-encrypt PDF Standard Security R≤4 \
             documents with R=6 (AES-256) under a non-FIPS provider before opening",
        ))
    }
}

// ---------------------------------------------------------------------------
// SignatureVerifier
// ---------------------------------------------------------------------------

struct AwsLcVerifier;

impl SignatureVerifier for AwsLcVerifier {
    fn verify_rsa_pkcs1v15(
        &self,
        pubkey: &RsaPublicKey<'_>,
        hash: HashAlgorithm,
        message: &[u8],
        signature: &[u8],
    ) -> Result<()> {
        // FIPS 140-3: SHA-1 is allowed for *verification* of historical
        // signatures (SP 800-131A) but not for new generation. Use the
        // dedicated legacy algorithm constant; aws-lc-rs still validates
        // via its FIPS module.
        let scheme: &dyn signature::VerificationAlgorithm = match hash {
            HashAlgorithm::Sha1 => &RSA_PKCS1_2048_8192_SHA1_FOR_LEGACY_USE_ONLY,
            HashAlgorithm::Sha256 => &RSA_PKCS1_2048_8192_SHA256,
            HashAlgorithm::Sha384 => &RSA_PKCS1_2048_8192_SHA384,
            HashAlgorithm::Sha512 => &RSA_PKCS1_2048_8192_SHA512,
            HashAlgorithm::Md5 => {
                return Err(not_permitted(
                    AlgorithmKind::SignatureVerify,
                    "RSA-PKCS#1-v1.5+MD5",
                    "FIPS 140-3 disallows MD5 for signature verification",
                ));
            },
        };
        // aws-lc-rs takes the RSA public key as DER-encoded SubjectPublicKeyInfo.
        let spki = encode_rsa_public_key_der(pubkey)?;
        let key = UnparsedPublicKey::new(scheme, &spki);
        key.verify(message, signature)
            .map_err(|_| Error::Verification("RSA-PKCS#1-v1.5 signature did not verify"))
    }

    fn verify_rsa_pss(
        &self,
        pubkey: &RsaPublicKey<'_>,
        hash: HashAlgorithm,
        message: &[u8],
        signature: &[u8],
    ) -> Result<()> {
        let scheme: &dyn signature::VerificationAlgorithm = match hash {
            HashAlgorithm::Sha256 => &RSA_PSS_2048_8192_SHA256,
            HashAlgorithm::Sha384 => &RSA_PSS_2048_8192_SHA384,
            HashAlgorithm::Sha512 => &RSA_PSS_2048_8192_SHA512,
            HashAlgorithm::Sha1 | HashAlgorithm::Md5 => {
                return Err(not_permitted(
                    AlgorithmKind::SignatureVerify,
                    "RSA-PSS-SHA1/MD5",
                    "FIPS 140-3 disallows PSS with SHA-1 or MD5",
                ));
            },
        };
        let spki = encode_rsa_public_key_der(pubkey)?;
        let key = UnparsedPublicKey::new(scheme, &spki);
        key.verify(message, signature)
            .map_err(|_| Error::Verification("RSA-PSS signature did not verify"))
    }

    fn verify_ecdsa(
        &self,
        curve: EcCurve,
        pubkey_sec1: &[u8],
        message: &[u8],
        signature_der: &[u8],
    ) -> Result<()> {
        let scheme: &dyn signature::VerificationAlgorithm = match curve {
            EcCurve::P256 => &ECDSA_P256_SHA256_ASN1,
            EcCurve::P384 => &ECDSA_P384_SHA384_ASN1,
        };
        let key = UnparsedPublicKey::new(scheme, pubkey_sec1);
        key.verify(message, signature_der)
            .map_err(|_| Error::Verification("ECDSA signature did not verify"))
    }
}

/// Encode a raw `(modulus_be, exponent_be)` RSA public key as DER
/// `SubjectPublicKeyInfo` so `aws_lc_rs::signature::UnparsedPublicKey`
/// can consume it. Hand-rolled minimal ASN.1 — `aws-lc-rs` itself
/// doesn't ship a key-encoder, and adding `der`/`spki` here would
/// pull in the full RustCrypto stack (defeating the FIPS isolation
/// the provider is supposed to give).
fn encode_rsa_public_key_der(pubkey: &RsaPublicKey<'_>) -> Result<Vec<u8>> {
    // RSAPublicKey ::= SEQUENCE { modulus INTEGER, publicExponent INTEGER }
    let mut rsapubkey = Vec::new();
    der_seq(&mut rsapubkey, |inner| {
        der_unsigned_int(inner, pubkey.modulus_be);
        der_unsigned_int(inner, pubkey.exponent_be);
    });

    // SubjectPublicKeyInfo ::= SEQUENCE {
    //   algorithm AlgorithmIdentifier,
    //   subjectPublicKey BIT STRING }
    // AlgorithmIdentifier for rsaEncryption = SEQUENCE {
    //   algorithm OID 1.2.840.113549.1.1.1,
    //   parameters NULL }
    const RSA_ENCRYPTION_OID_DER: &[u8] = &[
        0x06, 0x09, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x01,
    ];
    const NULL_DER: &[u8] = &[0x05, 0x00];

    let mut algo_id = Vec::new();
    algo_id.extend_from_slice(RSA_ENCRYPTION_OID_DER);
    algo_id.extend_from_slice(NULL_DER);

    let mut subject_pk_bitstring = Vec::with_capacity(1 + rsapubkey.len());
    subject_pk_bitstring.push(0x00); // unused-bits prefix
    subject_pk_bitstring.extend_from_slice(&rsapubkey);

    let mut spki = Vec::new();
    der_seq(&mut spki, |inner| {
        der_seq(inner, |a| a.extend_from_slice(&algo_id));
        der_tag_value(inner, 0x03, &subject_pk_bitstring); // BIT STRING
    });
    Ok(spki)
}

fn der_seq<F: FnOnce(&mut Vec<u8>)>(out: &mut Vec<u8>, content: F) {
    let mut inner = Vec::new();
    content(&mut inner);
    der_tag_value(out, 0x30, &inner);
}

fn der_unsigned_int(out: &mut Vec<u8>, be_bytes: &[u8]) {
    // Strip leading zeros, then prepend a 0x00 if MSB is set so DER
    // sees an unsigned integer.
    let mut view = be_bytes;
    while view.len() > 1 && view[0] == 0 {
        view = &view[1..];
    }
    let needs_leading_zero = !view.is_empty() && view[0] & 0x80 != 0;
    let total_len = view.len() + if needs_leading_zero { 1 } else { 0 };
    out.push(0x02);
    der_length(out, total_len);
    if needs_leading_zero {
        out.push(0x00);
    }
    out.extend_from_slice(view);
}

fn der_tag_value(out: &mut Vec<u8>, tag: u8, value: &[u8]) {
    out.push(tag);
    der_length(out, value.len());
    out.extend_from_slice(value);
}

fn der_length(out: &mut Vec<u8>, len: usize) {
    if len < 0x80 {
        out.push(len as u8);
    } else {
        // Long-form length. Up to 4 bytes covers all PDF cert sizes.
        let mut bytes = Vec::with_capacity(4);
        let mut n = len;
        while n > 0 {
            bytes.insert(0, (n & 0xff) as u8);
            n >>= 8;
        }
        out.push(0x80 | bytes.len() as u8);
        out.extend_from_slice(&bytes);
    }
}

// ---------------------------------------------------------------------------
// Tests — focused on policy + the implemented algorithm subset.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    fn provider() -> AwsLcProvider {
        AwsLcProvider::new()
    }

    #[test]
    fn name_and_legacy_policy() {
        let p = provider();
        assert_eq!(p.name(), "aws-lc-rs");
        assert!(!p.is_legacy_allowed());
    }

    #[test]
    fn md5_rejected() {
        let p = provider();
        let result = p.hasher(HashAlgorithm::Md5);
        assert!(matches!(result, Err(Error::AlgorithmNotPermitted { .. })));
    }

    #[test]
    fn rc4_rejected() {
        let p = provider();
        let result = p.symmetric().rc4(b"key", b"data");
        assert!(matches!(result, Err(Error::AlgorithmNotPermitted { .. })));
    }

    #[test]
    fn pss_sha1_rejected() {
        let p = provider();
        let pubkey = RsaPublicKey {
            modulus_be: &[0u8; 256],
            exponent_be: &[0x01, 0x00, 0x01],
        };
        let result =
            p.verifier()
                .verify_rsa_pss(&pubkey, HashAlgorithm::Sha1, b"message", &[0u8; 256]);
        assert!(matches!(result, Err(Error::AlgorithmNotPermitted { .. })));
    }

    // --- Hash vectors against NIST FIPS 180-4 ---

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

    // --- AES round-trip ---

    #[test]
    fn aes128_cbc_pkcs7_round_trip() {
        let p = provider();
        let key = [0x42u8; 16];
        let iv = [0x13u8; 16];
        let plaintext = b"PDF Standard Security V=4 stream content.";
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
        let iv = [0u8; 16];
        let plaintext = [0xa5u8; 32];
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
    fn random_bytes_non_constant() {
        let p = provider();
        let mut a = [0u8; 32];
        let mut b = [0u8; 32];
        p.random_bytes(&mut a).unwrap();
        p.random_bytes(&mut b).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn cross_provider_aes_compat() {
        // AES-128 is deterministic given key+IV+plaintext+padding —
        // so the FIPS provider must produce byte-identical ciphertext
        // to the default RustCryptoProvider. This catches API
        // mismatches (wrong padding mode, wrong IV interpretation)
        // immediately.
        use super::super::RustCryptoProvider;

        let key = [0x42u8; 16];
        let iv = [0x13u8; 16];
        let plaintext = b"FIPS-vs-rust-crypto byte-equality check.";

        let aws_lc = AwsLcProvider::new();
        let rust = RustCryptoProvider::new();

        let aws_ct = aws_lc
            .symmetric()
            .aes_cbc_encrypt(AesKeySize::Aes128, &key, &iv, plaintext, Padding::Pkcs7)
            .unwrap();
        let rust_ct = rust
            .symmetric()
            .aes_cbc_encrypt(AesKeySize::Aes128, &key, &iv, plaintext, Padding::Pkcs7)
            .unwrap();

        assert_eq!(aws_ct, rust_ct, "FIPS and rust-crypto AES-CBC must match");
    }
}
