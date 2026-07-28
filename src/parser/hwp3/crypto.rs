//! HWP3 비밀번호 암호문 복호화.
//!
//! HWP3 암호 문서는 본문 압축 payload에 DES-ECB를 적용한다. 비밀번호는 UTF-16LE
//! 바이트를 순환 XOR/rotate한 56-bit 재료에서 DES 키로 유도한다. 복호화 뒤에는 raw
//! DEFLATE 본문과 뒤따르는 CRC32/ISIZE trailer를 모두 검증해서, 오입력과 손상된
//! 암호문을 같은 안전한 오류로 처리한다.

use std::fmt;

use crc32fast::Hasher;
use des::cipher::{Block, BlockDecrypt, KeyInit};
use des::Des;
use flate2::{Decompress, FlushDecompress, Status};

const HWP3_MAGIC: &[u8; 30] = b"HWP Document File V3.00 \x1a\x01\x02\x03\x04\x05";
const DOCUMENT_INFO_OFFSET: usize = 30;
const DOCUMENT_INFO_BYTES: usize = 128;
const DOCUMENT_SUMMARY_BYTES: usize = 1008;
const FIXED_HEADER_BYTES: usize =
    DOCUMENT_INFO_OFFSET + DOCUMENT_INFO_BYTES + DOCUMENT_SUMMARY_BYTES;
const PASSWORD_FLAG_OFFSET: usize = DOCUMENT_INFO_OFFSET + 96;
const COMPRESSION_FLAG_OFFSET: usize = DOCUMENT_INFO_OFFSET + 124;
const INFO_BLOCK_LENGTH_OFFSET: usize = DOCUMENT_INFO_OFFSET + 126;
/// HWP3 압축 암호 평문 앞의 암호 확인용 고정 블록 길이.
///
/// DES 복호화 뒤 raw DEFLATE 결과에는 256바이트의 HWP3 전용 prefix가 먼저 온다.
/// 일반 HWP3 parser가 읽는 글꼴 테이블은 그 직후부터 시작한다. 독립 복호화 도구는
/// flag만 해제한 호환 파일을 만들기 위해 이를 유지할 수 있지만, rhwp 내부 parser에는
/// 본문이 아닌 이 prefix를 넘기면 안 된다.
const HWP3_PASSWORD_PREFIX_BYTES: usize = 256;

/// HWP3 비밀번호 암호 payload가 압축 해제된 뒤 가질 수 있는 최대 크기.
///
/// HWP5 암호 스트림과 같은 512 MiB 상한을 적용한다. HWP3는 하나의 raw DEFLATE
/// payload에 문서 본문 전체를 담으므로, trailer 검증 전에도 이 상한으로 메모리
/// 확장 폭주를 막는다.
pub const MAX_HWP3_PASSWORD_DECOMPRESSED_BYTES: usize = 512 * 1024 * 1024;

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

#[derive(Debug, Clone, Copy)]
struct Hwp3Layout {
    payload_offset: usize,
    compressed: bool,
}

