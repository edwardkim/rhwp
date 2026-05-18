//! HWP 배포용 문서 복호화
//!
//! ViewText 스트림 복호화 흐름:
//! 1. ViewText/Section{N} 원본 읽기
//! 2. 첫 번째 레코드(DISTRIBUTE_DOC_DATA, 256바이트) 파싱
//! 3. LCG + XOR로 256바이트 복호화
//! 4. 복호화된 데이터에서 AES-128 키 추출
//! 5. 나머지 데이터를 AES-128 ECB로 복호화
//! 6. zlib/deflate 압축 해제
//!
//! 참조: /home/edward/vsworks/shwp/hwp_semantic/crypto.py

use super::cfb_reader::decompress_stream;
use super::record::Record;
use super::tags;

/// 배포용 문서 복호화 에러
#[derive(Debug)]
pub enum CryptoError {
    /// DISTRIBUTE_DOC_DATA 레코드 없음
    NoDistributeData,
    /// 페이로드 크기 오류
    InvalidPayloadSize(usize),
    /// AES 키 추출 실패
    KeyExtractionFailed(String),
    /// 복호화 실패
    DecryptionFailed(String),
    /// 레코드 파싱 실패
    RecordError(String),
    /// 압축 해제 실패
    DecompressError(String),
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoError::NoDistributeData => write!(f, "DISTRIBUTE_DOC_DATA 레코드 없음"),
            CryptoError::InvalidPayloadSize(s) => {
                write!(f, "DISTRIBUTE_DOC_DATA 크기 오류: {}바이트 (필요: 256)", s)
            }
            CryptoError::KeyExtractionFailed(e) => write!(f, "AES 키 추출 실패: {}", e),
            CryptoError::DecryptionFailed(e) => write!(f, "복호화 실패: {}", e),
            CryptoError::RecordError(e) => write!(f, "레코드 파싱 실패: {}", e),
            CryptoError::DecompressError(e) => write!(f, "압축 해제 실패: {}", e),
        }
    }
}

impl std::error::Error for CryptoError {}

// ============================================================
// MSVC LCG (Linear Congruential Generator)
// ============================================================

/// MSVC srand()/rand() 호환 난수 생성기
struct MsvcLcg {
    seed: u32,
}

impl MsvcLcg {
    fn new(seed: u32) -> Self {
        MsvcLcg { seed }
    }

    /// 다음 난수 생성 (0 ~ 32767)
    fn rand(&mut self) -> u32 {
        self.seed = self.seed.wrapping_mul(214013).wrapping_add(2531011);
        (self.seed >> 16) & 0x7FFF
    }
}

// ============================================================
// DISTRIBUTE_DOC_DATA 복호화
// ============================================================

/// DISTRIBUTE_DOC_DATA 256바이트 페이로드 복호화 (LCG + XOR)
fn decrypt_distribute_doc_data(data: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if data.len() < 256 {
        return Err(CryptoError::InvalidPayloadSize(data.len()));
    }

    let mut result = data[..256].to_vec();

    // 첫 4바이트를 시드로 사용
    let seed = u32::from_le_bytes([result[0], result[1], result[2], result[3]]);
    let mut lcg = MsvcLcg::new(seed);

    // XOR 복호화
    let mut i = 0usize;
    let mut n = 0u32;
    let mut key = 0u8;

    while i < 256 {
        if n == 0 {
            key = (lcg.rand() & 0xFF) as u8;
            n = (lcg.rand() & 0xF) + 1;
        }
        if i >= 4 {
            result[i] ^= key;
        }
        i += 1;
        n -= 1;
    }

    Ok(result)
}

/// 복호화된 DISTRIBUTE_DOC_DATA에서 AES-128 키 추출 (16바이트)
fn extract_aes_key(decrypted_data: &[u8]) -> Result<[u8; 16], CryptoError> {
    if decrypted_data.len() < 256 {
        return Err(CryptoError::KeyExtractionFailed(
            "데이터가 256바이트 미만".to_string(),
        ));
    }

    let offset = 4 + (decrypted_data[0] & 0xF) as usize;

    if offset + 16 > decrypted_data.len() {
        return Err(CryptoError::KeyExtractionFailed(format!(
            "오프셋 {}에서 16바이트 부족",
            offset
        )));
    }

    let mut key = [0u8; 16];
    key.copy_from_slice(&decrypted_data[offset..offset + 16]);
    Ok(key)
}

// ============================================================
// AES-128 ECB 복호화 (순수 Rust 구현)
// ============================================================

