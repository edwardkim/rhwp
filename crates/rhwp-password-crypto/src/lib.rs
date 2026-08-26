//! HWP3, HWP5, HWPX 비밀번호 보호의 공통 암호 구현.
//!
//! parser와 serializer는 컨테이너와 문서 레이아웃만 담당한다. 이 모듈은 세 형식의
//! 키 유도, 암·복호화, HWP3 raw-DEFLATE와 HWPX ODF manifest 계약을 한곳에서 소유한다.

use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::io::{Cursor, Read, Write};

use aes::{Aes128, Aes256};
use base64::Engine as _;
use cbc::cipher::{
    block_padding::NoPadding, Block, BlockCipherDecrypt, BlockCipherEncrypt, BlockModeDecrypt,
    BlockModeEncrypt, KeyInit, KeyIvInit,
};
use cbc::{Decryptor, Encryptor};
use crc32fast::Hasher;
use des::Des;
use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use flate2::{Compression, Decompress, FlushDecompress, Status};
use hmac::Hmac;
use pbkdf2::pbkdf2;
use quick_xml::events::Event;
use quick_xml::{Reader, Writer};
use roxmltree::{Document as XmlDocument, Node};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

/// 한글 7.0 이후 HWP5 비밀번호 보호의 FileHeader EncryptVersion 값.
pub const HWP5_ENCRYPT_VERSION: u32 = 4;
/// HWP3 비밀번호 암호 payload의 압축 해제 상한.
pub const MAX_HWP3_PASSWORD_DECOMPRESSED_BYTES: usize = 512 * 1024 * 1024;

const HWP3_MAGIC: &[u8; 30] = b"HWP Document File V3.00 \x1a\x01\x02\x03\x04\x05";
const HWP3_DOCUMENT_INFO_OFFSET: usize = 30;
const HWP3_FIXED_HEADER_BYTES: usize = HWP3_DOCUMENT_INFO_OFFSET + 128 + 1008;
const HWP3_PASSWORD_FLAG_OFFSET: usize = HWP3_DOCUMENT_INFO_OFFSET + 96;
const HWP3_COMPRESSION_FLAG_OFFSET: usize = HWP3_DOCUMENT_INFO_OFFSET + 124;
const HWP3_INFO_BLOCK_LENGTH_OFFSET: usize = HWP3_DOCUMENT_INFO_OFFSET + 126;
const HWP3_PASSWORD_PREFIX_BYTES: usize = 256;

const HWPX_MANIFEST_PATH: &str = "META-INF/manifest.xml";
const HWPX_AES_256_CBC: &str = "http://www.w3.org/2001/04/xmlenc#aes256-cbc";
const HWPX_SHA_256_START_KEY: &str = "http://www.w3.org/2000/09/xmldsig#sha256";
const HWPX_SHA_256_1K_SUFFIX: &str = "#sha256-1k";
const HWPX_PBKDF2_NAME: &str = "urn:oasis:names:tc:opendocument:xmlns:manifest:1.0#pbkdf2";
const HWPX_MAX_PBKDF2_ITERATIONS: u32 = 1_000_000;
const HWPX_MAX_XML_SIZE: usize = 256 * 1024 * 1024;
const HWPX_MAX_BINDATA_SIZE: usize = 512 * 1024 * 1024;

type HmacSha1 = Hmac<Sha1>;
type HmacSha256 = Hmac<Sha256>;

/// 공통 암호 계층의 오류.
///
/// parser는 이 오류를 기존 형식별 오류로 변환한다. 암호나 파생 키는 오류에 포함하지 않는다.
#[derive(Debug)]
pub enum PasswordCryptoError {
    InvalidHwp3(&'static str),
    Hwp3PasswordEncoding,
    Hwp3UnsupportedUncompressed,
    DecompressedLimitExceeded { max_bytes: usize },
    WrongPasswordOrCorruptPayload,
    HwpxUnsupported(String),
    HwpxZip(String),
    HwpxXml(String),
    HwpxMissingEntry(String),
    HwpxEntryLimitExceeded { path: String, max_bytes: usize },
    Random(String),
}

impl std::fmt::Display for PasswordCryptoError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidHwp3(message) => write!(formatter, "잘못된 HWP3 암호 문서: {message}"),
            Self::Hwp3PasswordEncoding => {
                write!(formatter, "HWP3 비밀번호는 UTF-8 텍스트여야 합니다")
            }
            Self::Hwp3UnsupportedUncompressed => write!(
                formatter,
                "지원하지 않는 HWP3 암호화 방식: 압축되지 않은 암호 본문"
            ),
            Self::DecompressedLimitExceeded { max_bytes } => write!(
                formatter,
                "비밀번호 암호문의 압축 해제 결과가 {max_bytes} 바이트 상한을 초과했습니다"
            ),
            Self::WrongPasswordOrCorruptPayload => write!(
                formatter,
                "비밀번호가 일치하지 않거나 암호화 데이터가 손상되었습니다"
            ),
            Self::HwpxUnsupported(message) => {
                write!(formatter, "지원하지 않는 HWPX 암호화: {message}")
            }
            Self::HwpxZip(message) => write!(formatter, "HWPX ZIP 오류: {message}"),
            Self::HwpxXml(message) => write!(formatter, "HWPX XML 오류: {message}"),
            Self::HwpxMissingEntry(path) => write!(formatter, "HWPX 항목이 없습니다: {path}"),
            Self::HwpxEntryLimitExceeded { path, max_bytes } => {
                write!(
                    formatter,
                    "{path}의 복호화 결과가 {max_bytes} 바이트 상한을 초과했습니다"
                )
            }
            Self::Random(message) => write!(formatter, "안전한 난수 생성 실패: {message}"),
        }
    }
}

