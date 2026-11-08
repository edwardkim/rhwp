//! PDF encryption support.
//!
//! This module implements PDF encryption/decryption according to the PDF specification
//! (ISO 32000-1:2008, Section 7.6). It supports:
//!
//! - RC4 encryption (40-bit and 128-bit) for PDF 1.4-1.5
//! - AES encryption (128-bit and 256-bit) for PDF 1.6+
//! - Standard Security Handler (password validation, permissions)
//!
//! # Encryption Algorithms
//!
//! ## RC4 (PDF 1.4-1.5)
//! - RC4-40: 40-bit key length (weak, legacy)
//! - RC4-128: 128-bit key length
//!
//! ## AES (PDF 1.6+)
//! - AES-128: 128-bit key length with CBC mode
//! - AES-256: 256-bit key length with CBC mode (PDF 2.0)
//!
//! # Security Considerations
//!
//! - RC4-40 is cryptographically weak and should only be used for legacy documents
//! - Password validation uses constant-time comparison to prevent timing attacks
//! - Key derivation follows PDF specification algorithms (using MD5 or SHA-256)
//!
//! # References
//!
//! - PDF Spec Section 7.6: Encryption
//! - PDF Spec Section 7.6.3: Standard Security Handler
//! - PDF Spec Section 7.6.5: Algorithm 2 (Key Derivation)

use crate::error::{Error, Result};
use crate::object::Object;

mod aes;
mod algorithms;
mod certificate;
mod handler;
pub mod permissions;
// `pub(crate)` so the `crypto::RustCryptoProvider::SymmetricCipher`
// impl in `src/crypto/rust_provider.rs` can call
// `rc4::rc4_crypt_impl` (the cipher-only entry point that does NOT
// re-route through the active provider, breaking the
// `provider.rc4() → rc4_crypt → provider.rc4()` cycle). RC4 is
// required by PDF Standard Security Handler R≤4 (ISO 32000-1 §7.6.3).
pub(crate) mod rc4;
mod write_handler;

pub use algorithms::{
    compute_encryption_key, compute_owner_password_hash, compute_user_password_hash,
};
pub use certificate::{
    CertEncryptDict, CertSubFilter, CertificateEncryption, CertificateEncryptionHandler,
    KeyTransportAlgorithm, RecipientInfo, RecipientPermissions,
};
pub use handler::EncryptionHandler;
pub use permissions::PdfPermissions;
pub use write_handler::EncryptionWriteHandler;

/// A fresh MD5 hasher for the ISO 32000-1 §7.6.3 Standard-Security-
/// Handler password key-derivation, obtained through the active
/// [`crate::crypto`] provider (#230 Phase C).
///
/// Routing the *primitive* through the provider — not just the
/// operation gate in [`handler::EncryptionHandler::new`] /
/// [`write_handler`] — lets a `strict`/`fips-strict`
/// `SecurityPolicy` (or a FIPS provider) deny weak-crypto key
/// derivation at the boundary it actually happens, closing the
/// "operation-gated but primitive-ungated" gap (feature-230 §3.5 risk
/// table). Under the default `compat` policy the returned hasher is
/// the same `md5::Md5`, so every existing encrypted PDF still decrypts
/// and newly written ones are **byte-for-byte unchanged**.
///
/// The *non-security* opaque MD5 uses — the file identifier
/// ([`generate_file_id`]) and the embedded-file `/CheckSum`
/// (`writer::embedded_files`) — are deliberately **not** routed here:
/// they are not security decisions, and `AlgorithmUse` has no
/// non-security variant, so gating them would make a strict policy
/// refuse to write otherwise-compliant AES-256 documents. They remain
/// direct `md5` calls (feature-230 §2.2).
///
/// # Errors
/// Maps the provider/policy denial
/// ([`crate::crypto::Error::AlgorithmNotPermitted`]) to a clear
/// [`Error::InvalidPdf`], consistent with the existing R≤4 / legacy
/// rejection messages in this module.
#[cfg(feature = "legacy-crypto")]
pub(crate) fn md5_kdf_hasher() -> Result<Box<dyn crate::crypto::Hasher>> {
    crate::crypto::active()
        .hasher(crate::crypto::HashAlgorithm::Md5)
        .map_err(|e| {
            Error::InvalidPdf(format!(
                "MD5 key derivation is not permitted by the active crypto \
                 policy/provider (ISO 32000-1 §7.6.3 Standard Security \
                 Handler, R≤4): {e}"
            ))
        })
}

/// Encryption algorithm used in the PDF.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    /// No encryption
    None,
    /// RC4 with 40-bit key (PDF 1.4, V=1, R=2)
    RC4_40,
    /// RC4 with 128-bit key (PDF 1.5, V=2, R=3)
    Rc4_128,
    /// AES with 128-bit key in CBC mode (PDF 1.6, V=4, R=4)
    Aes128,
    /// AES with 256-bit key in CBC mode (PDF 2.0, V=5, R=5/6)
    Aes256,
}

impl Algorithm {
    /// Get the key length in bytes for this algorithm.
    pub fn key_length(&self) -> usize {
        match self {
            Algorithm::None => 0,
            Algorithm::RC4_40 => 5,   // 40 bits = 5 bytes
            Algorithm::Rc4_128 => 16, // 128 bits = 16 bytes
            Algorithm::Aes128 => 16,  // 128 bits = 16 bytes
            Algorithm::Aes256 => 32,  // 256 bits = 32 bytes
        }
    }

    /// Check if this is an AES algorithm.
    pub fn is_aes(&self) -> bool {
        matches!(self, Algorithm::Aes128 | Algorithm::Aes256)
    }

    /// Check if this is an RC4 algorithm.
    pub fn is_rc4(&self) -> bool {
        matches!(self, Algorithm::RC4_40 | Algorithm::Rc4_128)
    }
}

/// PDF encryption dictionary (/Encrypt entry in trailer).
///
/// PDF Spec: Section 7.6.1 - General
#[derive(Debug, Clone)]
pub struct EncryptDict {
    /// Filter name (should be "Standard")
    pub filter: String,
    /// SubFilter name (optional, for public-key security)
    pub sub_filter: Option<String>,
    /// Algorithm version (V): 1=RC4-40, 2=RC4-128, 4=AES-128, 5=AES-256
    pub version: u32,
    /// Key length in bits (Length): 40-128 for RC4, 128/256 for AES
    pub length: Option<u32>,
    /// Revision number (R): 2, 3, 4, 5, or 6
    pub revision: u32,
    /// Owner password hash (O): 32 or 48 bytes
    pub owner_password: Vec<u8>,
    /// User password hash (U): 32 or 48 bytes
    pub user_password: Vec<u8>,
    /// User permissions (P): 32-bit integer
    pub permissions: i32,
    /// Encrypt metadata flag (EncryptMetadata): true by default
    pub encrypt_metadata: bool,
    /// Additional encryption parameters (for V=5/R=6)
    pub owner_encryption: Option<Vec<u8>>, // OE
    /// User encryption key (UE entry, for V=5/R=6)
    pub user_encryption: Option<Vec<u8>>, // UE
    /// Encrypted permissions (Perms entry, for V=5/R=6)
    pub perms: Option<Vec<u8>>, // Perms (encrypted permissions)
    /// Stream crypt filter method (CFM from /CF dictionary, for V=4).
    /// "V2" = RC4-128, "AESV2" = AES-128. None means not specified (defaults to AES-128).
    pub stream_crypt_method: Option<String>,
}

