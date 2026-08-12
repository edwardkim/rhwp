//! HWP 배포용 문서 복호화와 HWP5 비밀번호 stream 어댑터.
//!
//! HWP3/HWP5/HWPX 비밀번호 암호 알고리즘은 `crate::password_crypto`가 단일
//! 소유한다. 이 파일은 배포용 ViewText의 별도 계약과 parser 오류형만 유지한다.

use aes::cipher::{Block, BlockCipherDecrypt, KeyInit};
use aes::Aes128;

use super::cfb_reader::{decompress_stream, decompress_stream_limited, CfbError};
use super::record::Record;
use super::tags;

/// 현재 지원하는 HWP5 비밀번호 암호화 방식(FileHeader EncryptVersion).
pub use crate::password_crypto::HWP5_ENCRYPT_VERSION as SUPPORTED_PASSWORD_ENCRYPT_VERSION;

/// 기존 crypto 호출자가 명시적으로 선택할 수 있도록 남긴 호환 상한값.
///
/// 새 호출은 consumer 경계에서 자기 자원 정책을 정한 뒤
/// `decrypt_*_limited(..., max_bytes)`에 그 값을 전달해야 한다. crypto helper는 이 값을
/// 자동으로 선택하지 않는다.
#[deprecated(
    note = "choose an explicit consumer-boundary limit and pass it to decrypt_*_limited(..., max_bytes); crypto helpers no longer select this default"
)]
pub const MAX_PASSWORD_DECOMPRESSED_STREAM_BYTES: usize = 512 * 1024 * 1024;

#[derive(Debug)]
pub enum CryptoError {
    NoDistributeData,
    InvalidPayloadSize(usize),
    KeyExtractionFailed(String),
    DecryptionFailed(String),
    RecordError(String),
    DecompressError(String),
    DecompressedStreamLimitExceeded { max_bytes: usize },
    WrongPassword,
    UnsupportedScheme { encrypt_version: u32 },
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoDistributeData => write!(f, "DISTRIBUTE_DOC_DATA 레코드 없음"),
            Self::InvalidPayloadSize(size) => {
                write!(f, "DISTRIBUTE_DOC_DATA 크기 오류: {size}바이트 (필요: 256)")
            }
            Self::KeyExtractionFailed(error) => write!(f, "AES 키 추출 실패: {error}"),
            Self::DecryptionFailed(error) => write!(f, "복호화 실패: {error}"),
            Self::RecordError(error) => write!(f, "레코드 파싱 실패: {error}"),
            Self::DecompressError(error) => write!(f, "압축 해제 실패: {error}"),
            Self::DecompressedStreamLimitExceeded { max_bytes } => write!(
                f,
                "암호화 스트림의 압축 해제 결과가 {max_bytes} 바이트 상한을 초과했습니다"
            ),
            Self::WrongPassword => {
                write!(f, "비밀번호가 일치하지 않거나 암호화 데이터가 손상되었습니다")
            }
            Self::UnsupportedScheme { encrypt_version } => write!(
                f,
                "지원하지 않는 암호화 방식: EncryptVersion {encrypt_version} (지원: {SUPPORTED_PASSWORD_ENCRYPT_VERSION})"
            ),
        }
    }
}

impl std::error::Error for CryptoError {}

struct MsvcLcg {
    seed: u32,
}

impl MsvcLcg {
    fn new(seed: u32) -> Self {
        Self { seed }
    }

    fn rand(&mut self) -> u32 {
        self.seed = self.seed.wrapping_mul(214013).wrapping_add(2531011);
        (self.seed >> 16) & 0x7fff
    }
}

fn decrypt_distribute_doc_data(data: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if data.len() < 256 {
        return Err(CryptoError::InvalidPayloadSize(data.len()));
    }
    let mut output = data[..256].to_vec();
    let seed = u32::from_le_bytes(output[..4].try_into().expect("four-byte seed"));
    let mut random = MsvcLcg::new(seed);
    let (mut remaining, mut key) = (0_u32, 0_u8);
    for (index, byte) in output.iter_mut().enumerate() {
        if remaining == 0 {
            key = random.rand() as u8;
            remaining = (random.rand() & 0x0f) + 1;
        }
        if index >= 4 {
            *byte ^= key;
        }
        remaining -= 1;
    }
    Ok(output)
}

fn extract_aes_key(data: &[u8]) -> Result<[u8; 16], CryptoError> {
    let offset = 4 + usize::from(
        *data
            .first()
            .ok_or_else(|| CryptoError::KeyExtractionFailed("데이터가 비었습니다".to_string()))?
            & 0x0f,
    );
    let key = data.get(offset..offset + 16).ok_or_else(|| {
        CryptoError::KeyExtractionFailed(format!("오프셋 {offset}에서 16바이트 부족"))
    })?;
    Ok(key.try_into().expect("16-byte key slice"))
}

fn decrypt_aes_ecb(data: &[u8], key: &[u8; 16]) -> Vec<u8> {
    let cipher = Aes128::new_from_slice(key).expect("AES-128 key size");
    let mut output = Vec::with_capacity(data.len());
    for chunk in data.chunks(16) {
        let mut block = Block::<Aes128>::default();
        block[..chunk.len()].copy_from_slice(chunk);
        cipher.decrypt_block(&mut block);
        output.extend_from_slice(&block);
    }
    output
}

/// HWP5 EncryptVersion 4 raw stream을 복호화한다.
pub fn decrypt_password_stream(raw: &[u8], password: &[u8]) -> Vec<u8> {
    crate::password_crypto::decrypt_hwp5_stream(raw, password)
}

