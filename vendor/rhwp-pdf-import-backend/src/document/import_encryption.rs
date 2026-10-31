//! Admission and allocation boundaries for Standard Security import (#8415).
//! Authentication is required; PDF copy/modify flags are advisory product policy.
use super::*;

impl PdfDocument {
    // Both the in-memory bounded parser and reader parser converge here before
    // cache insertion. ObjStm children are already plaintext and never call this.
    pub(super) fn decrypt_loaded_object(&self, obj: &mut Object, obj_ref: ObjectRef) -> Result<()> {
        // Decrypt string values inside this uncompressed object before
        // caching. Skip the /Encrypt dict (its entries are key material)
        // and the non-authenticated case (no key derived yet). Strings
        // inside compressed objects ride along with the ObjStm payload
        // and are already in clear text per ISO 32000-1:2008 §7.6.2.
        let is_encrypt_dict = *self.encrypt_dict_ref.lock_or_recover() == Some(obj_ref);
        let is_xref = obj
            .as_dict()
            .and_then(|d| d.get("Type"))
            .and_then(Object::as_name)
            == Some("XRef");
        if !is_encrypt_dict && !is_xref {
            let handler_guard = self.encryption_handler.lock_or_recover();
            if let Some(handler) = handler_guard.as_ref() {
                if handler.is_authenticated() {
                    Self::decrypt_strings_in_object(
                        handler,
                        obj,
                        obj_ref.id,
                        obj_ref.gen as u32,
                        self.import_budget.as_deref(),
                    )?;
                }
            }
        }

        if !is_encrypt_dict {
            self.decrypt_import_stream(obj, obj_ref)?;
        }
        Ok(())
    }

    pub(super) fn validate_import_encryption(
        &self,
        object: &mut Object,
        file_id: &[u8],
    ) -> Result<()> {
        let Some(budget) = self.import_budget.as_deref() else {
            return Ok(());
        };
        let Object::Dictionary(dict) = object else {
            return Err(Error::EncryptedPdf);
        };
        if dict.get("Filter").and_then(Object::as_name) != Some("Standard")
            || dict.contains_key("SubFilter")
        {
            return Err(Error::EncryptedPdf);
        }
        let revision = dict
            .get("R")
            .and_then(Object::as_integer)
            .ok_or(Error::EncryptedPdf)?;
        let version = dict
            .get("V")
            .and_then(Object::as_integer)
            .ok_or(Error::EncryptedPdf)?;
        if !matches!((version, revision), (1, 2) | (2, 3) | (4, 4) | (5, 5 | 6)) {
            return Err(Error::EncryptedPdf);
        }
        if file_id.len() > 1024 {
            return Err(budget.reject("encryption file identifier"));
        }
        let require_bytes = |key: &str, length: usize| -> Result<()> {
            if dict
                .get(key)
                .and_then(Object::as_string)
                .is_none_or(|v| v.len() < length || (revision < 5 && v.len() != length))
            {
                return Err(Error::InvalidPdf("invalid encryption key material length".into()));
            }
            Ok(())
        };
        require_bytes("O", if revision >= 5 { 48 } else { 32 })?;
        require_bytes("U", if revision >= 5 { 48 } else { 32 })?;
        if revision >= 5 {
            require_bytes("OE", 32)?;
            require_bytes("UE", 32)?;
            require_bytes("Perms", 16)?;
        }
        let bits = dict
            .get("Length")
            .and_then(Object::as_integer)
            .unwrap_or(40);
        if !(match version {
            1 => bits == 40,
            2 => (40..=128).contains(&bits) && bits % 8 == 0,
            4 => bits == 128,
            5 => bits == 256,
            _ => false,
        }) {
            return Err(Error::EncryptedPdf);
        }
        // The existing handler uses one algorithm for strings and streams.
        // Reject mixed/unknown filters rather than applying that algorithm to
        // plaintext. Supporting those combinations needs distinct handlers.
        if version >= 4 {
            let stream = dict.get("StmF").and_then(Object::as_name);
            let string = dict.get("StrF").and_then(Object::as_name);
            if stream.is_none() || stream == Some("Identity") || stream != string {
                return Err(Error::EncryptedPdf);
            }
            let method = dict
                .get("CF")
                .and_then(Object::as_dict)
                .and_then(|cf| cf.get(stream.unwrap()))
                .and_then(Object::as_dict)
                .and_then(|cf| cf.get("CFM"))
                .and_then(Object::as_name);
            if !matches!((version, method), (4, Some("AESV2" | "V2")) | (5, Some("AESV3"))) {
                return Err(Error::EncryptedPdf);
            }
        }
        // V=5 fields use fixed prefixes; valid files can include trailing bytes
        // (QPDF Encryption::initialize also admits lengths above the minimum).
        // Normalize before EncryptionHandler clones the values. The original
        // parsing/copying remains charged to the same request; no data is grown.
        if revision >= 5 {
            for (name, length) in [("O", 48), ("U", 48), ("OE", 32), ("UE", 32), ("Perms", 16)] {
                if let Some(Object::String(bytes)) = dict.get_mut(name) {
                    bytes.truncate(length);
                }
            }
        } else {
            // These fields have no meaning before V5. Do not let unused
            // attacker-supplied strings create unreserved handler copies.
            for name in ["OE", "UE", "Perms"] {
                dict.remove(name);
            }
        }
        // Fixed-size key/hash fields and their owned copies in EncryptionHandler.
        budget.charge_decoded(4096)?;
        Ok(())
    }