/// AES S-Box
#[rustfmt::skip]
const S_BOX: [u8; 256] = [
    0x63,0x7c,0x77,0x7b,0xf2,0x6b,0x6f,0xc5,0x30,0x01,0x67,0x2b,0xfe,0xd7,0xab,0x76,
    0xca,0x82,0xc9,0x7d,0xfa,0x59,0x47,0xf0,0xad,0xd4,0xa2,0xaf,0x9c,0xa4,0x72,0xc0,
    0xb7,0xfd,0x93,0x26,0x36,0x3f,0xf7,0xcc,0x34,0xa5,0xe5,0xf1,0x71,0xd8,0x31,0x15,
    0x04,0xc7,0x23,0xc3,0x18,0x96,0x05,0x9a,0x07,0x12,0x80,0xe2,0xeb,0x27,0xb2,0x75,
    0x09,0x83,0x2c,0x1a,0x1b,0x6e,0x5a,0xa0,0x52,0x3b,0xd6,0xb3,0x29,0xe3,0x2f,0x84,
    0x53,0xd1,0x00,0xed,0x20,0xfc,0xb1,0x5b,0x6a,0xcb,0xbe,0x39,0x4a,0x4c,0x58,0xcf,
    0xd0,0xef,0xaa,0xfb,0x43,0x4d,0x33,0x85,0x45,0xf9,0x02,0x7f,0x50,0x3c,0x9f,0xa8,
    0x51,0xa3,0x40,0x8f,0x92,0x9d,0x38,0xf5,0xbc,0xb6,0xda,0x21,0x10,0xff,0xf3,0xd2,
    0xcd,0x0c,0x13,0xec,0x5f,0x97,0x44,0x17,0xc4,0xa7,0x7e,0x3d,0x64,0x5d,0x19,0x73,
    0x60,0x81,0x4f,0xdc,0x22,0x2a,0x90,0x88,0x46,0xee,0xb8,0x14,0xde,0x5e,0x0b,0xdb,
    0xe0,0x32,0x3a,0x0a,0x49,0x06,0x24,0x5c,0xc2,0xd3,0xac,0x62,0x91,0x95,0xe4,0x79,
    0xe7,0xc8,0x37,0x6d,0x8d,0xd5,0x4e,0xa9,0x6c,0x56,0xf4,0xea,0x65,0x7a,0xae,0x08,
    0xba,0x78,0x25,0x2e,0x1c,0xa6,0xb4,0xc6,0xe8,0xdd,0x74,0x1f,0x4b,0xbd,0x8b,0x8a,
    0x70,0x3e,0xb5,0x66,0x48,0x03,0xf6,0x0e,0x61,0x35,0x57,0xb9,0x86,0xc1,0x1d,0x9e,
    0xe1,0xf8,0x98,0x11,0x69,0xd9,0x8e,0x94,0x9b,0x1e,0x87,0xe9,0xce,0x55,0x28,0xdf,
    0x8c,0xa1,0x89,0x0d,0xbf,0xe6,0x42,0x68,0x41,0x99,0x2d,0x0f,0xb0,0x54,0xbb,0x16,
];

/// AES Inverse S-Box
#[rustfmt::skip]
const INV_S_BOX: [u8; 256] = [
    0x52,0x09,0x6a,0xd5,0x30,0x36,0xa5,0x38,0xbf,0x40,0xa3,0x9e,0x81,0xf3,0xd7,0xfb,
    0x7c,0xe3,0x39,0x82,0x9b,0x2f,0xff,0x87,0x34,0x8e,0x43,0x44,0xc4,0xde,0xe9,0xcb,
    0x54,0x7b,0x94,0x32,0xa6,0xc2,0x23,0x3d,0xee,0x4c,0x95,0x0b,0x42,0xfa,0xc3,0x4e,
    0x08,0x2e,0xa1,0x66,0x28,0xd9,0x24,0xb2,0x76,0x5b,0xa2,0x49,0x6d,0x8b,0xd1,0x25,
    0x72,0xf8,0xf6,0x64,0x86,0x68,0x98,0x16,0xd4,0xa4,0x5c,0xcc,0x5d,0x65,0xb6,0x92,
    0x6c,0x70,0x48,0x50,0xfd,0xed,0xb9,0xda,0x5e,0x15,0x46,0x57,0xa7,0x8d,0x9d,0x84,
    0x90,0xd8,0xab,0x00,0x8c,0xbc,0xd3,0x0a,0xf7,0xe4,0x58,0x05,0xb8,0xb3,0x45,0x06,
    0xd0,0x2c,0x1e,0x8f,0xca,0x3f,0x0f,0x02,0xc1,0xaf,0xbd,0x03,0x01,0x13,0x8a,0x6b,
    0x3a,0x91,0x11,0x41,0x4f,0x67,0xdc,0xea,0x97,0xf2,0xcf,0xce,0xf0,0xb4,0xe6,0x73,
    0x96,0xac,0x74,0x22,0xe7,0xad,0x35,0x85,0xe2,0xf9,0x37,0xe8,0x1c,0x75,0xdf,0x6e,
    0x47,0xf1,0x1a,0x71,0x1d,0x29,0xc5,0x89,0x6f,0xb7,0x62,0x0e,0xaa,0x18,0xbe,0x1b,
    0xfc,0x56,0x3e,0x4b,0xc6,0xd2,0x79,0x20,0x9a,0xdb,0xc0,0xfe,0x78,0xcd,0x5a,0xf4,
    0x1f,0xdd,0xa8,0x33,0x88,0x07,0xc7,0x31,0xb1,0x12,0x10,0x59,0x27,0x80,0xec,0x5f,
    0x60,0x51,0x7f,0xa9,0x19,0xb5,0x4a,0x0d,0x2d,0xe5,0x7a,0x9f,0x93,0xc9,0x9c,0xef,
    0xa0,0xe0,0x3b,0x4d,0xae,0x2a,0xf5,0xb0,0xc8,0xeb,0xbb,0x3c,0x83,0x53,0x99,0x61,
    0x17,0x2b,0x04,0x7e,0xba,0x77,0xd6,0x26,0xe1,0x69,0x14,0x63,0x55,0x21,0x0c,0x7d,
];