impl EncryptDict {
    /// Parse an encryption dictionary from a PDF object.
    ///
    /// PDF Spec: Section 7.6.1 - General
    pub fn from_object(obj: &Object) -> Result<Self> {
        let dict = obj
            .as_dict()
            .ok_or_else(|| Error::InvalidPdf("Encrypt entry is not a dictionary".to_string()))?;

        // Extract required fields
        let filter = dict
            .get("Filter")
            .and_then(|o| o.as_name())
            .ok_or_else(|| Error::InvalidPdf("Encrypt dictionary missing /Filter".to_string()))?
            .to_string();

        let version = dict
            .get("V")
            .and_then(|o| match o {
                Object::Integer(i) => Some(*i as u32),
                _ => None,
            })
            .ok_or_else(|| Error::InvalidPdf("Encrypt dictionary missing /V".to_string()))?;

        let revision = dict
            .get("R")
            .and_then(|o| match o {
                Object::Integer(i) => Some(*i as u32),
                _ => None,
            })
            .ok_or_else(|| Error::InvalidPdf("Encrypt dictionary missing /R".to_string()))?;

        let owner_password = dict
            .get("O")
            .and_then(|o| o.as_string())
            .ok_or_else(|| Error::InvalidPdf("Encrypt dictionary missing /O".to_string()))?
            .to_vec();

        let user_password = dict
            .get("U")
            .and_then(|o| o.as_string())
            .ok_or_else(|| Error::InvalidPdf("Encrypt dictionary missing /U".to_string()))?
            .to_vec();

        let permissions = dict
            .get("P")
            .and_then(|o| match o {
                Object::Integer(i) => Some(*i as i32),
                _ => None,
            })
            .ok_or_else(|| Error::InvalidPdf("Encrypt dictionary missing /P".to_string()))?;

        // Optional fields
        let sub_filter = dict
            .get("SubFilter")
            .and_then(|o| o.as_name())
            .map(|s| s.to_string());

        let length = dict.get("Length").and_then(|o| match o {
            Object::Integer(i) => Some(*i as u32),
            _ => None,
        });

        let encrypt_metadata = dict
            .get("EncryptMetadata")
            .and_then(|o| match o {
                Object::Boolean(b) => Some(*b),
                _ => None,
            })
            .unwrap_or(true);

        // V=5/R=6 additional fields
        let owner_encryption = dict
            .get("OE")
            .and_then(|o| o.as_string())
            .map(|s| s.to_vec());

        let user_encryption = dict
            .get("UE")
            .and_then(|o| o.as_string())
            .map(|s| s.to_vec());

        let perms = dict
            .get("Perms")
            .and_then(|o| o.as_string())
            .map(|s| s.to_vec());

        // V=4: Parse /CF crypt filter dictionary to determine the actual algorithm.
        // /StmF names the crypt filter for streams; look up its /CFM entry.
        // CFM "V2" = RC4-128, "AESV2" = AES-128 (PDF Spec Table 25).
        let stream_crypt_method = if version == 4 {
            let stm_filter_name = dict
                .get("StmF")
                .and_then(|o| o.as_name())
                .unwrap_or("Identity");
            dict.get("CF")
                .and_then(|cf| cf.as_dict())
                .and_then(|cf_dict| cf_dict.get(stm_filter_name))
                .and_then(|filter_obj| filter_obj.as_dict())
                .and_then(|filter_dict| filter_dict.get("CFM"))
                .and_then(|cfm| cfm.as_name())
                .map(|s| s.to_string())
        } else {
            None
        };

        Ok(EncryptDict {
            filter,
            sub_filter,
            version,
            length,
            revision,
            owner_password,
            user_password,
            permissions,
            encrypt_metadata,
            owner_encryption,
            user_encryption,
            perms,
            stream_crypt_method,
        })
    }

    /// Determine the encryption algorithm from V and R values.
    ///
    /// PDF Spec: Table 20 - Encryption dictionary entries
    pub fn algorithm(&self) -> Result<Algorithm> {
        match (self.version, self.revision) {
            (1, 2) => Ok(Algorithm::RC4_40),
            (2, 3) => Ok(Algorithm::Rc4_128),
            (4, _) => {
                // V=4 means crypt-filter-based encryption. The actual algorithm
                // is determined by /CFM in the /CF dictionary (PDF Spec Table 25):
                //   "V2" = RC4-128
                //   "AESV2" = AES-128
                match self.stream_crypt_method.as_deref() {
                    Some("V2") => {
                        log::info!("V=4 R={}: CFM=V2 → RC4-128", self.revision);
                        Ok(Algorithm::Rc4_128)
                    },
                    Some("AESV2") | None => {
                        // None: /CF missing or unparseable — default to AES-128
                        // for backward compatibility.
                        Ok(Algorithm::Aes128)
                    },
                    Some(other) => {
                        log::warn!(
                            "V=4 R={}: unknown CFM '{}', falling back to AES-128",
                            self.revision,
                            other
                        );
                        Ok(Algorithm::Aes128)
                    },
                }
            },
            (5, 5) | (5, 6) => Ok(Algorithm::Aes256),
            // Lenient: V determines algorithm, R may be non-standard
            (1, r) => {
                log::warn!("Non-standard encryption V=1, R={}, using RC4-40", r);
                Ok(Algorithm::RC4_40)
            },
            (2, r) => {
                log::warn!("Non-standard encryption V=2, R={}, using RC4-128", r);
                Ok(Algorithm::Rc4_128)
            },
            _ => Err(Error::Unsupported(format!(
                "Unsupported encryption version V={}, R={}",
                self.version, self.revision
            ))),
        }
    }

    /// Get the effective key length in bytes.
    pub fn key_length_bytes(&self) -> usize {
        if let Some(length) = self.length {
            (length / 8) as usize
        } else {
            // Default key lengths based on version
            match self.version {
                1 => 5,  // 40 bits
                2 => 16, // 128 bits
                4 => 16, // 128 bits (AES-128)
                5 => 32, // 256 bits (AES-256)
                _ => 16, // Default to 128 bits
            }
        }
    }