impl std::error::Error for PasswordCryptoError {}

/// HWP5 비밀번호에서 AES-128 키를 유도한다.
fn derive_hwp5_key(password: &[u8]) -> [u8; 16] {
    let mut input = vec![0_u8; password.len() * 2];
    for (index, &byte) in password.iter().enumerate() {
        let previous = if index == 0 {
            0xec
        } else {
            password[index - 1]
        };
        input[index * 2] = previous.rotate_left(1);
        input[index * 2 + 1] = byte;
    }
    let digest = Sha1::digest(input);
    let mut key = [0_u8; 16];
    key.copy_from_slice(&digest[..16]);
    key
}

fn hwp5_padded(data: &[u8]) -> Vec<u8> {
    let remainder = data.len() % 16;
    if remainder == 0 {
        return data.to_vec();
    }
    let mut output = data.to_vec();
    output.resize(data.len() + 16 - remainder, (16 - remainder) as u8);
    output
}

fn hwp5_shift_register(register: &mut [u8; 16], feedback_bit: u8) {
    for index in 0..15 {
        register[index] = (register[index] << 1) | (register[index + 1] >> 7);
    }
    register[15] = (register[15] << 1) | (feedback_bit & 1);
}

fn aes_msb(cipher: &Aes128, register: &[u8; 16]) -> u8 {
    let mut block = Block::<Aes128>::from(*register);
    cipher.encrypt_block(&mut block);
    block[0] >> 7
}

/// HWP5 EncryptVersion 4 stream을 복호화한다.
pub fn decrypt_hwp5_stream(ciphertext: &[u8], password: &[u8]) -> Vec<u8> {
    hwp5_transform(ciphertext, password, false)
}

/// HWP5 EncryptVersion 4 stream을 암호화한다.
pub fn encrypt_hwp5_stream(plaintext: &[u8], password: &[u8]) -> Vec<u8> {
    hwp5_transform(plaintext, password, true)
}

fn hwp5_transform(input: &[u8], password: &[u8], encrypt: bool) -> Vec<u8> {
    let cipher = Aes128::new_from_slice(&derive_hwp5_key(password)).expect("AES-128 key size");
    let padded = hwp5_padded(input);
    let mut register = [0_u8; 16];
    let mut output = Vec::with_capacity(input.len());

    for block in padded.chunks_exact(16) {
        let mut transformed = [0_u8; 16];
        for bit_index in 0..128 {
            let byte_index = bit_index / 8;
            let bit_offset = bit_index % 8;
            let input_bit = (block[byte_index] >> (7 - bit_offset)) & 1;
            let result_bit = input_bit ^ aes_msb(&cipher, &register);
            let feedback_bit = if encrypt { result_bit } else { input_bit };
            hwp5_shift_register(&mut register, feedback_bit);
            transformed[byte_index] |= result_bit << (7 - bit_offset);
        }
        output.extend_from_slice(&transformed);
    }
    output.truncate(input.len());
    output
}

#[derive(Clone, Copy)]
struct Hwp3Layout {
    payload_offset: usize,
    compressed: bool,
}

