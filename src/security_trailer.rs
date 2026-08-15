//! HWP3 보안 트레일러 — 유효한 HWP3 파일에 강암호 페이로드를 append 해, 순정 한컴에서는
//! 정상 열람되고 rhwp 에서는 진짜 비밀을 복원하며, 어떤 상황에서도 에러 없이 정상화되는 구조.
//!
//! ## 3층 구조
//!
//! - **가시층(Visible)** — 민감정보를 뺀/가린 정상 HWP3 본문. 순정 한컴 + 모두가 읽는다.
//! - **봉인 트레일러(Sealed)** — 진짜 비밀(AEAD 암호문). rhwp + 비밀번호만 연다.
//! - **정상화(Normalize)** — 감지·검증·fallback. rhwp 런타임이 4상태로 수렴시킨다.
//!
//! ## 정보이론적 정직 조항
//!
//! 순정 한컴이 보여주는 **가시 내용의 기밀성은 여전히 56비트(DES) 상한**이다 — 그건 이
//! 설계로 못 올린다. 이 설계는 "가시 내용을 더 세게 지진다"가 아니라, **진짜 비밀을 애초에
//! 가시층에서 빼서 강암호 트레일러로 따로 보호한다**. 트레일러가 지키는 비밀에는 상한이 없다.
//!
//! ## 정당한 용도와 경고 (검사 우회 도구가 아니다)
//!
//! 이 모듈은 **권한자 복원이 가능한 리댁션**만 한다(REDACTED 전용) — 가시층은 실제 리댁션된
//! 문서이지 위장 문서가 아니고, 문서 전체를 숨기는 decoy 모드는 넣지 않는다. 정당한 용도는
//! 민감 필드(주민번호·계좌 등)를 문서에서 가리되 권한자(비밀번호 보유)가 원값을 복원하는 것이다.
//!
//! - **은닉 아님·탐지 가능**: 트레일러는 평문 매직마커(`RHWPSEC1`/`RHWPEND1`)로 시작·끝난다.
//!   어떤 검사 도구든 이 마커를 스캔해 "봉인 트레일러 있음"을 즉시 식별할 수 있다. 이 설계는
//!   내용을 **암호화**할 뿐, 트레일러의 **존재를 은폐(스테가노그래피)하지 않는다**.
//! - **가시 기밀성 한계**: 순정 한컴이 보여주는 가시 내용의 기밀성은 여전히 56비트(DES) 상한.
//! - **사용자 실수 위험**: 한컴이 재저장하면 트레일러가 소실될 수 있다 → 경고·읽기 전용 배포·
//!   교육이 필요하다.
//! - **법적/조직 준수**: 개인정보·금융정보 처리 규정, 보존정책, 로그·감사, 키 관리 정책을
//!   조직 기준에 맞춰 문서화한 뒤 운용한다.
//!
//! ## 파일 레이아웃 (append; 순정 한컴은 EOF 까지만 읽는다)
//!
//! ```text
//! [ 원본 HWP3 바이트 (완전히 유효한 문서) ]  <- 순정 한컴은 여기까지
//! MAGIC_START [8]  "RHWPSEC1"
//! version     [2]  u16 LE
//! flags       [2]  u16 LE   (REDACTED=0x01; 그 외 미지원)
//! kdf_algo    [1]  1=Argon2id
//! aead_algo   [1]  1=XChaCha20-Poly1305
//! salt        [16]
//! nonce       [24]
//! ct_len      [4]  u32 LE
//! ciphertext  [ct_len]      (AEAD, AAD = 위 헤더 전체)
//! trailer_len [4]  u32 LE   (MAGIC_START..MAGIC_END 총길이)
//! MAGIC_END   [8]  "RHWPEND1"
//! ```
//!
//! 탐지는 뒤에서부터: 끝 8바이트가 MAGIC_END 인지 → trailer_len 으로 시작 위치를 역산해
//! MAGIC_START 확인. 둘 중 하나라도 안 맞으면 트레일러가 없는 것으로 본다(우연 일치 방어).

use chacha20poly1305::aead::{Aead, Payload};
use chacha20poly1305::{Key, KeyInit, XChaCha20Poly1305, XNonce};
use ml_kem::kem::{Decapsulate, Encapsulate};
use ml_kem::{Ciphertext, Encoded, EncodedSizeUser, KemCore, MlKem768};
use zeroize::Zeroizing;

pub const MAGIC_START: &[u8; 8] = b"RHWPSEC1";
pub const MAGIC_END: &[u8; 8] = b"RHWPEND1";

/// **유일 지원 모드** — 민감 스팬만 가리고 진짜 값은 권한자 복원용으로 트레일러에 둔다.
/// 가시층은 decoy 가 아니라 **실제 리댁션된 문서**다: 표준 뷰어가 보는 것이 진짜 문서(의
/// 가린 판)이지, 전혀 다른 위장 문서가 아니다. 이 도구는 "검사 우회"가 아니라 "권한자
/// 복원 가능한 리댁션"만 수행한다. 문서 전체를 위장 문서 뒤에 숨기는 모드(구 0x02)는
/// 의도적으로 넣지 않는다.
pub const FLAG_REDACTED: u16 = 0x01;