    /// Serialize the encryption dictionary to a PDF Object.
    ///
    /// This creates a dictionary object suitable for the /Encrypt entry in the trailer.
    pub fn to_object(&self) -> Object {
        use std::collections::HashMap;

        let mut dict: HashMap<String, Object> = HashMap::new();

        // Required entries
        dict.insert("Filter".to_string(), Object::Name(self.filter.clone()));
        dict.insert("V".to_string(), Object::Integer(self.version as i64));
        dict.insert("R".to_string(), Object::Integer(self.revision as i64));
        dict.insert("O".to_string(), Object::String(self.owner_password.clone()));
        dict.insert("U".to_string(), Object::String(self.user_password.clone()));
        dict.insert("P".to_string(), Object::Integer(self.permissions as i64));

        // Optional entries
        if let Some(ref sub_filter) = self.sub_filter {
            dict.insert("SubFilter".to_string(), Object::Name(sub_filter.clone()));
        }

        if let Some(length) = self.length {
            dict.insert("Length".to_string(), Object::Integer(length as i64));
        }

        if !self.encrypt_metadata {
            dict.insert("EncryptMetadata".to_string(), Object::Boolean(false));
        }

        // V=5/R=6 specific entries
        if let Some(ref oe) = self.owner_encryption {
            dict.insert("OE".to_string(), Object::String(oe.clone()));
        }

        if let Some(ref ue) = self.user_encryption {
            dict.insert("UE".to_string(), Object::String(ue.clone()));
        }

        if let Some(ref perms) = self.perms {
            dict.insert("Perms".to_string(), Object::String(perms.clone()));
        }

        // For V=4 (crypt-filter-based), add crypt filter entries
        if self.version == 4 {
            let cfm = self
                .stream_crypt_method
                .as_deref()
                .unwrap_or("AESV2")
                .to_string();
            let mut cf_dict: HashMap<String, Object> = HashMap::new();
            let mut std_cf: HashMap<String, Object> = HashMap::new();
            std_cf.insert("CFM".to_string(), Object::Name(cfm));
            std_cf.insert("AuthEvent".to_string(), Object::Name("DocOpen".to_string()));
            std_cf.insert("Length".to_string(), Object::Integer(16));
            cf_dict.insert("StdCF".to_string(), Object::Dictionary(std_cf));
            dict.insert("CF".to_string(), Object::Dictionary(cf_dict));
            dict.insert("StmF".to_string(), Object::Name("StdCF".to_string()));
            dict.insert("StrF".to_string(), Object::Name("StdCF".to_string()));
        }

        // For V=5 (AES-256), add crypt filter entries
        if self.version == 5 {
            let mut cf_dict: HashMap<String, Object> = HashMap::new();
            let mut std_cf: HashMap<String, Object> = HashMap::new();
            std_cf.insert("CFM".to_string(), Object::Name("AESV3".to_string()));
            std_cf.insert("AuthEvent".to_string(), Object::Name("DocOpen".to_string()));
            std_cf.insert("Length".to_string(), Object::Integer(32));
            cf_dict.insert("StdCF".to_string(), Object::Dictionary(std_cf));
            dict.insert("CF".to_string(), Object::Dictionary(cf_dict));
            dict.insert("StmF".to_string(), Object::Name("StdCF".to_string()));
            dict.insert("StrF".to_string(), Object::Name("StdCF".to_string()));
        }

        Object::Dictionary(dict)
    }
}

/// Builder for creating encryption dictionaries.
///
/// This provides a convenient way to create properly configured encryption
/// for writing encrypted PDFs.
pub struct EncryptDictBuilder {
    algorithm: Algorithm,
    user_password: Vec<u8>,
    owner_password: Vec<u8>,
    permissions: i32,
    encrypt_metadata: bool,
}

impl EncryptDictBuilder {
    /// Create a new builder with the specified algorithm.
    pub fn new(algorithm: Algorithm) -> Self {
        Self {
            algorithm,
            user_password: Vec::new(),
            owner_password: Vec::new(),
            permissions: -1, // All permissions granted by default
            encrypt_metadata: true,
        }
    }

    /// Set the user password (required for opening the document).
    pub fn user_password(mut self, password: &[u8]) -> Self {
        self.user_password = password.to_vec();
        self
    }

    /// Set the owner password (required for full access).
    pub fn owner_password(mut self, password: &[u8]) -> Self {
        self.owner_password = password.to_vec();
        self
    }

    /// Set user permissions (P value).
    pub fn permissions(mut self, permissions: i32) -> Self {
        self.permissions = permissions;
        self
    }

    /// Set whether to encrypt metadata.
    pub fn encrypt_metadata(mut self, encrypt: bool) -> Self {
        self.encrypt_metadata = encrypt;
        self
    }

    /// Build the encryption dictionary.
    ///
    /// This computes all required hashes and returns the complete dictionary.
    ///
    /// For AES-256 (R6) the *file encryption key* is generated here and wrapped
    /// into `/UE` and `/OE`; callers that will go on to encrypt streams must use
    /// [`Self::build_with_key`] and encrypt with the key it returns. Using this
    /// method and then deriving a key separately produces a file whose `/UE`
    /// wraps one key while its streams are encrypted under another — see
    /// `build_with_key`.
    ///
    /// # Arguments
    /// * `file_id` - The first element of the PDF file identifier array
    pub fn build(self, file_id: &[u8]) -> Result<EncryptDict> {
        self.build_with_key(file_id).map(|(dict, _)| dict)
    }