fn hwp3_read_u16(input: &[u8], offset: usize) -> Result<u16, PasswordCryptoError> {
    let bytes = input
        .get(offset..offset + 2)
        .ok_or(PasswordCryptoError::InvalidHwp3("문서 정보가 잘렸습니다"))?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn parse_hwp3_layout(
    input: &[u8],
    password_required: bool,
) -> Result<Hwp3Layout, PasswordCryptoError> {
    if input.len() < HWP3_FIXED_HEADER_BYTES {
        return Err(PasswordCryptoError::InvalidHwp3("헤더가 너무 짧습니다"));
    }
    if input.get(..HWP3_MAGIC.len()) != Some(HWP3_MAGIC) {
        return Err(PasswordCryptoError::InvalidHwp3(
            "HWP3 시그니처가 일치하지 않습니다",
        ));
    }
    let password_flag = hwp3_read_u16(input, HWP3_PASSWORD_FLAG_OFFSET)?;
    if password_required && password_flag == 0 {
        return Err(PasswordCryptoError::InvalidHwp3(
            "암호 플래그가 설정되지 않았습니다",
        ));
    }
    if !password_required && password_flag != 0 {
        return Err(PasswordCryptoError::InvalidHwp3(
            "이미 암호 플래그가 설정되어 있습니다",
        ));
    }

    let info_block_length = usize::from(hwp3_read_u16(input, HWP3_INFO_BLOCK_LENGTH_OFFSET)?);
    let payload_offset = HWP3_FIXED_HEADER_BYTES
        .checked_add(info_block_length)
        .ok_or(PasswordCryptoError::InvalidHwp3(
            "정보 블록 길이가 넘칩니다",
        ))?;
    if payload_offset >= input.len() {
        return Err(PasswordCryptoError::InvalidHwp3("암호 본문이 없습니다"));
    }
    if password_required && !(input.len() - payload_offset).is_multiple_of(8) {
        return Err(PasswordCryptoError::InvalidHwp3(
            "암호 본문이 DES 블록 경계에 맞지 않습니다",
        ));
    }

    Ok(Hwp3Layout {
        payload_offset,
        compressed: input[HWP3_COMPRESSION_FLAG_OFFSET] != 0,
    })
}

/// 입력이 HWP3 비밀번호 암호 문서인지 확인한다.
pub fn is_hwp3_password_protected(input: &[u8]) -> Result<bool, PasswordCryptoError> {
    if input.len() < HWP3_PASSWORD_FLAG_OFFSET + 2 {
        return Err(PasswordCryptoError::InvalidHwp3(
            "문서 정보가 너무 짧습니다",
        ));
    }
    Ok(hwp3_read_u16(input, HWP3_PASSWORD_FLAG_OFFSET)? != 0)
}

/// HWP3의 UTF-16LE 비밀번호→DES 키 유도를 재현한다.
pub fn derive_hwp3_legacy_des_key(password: &str) -> [u8; 8] {
    let mut rolling = [0_u8; 7];
    for (index, byte) in password
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .enumerate()
    {
        let slot = index % rolling.len();
        rolling[slot] = (rolling[slot] ^ byte).rotate_left(1);
    }
    let mut key = [0_u8; 8];
    for bit_index in 0..56 {
        let source = rolling[bit_index / 8];
        let source_bit = (source >> (7 - (bit_index % 8))) & 1;
        key[bit_index / 7] |= source_bit << (7 - (bit_index % 7));
    }
    key
}

fn hwp3_des_ecb(payload: &mut [u8], key: &[u8; 8], encrypt: bool) {
    let cipher = Des::new_from_slice(key).expect("DES key size");
    for block in payload.chunks_exact_mut(8) {
        let block: &mut [u8; 8] = block.try_into().expect("HWP3 DES block size");
        let block: &mut Block<Des> = block.into();
        if encrypt {
            cipher.encrypt_block(block);
        } else {
            cipher.decrypt_block(block);
        }
    }
}

fn inflate_hwp3_raw_deflate(
    payload: &[u8],
    max_bytes: usize,
) -> Result<Vec<u8>, PasswordCryptoError> {
    let mut decoder = Decompress::new(false);
    let mut checksum = Hasher::new();
    let mut chunk = [0_u8; 64 * 1024];
    let mut input_offset = 0_usize;
    let mut output = Vec::new();

    loop {
        let input_before = decoder.total_in();
        let output_before = decoder.total_out();
        let status = decoder
            .decompress(&payload[input_offset..], &mut chunk, FlushDecompress::None)
            .map_err(|_| PasswordCryptoError::WrongPasswordOrCorruptPayload)?;
        let consumed = usize::try_from(decoder.total_in())
            .map_err(|_| PasswordCryptoError::WrongPasswordOrCorruptPayload)?;
        let produced = usize::try_from(decoder.total_out() - output_before)
            .map_err(|_| PasswordCryptoError::WrongPasswordOrCorruptPayload)?;
        if consumed > payload.len() {
            return Err(PasswordCryptoError::WrongPasswordOrCorruptPayload);
        }
        let output_len = output
            .len()
            .checked_add(produced)
            .ok_or(PasswordCryptoError::DecompressedLimitExceeded { max_bytes })?;
        if output_len > max_bytes {
            return Err(PasswordCryptoError::DecompressedLimitExceeded { max_bytes });
        }
        checksum.update(&chunk[..produced]);
        output.extend_from_slice(&chunk[..produced]);

        if status == Status::StreamEnd {
            let trailer_end = consumed
                .checked_add(8)
                .ok_or(PasswordCryptoError::WrongPasswordOrCorruptPayload)?;
            let trailer = payload
                .get(consumed..trailer_end)
                .ok_or(PasswordCryptoError::WrongPasswordOrCorruptPayload)?;
            let expected_checksum = u32::from_le_bytes(
                trailer[..4]
                    .try_into()
                    .map_err(|_| PasswordCryptoError::WrongPasswordOrCorruptPayload)?,
            );
            let expected_size = u32::from_le_bytes(
                trailer[4..]
                    .try_into()
                    .map_err(|_| PasswordCryptoError::WrongPasswordOrCorruptPayload)?,
            );
            if output.is_empty()
                || u32::try_from(output.len())
                    .ok()
                    .filter(|size| *size == expected_size)
                    .is_none()
                || checksum.finalize() != expected_checksum
            {
                return Err(PasswordCryptoError::WrongPasswordOrCorruptPayload);
            }
            return Ok(output);
        }
        if status != Status::Ok
            || (decoder.total_in() == input_before && decoder.total_out() == output_before)
        {
            return Err(PasswordCryptoError::WrongPasswordOrCorruptPayload);
        }
        input_offset = consumed;
    }
}

/// HWP3 DES-ECB password document를 압축하지 않은 평문 HWP3로 바꾼다.
pub fn decrypt_hwp3_password_document(
    input: &[u8],
    password: &[u8],
) -> Result<Vec<u8>, PasswordCryptoError> {
    let layout = parse_hwp3_layout(input, true)?;
    if !layout.compressed {
        return Err(PasswordCryptoError::Hwp3UnsupportedUncompressed);
    }
    let password =
        std::str::from_utf8(password).map_err(|_| PasswordCryptoError::Hwp3PasswordEncoding)?;
    let mut payload = input[layout.payload_offset..].to_vec();
    hwp3_des_ecb(&mut payload, &derive_hwp3_legacy_des_key(password), false);
    let mut plain = inflate_hwp3_raw_deflate(&payload, MAX_HWP3_PASSWORD_DECOMPRESSED_BYTES)?;
    if plain.len() <= HWP3_PASSWORD_PREFIX_BYTES {
        return Err(PasswordCryptoError::InvalidHwp3(
            "HWP3 암호 확인 블록 뒤에 본문이 없습니다",
        ));
    }
    let body = plain.split_off(HWP3_PASSWORD_PREFIX_BYTES);
    let mut output = input[..layout.payload_offset].to_vec();
    output.extend_from_slice(&body);
    output[HWP3_PASSWORD_FLAG_OFFSET..HWP3_PASSWORD_FLAG_OFFSET + 2]
        .copy_from_slice(&0_u16.to_le_bytes());
    output[HWP3_COMPRESSION_FLAG_OFFSET] = 0;
    Ok(output)
}

/// 압축되지 않은 HWP3를 legacy DES-ECB password document로 만든다.
///
/// HWP3의 256-byte password confirmation 영역은 format header가 아니라 암호화 payload의
/// 일부다. 새 문서는 0으로 초기화한 영역을 사용하고, 기존 보호 문서의 암호 변경은 먼저
/// 복호화한 뒤 이 함수를 호출한다.
pub fn encrypt_hwp3_password_document(
    input: &[u8],
    password: &[u8],
) -> Result<Vec<u8>, PasswordCryptoError> {
    let layout = parse_hwp3_layout(input, false)?;
    if layout.compressed {
        return Err(PasswordCryptoError::Hwp3UnsupportedUncompressed);
    }
    let password =
        std::str::from_utf8(password).map_err(|_| PasswordCryptoError::Hwp3PasswordEncoding)?;
    let mut plain = vec![0_u8; HWP3_PASSWORD_PREFIX_BYTES];
    plain.extend_from_slice(&input[layout.payload_offset..]);
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(&plain)
        .map_err(|error| PasswordCryptoError::HwpxZip(error.to_string()))?;
    let mut payload = encoder
        .finish()
        .map_err(|error| PasswordCryptoError::HwpxZip(error.to_string()))?;
    payload.extend_from_slice(&crc32fast::hash(&plain).to_le_bytes());
    payload.extend_from_slice(
        &u32::try_from(plain.len())
            .map_err(|_| PasswordCryptoError::DecompressedLimitExceeded {
                max_bytes: u32::MAX as usize,
            })?
            .to_le_bytes(),
    );
    payload.resize((payload.len() + 7) & !7, 0);
    hwp3_des_ecb(&mut payload, &derive_hwp3_legacy_des_key(password), true);

    let mut output = input[..layout.payload_offset].to_vec();
    output[HWP3_PASSWORD_FLAG_OFFSET..HWP3_PASSWORD_FLAG_OFFSET + 2]
        .copy_from_slice(&2_u16.to_le_bytes());
    output[HWP3_COMPRESSION_FLAG_OFFSET] = 1;
    output.extend_from_slice(&payload);
    Ok(output)
}

#[derive(Clone, Copy)]
enum HwpxPbkdf2Prf {
    HmacSha1,
    HmacSha256,
}

#[derive(Debug)]
struct HwpxEntryCrypto {
    path: String,
    checksum: Vec<u8>,
    iv: Vec<u8>,
    salt: Vec<u8>,
    iterations: u32,
    key_size: usize,
}

fn hwpx_attribute(node: Node<'_, '_>, local: &str) -> Option<String> {
    node.attributes()
        .find(|attribute| attribute.name().rsplit(':').next() == Some(local))
        .map(|attribute| attribute.value().to_string())
}

fn hwpx_child<'a, 'input>(node: Node<'a, 'input>, local: &str) -> Option<Node<'a, 'input>> {
    node.children()
        .find(|candidate| candidate.is_element() && candidate.tag_name().name() == local)
}

fn hwpx_base64(value: Option<String>, field: &str) -> Result<Vec<u8>, PasswordCryptoError> {
    let value = value.ok_or_else(|| {
        PasswordCryptoError::HwpxUnsupported(format!("manifest {field} 값이 없습니다"))
    })?;
    base64::engine::general_purpose::STANDARD
        .decode(value)
        .map_err(|_| {
            PasswordCryptoError::HwpxUnsupported(format!("manifest {field} 값이 base64가 아닙니다"))
        })
}

fn parse_hwpx_manifest(data: &[u8]) -> Result<Vec<HwpxEntryCrypto>, PasswordCryptoError> {
    let text = std::str::from_utf8(data).map_err(|_| {
        PasswordCryptoError::HwpxUnsupported("manifest.xml이 UTF-8이 아닙니다".to_string())
    })?;
    let document = XmlDocument::parse(text).map_err(|_| {
        PasswordCryptoError::HwpxUnsupported("manifest.xml 파싱에 실패했습니다".to_string())
    })?;
    let mut entries = Vec::new();
    let mut paths = HashSet::new();

    for file_entry in document
        .descendants()
        .filter(|node| node.is_element() && node.tag_name().name() == "file-entry")
    {
        let Some(encryption) = hwpx_child(file_entry, "encryption-data") else {
            continue;
        };
        let algorithm = hwpx_child(encryption, "algorithm").ok_or_else(|| {
            PasswordCryptoError::HwpxUnsupported(
                "manifest 암호 알고리즘 정보가 없습니다".to_string(),
            )
        })?;
        let derivation = hwpx_child(encryption, "key-derivation").ok_or_else(|| {
            PasswordCryptoError::HwpxUnsupported("manifest 키 파생 정보가 없습니다".to_string())
        })?;
        let start_key = hwpx_child(encryption, "start-key-generation").ok_or_else(|| {
            PasswordCryptoError::HwpxUnsupported("manifest 시작 키 정보가 없습니다".to_string())
        })?;
        let path = hwpx_attribute(file_entry, "full-path").ok_or_else(|| {
            PasswordCryptoError::HwpxUnsupported("manifest 암호화 경로가 없습니다".to_string())
        })?;
        if path.is_empty() || !paths.insert(path.clone()) {
            return Err(PasswordCryptoError::HwpxUnsupported(
                "manifest 암호화 경로가 비어 있거나 중복됩니다".to_string(),
            ));
        }
        let iterations = hwpx_attribute(derivation, "iteration-count")
            .ok_or_else(|| {
                PasswordCryptoError::HwpxUnsupported("manifest 반복 횟수가 없습니다".to_string())
            })?
            .parse::<u32>()
            .map_err(|_| {
                PasswordCryptoError::HwpxUnsupported(
                    "manifest 반복 횟수가 올바르지 않습니다".to_string(),
                )
            })?;
        let key_size = hwpx_attribute(derivation, "key-size")
            .ok_or_else(|| {
                PasswordCryptoError::HwpxUnsupported("manifest 키 크기가 없습니다".to_string())
            })?
            .parse::<usize>()
            .map_err(|_| {
                PasswordCryptoError::HwpxUnsupported(
                    "manifest 키 크기가 올바르지 않습니다".to_string(),
                )
            })?;
        let checksum_type = hwpx_attribute(encryption, "checksum-type").unwrap_or_default();
        let algorithm_name = hwpx_attribute(algorithm, "algorithm-name").unwrap_or_default();
        let start_key_algorithm =
            hwpx_attribute(start_key, "start-key-generation-name").unwrap_or_default();
        let checksum = hwpx_base64(hwpx_attribute(encryption, "checksum"), "checksum")?;
        let iv = hwpx_base64(
            hwpx_attribute(algorithm, "initialisation-vector"),
            "initialisation-vector",
        )?;
        let salt = hwpx_base64(hwpx_attribute(derivation, "salt"), "salt")?;

        if algorithm_name != HWPX_AES_256_CBC
            || key_size != 32
            || iv.len() != 16
            || salt.is_empty()
            || iterations == 0
            || iterations > HWPX_MAX_PBKDF2_ITERATIONS
            || start_key_algorithm != HWPX_SHA_256_START_KEY
            || !checksum_type.ends_with(HWPX_SHA_256_1K_SUFFIX)
            || checksum.len() != 32
        {
            return Err(PasswordCryptoError::HwpxUnsupported(
                "AES-256-CBC / SHA-256 / PBKDF2 ODF 계약과 다릅니다".to_string(),
            ));
        }
        entries.push(HwpxEntryCrypto {
            path,
            checksum,
            iv,
            salt,
            iterations,
            key_size,
        });
    }
    Ok(entries)
}

fn derive_hwpx_key(
    prf: HwpxPbkdf2Prf,
    password: &[u8],
    entry: &HwpxEntryCrypto,
) -> Result<Vec<u8>, PasswordCryptoError> {
    let start_key = Sha256::digest(password);
    let mut key = vec![0_u8; entry.key_size];
    let result = match prf {
        HwpxPbkdf2Prf::HmacSha1 => {
            pbkdf2::<HmacSha1>(&start_key, &entry.salt, entry.iterations, &mut key)
        }
        HwpxPbkdf2Prf::HmacSha256 => {
            pbkdf2::<HmacSha256>(&start_key, &entry.salt, entry.iterations, &mut key)
        }
    };
    result.map_err(|_| {
        PasswordCryptoError::HwpxUnsupported("PBKDF2 키 유도에 실패했습니다".to_string())
    })?;
    Ok(key)
}

fn hwpx_plaintext_limit(path: &str) -> usize {
    if path.to_ascii_lowercase().starts_with("bindata/") {
        HWPX_MAX_BINDATA_SIZE
    } else {
        HWPX_MAX_XML_SIZE
    }
}

fn inflate_hwpx_raw_deflate(data: &[u8], path: &str) -> Result<Vec<u8>, PasswordCryptoError> {
    let max_bytes = hwpx_plaintext_limit(path);
    let mut plaintext = Vec::new();
    DeflateDecoder::new(Cursor::new(data))
        .take((max_bytes as u64).saturating_add(1))
        .read_to_end(&mut plaintext)
        .map_err(|_| PasswordCryptoError::WrongPasswordOrCorruptPayload)?;
    if plaintext.len() > max_bytes {
        return Err(PasswordCryptoError::HwpxEntryLimitExceeded {
            path: path.to_string(),
            max_bytes,
        });
    }
    Ok(plaintext)
}

fn decrypt_hwpx_entry(
    password: &[u8],
    entry: &HwpxEntryCrypto,
    ciphertext: &[u8],
) -> Result<Vec<u8>, PasswordCryptoError> {
    if ciphertext.is_empty() || !ciphertext.len().is_multiple_of(16) {
        return Err(PasswordCryptoError::WrongPasswordOrCorruptPayload);
    }
    for prf in [HwpxPbkdf2Prf::HmacSha1, HwpxPbkdf2Prf::HmacSha256] {
        let key = derive_hwpx_key(prf, password, entry)?;
        let mut blocks = ciphertext.to_vec();
        let Ok(cipher) = Decryptor::<Aes256>::new_from_slices(&key, &entry.iv) else {
            continue;
        };
        let Ok(decrypted) = cipher.decrypt_padded::<NoPadding>(&mut blocks) else {
            continue;
        };
        let plaintext = match inflate_hwpx_raw_deflate(decrypted, &entry.path) {
            Ok(plaintext) => plaintext,
            Err(error @ PasswordCryptoError::HwpxEntryLimitExceeded { .. }) => return Err(error),
            Err(_) => continue,
        };
        let checksum = Sha256::digest(&plaintext[..plaintext.len().min(1024)]);
        if checksum.as_slice() == entry.checksum.as_slice() {
            return Ok(plaintext);
        }
    }
    Err(PasswordCryptoError::WrongPasswordOrCorruptPayload)
}

fn hwpx_local_name(name: &[u8]) -> &[u8] {
    name.rsplit(|byte| *byte == b':').next().unwrap_or(name)
}

fn strip_hwpx_encryption_data(manifest: &[u8]) -> Result<Vec<u8>, PasswordCryptoError> {
    let mut reader = Reader::from_reader(manifest);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new(Vec::new());
    let mut buffer = Vec::new();
    let mut skipped_depth = 0_u32;

    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Eof) => break,
            Ok(Event::Start(event))
                if hwpx_local_name(event.name().as_ref()) == b"encryption-data" =>
            {
                skipped_depth = 1;
            }
            Ok(Event::Start(_)) if skipped_depth > 0 => skipped_depth += 1,
            Ok(Event::End(_)) if skipped_depth > 1 => skipped_depth -= 1,
            Ok(Event::End(_)) if skipped_depth == 1 => skipped_depth = 0,
            Ok(Event::Empty(event)) if skipped_depth == 0 => writer
                .write_event(Event::Empty(event.into_owned()))
                .map_err(|_| {
                    PasswordCryptoError::HwpxXml("manifest 암호화 정보 제거 실패".to_string())
                })?,
            Ok(event) if skipped_depth == 0 => {
                writer.write_event(event.into_owned()).map_err(|_| {
                    PasswordCryptoError::HwpxXml("manifest 암호화 정보 제거 실패".to_string())
                })?
            }
            Ok(_) => {}
            Err(_) => {
                return Err(PasswordCryptoError::HwpxXml(
                    "manifest 암호화 정보 제거 중 XML 오류".to_string(),
                ));
            }
        }
        buffer.clear();
    }
    Ok(writer.into_inner())
}