pub fn decrypt_password_protected(
    raw: &[u8],
    password: &[u8],
    compressed: bool,
) -> Result<Vec<u8>, CryptoError> {
    let decrypted = decrypt_password_stream(raw, password);
    if compressed {
        decompress_stream(&decrypted).map_err(|_| CryptoError::WrongPassword)
    } else {
        Ok(decrypted)
    }
}

pub fn decrypt_password_protected_limited(
    raw: &[u8],
    password: &[u8],
    compressed: bool,
    max_bytes: usize,
) -> Result<Vec<u8>, CryptoError> {
    let decrypted = decrypt_password_stream(raw, password);
    if compressed {
        decompress_stream_limited(&decrypted, max_bytes).map_err(|error| match error {
            CfbError::LimitExceeded(_) => {
                CryptoError::DecompressedStreamLimitExceeded { max_bytes }
            }
            _ => CryptoError::WrongPassword,
        })
    } else if decrypted.len() > max_bytes {
        Err(CryptoError::DecompressedStreamLimitExceeded { max_bytes })
    } else {
        Ok(decrypted)
    }
}

#[cfg(test)]
pub(super) fn encrypt_password_stream_for_test(raw: &[u8], password: &[u8]) -> Vec<u8> {
    crate::password_crypto::encrypt_hwp5_stream(raw, password)
}

/// 배포용 ViewText section을 복호화한다.
pub fn decrypt_viewtext_section(
    section_data: &[u8],
    compressed: bool,
) -> Result<Vec<u8>, CryptoError> {
    let decrypted = decrypt_viewtext_payload(section_data)?;
    if compressed {
        decompress_stream(&decrypted)
            .map_err(|error| CryptoError::DecompressError(error.to_string()))
    } else {
        Ok(decrypted)
    }
}

pub fn decrypt_viewtext_section_limited(
    section_data: &[u8],
    compressed: bool,
    max_bytes: usize,
) -> Result<Vec<u8>, CryptoError> {
    let decrypted = decrypt_viewtext_payload(section_data)?;
    if compressed {
        decompress_stream_limited(&decrypted, max_bytes).map_err(|error| match error {
            CfbError::LimitExceeded(_) => {
                CryptoError::DecompressedStreamLimitExceeded { max_bytes }
            }
            _ => CryptoError::DecompressError(error.to_string()),
        })
    } else if decrypted.len() > max_bytes {
        Err(CryptoError::DecompressedStreamLimitExceeded { max_bytes })
    } else {
        Ok(decrypted)
    }
}

fn decrypt_viewtext_payload(section_data: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let first = read_first_record(section_data).map_err(CryptoError::RecordError)?;
    if first.tag_id != tags::HWPTAG_DISTRIBUTE_DOC_DATA {
        return Err(CryptoError::NoDistributeData);
    }
    let key = extract_aes_key(&decrypt_distribute_doc_data(&first.data)?)?;
    let header_size = if first.size >= 0xfff { 8 } else { 4 };
    let encrypted = section_data
        .get(header_size + first.size as usize..)
        .ok_or_else(|| CryptoError::DecryptionFailed("암호화된 본문 데이터 없음".to_string()))?;
    Ok(decrypt_aes_ecb(encrypted, &key))
}

fn read_first_record(data: &[u8]) -> Result<Record, String> {
    use byteorder::{LittleEndian, ReadBytesExt};
    use std::io::{Cursor, Read};

    let mut cursor = Cursor::new(data);
    let header = cursor
        .read_u32::<LittleEndian>()
        .map_err(|error| error.to_string())?;
    let tag_id = (header & 0x3ff) as u16;
    let level = ((header >> 10) & 0x3ff) as u16;
    let mut size = header >> 20;
    if size == 0xfff {
        size = cursor
            .read_u32::<LittleEndian>()
            .map_err(|error| error.to_string())?;
    }
    let position = cursor.position() as usize;
    let end = position
        .checked_add(size as usize)
        .filter(|end| *end <= data.len())
        .ok_or_else(|| "레코드 데이터 부족".to_string())?;
    let mut record_data = vec![0_u8; end - position];
    cursor
        .read_exact(&mut record_data)
        .map_err(|error| error.to_string())?;
    Ok(Record {
        tag_id,
        level,
        size,
        data: record_data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distribute_lcg_is_deterministic() {
        let mut left = MsvcLcg::new(12345);
        let mut right = MsvcLcg::new(12345);
        for _ in 0..10 {
            assert_eq!(left.rand(), right.rand());
        }
    }

    #[test]
    fn distribute_aes_matches_nist_vector() {
        let key = [
            0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf,
            0x4f, 0x3c,
        ];
        let ciphertext = [
            0x39, 0x25, 0x84, 0x1d, 0x02, 0xdc, 0x09, 0xfb, 0xdc, 0x11, 0x85, 0x97, 0x19, 0x6a,
            0x0b, 0x32,
        ];
        assert_eq!(
            decrypt_aes_ecb(&ciphertext, &key),
            [
                0x32, 0x43, 0xf6, 0xa8, 0x88, 0x5a, 0x30, 0x8d, 0x31, 0x31, 0x98, 0xa2, 0xe0, 0x37,
                0x07, 0x34,
            ]
        );
    }

    #[test]
    fn password_limit_rejects_expanded_stream() {
        use flate2::write::DeflateEncoder;
        use flate2::Compression;
        use std::io::Write;

        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(&vec![b'A'; 1025]).expect("deflate input");
        let encrypted =
            encrypt_password_stream_for_test(&encoder.finish().expect("deflate"), b"pw");
        assert!(matches!(
            decrypt_password_protected_limited(&encrypted, b"pw", true, 1024),
            Err(CryptoError::DecompressedStreamLimitExceeded { max_bytes: 1024 })
        ));
    }
}