fn read_u16_le(input: &[u8], offset: usize) -> Result<u16, Hwp3CryptoError> {
    let bytes = input
        .get(offset..offset + 2)
        .ok_or(Hwp3CryptoError::InvalidFormat("문서 정보가 잘렸습니다"))?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn parse_hwp3_layout(input: &[u8]) -> Result<Hwp3Layout, Hwp3CryptoError> {
    if input.len() < FIXED_HEADER_BYTES {
        return Err(Hwp3CryptoError::InvalidFormat("헤더가 너무 짧습니다"));
    }
    if input.get(..HWP3_MAGIC.len()) != Some(HWP3_MAGIC) {
        return Err(Hwp3CryptoError::InvalidFormat(
            "HWP3 시그니처가 일치하지 않습니다",
        ));
    }
    if read_u16_le(input, PASSWORD_FLAG_OFFSET)? == 0 {
        return Err(Hwp3CryptoError::InvalidFormat(
            "암호 플래그가 설정되지 않았습니다",
        ));
    }

    let info_block_length = usize::from(read_u16_le(input, INFO_BLOCK_LENGTH_OFFSET)?);
    let payload_offset = FIXED_HEADER_BYTES
        .checked_add(info_block_length)
        .ok_or(Hwp3CryptoError::InvalidFormat("정보 블록 길이가 넘칩니다"))?;
    if payload_offset >= input.len() {
        return Err(Hwp3CryptoError::InvalidFormat("암호 본문이 없습니다"));
    }
    if !(input.len() - payload_offset).is_multiple_of(8) {
        return Err(Hwp3CryptoError::InvalidFormat(
            "암호 본문이 DES 블록 경계에 맞지 않습니다",
        ));
    }

    Ok(Hwp3Layout {
        payload_offset,
        compressed: input[COMPRESSION_FLAG_OFFSET] != 0,
    })
}

/// 입력이 HWP3 비밀번호 암호 문서인지 확인한다.
pub fn is_hwp3_password_protected(input: &[u8]) -> Result<bool, Hwp3CryptoError> {
    if input.len() < PASSWORD_FLAG_OFFSET + 2 {
        return Err(Hwp3CryptoError::InvalidFormat("문서 정보가 너무 짧습니다"));
    }
    Ok(read_u16_le(input, PASSWORD_FLAG_OFFSET)? != 0)
}

/// HWP3의 UTF-16LE 비밀번호→DES 키 유도를 재현한다.
pub fn derive_legacy_des_key(password: &str) -> [u8; 8] {
    let mut rolling = [0u8; 7];
    for (index, byte) in password
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .enumerate()
    {
        let slot = index % rolling.len();
        rolling[slot] = (rolling[slot] ^ byte).rotate_left(1);
    }

    let mut key = [0u8; 8];
    for bit_index in 0..56 {
        let source = rolling[bit_index / 8];
        let source_bit = (source >> (7 - (bit_index % 8))) & 1;
        let destination_byte = bit_index / 7;
        let destination_bit = 7 - (bit_index % 7);
        key[destination_byte] |= source_bit << destination_bit;
    }
    key
}

fn decrypt_des_ecb_in_place(payload: &mut [u8], key: &[u8; 8]) {
    let cipher = Des::new_from_slice(key).expect("고정 길이 DES 키");
    for block in payload.chunks_exact_mut(8) {
        cipher.decrypt_block(Block::<Des>::from_mut_slice(block));
    }
}

fn inflate_raw_deflate_checked(
    payload: &[u8],
    max_bytes: usize,
) -> Result<Vec<u8>, Hwp3CryptoError> {
    let mut decoder = Decompress::new(false);
    let mut checksum = Hasher::new();
    let mut output = [0u8; 64 * 1024];
    let mut input_offset = 0usize;
    let mut decompressed = Vec::new();

    loop {
        let input_before = decoder.total_in();
        let output_before = decoder.total_out();
        let status = decoder
            .decompress(&payload[input_offset..], &mut output, FlushDecompress::None)
            .map_err(|_| Hwp3CryptoError::WrongPasswordOrCorruptPayload)?;
        let consumed = usize::try_from(decoder.total_in())
            .map_err(|_| Hwp3CryptoError::WrongPasswordOrCorruptPayload)?;
        let produced = usize::try_from(decoder.total_out() - output_before)
            .map_err(|_| Hwp3CryptoError::WrongPasswordOrCorruptPayload)?;
        if consumed > payload.len() {
            return Err(Hwp3CryptoError::WrongPasswordOrCorruptPayload);
        }
        let output_len = decompressed
            .len()
            .checked_add(produced)
            .ok_or(Hwp3CryptoError::DecompressedPayloadLimitExceeded { max_bytes })?;
        if output_len > max_bytes {
            return Err(Hwp3CryptoError::DecompressedPayloadLimitExceeded { max_bytes });
        }
        checksum.update(&output[..produced]);
        decompressed.extend_from_slice(&output[..produced]);

        if status == Status::StreamEnd {
            let trailer_end = consumed
                .checked_add(8)
                .ok_or(Hwp3CryptoError::WrongPasswordOrCorruptPayload)?;
            let trailer = payload
                .get(consumed..trailer_end)
                .ok_or(Hwp3CryptoError::WrongPasswordOrCorruptPayload)?;
            let expected_checksum = u32::from_le_bytes(
                trailer[..4]
                    .try_into()
                    .map_err(|_| Hwp3CryptoError::WrongPasswordOrCorruptPayload)?,
            );
            let expected_size = u32::from_le_bytes(
                trailer[4..]
                    .try_into()
                    .map_err(|_| Hwp3CryptoError::WrongPasswordOrCorruptPayload)?,
            );
            let actual_size = u32::try_from(decompressed.len())
                .map_err(|_| Hwp3CryptoError::WrongPasswordOrCorruptPayload)?;
            if actual_size == 0
                || expected_checksum != checksum.finalize()
                || expected_size != actual_size
            {
                return Err(Hwp3CryptoError::WrongPasswordOrCorruptPayload);
            }
            return Ok(decompressed);
        }
        if status != Status::Ok
            || (decoder.total_in() == input_before && decoder.total_out() == output_before)
        {
            return Err(Hwp3CryptoError::WrongPasswordOrCorruptPayload);
        }
        input_offset = consumed;
    }
}

/// 압축된 HWP3 비밀번호 문서의 본문을 복호화한다.
///
/// 반환값은 암호 플래그와 압축 플래그를 해제하고, 본문을 검증된 평문으로 바꾼 HWP3
/// 바이트다. HWP3의 256바이트 암호 확인 prefix, CRC32/ISIZE trailer와 패딩은 parser에
/// 넘기지 않는다. 입력 비밀번호는 UTF-8로 받아 HWP3 규칙의 UTF-16LE 키 유도에만 사용하고
/// 보관하지 않는다.
pub fn decrypt_hwp3_password_document(
    input: &[u8],
    password: &[u8],
) -> Result<Vec<u8>, Hwp3CryptoError> {
    let layout = parse_hwp3_layout(input)?;
    if !layout.compressed {
        return Err(Hwp3CryptoError::UnsupportedUncompressedPayload);
    }
    let password = std::str::from_utf8(password).map_err(|_| Hwp3CryptoError::PasswordEncoding)?;

    let key = derive_legacy_des_key(password);
    let mut output = input.to_vec();
    decrypt_des_ecb_in_place(&mut output[layout.payload_offset..], &key);
    let mut decompressed = inflate_raw_deflate_checked(
        &output[layout.payload_offset..],
        MAX_HWP3_PASSWORD_DECOMPRESSED_BYTES,
    )?;
    if decompressed.len() <= HWP3_PASSWORD_PREFIX_BYTES {
        return Err(Hwp3CryptoError::InvalidFormat(
            "HWP3 암호 확인 블록 뒤에 본문이 없습니다",
        ));
    }
    let body = decompressed.split_off(HWP3_PASSWORD_PREFIX_BYTES);
    output.truncate(layout.payload_offset);
    output.extend_from_slice(&body);
    output[PASSWORD_FLAG_OFFSET..PASSWORD_FLAG_OFFSET + 2].copy_from_slice(&0u16.to_le_bytes());
    output[COMPRESSION_FLAG_OFFSET] = 0;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use des::cipher::BlockEncrypt;
    use flate2::write::DeflateEncoder;
    use flate2::Compression;

    use super::*;

    fn encrypt_des_ecb_in_place(payload: &mut [u8], key: &[u8; 8]) {
        let cipher = Des::new_from_slice(key).expect("고정 길이 DES 키");
        for block in payload.chunks_exact_mut(8) {
            cipher.encrypt_block(Block::<Des>::from_mut_slice(block));
        }
    }

    fn encrypted_fixture(password: &str, plain: &[u8]) -> Vec<u8> {
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        let mut hwp3_plain = vec![0xA5; HWP3_PASSWORD_PREFIX_BYTES];
        hwp3_plain.extend_from_slice(plain);
        encoder.write_all(&hwp3_plain).expect("테스트 본문 압축");
        let mut payload = encoder.finish().expect("테스트 본문 마감");
        payload.extend_from_slice(&crc32fast::hash(&hwp3_plain).to_le_bytes());
        payload.extend_from_slice(&(hwp3_plain.len() as u32).to_le_bytes());
        payload.resize((payload.len() + 7) & !7, 0);
        let key = derive_legacy_des_key(password);
        encrypt_des_ecb_in_place(&mut payload, &key);

        let mut input = vec![0u8; FIXED_HEADER_BYTES];
        input[..HWP3_MAGIC.len()].copy_from_slice(HWP3_MAGIC);
        input[PASSWORD_FLAG_OFFSET..PASSWORD_FLAG_OFFSET + 2].copy_from_slice(&2u16.to_le_bytes());
        input[COMPRESSION_FLAG_OFFSET] = 1;
        input.extend_from_slice(&payload);
        input
    }

    #[test]
    fn derives_hancom_known_answer_key() {
        assert_eq!(
            derive_legacy_des_key("123456"),
            [0xc4, 0x34, 0xb2, 0x0c, 0xcc, 0x60, 0x00, 0xd0]
        );
    }

    #[test]
    fn decrypts_and_clears_password_flag() {
        let input = encrypted_fixture("correct-password", b"HWP3 test payload");
        let output = decrypt_hwp3_password_document(&input, b"correct-password").expect("복호화");
        assert!(!is_hwp3_password_protected(&output).expect("암호 플래그"));
        assert_eq!(output[COMPRESSION_FLAG_OFFSET], 0);
    }

    #[test]
    fn rejects_wrong_password_before_returning_output() {
        let input = encrypted_fixture("correct-password", b"HWP3 test payload");
        assert!(matches!(
            decrypt_hwp3_password_document(&input, b"wrong-password"),
            Err(Hwp3CryptoError::WrongPasswordOrCorruptPayload)
        ));
    }

    #[test]
    fn rejects_expansion_over_the_configured_limit() {
        let input = encrypted_fixture("correct-password", &[b'x'; 128]);
        let layout = parse_hwp3_layout(&input).expect("암호 payload 위치");
        let key = derive_legacy_des_key("correct-password");
        let mut payload = input[layout.payload_offset..].to_vec();
        decrypt_des_ecb_in_place(&mut payload, &key);
        assert!(matches!(
            inflate_raw_deflate_checked(&payload, 127),
            Err(Hwp3CryptoError::DecompressedPayloadLimitExceeded { max_bytes: 127 })
        ));
    }
}