fn read_zip_entry_limited<R: Read>(
    reader: &mut R,
    path: &str,
    max_bytes: usize,
) -> Result<Vec<u8>, PasswordCryptoError> {
    let mut data = Vec::new();
    reader
        .take((max_bytes as u64).saturating_add(1))
        .read_to_end(&mut data)
        .map_err(|error| PasswordCryptoError::HwpxZip(format!("{path} 읽기 실패: {error}")))?;
    if data.len() > max_bytes {
        return Err(PasswordCryptoError::HwpxEntryLimitExceeded {
            path: path.to_string(),
            max_bytes,
        });
    }
    Ok(data)
}

/// 암호 HWPX만 평문 ZIP으로 복원한다. 평문 HWPX는 `None`을 반환한다.
pub fn decrypt_hwpx_package(
    data: &[u8],
    password: &[u8],
) -> Result<Option<Vec<u8>>, PasswordCryptoError> {
    let mut source = ZipArchive::new(Cursor::new(data.to_vec()))
        .map_err(|error| PasswordCryptoError::HwpxZip(error.to_string()))?;
    let manifest = match source.by_name(HWPX_MANIFEST_PATH) {
        Ok(mut file) => read_zip_entry_limited(&mut file, HWPX_MANIFEST_PATH, HWPX_MAX_XML_SIZE)?,
        Err(zip::result::ZipError::FileNotFound) => return Ok(None),
        Err(error) => return Err(PasswordCryptoError::HwpxZip(error.to_string())),
    };
    let protected = parse_hwpx_manifest(&manifest)?;
    if protected.is_empty() {
        return Ok(None);
    }
    let protected_by_path: HashMap<&str, &HwpxEntryCrypto> = protected
        .iter()
        .map(|entry| (entry.path.as_str(), entry))
        .collect();
    let plain_manifest = strip_hwpx_encryption_data(&manifest)?;
    let mut selected = HashSet::new();
    let mut destination = ZipWriter::new(Cursor::new(Vec::new()));

    for index in 0..source.len() {
        let mut input = source
            .by_index(index)
            .map_err(|error| PasswordCryptoError::HwpxZip(error.to_string()))?;
        let name = input.name().to_string();
        if input.is_dir() {
            destination
                .add_directory(name, SimpleFileOptions::default())
                .map_err(|error| PasswordCryptoError::HwpxZip(error.to_string()))?;
            continue;
        }
        let bytes = read_zip_entry_limited(&mut input, &name, HWPX_MAX_BINDATA_SIZE)?;
        let source_method = input.compression();
        let (payload, method) = if name == HWPX_MANIFEST_PATH {
            (plain_manifest.clone(), CompressionMethod::Deflated)
        } else if let Some(entry) = protected_by_path.get(name.as_str()) {
            selected.insert(entry.path.as_str());
            (
                decrypt_hwpx_entry(password, entry, &bytes)?,
                CompressionMethod::Deflated,
            )
        } else {
            (bytes, source_method)
        };
        destination
            .start_file(
                name,
                SimpleFileOptions::default().compression_method(method),
            )
            .map_err(|error| PasswordCryptoError::HwpxZip(error.to_string()))?;
        destination
            .write_all(&payload)
            .map_err(|error| PasswordCryptoError::HwpxZip(error.to_string()))?;
    }

    if selected.len() != protected.len() {
        let missing = protected
            .iter()
            .find(|entry| !selected.contains(entry.path.as_str()))
            .map(|entry| entry.path.clone())
            .unwrap_or_else(|| "알 수 없는 경로".to_string());
        return Err(PasswordCryptoError::HwpxMissingEntry(missing));
    }
    destination
        .finish()
        .map(|cursor| Some(cursor.into_inner()))
        .map_err(|error| PasswordCryptoError::HwpxZip(error.to_string()))
}