/// AES 라운드 상수
const RCON: [u8; 10] = [0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36];

/// GF(2^8) xtime 연산
fn xtime(a: u8) -> u8 {
    if a & 0x80 != 0 {
        ((a as u16) << 1 ^ 0x1b) as u8
    } else {
        a << 1
    }
}

/// GF(2^8) 곱셈
fn gf_multiply(mut a: u8, mut b: u8) -> u8 {
    let mut result = 0u8;
    for _ in 0..8 {
        if b & 1 != 0 {
            result ^= a;
        }
        a = xtime(a);
        b >>= 1;
    }
    result
}

/// AES-128 키 확장 (16바이트 → 176바이트)
fn key_expansion(key: &[u8; 16]) -> Vec<u8> {
    let mut w = key.to_vec();

    for i in 4..44 {
        let start = (i - 1) * 4;
        let mut temp = [w[start], w[start + 1], w[start + 2], w[start + 3]];

        if i % 4 == 0 {
            // RotWord + SubWord + Rcon
            temp = [
                S_BOX[temp[1] as usize] ^ RCON[(i / 4) - 1],
                S_BOX[temp[2] as usize],
                S_BOX[temp[3] as usize],
                S_BOX[temp[0] as usize],
            ];
        }

        let prev_start = (i - 4) * 4;
        for j in 0..4 {
            w.push(w[prev_start + j] ^ temp[j]);
        }
    }

    w
}

/// AES Inverse SubBytes
fn inv_sub_bytes(state: &mut [u8; 16]) {
    for byte in state.iter_mut() {
        *byte = INV_S_BOX[*byte as usize];
    }
}

/// AES Inverse ShiftRows
fn inv_shift_rows(state: &mut [u8; 16]) {
    let s = *state;
    *state = [
        s[0], s[13], s[10], s[7], s[4], s[1], s[14], s[11], s[8], s[5], s[2], s[15], s[12], s[9],
        s[6], s[3],
    ];
}

/// AES Inverse MixColumns
fn inv_mix_columns(state: &mut [u8; 16]) {
    let s = *state;
    for c in 0..4 {
        let i = c * 4;
        state[i] = gf_multiply(0x0e, s[i])
            ^ gf_multiply(0x0b, s[i + 1])
            ^ gf_multiply(0x0d, s[i + 2])
            ^ gf_multiply(0x09, s[i + 3]);
        state[i + 1] = gf_multiply(0x09, s[i])
            ^ gf_multiply(0x0e, s[i + 1])
            ^ gf_multiply(0x0b, s[i + 2])
            ^ gf_multiply(0x0d, s[i + 3]);
        state[i + 2] = gf_multiply(0x0d, s[i])
            ^ gf_multiply(0x09, s[i + 1])
            ^ gf_multiply(0x0e, s[i + 2])
            ^ gf_multiply(0x0b, s[i + 3]);
        state[i + 3] = gf_multiply(0x0b, s[i])
            ^ gf_multiply(0x0d, s[i + 1])
            ^ gf_multiply(0x09, s[i + 2])
            ^ gf_multiply(0x0e, s[i + 3]);
    }
}