    /// Build the encryption dictionary **and return the file encryption key**
    /// that its `/UE` and `/OE` entries wrap, when the revision has one.
    ///
    /// ISO 32000-2 Algorithm 8 generates a single random file encryption key,
    /// wraps it into `/UE` under a key derived from the user password, and
    /// requires every string and stream to be encrypted with that same key. The
    /// key therefore has to leave this function: `build` computed it, used it
    /// for `/UE` and `/OE`, and dropped it, so the writer derived a *second*
    /// random key for the streams. The result authenticated — the `/U` hash is
    /// computed from the password correctly — and then decrypted every stream
    /// to noise, in this library and in any other.
    ///
    /// Returns `None` for R≤4, where the key is derived from the password and
    /// the file id rather than generated, so the writer can recompute it.
    pub fn build_with_key(self, file_id: &[u8]) -> Result<(EncryptDict, Option<Vec<u8>>)> {
        let (version, revision) = match self.algorithm {
            Algorithm::None => (0, 0),
            Algorithm::RC4_40 => (1, 2),
            Algorithm::Rc4_128 => (2, 3),
            Algorithm::Aes128 => (4, 4),
            Algorithm::Aes256 => (5, 6),
        };

        // FIPS gate: the FIPS-validated `AwsLcProvider`
        // refuses MD5 / RC4 entirely, so writing an R≤4 dict under it
        // would produce ciphertext that the same provider can't read
        // back. Reject up front with a clear error rather than letting
        // the deeper RC4 path return AlgorithmNotPermitted.
        if revision > 0 && revision <= 4 && !crate::crypto::active().is_legacy_allowed() {
            return Err(crate::Error::InvalidPdf(format!(
                "active CryptoProvider '{}' rejects PDF Standard Security R={} \
                 (R≤4 requires MD5 + RC4; FIPS 140-3 forbids both). \
                 Use Algorithm::Aes256 (R=6) or build pdf_oxide \
                 without the 'fips' feature.",
                crate::crypto::active().name(),
                revision
            )));
        }

        // Runtime crypto-governance policy (#230). No-op under the
        // default `compat` policy (byte-stable); `strict`/`fips-strict`
        // forbid *writing* legacy R≤4 (it fundamentally needs the MD5
        // KDF, ISO 32000-1 §7.6.3) while still allowing legacy reads.
        if revision > 0 && revision <= 4 {
            crate::crypto::record_algorithm_use(crate::crypto::AlgorithmId::HashMd5);
        }
        if revision > 0
            && revision <= 4
            && !crate::crypto::active_policy()
                .allows(crate::crypto::AlgorithmId::HashMd5, crate::crypto::AlgorithmUse::Write)
        {
            return Err(crate::Error::InvalidPdf(format!(
                "active crypto SecurityPolicy (mode={}) forbids writing PDF \
                 Standard Security R={} (R≤4 requires MD5; denied for write). \
                 Use Algorithm::Aes256 (R=6) or set a 'compat' crypto policy.",
                crate::crypto::active_policy().mode().token(),
                revision
            )));
        }

        let key_length = self.algorithm.key_length();

        // Use owner password if provided, otherwise use user password
        let owner_pass = if self.owner_password.is_empty() {
            self.user_password.clone()
        } else {
            self.owner_password.clone()
        };

        if revision >= 5 {
            // AES-256 (R6): file key is random; U/UE and O/OE are computed per
            // PDF 2.0 Algorithm 8 and 9 using the actual passwords.
            let (user_hash, user_encryption, file_key) =
                algorithms::compute_u_and_ue(&self.user_password, key_length, revision)?;
            let (owner_hash, owner_encryption) = algorithms::compute_o_and_oe(
                &owner_pass,
                &self.user_password,
                &file_key,
                &user_hash,
                revision,
            )?;
            return Ok((
                EncryptDict {
                    filter: "Standard".to_string(),
                    sub_filter: None,
                    version,
                    length: Some((key_length * 8) as u32),
                    revision,
                    owner_password: owner_hash,
                    user_password: user_hash,
                    permissions: self.permissions,
                    encrypt_metadata: self.encrypt_metadata,
                    owner_encryption: Some(owner_encryption),
                    user_encryption: Some(user_encryption),
                    perms: None,
                    stream_crypt_method: None,
                },
                // The key /UE and /OE wrap. Streams must use this one.
                Some(file_key),
            ));
        }

        // Compute owner password hash (O value)
        let owner_hash = algorithms::compute_owner_password_hash(
            &owner_pass,
            &self.user_password,
            revision,
            key_length,
        )?;

        // Compute encryption key from user password
        let encryption_key = algorithms::compute_encryption_key(
            &self.user_password,
            &owner_hash,
            self.permissions,
            file_id,
            revision,
            key_length,
            self.encrypt_metadata,
        )?;

        // Compute user password hash (U value)
        let user_hash = algorithms::compute_user_password_hash(&encryption_key, file_id, revision)?;

        Ok((
            EncryptDict {
                filter: "Standard".to_string(),
                sub_filter: None,
                version,
                length: Some((key_length * 8) as u32),
                revision,
                owner_password: owner_hash,
                user_password: user_hash,
                permissions: self.permissions,
                encrypt_metadata: self.encrypt_metadata,
                owner_encryption: None,
                user_encryption: None,
                perms: None,
                stream_crypt_method: None,
            },
            // R<=4 derives the key from the password, the owner hash, the
            // permissions and the file id, so the writer recomputes the same
            // value rather than being handed one.
            None,
        ))
    }
}

/// PDF encryption permissions (P field).
///
/// PDF Spec: Table 22 - User access permissions
#[derive(Debug, Clone, Copy)]
pub struct Permissions {
    bits: i32,
}

impl Permissions {
    /// Create permissions from the P field value.
    pub fn from_bits(bits: i32) -> Self {
        Self { bits }
    }

    /// Decoded view per [`PdfPermissions`]. The bit decoding lives
    /// in one place; this method-style API delegates so the two
    /// decoders cannot drift apart.
    #[inline]
    fn decoded(&self) -> PdfPermissions {
        PdfPermissions::from_p_flag(self.bits)
    }

    /// Check if printing is allowed.
    pub fn can_print(&self) -> bool {
        self.decoded().print_low_res
    }

    /// Check if modifying the document is allowed.
    pub fn can_modify(&self) -> bool {
        self.decoded().modify
    }

    /// Check if copying text/graphics is allowed.
    pub fn can_copy(&self) -> bool {
        self.decoded().copy
    }

    /// Check if adding/modifying annotations is allowed.
    pub fn can_annotate(&self) -> bool {
        self.decoded().annotate
    }

    /// Check if filling form fields is allowed (R>=3).
    pub fn can_fill_forms(&self) -> bool {
        self.decoded().fill_forms
    }

    /// Check if content extraction for accessibility is allowed (R>=3).
    pub fn can_extract_accessibility(&self) -> bool {
        self.decoded().accessibility
    }

    /// Check if assembling the document is allowed (R>=3).
    pub fn can_assemble(&self) -> bool {
        self.decoded().assemble
    }

    /// Check if high-quality printing is allowed (R>=3).
    pub fn can_print_high_quality(&self) -> bool {
        self.decoded().print_high_res
    }
}

