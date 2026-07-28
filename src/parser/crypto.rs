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

use super::cfb_reader::{decompress_stream, decompress_stream_limited, CfbError};
use super::record::Record;
use super::tags;

/// 현재 지원하는 HWP 5 비밀번호 암호화 방식(FileHeader EncryptVersion).
pub const SUPPORTED_PASSWORD_ENCRYPT_VERSION: u32 = 4;

/// HWP5 비밀번호 암호 스트림 하나가 압축 해제된 뒤 가질 수 있는 최대 크기.
///
/// HWPX BinData 입력 상한과 같은 512 MiB를 적용한다. 이 값은 DocInfo, BodyText,
/// 즉시·지연 BinData의 공통 상한이며, 암호문 크기와 별개로 deflate 확장 폭주를 막는다.
pub const MAX_PASSWORD_DECOMPRESSED_STREAM_BYTES: usize = 512 * 1024 * 1024;

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
    /// 비밀번호 암호 스트림의 복호화 후 크기가 상한을 초과함
    DecompressedStreamLimitExceeded { max_bytes: usize },
    /// 비밀번호 불일치 또는 암호문 손상 (복호화 결과가 유효한 데이터가 아님)
    WrongPassword,
    /// 지원하지 않는 HWP 비밀번호 암호화 버전
    UnsupportedScheme { encrypt_version: u32 },
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
            CryptoError::DecompressedStreamLimitExceeded { max_bytes } => write!(
                f,
                "비밀번호 암호 스트림의 압축 해제 결과가 {} 바이트 상한을 초과했습니다",
                max_bytes
            ),
            CryptoError::WrongPassword => {
                write!(
                    f,
                    "비밀번호가 일치하지 않거나 암호화 데이터가 손상되었습니다"
                )
            }
            CryptoError::UnsupportedScheme { encrypt_version } => write!(
                f,
                "지원하지 않는 암호화 방식: EncryptVersion {} (지원: {})",
                encrypt_version, SUPPORTED_PASSWORD_ENCRYPT_VERSION
            ),
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
// AES-128 ECB 암호화 (정방향) — 비밀번호 복호화의 CFB 모드가
// AES *암호화*를 기본 연산으로 사용하므로 정방향 경로가 필요하다.
// ============================================================

/// AES SubBytes
fn sub_bytes(state: &mut [u8; 16]) {
    for byte in state.iter_mut() {
        *byte = S_BOX[*byte as usize];
    }
}

/// AES ShiftRows (정방향)
fn shift_rows(state: &mut [u8; 16]) {
    let s = *state;
    *state = [
        s[0], s[5], s[10], s[15], s[4], s[9], s[14], s[3], s[8], s[13], s[2], s[7], s[12], s[1],
        s[6], s[11],
    ];
}

/// AES MixColumns (정방향)
fn mix_columns(state: &mut [u8; 16]) {
    let s = *state;
    for c in 0..4 {
        let i = c * 4;
        state[i] = gf_multiply(0x02, s[i]) ^ gf_multiply(0x03, s[i + 1]) ^ s[i + 2] ^ s[i + 3];
        state[i + 1] = s[i] ^ gf_multiply(0x02, s[i + 1]) ^ gf_multiply(0x03, s[i + 2]) ^ s[i + 3];
        state[i + 2] = s[i] ^ s[i + 1] ^ gf_multiply(0x02, s[i + 2]) ^ gf_multiply(0x03, s[i + 3]);
        state[i + 3] = gf_multiply(0x03, s[i]) ^ s[i + 1] ^ s[i + 2] ^ gf_multiply(0x02, s[i + 3]);
    }
}

/// AES-128 ECB 단일 블록 암호화 (16바이트)
fn encrypt_block(block: &[u8; 16], expanded_key: &[u8]) -> [u8; 16] {
    let mut state = *block;

    // Initial round key addition (round 0)
    add_round_key(&mut state, &expanded_key[0..16]);

    // 9 main rounds (round 1 → 9)
    for round in 1..=9 {
        sub_bytes(&mut state);
        shift_rows(&mut state);
        mix_columns(&mut state);
        add_round_key(&mut state, &expanded_key[round * 16..(round + 1) * 16]);
    }

    // Final round (round 10) — MixColumns 없음
    sub_bytes(&mut state);
    shift_rows(&mut state);
    add_round_key(&mut state, &expanded_key[160..176]);

    state
}

