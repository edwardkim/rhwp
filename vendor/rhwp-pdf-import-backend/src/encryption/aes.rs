//! AES encryption/decryption for PDF.
//!
//! AES (Advanced Encryption Standard) is used in PDF 1.6+ for stronger encryption.
//! PDFs use AES in CBC (Cipher Block Chaining) mode with PKCS#7 padding.
//!
//! Supported algorithms:
//! - AES-128: 16-byte key (PDF 1.6+, V=4, R=4)
//! - AES-256: 32-byte key (PDF 2.0, V=5, R=5/6)
//!
//! PDF Spec: Section 7.6.2 - General Encryption Algorithm
//!
//! All functions in this module delegate to
//! [`crate::crypto::active`]'s [`SymmetricCipher`] implementation
//! so the FIPS-validated `AwsLcProvider` (Phase 6) can swap in for
//! the default `RustCryptoProvider` without touching any caller.
//! Issue #236.
//!
//! [`SymmetricCipher`]: crate::crypto::SymmetricCipher

use crate::crypto::{active, AesKeySize, Padding};

fn map_err(e: crate::crypto::Error) -> &'static str {
    // Preserve the actionable distinction between provider variants
    // even though the existing public API surface only carries
    // `&'static str`. The richer variants in `crypto::Error` already
    // hold `&'static str` payloads, so we forward those without
    // allocating; only the structured `AlgorithmNotPermitted` and
    // unknown-future variants get folded to a generic message.
    match e {
        crate::crypto::Error::InvalidInput(s) => s,
        crate::crypto::Error::Verification(s) => s,
        crate::crypto::Error::Backend(s) => s,
        crate::crypto::Error::AlgorithmNotPermitted { .. } => {
            "AES algorithm rejected by active CryptoProvider's policy"
        },
    }
}

/// Encrypt data using AES-128 in CBC mode with PKCS#7 padding.
///
/// # Arguments
///
/// * `key` - The 16-byte encryption key
/// * `iv` - The 16-byte initialization vector
/// * `data` - The data to encrypt
///
/// # Returns
///
/// The encrypted data with PKCS#7 padding, or an error if encryption fails
#[allow(dead_code)]
pub fn aes128_encrypt(key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>, &'static str> {
    active()
        .symmetric()
        .aes_cbc_encrypt(AesKeySize::Aes128, key, iv, data, Padding::Pkcs7)
        .map_err(map_err)
}

/// Encrypt data using AES-128 in CBC mode WITHOUT padding.
///
/// Used by Algorithm 2.B (R=6) which handles its own data alignment.
/// Data length must be a multiple of 16.
pub fn aes128_encrypt_no_padding(
    key: &[u8],
    iv: &[u8],
    data: &[u8],
) -> Result<Vec<u8>, &'static str> {
    if data.is_empty() {
        return Ok(Vec::new());
    }
    active()
        .symmetric()
        .aes_cbc_encrypt(AesKeySize::Aes128, key, iv, data, Padding::None)
        .map_err(map_err)
}

/// Decrypt data using AES-256 in CBC mode WITHOUT padding.
///
/// Used for R=6 file encryption key unwrapping (UE/OE decryption).
/// Data length must be a multiple of 16.
pub fn aes256_decrypt_no_padding(
    key: &[u8],
    iv: &[u8],
    data: &[u8],
) -> Result<Vec<u8>, &'static str> {
    if data.is_empty() {
        return Ok(Vec::new());
    }
    active()
        .symmetric()
        .aes_cbc_decrypt(AesKeySize::Aes256, key, iv, data, Padding::None)
        .map_err(map_err)
}

/// Encrypt data using AES-256 in CBC mode without padding.
///
/// Used for R>=5 file encryption key wrapping (UE/OE encryption).
/// Data length must be a multiple of 16.
pub fn aes256_encrypt_no_padding(
    key: &[u8],
    iv: &[u8],
    data: &[u8],
) -> Result<Vec<u8>, &'static str> {
    if data.is_empty() {
        return Ok(Vec::new());
    }
    active()
        .symmetric()
        .aes_cbc_encrypt(AesKeySize::Aes256, key, iv, data, Padding::None)
        .map_err(map_err)
}

/// Decrypt data using AES-128 in CBC mode and remove PKCS#7 padding.
///
/// # Arguments
///
/// * `key` - The 16-byte encryption key
/// * `iv` - The 16-byte initialization vector
/// * `data` - The encrypted data
///
/// # Returns
///
/// The decrypted data with padding removed, or an error if decryption fails
pub fn aes128_decrypt(key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>, &'static str> {
    if data.is_empty() {
        return Ok(Vec::new());
    }
    active()
        .symmetric()
        .aes_cbc_decrypt(AesKeySize::Aes128, key, iv, data, Padding::Pkcs7)
        .map_err(map_err)
}

/// Encrypt data using AES-256 in CBC mode with PKCS#7 padding.
pub fn aes256_encrypt(key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>, &'static str> {
    active()
        .symmetric()
        .aes_cbc_encrypt(AesKeySize::Aes256, key, iv, data, Padding::Pkcs7)
        .map_err(map_err)
}

/// Decrypt data using AES-256 in CBC mode and remove PKCS#7 padding.
pub fn aes256_decrypt(key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>, &'static str> {
    if data.is_empty() {
        return Ok(Vec::new());
    }
    active()
        .symmetric()
        .aes_cbc_decrypt(AesKeySize::Aes256, key, iv, data, Padding::Pkcs7)
        .map_err(map_err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aes128_encrypt_decrypt_round_trip() {
        let key = b"PDF-AES-128-key!"; // 16 bytes
        let iv = b"AES-IV-16-bytes!"; // 16 bytes
        let plaintext = b"PDF Standard Security Handler V=4 stream content.";
        let ciphertext = aes128_encrypt(key, iv, plaintext).unwrap();
        let decrypted = aes128_decrypt(key, iv, &ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn aes128_encrypt_block_aligned() {
        // 16-byte aligned plaintext — PKCS#7 must still pad with a
        // full block of 0x10.
        let key = [0x42u8; 16];
        let iv = [0x13u8; 16];
        let plaintext = [0x0au8; 16];
        let ciphertext = aes128_encrypt(&key, &iv, &plaintext).unwrap();
        assert_eq!(ciphertext.len(), 32, "PKCS#7 must add a full padding block");
        let decrypted = aes128_decrypt(&key, &iv, &ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn aes256_no_padding_round_trip() {
        let key = [0x07u8; 32];
        let iv = [0u8; 16]; // V=5 key wrap uses zero IV
        let plaintext = [0xa5u8; 32];
        let ciphertext = aes256_encrypt_no_padding(&key, &iv, &plaintext).unwrap();
        let decrypted = aes256_decrypt_no_padding(&key, &iv, &ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn aes_rejects_short_key() {
        let result = aes128_encrypt(&[0u8; 8], &[0u8; 16], b"data1234data1234");
        assert!(result.is_err());
    }
}