/// Generate a unique file ID for the PDF.
///
/// PDF Spec: Section 14.4 - File Identifiers
///
/// The file identifier array contains two strings:
/// - First string: A permanent identifier based on file contents at creation
/// - Second string: A changing identifier updated each time the file is saved
///
/// This function generates both strings as the same value (for new PDFs).
/// For incremental updates, the first ID should be preserved.
///
/// # Returns
///
/// A tuple of (permanent_id, changing_id) as 16-byte vectors
pub fn generate_file_id() -> (Vec<u8>, Vec<u8>) {
    let uuid = uuid::Uuid::new_v4();
    let uuid_bytes = uuid.as_bytes();

    // SystemTime::now() is not available on wasm32-unknown-unknown.
    // The UUID v4 already provides sufficient entropy for a unique opaque
    // file identifier (ISO 32000-1 §14.4), so we skip the time contribution
    // on that target.
    #[cfg(not(target_arch = "wasm32"))]
    let now_bytes: Option<[u8; 16]> = {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        Some(now.as_nanos().to_le_bytes())
    };
    #[cfg(target_arch = "wasm32")]
    let now_bytes: Option<[u8; 16]> = None;

    // PDF spec (ISO 32000-1 §14.4) specifies MD5; when legacy-crypto is off
    // we use SHA-256 truncated to 16 bytes — still a unique opaque identifier.
    // #230 Phase C: intentionally a *direct* MD5 — the file identifier
    // is a non-security opaque tag, not a key-derivation. It is NOT
    // routed through `md5_kdf_hasher` (see that fn's docs): doing so
    // would let a strict policy refuse to write an otherwise-compliant
    // AES-256 document, and `AlgorithmUse` has no non-security variant.
    #[cfg(feature = "legacy-crypto")]
    let id = {
        use md5::{Digest, Md5};
        let mut h = Md5::new();
        h.update(uuid_bytes);
        if let Some(b) = now_bytes {
            h.update(b);
        }
        h.finalize().to_vec()
    };

    #[cfg(not(feature = "legacy-crypto"))]
    let id = {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(uuid_bytes);
        if let Some(b) = now_bytes {
            h.update(b);
        }
        h.finalize()[..16].to_vec()
    };

    // For new PDFs, both IDs are the same
    (id.clone(), id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::Object;
    use std::collections::HashMap;

    // === Algorithm tests ===

    #[test]
    fn test_algorithm_key_length_none() {
        assert_eq!(Algorithm::None.key_length(), 0);
    }

    #[test]
    fn test_algorithm_key_length_rc4_40() {
        assert_eq!(Algorithm::RC4_40.key_length(), 5);
    }

    #[test]
    fn test_algorithm_key_length_rc4_128() {
        assert_eq!(Algorithm::Rc4_128.key_length(), 16);
    }

    #[test]
    fn test_algorithm_key_length_aes128() {
        assert_eq!(Algorithm::Aes128.key_length(), 16);
    }

    #[test]
    fn test_algorithm_key_length_aes256() {
        assert_eq!(Algorithm::Aes256.key_length(), 32);
    }

    #[test]
    fn test_algorithm_is_aes() {
        assert!(!Algorithm::None.is_aes());
        assert!(!Algorithm::RC4_40.is_aes());
        assert!(!Algorithm::Rc4_128.is_aes());
        assert!(Algorithm::Aes128.is_aes());
        assert!(Algorithm::Aes256.is_aes());
    }

    #[test]
    fn test_algorithm_is_rc4() {
        assert!(!Algorithm::None.is_rc4());
        assert!(Algorithm::RC4_40.is_rc4());
        assert!(Algorithm::Rc4_128.is_rc4());
        assert!(!Algorithm::Aes128.is_rc4());
        assert!(!Algorithm::Aes256.is_rc4());
    }

    #[test]
    fn test_algorithm_none_not_aes_not_rc4() {
        assert!(!Algorithm::None.is_aes());
        assert!(!Algorithm::None.is_rc4());
    }

    #[test]
    fn test_algorithm_equality() {
        assert_eq!(Algorithm::RC4_40, Algorithm::RC4_40);
        assert_eq!(Algorithm::Aes256, Algorithm::Aes256);
        assert_ne!(Algorithm::RC4_40, Algorithm::Rc4_128);
        assert_ne!(Algorithm::Aes128, Algorithm::Aes256);
    }

    #[test]
    fn test_algorithm_clone_copy() {
        let a = Algorithm::Aes128;
        let b = a; // Copy
        let c = a;
        assert_eq!(a, b);
        assert_eq!(a, c);
    }

    // === Helper to build encryption dictionary Object ===

    fn make_encrypt_dict_obj(
        filter: &str,
        v: i64,
        r: i64,
        owner: &[u8],
        user: &[u8],
        p: i64,
    ) -> Object {
        let mut dict = HashMap::new();
        dict.insert("Filter".to_string(), Object::Name(filter.to_string()));
        dict.insert("V".to_string(), Object::Integer(v));
        dict.insert("R".to_string(), Object::Integer(r));
        dict.insert("O".to_string(), Object::String(owner.to_vec()));
        dict.insert("U".to_string(), Object::String(user.to_vec()));
        dict.insert("P".to_string(), Object::Integer(p));
        Object::Dictionary(dict)
    }

    // === EncryptDict::from_object tests ===

    #[test]
    fn test_encrypt_dict_from_object_rc4_40() {
        let obj = make_encrypt_dict_obj("Standard", 1, 2, &[0u8; 32], &[0u8; 32], -4);
        let ed = EncryptDict::from_object(&obj).unwrap();
        assert_eq!(ed.filter, "Standard");
        assert_eq!(ed.version, 1);
        assert_eq!(ed.revision, 2);
        assert_eq!(ed.permissions, -4);
        assert!(ed.encrypt_metadata); // default true
        assert!(ed.sub_filter.is_none());
        assert!(ed.length.is_none());
    }

    #[test]
    fn test_encrypt_dict_from_object_rc4_128() {
        let obj = make_encrypt_dict_obj("Standard", 2, 3, &[0u8; 32], &[0u8; 32], -1);
        let ed = EncryptDict::from_object(&obj).unwrap();
        assert_eq!(ed.version, 2);
        assert_eq!(ed.revision, 3);
    }

    #[test]
    fn test_encrypt_dict_from_object_aes128() {
        let obj = make_encrypt_dict_obj("Standard", 4, 4, &[0u8; 32], &[0u8; 32], -1);
        let ed = EncryptDict::from_object(&obj).unwrap();
        assert_eq!(ed.version, 4);
        assert_eq!(ed.revision, 4);
    }

    #[test]
    fn test_encrypt_dict_from_object_aes256() {
        let mut dict = HashMap::new();
        dict.insert("Filter".to_string(), Object::Name("Standard".to_string()));
        dict.insert("V".to_string(), Object::Integer(5));
        dict.insert("R".to_string(), Object::Integer(6));
        dict.insert("O".to_string(), Object::String(vec![0u8; 48]));
        dict.insert("U".to_string(), Object::String(vec![0u8; 48]));
        dict.insert("P".to_string(), Object::Integer(-1));
        dict.insert("OE".to_string(), Object::String(vec![0u8; 32]));
        dict.insert("UE".to_string(), Object::String(vec![0u8; 32]));
        dict.insert("Perms".to_string(), Object::String(vec![0u8; 16]));
        let obj = Object::Dictionary(dict);
        let ed = EncryptDict::from_object(&obj).unwrap();
        assert_eq!(ed.version, 5);
        assert_eq!(ed.revision, 6);
        assert!(ed.owner_encryption.is_some());
        assert!(ed.user_encryption.is_some());
        assert!(ed.perms.is_some());
    }

    #[test]
    fn test_encrypt_dict_from_object_with_optional_fields() {
        let mut dict = HashMap::new();
        dict.insert("Filter".to_string(), Object::Name("Standard".to_string()));
        dict.insert("SubFilter".to_string(), Object::Name("adbe.pkcs7.s4".to_string()));
        dict.insert("V".to_string(), Object::Integer(2));
        dict.insert("R".to_string(), Object::Integer(3));
        dict.insert("Length".to_string(), Object::Integer(128));
        dict.insert("O".to_string(), Object::String(vec![0u8; 32]));
        dict.insert("U".to_string(), Object::String(vec![0u8; 32]));
        dict.insert("P".to_string(), Object::Integer(-3904));
        dict.insert("EncryptMetadata".to_string(), Object::Boolean(false));
        let obj = Object::Dictionary(dict);
        let ed = EncryptDict::from_object(&obj).unwrap();
        assert_eq!(ed.sub_filter.as_deref(), Some("adbe.pkcs7.s4"));
        assert_eq!(ed.length, Some(128));
        assert!(!ed.encrypt_metadata);
    }

    #[test]
    fn test_encrypt_dict_from_object_not_dict() {
        let obj = Object::Integer(42);
        let result = EncryptDict::from_object(&obj);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_dict_from_object_missing_filter() {
        let mut dict = HashMap::new();
        dict.insert("V".to_string(), Object::Integer(1));
        dict.insert("R".to_string(), Object::Integer(2));
        dict.insert("O".to_string(), Object::String(vec![0u8; 32]));
        dict.insert("U".to_string(), Object::String(vec![0u8; 32]));
        dict.insert("P".to_string(), Object::Integer(-1));
        let obj = Object::Dictionary(dict);
        let result = EncryptDict::from_object(&obj);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_dict_from_object_missing_v() {
        let mut dict = HashMap::new();
        dict.insert("Filter".to_string(), Object::Name("Standard".to_string()));
        dict.insert("R".to_string(), Object::Integer(2));
        dict.insert("O".to_string(), Object::String(vec![0u8; 32]));
        dict.insert("U".to_string(), Object::String(vec![0u8; 32]));
        dict.insert("P".to_string(), Object::Integer(-1));
        let obj = Object::Dictionary(dict);
        let result = EncryptDict::from_object(&obj);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_dict_from_object_missing_r() {
        let mut dict = HashMap::new();
        dict.insert("Filter".to_string(), Object::Name("Standard".to_string()));
        dict.insert("V".to_string(), Object::Integer(1));
        dict.insert("O".to_string(), Object::String(vec![0u8; 32]));
        dict.insert("U".to_string(), Object::String(vec![0u8; 32]));
        dict.insert("P".to_string(), Object::Integer(-1));
        let obj = Object::Dictionary(dict);
        let result = EncryptDict::from_object(&obj);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_dict_from_object_missing_o() {
        let mut dict = HashMap::new();
        dict.insert("Filter".to_string(), Object::Name("Standard".to_string()));
        dict.insert("V".to_string(), Object::Integer(1));
        dict.insert("R".to_string(), Object::Integer(2));
        dict.insert("U".to_string(), Object::String(vec![0u8; 32]));
        dict.insert("P".to_string(), Object::Integer(-1));
        let obj = Object::Dictionary(dict);
        let result = EncryptDict::from_object(&obj);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_dict_from_object_missing_u() {
        let mut dict = HashMap::new();
        dict.insert("Filter".to_string(), Object::Name("Standard".to_string()));
        dict.insert("V".to_string(), Object::Integer(1));
        dict.insert("R".to_string(), Object::Integer(2));
        dict.insert("O".to_string(), Object::String(vec![0u8; 32]));
        dict.insert("P".to_string(), Object::Integer(-1));
        let obj = Object::Dictionary(dict);
        let result = EncryptDict::from_object(&obj);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_dict_from_object_missing_p() {
        let mut dict = HashMap::new();
        dict.insert("Filter".to_string(), Object::Name("Standard".to_string()));
        dict.insert("V".to_string(), Object::Integer(1));
        dict.insert("R".to_string(), Object::Integer(2));
        dict.insert("O".to_string(), Object::String(vec![0u8; 32]));
        dict.insert("U".to_string(), Object::String(vec![0u8; 32]));
        let obj = Object::Dictionary(dict);
        let result = EncryptDict::from_object(&obj);
        assert!(result.is_err());
    }

    // === EncryptDict::algorithm tests ===

    #[test]
    fn test_algorithm_detection_v1_r2() {
        let obj = make_encrypt_dict_obj("Standard", 1, 2, &[0u8; 32], &[0u8; 32], -1);
        let ed = EncryptDict::from_object(&obj).unwrap();
        assert_eq!(ed.algorithm().unwrap(), Algorithm::RC4_40);
    }

    #[test]
    fn test_algorithm_detection_v2_r3() {
        let obj = make_encrypt_dict_obj("Standard", 2, 3, &[0u8; 32], &[0u8; 32], -1);
        let ed = EncryptDict::from_object(&obj).unwrap();
        assert_eq!(ed.algorithm().unwrap(), Algorithm::Rc4_128);
    }

    #[test]
    fn test_algorithm_detection_v4_r4() {
        let obj = make_encrypt_dict_obj("Standard", 4, 4, &[0u8; 32], &[0u8; 32], -1);
        let ed = EncryptDict::from_object(&obj).unwrap();
        assert_eq!(ed.algorithm().unwrap(), Algorithm::Aes128);
    }

    #[test]
    fn test_algorithm_detection_v5_r5() {
        let obj = make_encrypt_dict_obj("Standard", 5, 5, &[0u8; 32], &[0u8; 32], -1);
        let ed = EncryptDict::from_object(&obj).unwrap();
        assert_eq!(ed.algorithm().unwrap(), Algorithm::Aes256);
    }

    #[test]
    fn test_algorithm_detection_v5_r6() {
        let obj = make_encrypt_dict_obj("Standard", 5, 6, &[0u8; 32], &[0u8; 32], -1);
        let ed = EncryptDict::from_object(&obj).unwrap();
        assert_eq!(ed.algorithm().unwrap(), Algorithm::Aes256);
    }

    #[test]
    fn test_algorithm_detection_lenient_v1_r3() {
        // Non-standard R for V=1 should still return RC4_40
        let obj = make_encrypt_dict_obj("Standard", 1, 3, &[0u8; 32], &[0u8; 32], -1);
        let ed = EncryptDict::from_object(&obj).unwrap();
        assert_eq!(ed.algorithm().unwrap(), Algorithm::RC4_40);
    }

    #[test]
    fn test_algorithm_detection_lenient_v2_r4() {
        // Non-standard R for V=2 should still return Rc4_128
        let obj = make_encrypt_dict_obj("Standard", 2, 4, &[0u8; 32], &[0u8; 32], -1);
        let ed = EncryptDict::from_object(&obj).unwrap();
        assert_eq!(ed.algorithm().unwrap(), Algorithm::Rc4_128);
    }

    #[test]
    fn test_algorithm_detection_lenient_v4_r5() {
        // Non-standard R for V=4 should still return Aes128
        let obj = make_encrypt_dict_obj("Standard", 4, 5, &[0u8; 32], &[0u8; 32], -1);
        let ed = EncryptDict::from_object(&obj).unwrap();
        assert_eq!(ed.algorithm().unwrap(), Algorithm::Aes128);
    }

    #[test]
    fn test_algorithm_detection_unsupported() {
        let obj = make_encrypt_dict_obj("Standard", 99, 99, &[0u8; 32], &[0u8; 32], -1);
        let ed = EncryptDict::from_object(&obj).unwrap();
        let result = ed.algorithm();
        assert!(result.is_err());
    }

    // === key_length_bytes tests ===

    #[test]
    fn test_key_length_bytes_with_length() {
        let obj = make_encrypt_dict_obj("Standard", 2, 3, &[0u8; 32], &[0u8; 32], -1);
        let mut ed = EncryptDict::from_object(&obj).unwrap();
        ed.length = Some(128);
        assert_eq!(ed.key_length_bytes(), 16);
    }

    #[test]
    fn test_key_length_bytes_with_length_40() {
        let obj = make_encrypt_dict_obj("Standard", 1, 2, &[0u8; 32], &[0u8; 32], -1);
        let mut ed = EncryptDict::from_object(&obj).unwrap();
        ed.length = Some(40);
        assert_eq!(ed.key_length_bytes(), 5);
    }

    #[test]
    fn test_key_length_bytes_default_v1() {
        let obj = make_encrypt_dict_obj("Standard", 1, 2, &[0u8; 32], &[0u8; 32], -1);
        let ed = EncryptDict::from_object(&obj).unwrap();
        assert_eq!(ed.key_length_bytes(), 5); // 40 bits
    }

    #[test]
    fn test_key_length_bytes_default_v2() {
        let obj = make_encrypt_dict_obj("Standard", 2, 3, &[0u8; 32], &[0u8; 32], -1);
        let ed = EncryptDict::from_object(&obj).unwrap();
        assert_eq!(ed.key_length_bytes(), 16); // 128 bits
    }

    #[test]
    fn test_key_length_bytes_default_v4() {
        let obj = make_encrypt_dict_obj("Standard", 4, 4, &[0u8; 32], &[0u8; 32], -1);
        let ed = EncryptDict::from_object(&obj).unwrap();
        assert_eq!(ed.key_length_bytes(), 16); // 128 bits
    }

    #[test]
    fn test_key_length_bytes_default_v5() {
        let obj = make_encrypt_dict_obj("Standard", 5, 6, &[0u8; 32], &[0u8; 32], -1);
        let ed = EncryptDict::from_object(&obj).unwrap();
        assert_eq!(ed.key_length_bytes(), 32); // 256 bits
    }

    #[test]
    fn test_key_length_bytes_default_unknown_v() {
        let obj = make_encrypt_dict_obj("Standard", 3, 3, &[0u8; 32], &[0u8; 32], -1);
        let ed = EncryptDict::from_object(&obj).unwrap();
        assert_eq!(ed.key_length_bytes(), 16); // default 128 bits
    }

    // === to_object tests ===

    #[test]
    fn test_to_object_basic() {
        let ed = EncryptDict {
            filter: "Standard".to_string(),
            sub_filter: None,
            version: 1,
            length: None,
            revision: 2,
            owner_password: vec![0u8; 32],
            user_password: vec![0u8; 32],
            permissions: -4,
            encrypt_metadata: true,
            owner_encryption: None,
            user_encryption: None,
            perms: None,
            stream_crypt_method: None,
        };
        let obj = ed.to_object();
        let dict = obj.as_dict().unwrap();
        assert!(dict.contains_key("Filter"));
        assert!(dict.contains_key("V"));
        assert!(dict.contains_key("R"));
        assert!(dict.contains_key("O"));
        assert!(dict.contains_key("U"));
        assert!(dict.contains_key("P"));
        // EncryptMetadata is true (default), should NOT be written
        assert!(!dict.contains_key("EncryptMetadata"));
    }

    #[test]
    fn test_to_object_with_sub_filter() {
        let ed = EncryptDict {
            filter: "Standard".to_string(),
            sub_filter: Some("adbe.pkcs7.s4".to_string()),
            version: 2,
            length: Some(128),
            revision: 3,
            owner_password: vec![0u8; 32],
            user_password: vec![0u8; 32],
            permissions: -1,
            encrypt_metadata: false,
            owner_encryption: None,
            user_encryption: None,
            perms: None,
            stream_crypt_method: None,
        };
        let obj = ed.to_object();
        let dict = obj.as_dict().unwrap();
        assert!(dict.contains_key("SubFilter"));
        assert!(dict.contains_key("Length"));
        assert!(dict.contains_key("EncryptMetadata"));
    }

    #[test]
    fn test_to_object_v4_has_crypt_filters() {
        let ed = EncryptDict {
            filter: "Standard".to_string(),
            sub_filter: None,
            version: 4,
            length: None,
            revision: 4,
            owner_password: vec![0u8; 32],
            user_password: vec![0u8; 32],
            permissions: -1,
            encrypt_metadata: true,
            owner_encryption: None,
            user_encryption: None,
            perms: None,
            stream_crypt_method: None,
        };
        let obj = ed.to_object();
        let dict = obj.as_dict().unwrap();
        assert!(dict.contains_key("CF"));
        assert!(dict.contains_key("StmF"));
        assert!(dict.contains_key("StrF"));
    }

    #[test]
    fn test_to_object_v5_has_crypt_filters() {
        let ed = EncryptDict {
            filter: "Standard".to_string(),
            sub_filter: None,
            version: 5,
            length: None,
            revision: 6,
            owner_password: vec![0u8; 48],
            user_password: vec![0u8; 48],
            permissions: -1,
            encrypt_metadata: true,
            owner_encryption: Some(vec![0u8; 32]),
            user_encryption: Some(vec![0u8; 32]),
            perms: Some(vec![0u8; 16]),
            stream_crypt_method: None,
        };
        let obj = ed.to_object();
        let dict = obj.as_dict().unwrap();
        assert!(dict.contains_key("CF"));
        assert!(dict.contains_key("StmF"));
        assert!(dict.contains_key("StrF"));
        assert!(dict.contains_key("OE"));
        assert!(dict.contains_key("UE"));
        assert!(dict.contains_key("Perms"));
    }

    // === Permissions tests ===

    #[test]
    fn test_permissions_all_granted() {
        let perms = Permissions::from_bits(-1i32); // all bits set
        assert!(perms.can_print());
        assert!(perms.can_modify());
        assert!(perms.can_copy());
        assert!(perms.can_annotate());
        assert!(perms.can_fill_forms());
        assert!(perms.can_extract_accessibility());
        assert!(perms.can_assemble());
        assert!(perms.can_print_high_quality());
    }

    #[test]
    fn test_permissions_none_granted() {
        let perms = Permissions::from_bits(0);
        assert!(!perms.can_print());
        assert!(!perms.can_modify());
        assert!(!perms.can_copy());
        assert!(!perms.can_annotate());
        assert!(!perms.can_fill_forms());
        assert!(!perms.can_extract_accessibility());
        assert!(!perms.can_assemble());
        assert!(!perms.can_print_high_quality());
    }

    #[test]
    fn test_permissions_print_only() {
        let perms = Permissions::from_bits(1 << 2);
        assert!(perms.can_print());
        assert!(!perms.can_modify());
        assert!(!perms.can_copy());
    }

    #[test]
    fn test_permissions_modify_only() {
        let perms = Permissions::from_bits(1 << 3);
        assert!(!perms.can_print());
        assert!(perms.can_modify());
        assert!(!perms.can_copy());
    }

    #[test]
    fn test_permissions_copy_only() {
        let perms = Permissions::from_bits(1 << 4);
        assert!(perms.can_copy());
        assert!(!perms.can_print());
        assert!(!perms.can_modify());
    }

    #[test]
    fn test_permissions_annotate_only() {
        let perms = Permissions::from_bits(1 << 5);
        assert!(perms.can_annotate());
    }

    #[test]
    fn test_permissions_fill_forms_only() {
        let perms = Permissions::from_bits(1 << 8);
        assert!(perms.can_fill_forms());
    }

    #[test]
    fn test_permissions_extract_accessibility_only() {
        let perms = Permissions::from_bits(1 << 9);
        assert!(perms.can_extract_accessibility());
    }

    #[test]
    fn test_permissions_assemble_only() {
        let perms = Permissions::from_bits(1 << 10);
        assert!(perms.can_assemble());
    }

    #[test]
    fn test_permissions_high_quality_print_only() {
        let perms = Permissions::from_bits(1 << 11);
        assert!(perms.can_print_high_quality());
    }

    #[test]
    fn test_permissions_combined() {
        // Print + Copy + Fill Forms
        let bits = (1 << 2) | (1 << 4) | (1 << 8);
        let perms = Permissions::from_bits(bits);
        assert!(perms.can_print());
        assert!(!perms.can_modify());
        assert!(perms.can_copy());
        assert!(!perms.can_annotate());
        assert!(perms.can_fill_forms());
    }

    #[test]
    fn test_permissions_clone_copy() {
        let p = Permissions::from_bits(-1);
        let p2 = p; // Copy
        let p3 = p;
        assert!(p2.can_print());
        assert!(p3.can_print());
    }

    // === EncryptDictBuilder tests ===

    #[cfg(feature = "legacy-crypto")]
    #[test]
    fn test_builder_rc4_40() {
        let file_id = vec![0u8; 16];
        let ed = EncryptDictBuilder::new(Algorithm::RC4_40)
            .user_password(b"user")
            .owner_password(b"owner")
            .permissions(-4)
            .build(&file_id)
            .unwrap();
        assert_eq!(ed.filter, "Standard");
        assert_eq!(ed.version, 1);
        assert_eq!(ed.revision, 2);
        assert_eq!(ed.permissions, -4);
        assert_eq!(ed.length, Some(40)); // 5 * 8
        assert!(ed.encrypt_metadata);
    }

    #[cfg(feature = "legacy-crypto")]
    #[test]
    fn test_builder_rc4_128() {
        let file_id = vec![0u8; 16];
        let ed = EncryptDictBuilder::new(Algorithm::Rc4_128)
            .user_password(b"pass")
            .build(&file_id)
            .unwrap();
        assert_eq!(ed.version, 2);
        assert_eq!(ed.revision, 3);
        assert_eq!(ed.length, Some(128)); // 16 * 8
    }

    #[cfg(feature = "legacy-crypto")]
    #[test]
    fn test_builder_aes128() {
        let file_id = vec![0u8; 16];
        let ed = EncryptDictBuilder::new(Algorithm::Aes128)
            .user_password(b"pass")
            .encrypt_metadata(false)
            .build(&file_id)
            .unwrap();
        assert_eq!(ed.version, 4);
        assert_eq!(ed.revision, 4);
        assert!(!ed.encrypt_metadata);
    }

    #[test]
    fn test_builder_aes256() {
        let file_id = vec![0u8; 16];
        let ed = EncryptDictBuilder::new(Algorithm::Aes256)
            .user_password(b"user")
            .owner_password(b"owner")
            .build(&file_id)
            .unwrap();
        assert_eq!(ed.version, 5);
        assert_eq!(ed.revision, 6);
        assert_eq!(ed.length, Some(256)); // 32 * 8
    }

    #[cfg(feature = "legacy-crypto")]
    #[test]
    fn test_builder_owner_password_defaults_to_user() {
        let file_id = vec![0u8; 16];
        let ed = EncryptDictBuilder::new(Algorithm::RC4_40)
            .user_password(b"user_pass")
            // No owner_password set -> should use user_password
            .build(&file_id)
            .unwrap();
        assert_eq!(ed.filter, "Standard");
        assert!(!ed.owner_password.is_empty());
    }

    #[cfg(feature = "legacy-crypto")]
    #[test]
    fn test_builder_default_permissions() {
        let file_id = vec![0u8; 16];
        let ed = EncryptDictBuilder::new(Algorithm::RC4_40)
            .user_password(b"pass")
            .build(&file_id)
            .unwrap();
        assert_eq!(ed.permissions, -1); // All permissions by default
    }

    // === generate_file_id tests ===

    #[test]
    fn test_generate_file_id_returns_16_bytes() {
        let (id1, id2) = generate_file_id();
        assert_eq!(id1.len(), 16);
        assert_eq!(id2.len(), 16);
    }

    #[test]
    fn test_generate_file_id_both_equal() {
        let (id1, id2) = generate_file_id();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_generate_file_id_unique() {
        // Two successive calls should produce different IDs
        let (id1, _) = generate_file_id();
        let (id2, _) = generate_file_id();
        // It's extremely unlikely these would be equal
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_generate_file_id_uniqueness() {
        let id1 = generate_file_id();
        let id2 = generate_file_id();
        assert_ne!(id1, id2, "file IDs must be unique across calls");
    }

    // === EncryptDict roundtrip tests ===

    #[test]
    fn test_encrypt_dict_roundtrip_v1() {
        let original = EncryptDict {
            filter: "Standard".to_string(),
            sub_filter: None,
            version: 1,
            length: Some(40),
            revision: 2,
            owner_password: vec![1u8; 32],
            user_password: vec![2u8; 32],
            permissions: -4,
            encrypt_metadata: true,
            owner_encryption: None,
            user_encryption: None,
            perms: None,
            stream_crypt_method: None,
        };
        let obj = original.to_object();
        let parsed = EncryptDict::from_object(&obj).unwrap();
        assert_eq!(parsed.filter, original.filter);
        assert_eq!(parsed.version, original.version);
        assert_eq!(parsed.revision, original.revision);
        assert_eq!(parsed.permissions, original.permissions);
        assert_eq!(parsed.owner_password, original.owner_password);
        assert_eq!(parsed.user_password, original.user_password);
    }

    #[test]
    fn test_encrypt_dict_roundtrip_v4() {
        let original = EncryptDict {
            filter: "Standard".to_string(),
            sub_filter: None,
            version: 4,
            length: None,
            revision: 4,
            owner_password: vec![3u8; 32],
            user_password: vec![4u8; 32],
            permissions: -1,
            encrypt_metadata: true,
            owner_encryption: None,
            user_encryption: None,
            perms: None,
            stream_crypt_method: None,
        };
        let obj = original.to_object();
        let parsed = EncryptDict::from_object(&obj).unwrap();
        assert_eq!(parsed.version, 4);
        assert_eq!(parsed.revision, 4);
        assert_eq!(parsed.algorithm().unwrap(), Algorithm::Aes128);
        assert_eq!(parsed.stream_crypt_method.as_deref(), Some("AESV2"));
    }

    #[test]
    fn test_v4_cfm_v2_selects_rc4_128() {
        // V=4 with CFM=V2 means RC4-128, NOT AES-128.
        // This is the exact case from issue #202 (OpenPDF 1.3.26).
        let ed = EncryptDict {
            filter: "Standard".to_string(),
            sub_filter: None,
            version: 4,
            length: Some(128),
            revision: 4,
            owner_password: vec![0u8; 32],
            user_password: vec![0u8; 32],
            permissions: -1580,
            encrypt_metadata: false,
            owner_encryption: None,
            user_encryption: None,
            perms: None,
            stream_crypt_method: Some("V2".to_string()),
        };
        assert_eq!(ed.algorithm().unwrap(), Algorithm::Rc4_128);
    }

    #[test]
    fn test_v4_cfm_v2_roundtrip() {
        // Build a V=4 dict with CFM=V2, serialize, and re-parse.
        let original = EncryptDict {
            filter: "Standard".to_string(),
            sub_filter: None,
            version: 4,
            length: Some(128),
            revision: 4,
            owner_password: vec![5u8; 32],
            user_password: vec![6u8; 32],
            permissions: -1580,
            encrypt_metadata: false,
            owner_encryption: None,
            user_encryption: None,
            perms: None,
            stream_crypt_method: Some("V2".to_string()),
        };
        let obj = original.to_object();
        let parsed = EncryptDict::from_object(&obj).unwrap();
        assert_eq!(parsed.version, 4);
        assert_eq!(parsed.stream_crypt_method.as_deref(), Some("V2"));
        assert_eq!(parsed.algorithm().unwrap(), Algorithm::Rc4_128);
    }
}