pub const VERSION: u16 = 1;
pub const KDF_ARGON2ID: u8 = 1;
/// **포스트양자 공개키 봉인** — kdf 슬롯을 재사용해 "키 유도 방식"을 ML-KEM-768(FIPS 203)
/// 격자 KEM 으로 표시한다. 비밀번호 없이 수신자 공개키로 봉인하며, 파생된 32바이트 공유비밀을
/// 그대로 AEAD 키로 쓴다. 대칭부(XChaCha20-Poly1305)는 이미 양자내성이고, Shor 에 깨지는
/// 비대칭 키교환만 이 격자 KEM 으로 대체한다.
pub const KDF_MLKEM768: u8 = 2;
pub const AEAD_XCHACHA20POLY1305: u8 = 1;

// ML-KEM-768 (FIPS 203, 보안 카테고리 3) 고정 크기. PQ 트레일러의 고정 프리픽스는
// MAGIC_START[8] version[2] flags[2] kdf[1] aead[1] encap_len[4] = 18 바이트.
const MLKEM768_EK_LEN: usize = 1184; // encapsulation key (공개키)
const MLKEM768_DK_LEN: usize = 2400; // decapsulation key (개인키)
const MLKEM768_CT_LEN: usize = 1088; // encapsulation (KEM 암호문)
const MLKEM768_SS_LEN: usize = 32; // shared secret == XChaCha20 키 길이
const PQ_HEADER_PREFIX: usize = 8 + 2 + 2 + 1 + 1 + 4; // = 18

// kdf_algo=1 의 고정 Argon2id 파라미터(seal/unseal 이 반드시 같아야 하므로 상수).
// OWASP 권고 하한 근방 — memory-hard 성질을 지키면서 CI·wasm 에서도 감당된다.
const ARGON2_MEM_KIB: u32 = 19_456; // 19 MiB
const ARGON2_TIME: u32 = 2;
const ARGON2_LANES: u32 = 1;

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 24;
const TAG_LEN: usize = 16; // Poly1305
const HEADER_LEN: usize = 8 + 2 + 2 + 1 + 1 + SALT_LEN + NONCE_LEN + 4; // MAGIC..ct_len = 58
const MIN_TRAILER_LEN: usize = HEADER_LEN + TAG_LEN + 4 + 8; // 빈 비밀 + trailer_len + MAGIC_END

#[derive(Debug)]
pub enum SealError {
    Kdf(String),
    Aead,
    Random(String),
    /// 수신자 공개키 길이가 ML-KEM-768 EK(1184바이트)와 다르다.
    BadPublicKey {
        expected: usize,
        got: usize,
    },
    /// ML-KEM 캡슐화 실패 — FIPS 203 상 실무 도달 불가지만, 절대 panic 하지 않도록 방어적으로 표면화.
    Kem,
}

impl std::fmt::Display for SealError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SealError::Kdf(e) => write!(f, "키 유도 실패: {e}"),
            SealError::Aead => write!(f, "암호화 실패"),
            SealError::Random(e) => write!(f, "엔트로피 획득 실패: {e}"),
            SealError::BadPublicKey { expected, got } => {
                write!(
                    f,
                    "공개키 길이 오류: {expected}바이트 기대, {got}바이트 받음"
                )
            }
            SealError::Kem => write!(f, "ML-KEM 캡슐화 실패"),
        }
    }
}

impl std::error::Error for SealError {}

/// 파일을 열었을 때의 정상화 결과 — rhwp 는 절대 에러로 죽지 않고 넷 중 하나로 수렴한다.
#[derive(Debug, PartialEq, Eq)]
pub enum Opened {
    /// 트레일러 없음 → 평범한 HWP3. (한컴 재저장으로 트레일러가 사라진 Stripped 도 여기로 온다.)
    Plain,
    /// 트레일러 복호 성공 → 진짜 비밀 복원.
    Sealed { plaintext: Vec<u8>, flags: u16 },
    /// 트레일러는 있으나 복호 실패(비밀번호 오류·변조) → 가시층은 그대로 열고 경고.
    Broken { reason: String },
}

fn derive_key(password: &[u8], salt: &[u8]) -> Result<Zeroizing<[u8; 32]>, SealError> {
    use argon2::{Algorithm, Argon2, Params, Version};
    let params = Params::new(ARGON2_MEM_KIB, ARGON2_TIME, ARGON2_LANES, Some(32))
        .map_err(|e| SealError::Kdf(e.to_string()))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = Zeroizing::new([0u8; 32]);
    argon2
        .hash_password_into(password, salt, key.as_mut())
        .map_err(|e| SealError::Kdf(e.to_string()))?;
    Ok(key)
}