// ============================================================
// SHA-1 (순수 Rust 구현) — 비밀번호 키 파생에 사용.
// rhwp는 blake3를 쓰지만 비밀번호 암호화 스펙이 SHA-1을 요구하므로
// WASM 바이너리 크기를 늘리지 않도록 직접 구현한다.
// ============================================================

/// SHA-1 해시 (FIPS 180-4)
fn sha1(data: &[u8]) -> [u8; 20] {
    let mut h: [u32; 5] = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0];

    // 패딩: 0x80 + 0x00... + 64비트 길이(비트). 블록은 64바이트 단위.
    let bit_len: u64 = (data.len() as u64).wrapping_mul(8);
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks(64) {
        let mut w = [0u32; 80];
        for (i, word) in chunk.chunks(4).enumerate().take(16) {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }

        let (mut a, mut b, mut c, mut d, mut e) = (h[0], h[1], h[2], h[3], h[4]);

        for i in 0..80 {
            let (f, k) = match i {
                0..=19 => ((b & c) | ((!b) & d), 0x5A827999u32),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
                _ => (b ^ c ^ d, 0xCA62C1D6),
            };
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(w[i]);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
    }

    let mut out = [0u8; 20];
    for (i, word) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

// ============================================================
// 비밀번호 암호화 문서 복호화 — EncryptVersion 4 (한글 7.0 이후)
//
// 알고리즘 참조: volexity/hwp-extract (BSD-3-Clause) 의 encrypt.py.
// Copyright © 2024 Volexity, Inc. 전체 고지는 THIRD_PARTY_LICENSES.md 참조.
// rhwp(MIT)와 라이선스 호환. 본 구현은 알고리즘을 직접 포팅했다.
//
// 흐름:
// 1. 비밀번호 → SHA-1 기반 키 파생 (16바이트 AES-128 키)
// 2. 원본 스트림을 16바이트 배수로 패드
// 3. AES-ECB 암호화를 기본 연산으로 쓰는 비트 단위 CFB 모드로 복호화
// 4. (압축 문서) zlib/raw-deflate 압축 해제
// ============================================================

/// 비밀번호에서 AES-128 키(16바이트)를 파생한다.
///
/// 각 비밀번호 바이트 앞에 "이전 바이트를 1비트 회전"한 값을 끼워 넣어
/// 인터리브 버퍼를 만들고, 그것의 SHA-1 다이제스트 앞 16바이트를 키로 쓴다.
/// 첫 바이트의 "이전 값"은 시드 상수 236(0xEC)이다.
fn derive_password_key(password: &[u8]) -> [u8; 16] {
    let mut buf = vec![0u8; password.len() * 2];
    for (i, &byte) in password.iter().enumerate() {
        // i==0 일 때 Python 코드는 password[-1] 대신 236을 사용한다.
        let v6 = if i == 0 { 236u8 } else { password[i - 1] };
        // (2*v6 | v6>>7) & 0xFF == ROL1(v6). u8 wrapping으로 동일.
        let v7 = v6.wrapping_mul(2) | (v6 >> 7);
        buf[i * 2] = v7;
        buf[i * 2 + 1] = byte;
    }
    let digest = sha1(&buf);
    let mut key = [0u8; 16];
    key.copy_from_slice(&digest[..16]);
    key
}

/// 데이터를 16바이트 배수로 PKCS#7-스타일 패드한다.
///
/// 이미 16바이트 배수면 패드를 추가하지 않는다(참조 구현과 동일).
/// 그 외에는 (16 - 나머지)바이트를 추가하며 각 패드 바이트 값은 추가 개수이다.
fn pad_to_block(data: &[u8]) -> Vec<u8> {
    let rem = data.len() % 16;
    if rem == 0 {
        return data.to_vec();
    }
    let amount = 16 - rem;
    let mut out = data.to_vec();
    out.resize(data.len() + amount, amount as u8);
    out
}

/// 비밀번호 기반 AES 비트 단위 CFB 복호화.
///
/// 16바이트 시프트 레지스터(tmp_in)를 유지하며, 각 16바이트 블록을
/// 128비트 단위로 처리한다. 각 비트마다 tmp_in을 AES-ECB 암호화한 결과의
/// MSB로 암호문 비트를 XOR하고, 암호문 비트를 시프트 레지스터로 되먹임한다.
/// 키는 스트림 전체에서 불변이므로 키 확장은 한 번만 수행한다.
fn shift_password_cfb_register(register: &mut [u8; 16], feedback_bit: u8) {
    let mut tmp = 1usize;
    for _ in 0..3 {
        let v14 = register[tmp];
        register[tmp - 1] = register[tmp - 1].wrapping_mul(2) | (register[tmp] >> 7);
        let v15 = register[tmp + 1];
        let v16 = v14.wrapping_mul(2) | (register[tmp + 1] >> 7);
        let v17 = register[tmp + 2];
        let v18 = v15.wrapping_mul(2) | (v17 >> 7);
        let v19 = register[tmp + 3];
        let v20 = v17.wrapping_mul(2) | (v19 >> 7);
        let v21 = v19.wrapping_mul(2) | (register[tmp + 4] >> 7);
        register[tmp] = v16;
        register[tmp + 1] = v18;
        register[tmp + 2] = v20;
        register[tmp + 3] = v21;
        tmp += 5;
    }
    register[15] = register[15].wrapping_mul(2) | (feedback_bit & 1);
}

fn decrypt_aes_cfb_password(key: &[u8; 16], data: &[u8]) -> Vec<u8> {
    let expanded_key = key_expansion(key);
    let mut tmp_in = [0u8; 16];
    let mut final_data = Vec::with_capacity(data.len());

    for block in data.chunks(16) {
        let mut real_input = [0u8; 16];
        let len = block.len().min(16);
        real_input[..len].copy_from_slice(&block[..len]);

        for i in 0..128usize {
            let enc = encrypt_block(&tmp_in, &expanded_key);
            let out = enc[0];
            let ff = i & 7;

            // 시프트 레지스터의 최하위 비트에 암호문 비트 1개를 되먹임.
            let bit_in = (real_input[i >> 3] >> (7 - ff)) & 1;
            shift_password_cfb_register(&mut tmp_in, bit_in);

            // AES 출력의 MSB를 암호문 비트 위치에 XOR → 평문 비트.
            real_input[i >> 3] ^= (out & 0x80) >> (i & 7);
        }

        final_data.extend_from_slice(&real_input[..len]);
    }

    final_data
}

/// 비밀번호 암호 스트림을 복호화하되 압축 상태는 그대로 둔다.
///
/// `Scripts/*`나 직렬화기가 모델링하지 않는 `BinData/*`처럼 raw 스트림 보존이
/// 필요한 경로에서 사용한다. 참조 구현이 마지막 부분 블록 계산을 위해 붙이는
/// 16바이트 정렬용 데이터는 결과에서 제거하여 CFB의 길이 보존 계약을 지킨다.
pub fn decrypt_password_stream(raw: &[u8], password: &[u8]) -> Vec<u8> {
    let key = derive_password_key(password);
    let padded = pad_to_block(raw);
    let mut decrypted = decrypt_aes_cfb_password(&key, &padded);
    decrypted.truncate(raw.len());
    decrypted
}

/// 비밀번호로 보호된 스트림(raw)을 복호화한다.
///
/// `compressed`가 true면 복호화 후 zlib/raw-deflate 압축 해제까지 수행한다. DocInfo,
/// BodyText, 즉시·지연 BinData가 모두 같은 상한을 사용하도록 기본 상한을 적용한다.
pub fn decrypt_password_protected(
    raw: &[u8],
    password: &[u8],
    compressed: bool,
) -> Result<Vec<u8>, CryptoError> {
    decrypt_password_protected_limited(
        raw,
        password,
        compressed,
        MAX_PASSWORD_DECOMPRESSED_STREAM_BYTES,
    )
}

/// 비밀번호 암호 스트림을 복호화하고, 결과 바이트 수를 `max_bytes`로 제한한다.
///
/// 압축 해제 형식 오류는 오답 비밀번호 또는 손상 암호문일 가능성이 높아
/// `WrongPassword`로 매핑하지만, 상한 초과는 호출자가 정책 위반으로 구분할 수 있게
/// 별도 오류로 보존한다.
pub fn decrypt_password_protected_limited(
    raw: &[u8],
    password: &[u8],
    compressed: bool,
    max_bytes: usize,
) -> Result<Vec<u8>, CryptoError> {
    let decrypted = decrypt_password_stream(raw, password);

    if compressed {
        match decompress_stream_limited(&decrypted, max_bytes) {
            Ok(output) => Ok(output),
            Err(CfbError::LimitExceeded(_)) => {
                Err(CryptoError::DecompressedStreamLimitExceeded { max_bytes })
            }
            Err(_) => Err(CryptoError::WrongPassword),
        }
    } else if decrypted.len() > max_bytes {
        Err(CryptoError::DecompressedStreamLimitExceeded { max_bytes })
    } else {
        Ok(decrypted)
    }
}

/// 테스트 픽스처용 비밀번호 암호 스트림 생성.
///
/// 제품 API는 읽기만 지원하므로 암호화 경로는 테스트 빌드에서만 노출한다.
#[cfg(test)]
pub(super) fn encrypt_password_stream_for_test(raw: &[u8], password: &[u8]) -> Vec<u8> {
    let key = derive_password_key(password);
    let padded = pad_to_block(raw);
    let expanded_key = key_expansion(&key);
    let mut register = [0u8; 16];
    let mut encrypted = Vec::with_capacity(padded.len());

    for block in padded.chunks_exact(16) {
        let mut output = [0u8; 16];
        for i in 0..128usize {
            let key_stream = encrypt_block(&register, &expanded_key)[0] >> 7;
            let plain_bit = (block[i >> 3] >> (7 - (i & 7))) & 1;
            let cipher_bit = plain_bit ^ key_stream;
            shift_password_cfb_register(&mut register, cipher_bit);
            output[i >> 3] |= cipher_bit << (7 - (i & 7));
        }
        encrypted.extend_from_slice(&output);
    }

    encrypted.truncate(raw.len());
    encrypted
}

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
    // record.rs 와 동일 근거: wasm32(usize 32비트)에서 `pos + size` 랩어라운드로
    // 경계 검사가 무력화되는 것을 checked_add 로 막는다.
    if pos
        .checked_add(size as usize)
        .map_or(true, |end| end > data.len())
    {
        return Err(format!(
            "레코드 데이터 부족: tag={}, 필요={}, 가용={}",
            tag_id,
            size,
            data.len().saturating_sub(pos)
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

    // ── SHA-1 벡터 (FIPS 180-4 APPENDIX B) ──

    #[test]
    fn test_sha1_empty() {
        assert_eq!(
            &sha1(b""),
            b"\xda\x39\xa3\xee\x5e\x6b\x4b\x0d\x32\x55\xbf\xef\x95\x60\x18\x90\xaf\xd8\x07\x09"
        );
    }

    #[test]
    fn test_sha1_abc() {
        assert_eq!(
            &sha1(b"abc"),
            b"\xa9\x99\x3e\x36\x47\x06\x81\x6a\xba\x3e\x25\x71\x78\x50\xc2\x6c\x9c\xd0\xd8\x9d"
        );
    }

    #[test]
    fn test_sha1_long() {
        let msg = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
        assert_eq!(
            &sha1(msg),
            b"\x84\x98\x3e\x44\x1c\x3b\xd2\x6e\xba\xae\x4a\xa1\xf9\x51\x29\xe5\xe5\x46\x70\xf1"
        );
    }

    // ── AES-128 정방향 (NIST FIPS-197 벡터) ──

    #[test]
    fn test_aes_encrypt_block_nist() {
        // 기존 test_aes_encrypt_decrypt_roundtrip 와 짝: 같은 key/plaintext.
        let key = [
            0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf,
            0x4f, 0x3c,
        ];
        let plaintext = [
            0x32, 0x43, 0xf6, 0xa8, 0x88, 0x5a, 0x30, 0x8d, 0x31, 0x31, 0x98, 0xa2, 0xe0, 0x37,
            0x07, 0x34,
        ];
        let expected_ciphertext = [
            0x39, 0x25, 0x84, 0x1d, 0x02, 0xdc, 0x09, 0xfb, 0xdc, 0x11, 0x85, 0x97, 0x19, 0x6a,
            0x0b, 0x32,
        ];
        let expanded = key_expansion(&key);
        assert_eq!(encrypt_block(&plaintext, &expanded), expected_ciphertext);
    }

    #[test]
    fn test_aes_ecb_encrypt_decrypt_roundtrip() {
        let key = [
            0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf,
            0x4f, 0x3c,
        ];
        let plaintext = [
            0x32, 0x43, 0xf6, 0xa8, 0x88, 0x5a, 0x30, 0x8d, 0x31, 0x31, 0x98, 0xa2, 0xe0, 0x37,
            0x07, 0x34, 0x32, 0x43, 0xf6, 0xa8, 0x88, 0x5a, 0x30, 0x8d, 0x31, 0x31, 0x98, 0xa2,
            0xe0, 0x37, 0x07, 0x34,
        ];
        // 복호화 후 재암호화(encrypt_block 직접) → 원문 복원 검증.
        let expanded = key_expansion(&key);
        let mut enc = Vec::new();
        for block in plaintext.chunks(16) {
            let mut b = [0u8; 16];
            b.copy_from_slice(block);
            enc.extend_from_slice(&encrypt_block(&b, &expanded));
        }
        let dec = decrypt_aes_ecb(&enc, &key);
        assert_eq!(dec, plaintext);
    }

    // ── 비밀번호 키 파생 ──

    #[test]
    fn test_derive_password_key_matches_volexity_vectors() {
        // volexity/hwp-extract e5f8b5e의 genkey()로 독립 생성한 고정 벡터.
        assert_eq!(
            derive_password_key(b"a"),
            [
                0xaf, 0xf8, 0x46, 0xf0, 0xdd, 0x70, 0x1f, 0xd6, 0xfb, 0x9a, 0xe6, 0xf2, 0x6a, 0x6d,
                0xb4, 0x0f,
            ]
        );
        assert_eq!(
            derive_password_key(b"helloworld"),
            [
                0x20, 0x7a, 0xe2, 0xca, 0x4c, 0xd8, 0x7f, 0xcf, 0x0d, 0x01, 0x56, 0x9d, 0x8c, 0x14,
                0x71, 0x5a,
            ]
        );
        assert_eq!(
            derive_password_key("한글".as_bytes()),
            [
                0xd9, 0xea, 0x31, 0xa9, 0x80, 0x12, 0x79, 0xbf, 0xc6, 0xfc, 0x29, 0xda, 0x14, 0xb4,
                0x3d, 0x83,
            ]
        );
    }

    #[test]
    fn test_derive_password_key_first_byte_seed() {
        // buf[0] 이 시드 상수 236(0xEC) 의 ROL1 = 0xD9 임을 sha1 입력에서 유추할 수 없으므로,
        // 동등성 대신 "빈 비밀번호도 길이 0 입력으로 처리된다" 만 확인 (빈 키 파생은 에러 아님).
        let key_empty = derive_password_key(b"");
        assert_eq!(key_empty.len(), 16);
    }

    // ── 패딩 ──

    #[test]
    fn test_pad_to_block_aligned() {
        // 16바이트 배수면 변경 없음.
        let data = vec![0u8; 32];
        assert_eq!(pad_to_block(&data), data);
        let data = vec![0u8; 16];
        assert_eq!(pad_to_block(&data), data);
    }

    #[test]
    fn test_pad_to_block_unaligned() {
        // 13바이트 → 16바이트로, 3바이트(값 3) 추가.
        let data = vec![0xABu8; 13];
        let padded = pad_to_block(&data);
        assert_eq!(padded.len(), 16);
        assert_eq!(&padded[..13], &data[..]);
        assert_eq!(&padded[13..], &[3, 3, 3]);
    }

    #[test]
    fn test_pad_to_block_empty() {
        // 빈 입력은 이미 0바이트(16 배수) → 변경 없음.
        let padded = pad_to_block(&[]);
        assert_eq!(padded, Vec::<u8>::new());
    }

    // ── 비밀번호 CFB 복호화 ──

    #[test]
    fn test_cfb_password_roundtrip() {
        for (idx, plaintext) in [
            &b"Hello, HWP!"[..],
            &b"0123456789abcdef"[..],       // 정확히 1블록
            &b"0123456789abcdefghijkl"[..], // 1.5블록
        ]
        .iter()
        .enumerate()
        {
            let encrypted = encrypt_password_stream_for_test(plaintext, b"test-password");
            let decrypted = decrypt_password_stream(&encrypted, b"test-password");
            assert_eq!(decrypted, *plaintext, "roundtrip case {idx}");
        }
    }

    #[test]
    fn test_cfb_password_decrypt_matches_volexity_vector() {
        // volexity/hwp-extract e5f8b5e의 decrypt_data(genkey("helloworld"),
        // bytes(range(32))) 결과. 자체 암·복호화 왕복만으로는 양쪽에 같은 포팅
        // 결함이 있을 때 놓칠 수 있으므로 외부 구현의 고정 벡터를 계약으로 둔다.
        let ciphertext: Vec<u8> = (0u8..32).collect();
        let decrypted = decrypt_password_stream(&ciphertext, b"helloworld");
        assert_eq!(
            decrypted,
            [
                0x00, 0x01, 0x3e, 0xec, 0x90, 0x3d, 0xbc, 0x26, 0xfa, 0xff, 0x9c, 0x6c, 0xfb, 0x35,
                0x48, 0x00, 0xbc, 0xaa, 0x14, 0x7b, 0x0e, 0xd1, 0x5c, 0x32, 0x21, 0x17, 0x37, 0xfa,
                0x97, 0x1d, 0xe3, 0x79,
            ]
        );
    }

    #[test]
    fn test_decrypt_password_protected_wrong_password() {
        // 임의의 바이트는 유효한 deflate 스트림이 아니므로 복호화 후 압축 해제 실패 →
        // WrongPassword. compressed=true 경로의 비밀번호 불일치 감지를 검증.
        let fake_encrypted = vec![0xAAu8; 64];
        let err = decrypt_password_protected(&fake_encrypted, b"wrongpwd", true).unwrap_err();
        assert!(matches!(err, CryptoError::WrongPassword));
    }

    #[test]
    fn test_decrypt_password_protected_uncompressed_returns_bytes() {
        // compressed=false 면 복호화된 바이트를 그대로 반환 (검증은 호출자/파서 책임).
        let fake = vec![0x11u8; 32];
        let result = decrypt_password_protected(&fake, b"pw", false).unwrap();
        assert_eq!(result.len(), 32);
    }

    #[test]
    fn test_decrypt_password_protected_limited_rejects_compressed_expansion() {
        use flate2::write::DeflateEncoder;
        use flate2::Compression;
        use std::io::Write;

        const TEST_PASSWORD: &[u8] = &[0x73, 0x69, 0x7a, 0x65, 0x2d, 0x74, 0x65, 0x73, 0x74];
        const MAX_BYTES: usize = 1024;
        let plaintext = vec![b'A'; MAX_BYTES + 1];
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(&plaintext).unwrap();
        let compressed = encoder.finish().unwrap();
        let encrypted = encrypt_password_stream_for_test(&compressed, TEST_PASSWORD);

        assert!(matches!(
            decrypt_password_protected_limited(&encrypted, TEST_PASSWORD, true, MAX_BYTES),
            Err(CryptoError::DecompressedStreamLimitExceeded {
                max_bytes: MAX_BYTES
            })
        ));
    }

    #[test]
    fn test_decrypt_password_protected_limited_rejects_uncompressed_oversize() {
        const TEST_PASSWORD: &[u8] = &[0x73, 0x69, 0x7a, 0x65, 0x2d, 0x74, 0x65, 0x73, 0x74];
        const MAX_BYTES: usize = 1024;
        let encrypted = encrypt_password_stream_for_test(&vec![0xA5; MAX_BYTES + 1], TEST_PASSWORD);

        assert!(matches!(
            decrypt_password_protected_limited(&encrypted, TEST_PASSWORD, false, MAX_BYTES),
            Err(CryptoError::DecompressedStreamLimitExceeded {
                max_bytes: MAX_BYTES
            })
        ));
    }
}