    pub(super) fn reserve_import_authentication(&self, password: &[u8]) -> Result<()> {
        let Some(budget) = self.import_budget.as_deref() else {
            return Ok(());
        };
        if password.len() > 1024 {
            return Err(budget.reject("PDF password length"));
        }
        // ISO 32000-2 Algorithm 2.B stops by round 287 (last byte <= 255).
        // Four hashes conservatively cover user+owner validation/key derivation.
        // Each round uses at most (127 password + 64 hash + 48 U) * 64 bytes,
        // plus AES copies/hash/normalization scratch. Reserve before entering
        // the existing KDF. This is cumulative work, not live heap or RSS.
        const KDF_BYTES: usize = 4 * 288 * ((127 + 64 + 48) * 64 * 3 + 1024);
        // Also honor a host's lower single-codec allocation cap. The largest
        // normalized-password/KDF scratch is conservatively bounded at 64 KiB.
        budget.charge_codec_allocation(64 * 1024)?;
        budget.charge_decoded(KDF_BYTES)?;
        budget.charge_codec_work(4 * 288 * (127 + 64 + 48) * 64)?;
        Ok(())
    }

    pub(super) fn decrypt_import_stream(
        &self,
        object: &mut Object,
        reference: ObjectRef,
    ) -> Result<()> {
        let Some(budget) = self.import_budget.as_deref() else {
            return Ok(());
        };
        let Object::Stream { dict, data } = object else {
            return Ok(());
        };
        let kind = dict.get("Type").and_then(Object::as_name);
        if kind == Some("XRef") {
            return Ok(());
        }
        let guard = self.encryption_handler.lock_or_recover();
        let Some(handler) = guard.as_ref() else {
            return Ok(());
        };
        if !handler.is_authenticated() {
            return Ok(());
        }
        if kind == Some("Metadata") && !handler.encrypts_metadata() {
            return Ok(());
        }
        if data.len() > budget.limits.stream_bytes {
            return Err(budget.reject("encrypted stream bytes"));
        }
        // CBC output is <= ciphertext; provider uses a mutable copy and output.
        budget.charge_codec_allocation(data.len())?;
        budget.charge_decoded(data.len().saturating_mul(2))?;
        budget.charge_codec_work(data.len())?;
        let plaintext = handler.decrypt_stream(data, reference.id, reference.gen as u32)?;
        *data = crate::import_limits::StreamData::bounded(plaintext, self.import_budget.as_ref());
        dict.insert("Length".into(), Object::Integer(data.len() as i64));
        Ok(())
    }
}