/// AES AddRoundKey
fn add_round_key(state: &mut [u8; 16], round_key: &[u8]) {
    for i in 0..16 {
        state[i] ^= round_key[i];
    }
}

/// AES-128 ECB 단일 블록 복호화 (16바이트)
fn decrypt_block(block: &[u8; 16], expanded_key: &[u8]) -> [u8; 16] {
    let mut state = *block;

    // Initial round key addition (round 10)
    add_round_key(&mut state, &expanded_key[160..176]);

    // 9 main rounds (round 9 → 1)
    for round in (1..=9).rev() {
        inv_shift_rows(&mut state);
        inv_sub_bytes(&mut state);
        add_round_key(&mut state, &expanded_key[round * 16..(round + 1) * 16]);
        inv_mix_columns(&mut state);
    }

    // Final round (round 0)
    inv_shift_rows(&mut state);
    inv_sub_bytes(&mut state);
    add_round_key(&mut state, &expanded_key[0..16]);

    state
}

/// AES-128 ECB 복호화
fn decrypt_aes_ecb(data: &[u8], key: &[u8; 16]) -> Vec<u8> {
    let expanded_key = key_expansion(key);
    let mut result = Vec::with_capacity(data.len());

    for chunk in data.chunks(16) {
        let mut block = [0u8; 16];
        let len = chunk.len().min(16);
        block[..len].copy_from_slice(&chunk[..len]);

        let decrypted = decrypt_block(&block, &expanded_key);
        result.extend_from_slice(&decrypted);
    }

    result
}

// ============================================================
// ViewText 섹션 복호화 (공개 API)
// ============================================================

/// ViewText 섹션 데이터 복호화
///
/// ViewText/Section{N} 원본 데이터를 받아:
/// 1. 첫 번째 레코드(DISTRIBUTE_DOC_DATA)에서 키 추출
/// 2. 나머지 데이터를 AES-128 ECB 복호화
/// 3. 압축 해제 (compressed=true일 때)
///
/// 반환값: 압축 해제된 레코드 데이터 (BodyText와 동일한 레코드 구조)
pub fn decrypt_viewtext_section(
    section_data: &[u8],
    compressed: bool,
) -> Result<Vec<u8>, CryptoError> {
    // 첫 번째 레코드만 파싱 (DISTRIBUTE_DOC_DATA)
    // 주의: Record::read_all을 사용하면 안 됨!
    // ViewText 섹션은 [DISTRIBUTE_DOC_DATA 레코드] + [AES 암호문] 구조이므로
    // 암호문 부분을 레코드로 파싱하면 실패한다.
    let first =
        read_first_record(section_data).map_err(|e| CryptoError::RecordError(e.to_string()))?;

    // DISTRIBUTE_DOC_DATA 확인
    if first.tag_id != tags::HWPTAG_DISTRIBUTE_DOC_DATA {
        return Err(CryptoError::NoDistributeData);
    }

    if first.data.len() != 256 {
        return Err(CryptoError::InvalidPayloadSize(first.data.len()));
    }

    // 256바이트 복호화 (LCG + XOR)
    let decrypted_header = decrypt_distribute_doc_data(&first.data)?;

    // AES 키 추출
    let aes_key = extract_aes_key(&decrypted_header)?;

    // 암호화된 본문 위치 계산
    // 레코드 헤더: 4바이트 (+ 확장 4바이트)
    let record_header_size = if first.size >= 0xFFF { 8 } else { 4 };
    let encrypted_start = record_header_size + first.size as usize;

    if section_data.len() <= encrypted_start {
        return Err(CryptoError::DecryptionFailed(
            "암호화된 본문 데이터 없음".to_string(),
        ));
    }

    let encrypted_body = &section_data[encrypted_start..];

    // AES-128 ECB 복호화
    let decrypted_body = decrypt_aes_ecb(encrypted_body, &aes_key);

    // 압축 해제
    if compressed {
        decompress_stream(&decrypted_body).map_err(|e| CryptoError::DecompressError(e.to_string()))
    } else {
        Ok(decrypted_body)
    }
}