struct HwpxZipEntry {
    name: String,
    data: Vec<u8>,
    method: CompressionMethod,
    is_dir: bool,
}

struct HwpxEncryptedEntry {
    path: String,
    checksum: Vec<u8>,
    iv: [u8; 16],
    salt: [u8; 16],
    size: usize,
}

fn is_hwpx_protected_path(path: &str) -> bool {
    path.starts_with("BinData/")
        // [#3546] 차트 XML 은 문서 내용(데이터 값·라벨)이다. 종전에는
        // BinData/*.ooxml_chart 로 방출돼 위 분기로 암호화됐으나, 원형
        // Chart/chartN.xml 방출로 바뀌며 평문으로 남지 않도록 포함한다.
        // 복호 측은 암호화 manifest 기반이라 왕복은 대칭이다.
        || path.starts_with("Chart/")
        || path.starts_with("Preview/")
        || path == "settings.xml"
        || path == "Contents/header.xml"
        || (path.starts_with("Contents/")
            && path.ends_with(".xml")
            && path != "Contents/content.hpf")
}

fn hwpx_random_16() -> Result<[u8; 16], PasswordCryptoError> {
    let mut output = [0_u8; 16];
    getrandom::fill(&mut output).map_err(|error| PasswordCryptoError::Random(error.to_string()))?;
    Ok(output)
}