/// 헤더(= AEAD 의 associated data). MAGIC..ct_len 전체를 묶어 어떤 헤더 필드(버전·플래그·
/// 알고리즘·salt·nonce·길이)를 변조해도 복호가 거부되게 한다.
fn build_header(
    flags: u16,
    salt: &[u8; SALT_LEN],
    nonce: &[u8; NONCE_LEN],
    ct_len: u32,
) -> Vec<u8> {
    let mut h = Vec::with_capacity(HEADER_LEN);
    h.extend_from_slice(MAGIC_START);
    h.extend_from_slice(&VERSION.to_le_bytes());
    h.extend_from_slice(&flags.to_le_bytes());
    h.push(KDF_ARGON2ID);
    h.push(AEAD_XCHACHA20POLY1305);
    h.extend_from_slice(salt);
    h.extend_from_slice(nonce);
    h.extend_from_slice(&ct_len.to_le_bytes());
    h
}

/// 유효한 HWP3 바이트에 리댁션된 값을 강암호 트레일러로 append 한다(REDACTED 전용).
/// `host_hwp3` 는 이미 민감 스팬이 가려진 **실제 리댁션 문서**여야 한다 — 이 함수는
/// 그 가려진 원값(`secret`)만 권한자 복원용으로 봉인한다.
pub fn seal(host_hwp3: &[u8], secret: &[u8], password: &[u8]) -> Result<Vec<u8>, SealError> {
    let mut salt = [0u8; SALT_LEN];
    let mut nonce = [0u8; NONCE_LEN];
    getrandom::fill(&mut salt).map_err(|e| SealError::Random(e.to_string()))?;
    getrandom::fill(&mut nonce).map_err(|e| SealError::Random(e.to_string()))?;

    let key = derive_key(password, &salt)?;
    let ct_len = (secret.len() + TAG_LEN) as u32;
    let header = build_header(FLAG_REDACTED, &salt, &nonce, ct_len);

    let cipher = XChaCha20Poly1305::new(Key::from_slice(key.as_ref()));
    let ciphertext = cipher
        .encrypt(
            XNonce::from_slice(&nonce),
            Payload {
                msg: secret,
                aad: &header,
            },
        )
        .map_err(|_| SealError::Aead)?;
    debug_assert_eq!(ciphertext.len(), ct_len as usize);

    let trailer_len = (header.len() + ciphertext.len() + 4 + 8) as u32;
    let mut out = Vec::with_capacity(host_hwp3.len() + trailer_len as usize);
    out.extend_from_slice(host_hwp3);
    out.extend_from_slice(&header);
    out.extend_from_slice(&ciphertext);
    out.extend_from_slice(&trailer_len.to_le_bytes());
    out.extend_from_slice(MAGIC_END);
    Ok(out)
}

/// 뒤에서부터 트레일러를 탐지 — 있으면 트레일러 시작 오프셋을 돌려준다. 끝이 MAGIC_END 이고
/// trailer_len 이 가리키는 시작이 MAGIC_START 여야 한다(우연 일치·재저장 잔여 방어).
pub fn detect_trailer(bytes: &[u8]) -> Option<usize> {
    if bytes.len() < MIN_TRAILER_LEN {
        return None;
    }
    let n = bytes.len();
    if &bytes[n - 8..] != MAGIC_END {
        return None;
    }
    let trailer_len = u32::from_le_bytes(bytes[n - 12..n - 8].try_into().ok()?) as usize;
    if trailer_len < MIN_TRAILER_LEN || trailer_len > n {
        return None;
    }
    let start = n - trailer_len;
    if &bytes[start..start + 8] != MAGIC_START {
        return None;
    }
    Some(start)
}

/// 가시층(트레일러를 뗀 원본 HWP3 바이트). 트레일러가 없으면 입력 그대로.
pub fn visible_layer(bytes: &[u8]) -> &[u8] {
    match detect_trailer(bytes) {
        Some(start) => &bytes[..start],
        None => bytes,
    }
}