/// 바이트 스트림에서 첫 번째 레코드만 파싱
fn read_first_record(data: &[u8]) -> Result<Record, String> {
    use byteorder::{LittleEndian, ReadBytesExt};
    use std::io::{Cursor, Read};

    if data.len() < 4 {
        return Err("데이터가 4바이트 미만".to_string());
    }

    let mut cursor = Cursor::new(data);
    let header = cursor
        .read_u32::<LittleEndian>()
        .map_err(|e| e.to_string())?;

    let tag_id = (header & 0x3FF) as u16;
    let level = ((header >> 10) & 0x3FF) as u16;
    let mut size = (header >> 20) as u32;

    if size == 0xFFF {
        size = cursor
            .read_u32::<LittleEndian>()
            .map_err(|e| e.to_string())?;
    }

    let pos = cursor.position() as usize;
    if pos + size as usize > data.len() {
        return Err(format!(
            "레코드 데이터 부족: tag={}, 필요={}, 가용={}",
            tag_id,
            size,
            data.len() - pos
        ));
    }

    let mut record_data = vec![0u8; size as usize];
    cursor
        .read_exact(&mut record_data)
        .map_err(|e| e.to_string())?;

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
    fn test_msvc_lcg() {
        let mut lcg = MsvcLcg::new(0);
        // MSVC rand() 결과 시퀀스 (시드 0)
        let first = lcg.rand();
        let second = lcg.rand();
        // 값이 0~32767 범위인지 확인
        assert!(first <= 0x7FFF);
        assert!(second <= 0x7FFF);
        // 서로 다른 값 생성
        assert_ne!(first, second);
    }

    #[test]
    fn test_lcg_deterministic() {
        let mut lcg1 = MsvcLcg::new(12345);
        let mut lcg2 = MsvcLcg::new(12345);
        // 같은 시드면 같은 시퀀스
        for _ in 0..10 {
            assert_eq!(lcg1.rand(), lcg2.rand());
        }
    }

    #[test]
    fn test_decrypt_distribute_doc_data() {
        // 256바이트 테스트 데이터
        let mut data = vec![0u8; 256];
        // 시드 = 0x00000001
        data[0] = 1;
        data[1] = 0;
        data[2] = 0;
        data[3] = 0;

        let result = decrypt_distribute_doc_data(&data).unwrap();
        assert_eq!(result.len(), 256);
        // 첫 4바이트는 변경 안됨 (시드)
        assert_eq!(result[0], 1);
        assert_eq!(result[1], 0);
        assert_eq!(result[2], 0);
        assert_eq!(result[3], 0);
    }

    #[test]
    fn test_decrypt_distribute_too_short() {
        let data = vec![0u8; 100];
        let result = decrypt_distribute_doc_data(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_aes_key() {
        let mut data = vec![0x42u8; 256];
        data[0] = 0x03; // offset = 4 + (0x03 & 0xF) = 7
                        // key는 data[7..23]

        let key = extract_aes_key(&data).unwrap();
        assert_eq!(key.len(), 16);
        assert_eq!(key, [0x42; 16]);
    }

    #[test]
    fn test_extract_aes_key_offset_0() {
        let mut data = vec![0xAB; 256];
        data[0] = 0x00; // offset = 4 + 0 = 4
        data[4..20].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);

        let key = extract_aes_key(&data).unwrap();
        assert_eq!(key, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    }

    #[test]
    fn test_aes_encrypt_decrypt_roundtrip() {
        // AES-128 ECB: 암호화 후 복호화 하면 원본 복원
        let key = [
            0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf,
            0x4f, 0x3c,
        ];
        let plaintext = [
            0x32, 0x43, 0xf6, 0xa8, 0x88, 0x5a, 0x30, 0x8d, 0x31, 0x31, 0x98, 0xa2, 0xe0, 0x37,
            0x07, 0x34,
        ];

        // NIST AES-128 테스트 벡터의 암호문
        let expected_ciphertext = [
            0x39, 0x25, 0x84, 0x1d, 0x02, 0xdc, 0x09, 0xfb, 0xdc, 0x11, 0x85, 0x97, 0x19, 0x6a,
            0x0b, 0x32,
        ];

        let decrypted = decrypt_aes_ecb(&expected_ciphertext, &key);
        assert_eq!(&decrypted[..16], &plaintext);
    }

    #[test]
    fn test_aes_key_expansion_length() {
        let key = [0u8; 16];
        let expanded = key_expansion(&key);
        // AES-128: 44 words × 4 bytes = 176 bytes
        assert_eq!(expanded.len(), 176);
    }

    #[test]
    fn test_gf_multiply() {
        // GF(2^8) 곱셈 검증
        assert_eq!(gf_multiply(0x57, 0x83), 0xc1);
    }

    #[test]
    fn test_xtime() {
        assert_eq!(xtime(0x57), 0xae);
        assert_eq!(xtime(0xae), 0x47);
    }

    #[test]
    fn test_no_distribute_data() {
        // 빈 데이터로 복호화 시도
        let result = decrypt_viewtext_section(&[], false);
        assert!(result.is_err());
    }
}