fn encrypt_hwpx_entry(
    path: &str,
    plaintext: &[u8],
    password: &[u8],
) -> Result<(Vec<u8>, HwpxEncryptedEntry), PasswordCryptoError> {
    let iv = hwpx_random_16()?;
    let salt = hwpx_random_16()?;
    let entry = HwpxEntryCrypto {
        path: path.to_string(),
        checksum: Sha256::digest(&plaintext[..plaintext.len().min(1024)]).to_vec(),
        iv: iv.to_vec(),
        salt: salt.to_vec(),
        iterations: 1024,
        key_size: 32,
    };
    let key = derive_hwpx_key(HwpxPbkdf2Prf::HmacSha1, password, &entry)?;
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(plaintext)
        .map_err(|error| PasswordCryptoError::HwpxZip(error.to_string()))?;
    let mut compressed = encoder
        .finish()
        .map_err(|error| PasswordCryptoError::HwpxZip(error.to_string()))?;
    compressed.resize((compressed.len() + 15) & !15, 0);
    let len = compressed.len();
    let ciphertext = Encryptor::<Aes256>::new_from_slices(&key, &iv)
        .map_err(|_| PasswordCryptoError::HwpxUnsupported("AES-256-CBC 초기화 실패".to_string()))?
        .encrypt_padded::<NoPadding>(&mut compressed, len)
        .map_err(|_| PasswordCryptoError::HwpxUnsupported("AES-256-CBC 암호화 실패".to_string()))?
        .to_vec();
    Ok((
        ciphertext,
        HwpxEncryptedEntry {
            path: path.to_string(),
            checksum: entry.checksum,
            iv,
            salt,
            size: plaintext.len(),
        },
    ))
}