/// 파일을 정상화해 연다 — 절대 Err 를 내지 않고 Plain/Sealed/Broken 중 하나로 수렴한다.
pub fn open(bytes: &[u8], password: &[u8]) -> Opened {
    let start = match detect_trailer(bytes) {
        Some(s) => s,
        None => return Opened::Plain,
    };
    let t = &bytes[start..];
    // 레이아웃: MAGIC_START[8] version[2] flags[2] kdf[1] aead[1] salt[16] nonce[24] ct_len[4] ct[..]
    let version = u16::from_le_bytes([t[8], t[9]]);
    if version != VERSION {
        // 버전 협상: 모르는 버전은 향후 포맷 진화로 보고 트레일러를 무시(Plain).
        return Opened::Plain;
    }
    let flags = u16::from_le_bytes([t[10], t[11]]);
    let kdf = t[12];
    let aead = t[13];
    if kdf != KDF_ARGON2ID || aead != AEAD_XCHACHA20POLY1305 {
        // kdf=2 는 공개키(ML-KEM) 봉인이다 — 비밀번호로는 못 연다. 안내 메시지만 개선하고
        // 동작은 그대로(Broken) 유지한다(비밀번호 경로는 kdf=1 전용).
        let reason = if kdf == KDF_MLKEM768 {
            "공개키(ML-KEM) 봉인 트레일러다 — 비밀번호가 아니라 개인키로 열어야 한다(open_with_privkey)"
                .to_string()
        } else {
            format!("미지원 알고리즘 (kdf={kdf}, aead={aead})")
        };
        return Opened::Broken { reason };
    }
    let salt: [u8; SALT_LEN] = t[14..30].try_into().expect("salt 16");
    let nonce: [u8; NONCE_LEN] = t[30..54].try_into().expect("nonce 24");
    let ct_len = u32::from_le_bytes([t[54], t[55], t[56], t[57]]) as usize;
    let ct_end = HEADER_LEN + ct_len;
    if ct_len < TAG_LEN || ct_end + 4 + 8 > t.len() {
        return Opened::Broken {
            reason: "트레일러 길이 불일치".to_string(),
        };
    }
    let ciphertext = &t[HEADER_LEN..ct_end];
    let header = build_header(flags, &salt, &nonce, ct_len as u32);

    let key = match derive_key(password, &salt) {
        Ok(k) => k,
        Err(e) => {
            return Opened::Broken {
                reason: e.to_string(),
            }
        }
    };
    let cipher = XChaCha20Poly1305::new(Key::from_slice(key.as_ref()));
    match cipher.decrypt(
        XNonce::from_slice(&nonce),
        Payload {
            msg: ciphertext,
            aad: &header,
        },
    ) {
        Ok(plaintext) => Opened::Sealed { plaintext, flags },
        Err(_) => Opened::Broken {
            reason: "복호 실패 (비밀번호 오류 또는 변조)".to_string(),
        },
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 포스트양자 공개키 봉인 (kdf=2, ML-KEM-768 / NIST FIPS 203)
//
// 왜 필요한가: 기존 비밀번호 모드(Argon2id → XChaCha20-Poly1305)는 **이미 양자내성**이다 —
// 대칭 암호는 Grover 로 유효 강도가 절반(256→128비트)으로 줄 뿐 여전히 안전하다. 양자
// (Shor)에 깨지는 건 **비대칭 공개키 교환**이다. 그래서 이 모드는 비밀번호 공유 없이
// 수신자의 공개키로 봉인하는 **양자안전 공개키 교환(ML-KEM 격자 KEM)** 을 추가한다.
//
// 흐름: 봉인자는 수신자 공개키(EK)로 `encapsulate` → (KEM 암호문 CT, 32바이트 공유비밀 SS).
// SS 를 그대로 XChaCha20-Poly1305 키로 쓴다(ML-KEM 공유비밀은 균일 난수라 별도 KDF 불필요).
// 수신자는 개인키(DK)로 `decapsulate(CT)` → 같은 SS 를 복원해 AEAD 를 푼다.
//
// PQ 트레일러 레이아웃(append; 비밀번호 트레일러와 매직마커·꼬리는 공유해 `detect_trailer`
// 가 그대로 판별한다):
//
// ```text
// MAGIC_START [8]  "RHWPSEC1"
// version     [2]  u16 LE
// flags       [2]  u16 LE   (REDACTED=0x01)
// kdf_algo    [1]  2=ML-KEM-768
// aead_algo   [1]  1=XChaCha20-Poly1305
// encap_len   [4]  u32 LE   (= 1088)
// encap       [encap_len]   KEM 암호문(CT)
// nonce       [24]
// ct_len      [4]  u32 LE
// ciphertext  [ct_len]      (AEAD, AAD = MAGIC..ct_len 전체 헤더)
// trailer_len [4]  u32 LE
// MAGIC_END   [8]  "RHWPEND1"
// ```
//
// 정당한 용도·경고는 비밀번호 모드와 동일하다 — REDACTED 전용, 평문 매직마커로 **탐지 가능**,
// decoy 모드 없음. 이 모드가 바꾸는 건 "비밀을 여는 자격"뿐이다(비밀번호 → 개인키 보유).
// ─────────────────────────────────────────────────────────────────────────────

/// `getrandom` 을 rand_core 0.6 `RngCore` + `CryptoRng` 로 감싼 어댑터 — ml-kem 0.2 의
/// `generate`/`encapsulate` 가 요구하는 `CryptoRngCore`(= `CryptoRng + RngCore` 블랭킷)를
/// 충족한다. 엔트로피는 OS(getrandom)에서 직접 뽑는다.
struct GetRandomRng;

impl rand_core::RngCore for GetRandomRng {
    fn next_u32(&mut self) -> u32 {
        let mut b = [0u8; 4];
        getrandom::fill(&mut b).expect("getrandom: next_u32");
        u32::from_le_bytes(b)
    }
    fn next_u64(&mut self) -> u64 {
        let mut b = [0u8; 8];
        getrandom::fill(&mut b).expect("getrandom: next_u64");
        u64::from_le_bytes(b)
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        getrandom::fill(dest).expect("getrandom: fill_bytes");
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl rand_core::CryptoRng for GetRandomRng {}

/// PQ 헤더(= AEAD associated data). MAGIC..ct_len 전체를 묶어 encap·nonce·길이·플래그 등
/// 어느 헤더 필드를 변조해도 복호가 거부되게 한다(비밀번호 모드 `build_header` 와 같은 원리).
fn build_pq_header(flags: u16, encap: &[u8], nonce: &[u8; NONCE_LEN], ct_len: u32) -> Vec<u8> {
    let mut h = Vec::with_capacity(PQ_HEADER_PREFIX + encap.len() + NONCE_LEN + 4);
    h.extend_from_slice(MAGIC_START);
    h.extend_from_slice(&VERSION.to_le_bytes());
    h.extend_from_slice(&flags.to_le_bytes());
    h.push(KDF_MLKEM768);
    h.push(AEAD_XCHACHA20POLY1305);
    h.extend_from_slice(&(encap.len() as u32).to_le_bytes());
    h.extend_from_slice(encap);
    h.extend_from_slice(nonce);
    h.extend_from_slice(&ct_len.to_le_bytes());
    h
}

/// ML-KEM-768 키쌍을 새로 만든다 → `(공개키 EK 바이트[1184], 개인키 DK 바이트[2400])`.
/// 공개키는 봉인자에게 배포하고, 개인키는 수신자만 보관해 `open_with_privkey` 로 연다.
pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
    let mut rng = GetRandomRng;
    // KemCore::generate → (decapsulation key, encapsulation key) = (dk, ek).
    let (dk, ek) = MlKem768::generate(&mut rng);
    let ek_bytes = ek.as_bytes().to_vec();
    let dk_bytes = dk.as_bytes().to_vec();
    debug_assert_eq!(ek_bytes.len(), MLKEM768_EK_LEN);
    debug_assert_eq!(dk_bytes.len(), MLKEM768_DK_LEN);
    (ek_bytes, dk_bytes)
}

/// 유효한 HWP3 바이트에 리댁션된 원값을 **수신자 공개키로** 봉인한 트레일러를 append 한다
/// (비밀번호 불필요, REDACTED 전용). `host_hwp3` 는 이미 민감 스팬이 가려진 실제 리댁션
/// 문서여야 한다 — 이 함수는 그 가려진 원값(`secret`)만 봉인한다.
pub fn seal_to_pubkey(
    host_hwp3: &[u8],
    secret: &[u8],
    recipient_public: &[u8],
) -> Result<Vec<u8>, SealError> {
    if recipient_public.len() != MLKEM768_EK_LEN {
        return Err(SealError::BadPublicKey {
            expected: MLKEM768_EK_LEN,
            got: recipient_public.len(),
        });
    }
    type Ek = <MlKem768 as KemCore>::EncapsulationKey;
    // 길이는 위에서 검증했지만 from_bytes 는 정확 크기 Array 를 요구하므로 try_from 으로 안전 변환.
    let ek_arr =
        Encoded::<Ek>::try_from(recipient_public).map_err(|_| SealError::BadPublicKey {
            expected: MLKEM768_EK_LEN,
            got: recipient_public.len(),
        })?;
    let ek = Ek::from_bytes(&ek_arr);

    let mut rng = GetRandomRng;
    // encapsulate → (KEM 암호문 CT, 32바이트 공유비밀 SS). FIPS 203 상 무오류지만 방어적으로 map_err.
    let (ct_kem, shared) = ek.encapsulate(&mut rng).map_err(|_| SealError::Kem)?;
    let encap = ct_kem.as_slice();
    debug_assert_eq!(encap.len(), MLKEM768_CT_LEN);
    debug_assert_eq!(shared.as_slice().len(), MLKEM768_SS_LEN);

    let mut nonce = [0u8; NONCE_LEN];
    getrandom::fill(&mut nonce).map_err(|e| SealError::Random(e.to_string()))?;

    let ct_len = (secret.len() + TAG_LEN) as u32;
    let header = build_pq_header(FLAG_REDACTED, encap, &nonce, ct_len);

    // AEAD 키 = 32바이트 공유비밀을 그대로 사용(ML-KEM SS 는 균일 난수). Zeroizing 로 소거 보장.
    let mut key = Zeroizing::new([0u8; 32]);
    key.copy_from_slice(shared.as_slice());
    let cipher = XChaCha20Poly1305::new(Key::from_slice(key.as_ref()));
    let ciphertext = cipher
        .encrypt(
            XNonce::from_slice(&nonce),
            Payload {
                msg: secret,
                aad: &header,
            },
        )
        .map_err(|_| SealError::Aead)?;
    debug_assert_eq!(ciphertext.len(), ct_len as usize);

    let trailer_len = (header.len() + ciphertext.len() + 4 + 8) as u32;
    let mut out = Vec::with_capacity(host_hwp3.len() + trailer_len as usize);
    out.extend_from_slice(host_hwp3);
    out.extend_from_slice(&header);
    out.extend_from_slice(&ciphertext);
    out.extend_from_slice(&trailer_len.to_le_bytes());
    out.extend_from_slice(MAGIC_END);
    Ok(out)
}

/// 공개키로 봉인된 파일을 **개인키로** 정상화해 연다 — 절대 Err/panic 없이
/// Plain/Sealed/Broken 중 하나로 수렴한다(모든 길이 읽기는 엄격 경계 검사).
pub fn open_with_privkey(bytes: &[u8], secret_key: &[u8]) -> Opened {
    let start = match detect_trailer(bytes) {
        Some(s) => s,
        None => return Opened::Plain,
    };
    let t = &bytes[start..];
    // detect_trailer 가 t.len() >= MIN_TRAILER_LEN 를 보장하지만, 여기서도 프리픽스 경계를 명시 검사.
    if t.len() < PQ_HEADER_PREFIX {
        return Opened::Broken {
            reason: "트레일러가 너무 짧다".to_string(),
        };
    }
    let version = u16::from_le_bytes([t[8], t[9]]);
    if version != VERSION {
        // 모르는 버전은 향후 포맷 진화로 보고 무시(Plain) — 비밀번호 경로와 동일 정책.
        return Opened::Plain;
    }
    let flags = u16::from_le_bytes([t[10], t[11]]);
    let kdf = t[12];
    let aead = t[13];
    if kdf != KDF_MLKEM768 {
        // 비밀번호(kdf=1) 트레일러를 개인키로 열려는 경우 등 → Broken(패닉 금지).
        return Opened::Broken {
            reason: "공개키 봉인이 아니다 (kdf != 2) — 비밀번호로 열어야 한다(open)".to_string(),
        };
    }
    if aead != AEAD_XCHACHA20POLY1305 {
        return Opened::Broken {
            reason: format!("미지원 AEAD (aead={aead})"),
        };
    }

    // ── 엄격 경계 파싱: 어떤 길이 필드가 조작돼도 인덱스 OOB/패닉 없이 Broken 으로 귀결 ──
    let encap_len = u32::from_le_bytes([t[14], t[15], t[16], t[17]]) as usize;
    let len_mismatch = || Opened::Broken {
        reason: "트레일러 길이 불일치".to_string(),
    };
    // encap 이 정확히 ML-KEM-768 CT 크기인지 먼저 확인(decapsulate 가 고정 크기 Array 를 요구).
    if encap_len != MLKEM768_CT_LEN {
        return Opened::Broken {
            reason: format!("encap 길이 불일치 ({encap_len})"),
        };
    }
    let encap_end = match PQ_HEADER_PREFIX.checked_add(encap_len) {
        Some(v) => v,
        None => return len_mismatch(),
    };
    let nonce_end = match encap_end.checked_add(NONCE_LEN) {
        Some(v) => v,
        None => return len_mismatch(),
    };
    let ctlen_end = match nonce_end.checked_add(4) {
        Some(v) => v,
        None => return len_mismatch(),
    };
    if ctlen_end > t.len() {
        return len_mismatch();
    }
    let encap = &t[PQ_HEADER_PREFIX..encap_end];
    let nonce: [u8; NONCE_LEN] = t[encap_end..nonce_end].try_into().expect("nonce 24");
    let ct_len = u32::from_le_bytes([
        t[nonce_end],
        t[nonce_end + 1],
        t[nonce_end + 2],
        t[nonce_end + 3],
    ]) as usize;
    let ct_start = ctlen_end;
    let ct_end = match ct_start.checked_add(ct_len) {
        Some(v) => v,
        None => return len_mismatch(),
    };
    // 암호문 뒤에는 trailer_len[4] + MAGIC_END[8] = 12바이트가 있어야 한다.
    let need_tail = match ct_end.checked_add(4 + 8) {
        Some(v) => v,
        None => return len_mismatch(),
    };
    if ct_len < TAG_LEN || need_tail > t.len() {
        return len_mismatch();
    }
    let ciphertext = &t[ct_start..ct_end];
    // AAD = MAGIC..ct_len 전체 헤더(암호문 직전까지). 원본 바이트를 그대로 써 봉인 시점과 바이트 동일.
    let aad = &t[..ctlen_end];

    // ── 개인키로 역캡슐화 → 공유비밀 복원 → AEAD 복호 ──
    type Dk = <MlKem768 as KemCore>::DecapsulationKey;
    let dk_arr = match Encoded::<Dk>::try_from(secret_key) {
        Ok(a) => a,
        Err(_) => {
            return Opened::Broken {
                reason: format!(
                    "개인키 길이 오류: {}바이트 기대, {}바이트 받음",
                    MLKEM768_DK_LEN,
                    secret_key.len()
                ),
            }
        }
    };
    let dk = Dk::from_bytes(&dk_arr);
    let ct_kem = match Ciphertext::<MlKem768>::try_from(encap) {
        Ok(c) => c,
        Err(_) => return len_mismatch(),
    };
    // ML-KEM 역캡슐화는 무오류다 — 틀린 개인키·변조 CT 여도 (묵시적 거부로) *다른* 공유비밀을
    // 돌려줄 뿐 Err 를 내지 않는다. 그래서 진짜 무결성 관문은 아래 AEAD 다: 공유비밀이 다르면
    // AEAD 키가 달라져 복호가 실패하고 Broken 이 된다.
    let shared = match dk.decapsulate(&ct_kem) {
        Ok(s) => s,
        Err(_) => {
            return Opened::Broken {
                reason: "역캡슐화 실패".to_string(),
            }
        }
    };
    let mut key = Zeroizing::new([0u8; 32]);
    key.copy_from_slice(shared.as_slice());
    let cipher = XChaCha20Poly1305::new(Key::from_slice(key.as_ref()));
    match cipher.decrypt(
        XNonce::from_slice(&nonce),
        Payload {
            msg: ciphertext,
            aad,
        },
    ) {
        Ok(plaintext) => Opened::Sealed { plaintext, flags },
        Err(_) => Opened::Broken {
            reason: "복호 실패 (개인키 불일치 또는 변조)".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOST: &[u8] = b"\x1b\x00\x00\x00HWP Document File V3.00\x00 ... valid hwp3 bytes ...";
    const SECRET: &[u8] = "진짜 비밀 — 주민번호 900101-1234567".as_bytes();

    /// 테스트 비밀번호는 **실행마다 새로 뽑는다**.
    ///
    /// 상수로 두면 (a) CodeQL 이 하드코딩 암호값(critical)으로 잡고, (b) 어느
    /// 경로가 특정 비밀번호에 우연히 기대게 되어도 드러나지 않는다. 난수면
    /// 왕복·변조·오답 판정이 값에 무관하게 성립함을 매 실행이 재확인한다.
    fn pw() -> Vec<u8> {
        let mut buf = [0u8; 32];
        getrandom::fill(&mut buf).expect("테스트 비밀번호 난수");
        buf.to_vec()
    }

    #[test]
    fn seal_then_open_roundtrips() {
        let pw = pw();
        let sealed = seal(HOST, SECRET, &pw).unwrap();
        // 가시층은 원본 그대로(순정 한컴이 읽는 것).
        assert_eq!(visible_layer(&sealed), HOST);
        // rhwp + 올바른 비밀번호 → 진짜 비밀 복원.
        match open(&sealed, &pw) {
            Opened::Sealed { plaintext, flags } => {
                assert_eq!(plaintext, SECRET);
                assert_eq!(flags, FLAG_REDACTED);
            }
            other => panic!("Sealed 를 기대: {other:?}"),
        }
    }

    #[test]
    fn wrong_password_is_broken_not_panic() {
        let (right, wrong) = (pw(), pw());
        let sealed = seal(HOST, SECRET, &right).unwrap();
        assert!(
            matches!(open(&sealed, &wrong), Opened::Broken { .. }),
            "다른 비밀번호는 Broken 이어야 한다"
        );
    }

    #[test]
    fn tampered_ciphertext_is_broken() {
        let pw = pw();
        let mut sealed = seal(HOST, SECRET, &pw).unwrap();
        let n = sealed.len();
        sealed[n - 20] ^= 0xFF; // 트레일러 내부(암호문/태그) 1비트 변조
        assert!(matches!(open(&sealed, &pw), Opened::Broken { .. }));
    }

    #[test]
    fn no_trailer_is_plain() {
        assert_eq!(open(HOST, &pw()), Opened::Plain);
        assert_eq!(visible_layer(HOST), HOST);
    }

    #[test]
    fn accidental_magic_end_in_body_is_plain() {
        // 원본이 우연히 MAGIC_END 로 끝나도, trailer_len 역산이 MAGIC_START 와 안 맞아 Plain.
        let mut tricky = HOST.to_vec();
        tricky.extend_from_slice(MAGIC_END);
        assert_eq!(open(&tricky, &pw()), Opened::Plain);
        assert!(detect_trailer(&tricky).is_none());
    }

    #[test]
    fn stripped_trailer_reopens_as_plain() {
        // 한컴 재저장 = 가시층만 남고 트레일러 소실 → Plain 으로 정상화(에러 없음).
        let pw = pw();
        let sealed = seal(HOST, SECRET, &pw).unwrap();
        let stripped = visible_layer(&sealed).to_vec();
        assert_eq!(open(&stripped, &pw), Opened::Plain);
    }

    #[test]
    fn empty_secret_roundtrips() {
        let pw = pw();
        let sealed = seal(HOST, b"", &pw).unwrap();
        assert!(matches!(open(&sealed, &pw), Opened::Sealed { .. }));
    }

    // ───────────────────────── 포스트양자 공개키 봉인 (kdf=2) ─────────────────────────

    #[test]
    fn pq_keypair_sizes_are_fips203() {
        let (ek, dk) = generate_keypair();
        assert_eq!(ek.len(), MLKEM768_EK_LEN, "EK=1184");
        assert_eq!(dk.len(), MLKEM768_DK_LEN, "DK=2400");
    }

    #[test]
    fn pq_generate_seal_open_roundtrips() {
        let (ek, dk) = generate_keypair();
        let sealed = seal_to_pubkey(HOST, SECRET, &ek).unwrap();
        // 가시층은 원본 그대로(순정 한컴이 읽는 것) — 트레일러만 append.
        assert_eq!(visible_layer(&sealed), HOST);
        // 수신자 개인키 → 진짜 비밀 정확 복원.
        match open_with_privkey(&sealed, &dk) {
            Opened::Sealed { plaintext, flags } => {
                assert_eq!(plaintext, SECRET);
                assert_eq!(flags, FLAG_REDACTED);
            }
            other => panic!("Sealed 를 기대: {other:?}"),
        }
    }

    #[test]
    fn pq_wrong_private_key_is_broken_not_panic() {
        let (ek, _dk) = generate_keypair();
        let (_ek2, dk2) = generate_keypair(); // 서로 다른 키쌍의 개인키
        let sealed = seal_to_pubkey(HOST, SECRET, &ek).unwrap();
        // 틀린 개인키 → ML-KEM 묵시적 거부로 *다른* 공유비밀 → AEAD 복호 실패 → Broken(패닉 없음).
        assert!(matches!(
            open_with_privkey(&sealed, &dk2),
            Opened::Broken { .. }
        ));
    }

    #[test]
    fn pq_tampered_ciphertext_is_broken() {
        let (ek, dk) = generate_keypair();
        let mut sealed = seal_to_pubkey(HOST, SECRET, &ek).unwrap();
        let n = sealed.len();
        // ciphertext 는 trailer_len[4]+MAGIC_END[8] 바로 앞에서 끝난다 → n-13 은 AEAD 태그 안.
        sealed[n - 13] ^= 0xFF;
        assert!(matches!(
            open_with_privkey(&sealed, &dk),
            Opened::Broken { .. }
        ));
    }

    #[test]
    fn pq_tampered_encap_header_is_broken() {
        // encap 은 AAD(헤더)의 일부다 → 1비트만 변조해도 복호가 거부된다(AAD 바인딩 + KEM 불일치).
        let (ek, dk) = generate_keypair();
        let mut sealed = seal_to_pubkey(HOST, SECRET, &ek).unwrap();
        let encap_off = HOST.len() + PQ_HEADER_PREFIX + 100; // encap 내부 임의 지점
        sealed[encap_off] ^= 0xFF;
        assert!(matches!(
            open_with_privkey(&sealed, &dk),
            Opened::Broken { .. }
        ));
    }

    #[test]
    fn pq_password_open_on_pq_trailer_is_broken_not_panic() {
        // 비밀번호 경로(open)로 공개키 트레일러를 열면 kdf=2 분기에서 Broken(패닉 없음).
        let (ek, _dk) = generate_keypair();
        let sealed = seal_to_pubkey(HOST, SECRET, &ek).unwrap();
        assert!(matches!(open(&sealed, &pw()), Opened::Broken { .. }));
    }

    #[test]
    fn pq_open_with_privkey_on_password_trailer_is_broken_not_panic() {
        // 공개키 경로(open_with_privkey)로 비밀번호(kdf=1) 트레일러를 열면 Broken(패닉 없음).
        let (_ek, dk) = generate_keypair();
        let sealed = seal(HOST, SECRET, &pw()).unwrap();
        assert!(matches!(
            open_with_privkey(&sealed, &dk),
            Opened::Broken { .. }
        ));
    }

    #[test]
    fn pq_no_trailer_is_plain() {
        let (_ek, dk) = generate_keypair();
        assert_eq!(open_with_privkey(HOST, &dk), Opened::Plain);
    }

    #[test]
    fn pq_stripped_trailer_reopens_as_plain() {
        // 한컴 재저장 = 트레일러 소실 → 개인키 경로도 Plain 으로 정상화(에러 없음).
        let (ek, dk) = generate_keypair();
        let sealed = seal_to_pubkey(HOST, SECRET, &ek).unwrap();
        let stripped = visible_layer(&sealed).to_vec();
        assert_eq!(open_with_privkey(&stripped, &dk), Opened::Plain);
    }

    #[test]
    fn pq_empty_secret_roundtrips() {
        let (ek, dk) = generate_keypair();
        let sealed = seal_to_pubkey(HOST, b"", &ek).unwrap();
        match open_with_privkey(&sealed, &dk) {
            Opened::Sealed { plaintext, .. } => assert_eq!(plaintext, b""),
            other => panic!("Sealed 를 기대: {other:?}"),
        }
    }

    #[test]
    fn pq_bad_public_key_length_is_error() {
        let short = vec![0u8; 100];
        assert!(matches!(
            seal_to_pubkey(HOST, SECRET, &short),
            Err(SealError::BadPublicKey { .. })
        ));
    }

    #[test]
    fn pq_bad_private_key_length_is_broken_not_panic() {
        let (ek, _dk) = generate_keypair();
        let sealed = seal_to_pubkey(HOST, SECRET, &ek).unwrap();
        let short = vec![0u8; 100];
        assert!(matches!(
            open_with_privkey(&sealed, &short),
            Opened::Broken { .. }
        ));
    }
}
