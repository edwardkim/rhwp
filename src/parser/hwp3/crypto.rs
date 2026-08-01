//! HWP3 parser의 비밀번호 암호 adapter.
//!
//! DES-ECB, UTF-16LE 키 유도, raw-DEFLATE와 CRC32 검증은
//! `crate::password_crypto`가 소유한다. 이 파일은 기존 HWP3 parser 오류 계약만 유지한다.

use std::fmt;

use crate::password_crypto::{self, PasswordCryptoError};

pub use crate::password_crypto::MAX_HWP3_PASSWORD_DECOMPRESSED_BYTES;

const WRONG_PASSWORD_MESSAGE: &str = "비밀번호가 일치하지 않거나 암호화 데이터가 손상되었습니다";

#[derive(Debug)]
pub enum Hwp3CryptoError {
    InvalidFormat(&'static str),
    PasswordEncoding,
    UnsupportedUncompressedPayload,
    DecompressedPayloadLimitExceeded { max_bytes: usize },
    WrongPasswordOrCorruptPayload,
}

impl fmt::Display for Hwp3CryptoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFormat(message) => write!(formatter, "잘못된 HWP3 암호 문서: {message}"),
            Self::PasswordEncoding => write!(formatter, "HWP3 비밀번호는 UTF-8 텍스트여야 합니다"),
            Self::UnsupportedUncompressedPayload => write!(
                formatter,
                "지원하지 않는 HWP3 암호화 방식: 압축되지 않은 암호 본문"
            ),
            Self::DecompressedPayloadLimitExceeded { max_bytes } => write!(
                formatter,
                "HWP3 암호 본문의 압축 해제 결과가 {max_bytes} 바이트 상한을 초과했습니다"
            ),
            Self::WrongPasswordOrCorruptPayload => write!(formatter, "{WRONG_PASSWORD_MESSAGE}"),
        }
    }
}

impl std::error::Error for Hwp3CryptoError {}

fn map_error(error: PasswordCryptoError) -> Hwp3CryptoError {
    match error {
        PasswordCryptoError::InvalidHwp3(message) => Hwp3CryptoError::InvalidFormat(message),
        PasswordCryptoError::Hwp3PasswordEncoding => Hwp3CryptoError::PasswordEncoding,
        PasswordCryptoError::Hwp3UnsupportedUncompressed => {
            Hwp3CryptoError::UnsupportedUncompressedPayload
        }
        PasswordCryptoError::DecompressedLimitExceeded { max_bytes } => {
            Hwp3CryptoError::DecompressedPayloadLimitExceeded { max_bytes }
        }
        PasswordCryptoError::WrongPasswordOrCorruptPayload => {
            Hwp3CryptoError::WrongPasswordOrCorruptPayload
        }
        _ => Hwp3CryptoError::WrongPasswordOrCorruptPayload,
    }
}

/// 입력이 HWP3 비밀번호 암호 문서인지 확인한다.
pub fn is_hwp3_password_protected(input: &[u8]) -> Result<bool, Hwp3CryptoError> {
    password_crypto::is_hwp3_password_protected(input).map_err(map_error)
}

/// HWP3의 UTF-16LE 비밀번호→DES 키 유도를 재현한다.
pub fn derive_legacy_des_key(password: &str) -> [u8; 8] {
    password_crypto::derive_hwp3_legacy_des_key(password)
}

/// 압축된 HWP3 비밀번호 문서의 본문을 복호화한다.
pub fn decrypt_hwp3_password_document(
    input: &[u8],
    password: &[u8],
) -> Result<Vec<u8>, Hwp3CryptoError> {
    password_crypto::decrypt_hwp3_password_document(input, password).map_err(map_error)
}