fn xml_attribute(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn hwpx_media_type(path: &str) -> &'static str {
    let lowercase = path.to_ascii_lowercase();
    if lowercase.ends_with(".png") {
        "image/png"
    } else if lowercase.ends_with(".bmp") {
        "image/bmp"
    } else if lowercase.ends_with(".jpg") || lowercase.ends_with(".jpeg") {
        "image/jpeg"
    } else if lowercase.ends_with(".xml") {
        "application/xml"
    } else if lowercase.ends_with(".txt") {
        "text/xml"
    } else {
        "application/octet-stream"
    }
}

fn build_hwpx_manifest(entries: &[HwpxEncryptedEntry]) -> String {
    let mut output = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes" ?><odf:manifest xmlns:odf="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0">"#,
    );
    for entry in entries {
        let checksum = base64::engine::general_purpose::STANDARD.encode(&entry.checksum);
        let iv = base64::engine::general_purpose::STANDARD.encode(entry.iv);
        let salt = base64::engine::general_purpose::STANDARD.encode(entry.salt);
        let _ = write!(
            output,
            r#"<odf:file-entry full-path="{}" media-type="{}" size="{}"><odf:encryption-data checksum-type="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0#sha256-1k" checksum="{}"><odf:algorithm algorithm-name="{}" initialisation-vector="{}"/><odf:key-derivation key-derivation-name="{}" key-size="32" iteration-count="1024" salt="{}"/><odf:start-key-generation start-key-generation-name="{}" key-size="32"/></odf:encryption-data></odf:file-entry>"#,
            xml_attribute(&entry.path),
            hwpx_media_type(&entry.path),
            entry.size,
            checksum,
            HWPX_AES_256_CBC,
            iv,
            HWPX_PBKDF2_NAME,
            salt,
            HWPX_SHA_256_START_KEY,
        );
    }
    output.push_str("</odf:manifest>");
    output
}

