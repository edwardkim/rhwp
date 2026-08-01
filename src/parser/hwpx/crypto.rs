//! HWPX parser의 password package adapter.
//!
//! ODF manifest, PBKDF2, AES-256-CBC와 raw-DEFLATE 구현은
//! `crate::password_crypto`에 있으며, 여기서는 기존 `HwpxError` 계약만 유지한다.

use crate::password_crypto::{self, PasswordCryptoError};

use super::HwpxError;

fn map_error(error: PasswordCryptoError) -> HwpxError {
    match error {
        PasswordCryptoError::WrongPasswordOrCorruptPayload => {
            HwpxError::WrongPasswordOrCorruptPayload
        }
        PasswordCryptoError::HwpxZip(message) => HwpxError::ZipError(message),
        PasswordCryptoError::HwpxXml(message) => HwpxError::XmlError(message),
        PasswordCryptoError::HwpxMissingEntry(path) => HwpxError::MissingFile(path),
        PasswordCryptoError::HwpxEntryLimitExceeded { path, max_bytes } => {
            HwpxError::DecryptedEntryLimitExceeded { path, max_bytes }
        }
        PasswordCryptoError::HwpxUnsupported(message) => HwpxError::UnsupportedEncryption(message),
        other => HwpxError::UnsupportedEncryption(other.to_string()),
    }
}

/// 암호 HWPX만 메모리의 평문 ZIP으로 바꾼다. 평문 HWPX이면 `None`을 반환한다.
pub(super) fn decrypt_hwpx_package(
    data: &[u8],
    password: &[u8],
) -> Result<Option<Vec<u8>>, HwpxError> {
    password_crypto::decrypt_hwpx_package(data, password).map_err(map_error)
}