/// 평문 HWPX ZIP을 한컴 ODF AES-256-CBC password package로 만든다.
pub fn encrypt_hwpx_package(data: &[u8], password: &[u8]) -> Result<Vec<u8>, PasswordCryptoError> {
    let mut source = ZipArchive::new(Cursor::new(data.to_vec()))
        .map_err(|error| PasswordCryptoError::HwpxZip(error.to_string()))?;
    let mut entries = Vec::with_capacity(source.len());
    let mut manifest = None;

    for index in 0..source.len() {
        let mut input = source
            .by_index(index)
            .map_err(|error| PasswordCryptoError::HwpxZip(error.to_string()))?;
        let name = input.name().to_string();
        let is_dir = input.is_dir();
        let data = if is_dir {
            Vec::new()
        } else {
            read_zip_entry_limited(&mut input, &name, HWPX_MAX_BINDATA_SIZE)?
        };
        if name == HWPX_MANIFEST_PATH {
            manifest = Some(data.clone());
        }
        entries.push(HwpxZipEntry {
            name,
            data,
            method: input.compression(),
            is_dir,
        });
    }
    let manifest = manifest
        .ok_or_else(|| PasswordCryptoError::HwpxMissingEntry(HWPX_MANIFEST_PATH.to_string()))?;
    if !parse_hwpx_manifest(&manifest)?.is_empty() {
        return Err(PasswordCryptoError::HwpxUnsupported(
            "이미 암호화된 HWPX의 암호 변경은 지원하지 않습니다".to_string(),
        ));
    }

    let mut encrypted_entries = Vec::new();
    for entry in &mut entries {
        if is_hwpx_protected_path(&entry.name) {
            let (ciphertext, metadata) = encrypt_hwpx_entry(&entry.name, &entry.data, password)?;
            entry.data = ciphertext;
            entry.method = CompressionMethod::Stored;
            encrypted_entries.push(metadata);
        }
    }
    if encrypted_entries.is_empty() {
        return Err(PasswordCryptoError::HwpxUnsupported(
            "암호화할 HWPX 문서 항목이 없습니다".to_string(),
        ));
    }
    let encrypted_manifest = build_hwpx_manifest(&encrypted_entries).into_bytes();
    let mut destination = ZipWriter::new(Cursor::new(Vec::new()));
    for entry in entries {
        if entry.is_dir {
            destination
                .add_directory(entry.name, SimpleFileOptions::default())
                .map_err(|error| PasswordCryptoError::HwpxZip(error.to_string()))?;
            continue;
        }
        let (data, method) = if entry.name == HWPX_MANIFEST_PATH {
            (encrypted_manifest.as_slice(), CompressionMethod::Deflated)
        } else {
            (entry.data.as_slice(), entry.method)
        };
        destination
            .start_file(
                entry.name,
                SimpleFileOptions::default().compression_method(method),
            )
            .map_err(|error| PasswordCryptoError::HwpxZip(error.to_string()))?;
        destination
            .write_all(data)
            .map_err(|error| PasswordCryptoError::HwpxZip(error.to_string()))?;
    }
    destination
        .finish()
        .map(|cursor| cursor.into_inner())
        .map_err(|error| PasswordCryptoError::HwpxZip(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hwp5_stream_roundtrip_preserves_partial_block() {
        for plaintext in [b"HWP5 password".as_slice(), b"0123456789abcdef".as_slice()] {
            let ciphertext = encrypt_hwp5_stream(plaintext, b"password");
            assert_ne!(ciphertext, plaintext);
            assert_eq!(decrypt_hwp5_stream(&ciphertext, b"password"), plaintext);
        }
    }

    #[test]
    fn hwp5_decrypt_matches_external_vector() {
        let ciphertext: Vec<u8> = (0_u8..32).collect();
        assert_eq!(
            decrypt_hwp5_stream(&ciphertext, b"helloworld"),
            [
                0x00, 0x01, 0x3e, 0xec, 0x90, 0x3d, 0xbc, 0x26, 0xfa, 0xff, 0x9c, 0x6c, 0xfb, 0x35,
                0x48, 0x00, 0xbc, 0xaa, 0x14, 0x7b, 0x0e, 0xd1, 0x5c, 0x32, 0x21, 0x17, 0x37, 0xfa,
                0x97, 0x1d, 0xe3, 0x79,
            ]
        );
    }
}
