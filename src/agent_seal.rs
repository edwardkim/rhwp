//! 에이전트 전용 봉인(agent seal) — 사람 암호가 없는 완전 엔트로피 봉인과
//! 정보이론적 일회용 패드(OTP) 봉인.
//!
//! ## 왜 "에이전트 전용"인가 — 약한 고리는 사람이 고른 암호다
//!
//! 문서 암호화의 실질 강도는 알고리즘이 아니라 **키의 엔트로피**가 정한다.
//! 사람은 낮은 엔트로피 암호를 고르고(사전 단어·생일·재사용), Argon2id 같은
//! 강한 키유도(KDF)조차 이 사실을 늦출 뿐 없애지 못한다. 반면 **에이전트는
//! 완전 엔트로피 기계 키**(OS 난수 32바이트)를 직접 보관·전달할 수 있다.
//! 이 모듈의 1번 모드는 그 전제를 이용한다 — 암호도 KDF도 없이, 난수 키
//! 그대로가 "에이전트 암호"다. 이 키를 base64/hex 로 인코딩해 다루는 것은
//! 호출자 몫이다.
//!
//! ## "양자보다 강하다"는 말의 유일한 정직한 형태 — OTP
//!
//! 어떤 암호도 "양자내성(quantum-resistant)보다 더 강하다"고 말할 수 없다.
//! **딱 하나 예외가 정보이론적 안전성(일회용 패드)이다** — 어떤 컴퓨터로도,
//! 양자든 고전이든, 무한한 시간을 줘도 깰 수 없다(Shannon 완전비밀).
//! 그러나 이는 **오직** 다음이 모두 성립할 때만 참이다:
//!
//! 1. 패드가 **진짜 난수**여야 한다(의사난수 PRNG 는 안 된다).
//! 2. 패드 길이가 **메시지 길이 이상**이어야 한다.
//! 3. 패드를 **정확히 한 번만** 써야 한다(재사용 시 완전히 깨진다).
//! 4. 패드를 **대역 외(out-of-band)로 안전하게** 공유해야 한다.
//!
//! 이 조건을 못 지키면 OTP 는 오히려 약한 XOR 암호로 전락한다. 그리고 OTP
//! 는 **기밀성만** 준다 — 무결성/인증이 없어 비트 뒤집기 변조를 탐지하지
//! 못한다(1번 모드의 AEAD 와 대비되는 정직한 한계다).
//!
//! 이 모듈은 두 가지를 **정직하게** 제공한다. 마법의 "양자 초월 암호" 같은
//! 것은 없다:
//!
//! - **1번 모드 — 완전 엔트로피 에이전트 키(계산적 안전, 양자내성)**:
//!   XChaCha20-Poly1305 AEAD. 사람 암호·KDF 없음. 인증 있음.
//! - **2번 모드 — 일회용 패드(정보이론적 안전)**: XOR. 위 4조건이 지켜질
//!   때에 한해 무조건적으로 안전. 인증 없음.
//!
//! ## 컨테이너 형식 — 호스트 문서 뒤에 덧붙는 자기서술 트레일러
//!
//! 봉인은 호스트 문서 **뒤에** 트레일러로 붙는다. 평범한 뷰어는 문서 끝의
//! 낯선 바이트를 무시하므로 호스트는 그대로 열린다. 트레일러는 이 모듈만의
//! 마법 마커(`RHWPAGT1` … `RHWPAGND`)로 감싸며, `security_trailer` 의 공개키
//! 형식과 **완전히 독립**이다.
//!
//! ```text
//!   MAGIC_START "RHWPAGT1"  (8)
//!   version                 (1)  = 1
//!   algo                    (1)  = 1(XChaCha20-Poly1305) | 2(OTP-XOR)
//!   nonce_len               (1)  = 24(XChaCha) | 0(OTP)
//!   nonce                   (nonce_len)
//!   ── 위까지가 header = AEAD 의 AAD ──
//!   ciphertext              (가변)   // XChaCha: AEAD(secret)+16B 태그, OTP: secret XOR pad
//!   trailer_len  u64 LE     (8)      // 양 마법을 포함한 트레일러 총 길이
//!   MAGIC_END "RHWPAGND"    (8)
//! ```
//!
//! 트레일러는 꼬리에서부터 탐지한다: 마지막 8바이트가 `RHWPAGND` 인지 보고,
//! 그 앞 8바이트(`trailer_len`)로 시작 위치를 되짚어 `RHWPAGT1` 을 확인한다.
//! 모든 인덱싱은 경계 검사를 거치며, 어떤 손상된 입력에도 **패닉하지 않는다**.

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};

/// 트레일러 시작 마법. `security_trailer` 형식과 절대 겹치지 않는 고유 마커다.
const MAGIC_START: [u8; 8] = *b"RHWPAGT1";
/// 트레일러 끝 마법.
const MAGIC_END: [u8; 8] = *b"RHWPAGND";
/// 트레일러 형식 버전. 향후 변경 시 이 값으로 분기한다.
const FORMAT_VERSION: u8 = 1;
/// algo 바이트 — 1번 모드(완전 엔트로피 AEAD).
const ALGO_XCHACHA20_POLY1305: u8 = 1;
/// algo 바이트 — 2번 모드(정보이론적 일회용 패드).
const ALGO_OTP_XOR: u8 = 2;
/// XChaCha20-Poly1305 의 논스 길이(바이트).
const XNONCE_LEN: usize = 24;
/// 고정 헤더 접두(마법 8 + 버전 1 + algo 1 + nonce_len 1)의 길이.
const HEADER_PREFIX_LEN: usize = 8 + 1 + 1 + 1;
/// 고정 꼬리(trailer_len 8 + 끝 마법 8)의 길이.
const FOOTER_LEN: usize = 8 + 8;
/// 가능한 최소 트레일러 길이(nonce_len=0, ciphertext 없음: OTP 로 빈 secret 을 봉인한 경우).
const MIN_TRAILER_LEN: usize = HEADER_PREFIX_LEN + FOOTER_LEN;

/// `open_with_key` / `otp_open` 의 결과.
///
/// 이 세 갈래는 서로 배타적이다:
/// - `Plain`: 트레일러가 없다(우리 봉인이 아닌 평문 호스트 문서).
/// - `Sealed`: 봉인을 풀었고 `plaintext` 가 복원된 비밀이다.
/// - `Broken`: 트레일러는 우리 것이 맞지만 열 수 없다(키 불일치·변조·손상·형식
///   오류). 어떤 경우에도 패닉 대신 이 값을 돌려준다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Opened {
    /// 봉인 트레일러가 없다 — 평문 호스트 문서 그대로다.
    Plain,
    /// 봉인을 풀었다. `plaintext` 는 `seal`/`otp_seal` 에 넣은 비밀이다.
    Sealed {
        /// 복원된 비밀 평문.
        plaintext: Vec<u8>,
    },
    /// 우리 트레일러이나 열 수 없다(사유 포함).
    Broken {
        /// 사람이 읽을 수 있는 실패 사유.
        reason: String,
    },
}

/// 봉인 과정에서 발생할 수 있는 오류.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentSealError {
    /// OTP 패드가 메시지보다 짧다 — 일회용 패드 규칙(패드 길이 ≥ 메시지 길이)
    /// 위반이라, 잘린 봉인을 만드는 대신 오류를 돌려 규칙을 강제한다.
    PadTooShort {
        /// 필요한 패드 길이(= 메시지 길이).
        needed: usize,
        /// 실제 패드 길이.
        got: usize,
    },
}

impl std::fmt::Display for AgentSealError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentSealError::PadTooShort { needed, got } => write!(
                f,
                "OTP 패드가 너무 짧습니다: {needed}바이트 필요, {got}바이트 제공 (패드는 메시지 이상이어야 하고 한 번만 써야 합니다)"
            ),
        }
    }
}

impl std::error::Error for AgentSealError {}

// ── 1번 모드 — 완전 엔트로피 에이전트 키 ─────────────────────────────────────

/// OS 엔트로피에서 32바이트 완전 엔트로피 에이전트 키를 만든다.
///
/// 이 원시 키 **자체가** "에이전트 암호"다 — 사람 암호도 Argon2id 도 없다.
/// 호출자는 이 키를 base64/hex 로 인코딩해 보관·전달한다. 키를 잃으면 봉인은
/// 복구 불가다(그것이 요점이다).
///
/// OS 엔트로피 획득 실패는 복구 불가능한 호스트 환경 장애이므로 패닉한다
/// (문서 입력으로 유발되는 경로가 아니다).
pub fn agent_keygen() -> [u8; 32] {
    let mut key = [0u8; 32];
    getrandom::fill(&mut key).expect("OS 엔트로피(getrandom)를 사용할 수 없습니다");
    key
}

/// 완전 엔트로피 키로 비밀을 봉인해 `host || trailer` 를 돌려준다.
///
/// XChaCha20-Poly1305 AEAD 로 `secret` 을 암호화한다. 논스는 매 호출 새로
/// 뽑은 24바이트 난수이고, 트레일러 헤더(마법·버전·algo·nonce_len·nonce)를
/// AAD 로 묶어 헤더 변조를 탐지한다. 트레일러는 `host` **뒤에** 붙으므로
/// 평범한 뷰어는 `host` 를 그대로 연다(가시 계층 = `host`).
///
/// 반환값은 항상 `Vec<u8>` 다. AEAD 암호화는 유효한 키·논스에 대해 사실상
/// 실패하지 않는다(메시지가 ~256GB 를 넘는 비현실적 경우에만 실패).
pub fn seal_with_key(host: &[u8], secret: &[u8], key: &[u8; 32]) -> Vec<u8> {
    let mut nonce = [0u8; XNONCE_LEN];
    getrandom::fill(&mut nonce).expect("OS 엔트로피(getrandom)를 사용할 수 없습니다");

    // 헤더 = AAD. 암호화 이전에 확정되는 바이트만 담는다(논스 포함).
    let header = build_header(ALGO_XCHACHA20_POLY1305, &nonce);

    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key[..]));
    let ciphertext = cipher
        .encrypt(
            XNonce::from_slice(&nonce),
            Payload {
                msg: secret,
                aad: &header,
            },
        )
        .expect("XChaCha20-Poly1305 암호화는 유효한 키/논스에서 실패하지 않습니다");

    let trailer = finish_trailer(header, &ciphertext);

    let mut out = Vec::with_capacity(host.len() + trailer.len());
    out.extend_from_slice(host);
    out.extend_from_slice(&trailer);
    out
}

/// `seal_with_key` 로 봉인한 바이트에서 완전 엔트로피 키로 비밀을 복원한다.
///
/// 트레일러를 꼬리에서 탐지한다. 트레일러가 없으면 `Plain`, 우리 트레일러이나
/// 키 불일치·변조·손상이면 `Broken`, 성공하면 `Sealed{plaintext}` 를 돌려준다.
/// 어떤 손상된 입력에도 **패닉하지 않는다**(엄격한 경계 검사).
pub fn open_with_key(bytes: &[u8], key: &[u8; 32]) -> Opened {
    let parsed = match parse_trailer(bytes) {
        ParseResult::NoTrailer => return Opened::Plain,
        ParseResult::Broken(reason) => return Opened::Broken { reason },
        ParseResult::Found(p) => p,
    };

    if parsed.algo != ALGO_XCHACHA20_POLY1305 {
        return Opened::Broken {
            reason: format!(
                "XChaCha20-Poly1305 봉인이 아닙니다(algo={}). OTP 봉인은 otp_open 으로 여세요.",
                parsed.algo
            ),
        };
    }
    if parsed.nonce.len() != XNONCE_LEN {
        return Opened::Broken {
            reason: format!("논스 길이가 잘못되었습니다: {}바이트", parsed.nonce.len()),
        };
    }

    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key[..]));
    match cipher.decrypt(
        XNonce::from_slice(parsed.nonce),
        Payload {
            msg: parsed.ciphertext,
            aad: parsed.aad,
        },
    ) {
        Ok(plaintext) => Opened::Sealed { plaintext },
        Err(_) => Opened::Broken {
            reason: "키 불일치 또는 암호문 변조(AEAD 인증 실패)".to_string(),
        },
    }
}

// ── 2번 모드 — 일회용 패드(정보이론적 안전) ──────────────────────────────────

/// OS 엔트로피에서 `len` 바이트의 일회용 패드를 만든다.
///
/// **주의**: 정보이론적 안전은 이 패드가 (1) 진짜 난수이고, (2) 메시지 길이
/// 이상이며, (3) **정확히 한 번만** 쓰이고, (4) 대역 외로 안전하게 공유될
/// 때에만 성립한다. 이 함수는 (1) 진짜 난수와 (2) 원하는 길이만 보장한다 —
/// 재사용 금지와 안전한 배포는 호출자의 책임이다. 한 패드를 두 메시지에
/// 쓰면 보안이 **완전히** 무너진다.
pub fn otp_generate_pad(len: usize) -> Vec<u8> {
    let mut pad = vec![0u8; len];
    getrandom::fill(&mut pad).expect("OS 엔트로피(getrandom)를 사용할 수 없습니다");
    pad
}

/// 일회용 패드로 비밀을 봉인한다. `ciphertext = secret XOR pad[..secret.len()]`.
///
/// `pad.len() >= secret.len()` 을 요구한다 — 짧으면 잘린 봉인을 만드는 대신
/// [`AgentSealError::PadTooShort`] 를 돌려 OTP 규칙을 강제한다. 결과는 1번
/// 모드와 **동일한 트레일러 컨테이너**(algo 바이트만 다름)라 자기서술적이다.
/// 호스트 접두는 붙이지 않는다(독립 봉인 블롭).
///
/// **정직한 한계**: OTP 는 기밀성만 준다. 무결성/인증이 없어 `otp_open` 은
/// 비트 뒤집기 변조를 탐지하지 못한다. 인증이 필요하면 1번 모드를 쓰라.
pub fn otp_seal(secret: &[u8], pad: &[u8]) -> Result<Vec<u8>, AgentSealError> {
    if pad.len() < secret.len() {
        return Err(AgentSealError::PadTooShort {
            needed: secret.len(),
            got: pad.len(),
        });
    }
    let ciphertext: Vec<u8> = secret.iter().zip(pad.iter()).map(|(s, p)| s ^ p).collect();

    // OTP 는 논스가 없다(nonce_len=0). 헤더는 자기서술을 위해서만 쓴다.
    let header = build_header(ALGO_OTP_XOR, &[]);
    Ok(finish_trailer(header, &ciphertext))
}

/// 일회용 패드로 봉인한 바이트에서 비밀을 복원한다. XOR 로 되돌린다.
///
/// 트레일러가 없으면 `Plain`, OTP 트레일러가 아니거나 손상되었으면 `Broken`,
/// 성공하면 `Sealed{plaintext}` 를 돌려준다. 패드가 암호문보다 짧으면(복원
/// 불가) `Broken` 이다. 어떤 손상된 입력에도 **패닉하지 않는다**.
pub fn otp_open(sealed: &[u8], pad: &[u8]) -> Opened {
    let parsed = match parse_trailer(sealed) {
        ParseResult::NoTrailer => return Opened::Plain,
        ParseResult::Broken(reason) => return Opened::Broken { reason },
        ParseResult::Found(p) => p,
    };

    if parsed.algo != ALGO_OTP_XOR {
        return Opened::Broken {
            reason: format!(
                "OTP 봉인이 아닙니다(algo={}). XChaCha20-Poly1305 봉인은 open_with_key 로 여세요.",
                parsed.algo
            ),
        };
    }
    if pad.len() < parsed.ciphertext.len() {
        return Opened::Broken {
            reason: format!(
                "패드가 암호문보다 짧습니다: {}바이트 필요, {}바이트 제공",
                parsed.ciphertext.len(),
                pad.len()
            ),
        };
    }
    let plaintext: Vec<u8> = parsed
        .ciphertext
        .iter()
        .zip(pad.iter())
        .map(|(c, p)| c ^ p)
        .collect();
    Opened::Sealed { plaintext }
}

// ── 내부 트레일러 조립/해석 ──────────────────────────────────────────────────

/// 헤더(= AAD 후보) 바이트를 만든다: 마법 || 버전 || algo || nonce_len || nonce.
fn build_header(algo: u8, nonce: &[u8]) -> Vec<u8> {
    let mut header = Vec::with_capacity(HEADER_PREFIX_LEN + nonce.len());
    header.extend_from_slice(&MAGIC_START);
    header.push(FORMAT_VERSION);
    header.push(algo);
    // nonce.len() 은 24(XChaCha) 또는 0(OTP) 이라 항상 u8 에 들어간다.
    header.push(nonce.len() as u8);
    header.extend_from_slice(nonce);
    header
}

/// 헤더와 암호문에 꼬리(trailer_len + 끝 마법)를 붙여 완성한 트레일러를 돌려준다.
fn finish_trailer(mut header_and_ct: Vec<u8>, ciphertext: &[u8]) -> Vec<u8> {
    header_and_ct.extend_from_slice(ciphertext);
    let trailer_len = (header_and_ct.len() + FOOTER_LEN) as u64;
    header_and_ct.extend_from_slice(&trailer_len.to_le_bytes());
    header_and_ct.extend_from_slice(&MAGIC_END);
    header_and_ct
}

/// 파싱된 트레일러 뷰(입력 바이트를 빌려 참조한다).
struct ParsedTrailer<'a> {
    algo: u8,
    nonce: &'a [u8],
    ciphertext: &'a [u8],
    /// AEAD 검증에 쓸 AAD = 헤더(마법..nonce). seal 시의 AAD 와 바이트가 같다.
    aad: &'a [u8],
}

/// `parse_trailer` 의 결과.
enum ParseResult<'a> {
    /// 트레일러 없음 → 평문.
    NoTrailer,
    /// 우리 트레일러이나 손상됨 → Broken(사유).
    Broken(String),
    /// 정상 파싱됨.
    Found(ParsedTrailer<'a>),
}

/// 꼬리에서부터 트레일러를 탐지·해석한다. 모든 인덱싱은 경계 검사를 거친다.
fn parse_trailer(bytes: &[u8]) -> ParseResult<'_> {
    let n = bytes.len();

    // 1) 끝 마법이 없으면 우리 봉인이 아니다 → 평문.
    if n < MAGIC_END.len() || bytes[n - MAGIC_END.len()..] != MAGIC_END {
        return ParseResult::NoTrailer;
    }
    // 끝 마법은 있는데 최소 크기에도 못 미치면 손상된 우리 트레일러다.
    if n < MIN_TRAILER_LEN {
        return ParseResult::Broken("트레일러가 최소 길이보다 짧습니다".to_string());
    }

    // 2) trailer_len 을 읽어 시작 위치를 되짚는다.
    let tl_bytes: [u8; 8] = match bytes.get(n - FOOTER_LEN..n - MAGIC_END.len()) {
        Some(s) => match s.try_into() {
            Ok(a) => a,
            Err(_) => return ParseResult::Broken("trailer_len 필드가 잘렸습니다".to_string()),
        },
        None => return ParseResult::Broken("trailer_len 필드가 잘렸습니다".to_string()),
    };
    let trailer_len_u64 = u64::from_le_bytes(tl_bytes);
    let trailer_len = match usize::try_from(trailer_len_u64) {
        Ok(v) => v,
        Err(_) => {
            return ParseResult::Broken("trailer_len 이 usize 범위를 벗어났습니다".to_string())
        }
    };
    if trailer_len < MIN_TRAILER_LEN || trailer_len > n {
        return ParseResult::Broken("trailer_len 이 범위를 벗어났습니다".to_string());
    }

    // 3) 시작 마법 확인.
    let start = n - trailer_len;
    let body = &bytes[start..n];
    if body[..MAGIC_START.len()] != MAGIC_START {
        return ParseResult::Broken("시작 마법이 없습니다".to_string());
    }

    // 4) 헤더 필드.
    let version = body[8];
    if version != FORMAT_VERSION {
        return ParseResult::Broken(format!("알 수 없는 형식 버전: {version}"));
    }
    let algo = body[9];
    let nonce_len = body[10] as usize;

    // 헤더 = body[0..header_end], header_end = 고정접두 + nonce_len.
    let header_end = HEADER_PREFIX_LEN + nonce_len;
    // ciphertext 는 header_end..(m - FOOTER_LEN) 이어야 한다. 경계 검사.
    let m = body.len();
    let ct_end = match m.checked_sub(FOOTER_LEN) {
        Some(v) => v,
        None => return ParseResult::Broken("트레일러 꼬리가 잘렸습니다".to_string()),
    };
    if header_end > ct_end {
        return ParseResult::Broken("헤더(논스)가 트레일러 경계를 넘습니다".to_string());
    }

    let aad = &body[..header_end];
    let nonce = &body[HEADER_PREFIX_LEN..header_end];
    let ciphertext = &body[header_end..ct_end];

    ParseResult::Found(ParsedTrailer {
        algo,
        nonce,
        ciphertext,
        aad,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOST: &[u8] = b"\x50\x4b\x03\x04 pretend this is an HWPX/HWP host document body ...";

    // ── 1번 모드 — 완전 엔트로피 키 ──

    #[test]
    fn keygen_is_32_bytes_and_random() {
        let a = agent_keygen();
        let b = agent_keygen();
        assert_eq!(a.len(), 32);
        // 두 키가 같을 확률은 2^-256 — 사실상 불가능.
        assert_ne!(a, b, "연속 키생성이 같은 값을 냈습니다(엔트로피 이상)");
    }

    #[test]
    fn seal_open_roundtrip_recovers_secret() {
        let key = agent_keygen();
        let secret = b"top secret agent payload \x00\xff\x10";
        let sealed = seal_with_key(HOST, secret, &key);

        // 가시 계층 = 호스트: 봉인 바이트의 접두가 호스트와 정확히 같다.
        assert_eq!(&sealed[..HOST.len()], HOST, "호스트가 가시 접두여야 합니다");
        assert!(sealed.len() > HOST.len(), "트레일러가 붙어야 합니다");

        match open_with_key(&sealed, &key) {
            Opened::Sealed { plaintext } => assert_eq!(plaintext, secret),
            other => panic!("Sealed 를 기대했으나 {other:?}"),
        }
    }

    #[test]
    fn empty_secret_roundtrips() {
        let key = agent_keygen();
        let sealed = seal_with_key(HOST, b"", &key);
        assert_eq!(&sealed[..HOST.len()], HOST);
        match open_with_key(&sealed, &key) {
            Opened::Sealed { plaintext } => assert_eq!(plaintext, b""),
            other => panic!("빈 비밀 Sealed 를 기대했으나 {other:?}"),
        }
    }

    #[test]
    fn wrong_key_is_broken_not_panic() {
        let key = agent_keygen();
        let mut wrong = agent_keygen();
        // 두 키가 우연히 같지 않도록 보장.
        if wrong == key {
            wrong[0] ^= 0xff;
        }
        let sealed = seal_with_key(HOST, b"secret", &key);
        match open_with_key(&sealed, &wrong) {
            Opened::Broken { .. } => {}
            other => panic!("Broken 을 기대했으나 {other:?}"),
        }
    }

    #[test]
    fn random_key_cannot_open_another_keys_seal() {
        // 명시적으로: 서로 다른 두 무작위 키는 서로의 봉인을 못 연다.
        let k1 = agent_keygen();
        let k2 = agent_keygen();
        assert_ne!(k1, k2);
        let sealed = seal_with_key(HOST, b"cross-key secret", &k1);
        assert!(matches!(open_with_key(&sealed, &k2), Opened::Broken { .. }));
        // 반대 방향도.
        let sealed2 = seal_with_key(HOST, b"cross-key secret", &k2);
        assert!(matches!(
            open_with_key(&sealed2, &k1),
            Opened::Broken { .. }
        ));
    }

    #[test]
    fn tampered_ciphertext_is_broken() {
        let key = agent_keygen();
        let mut sealed = seal_with_key(HOST, b"authenticate me", &key);
        // 암호문 영역(호스트와 트레일러 꼬리 사이)의 한 바이트를 뒤집는다.
        // 호스트 바로 뒤가 헤더, 그 뒤가 암호문이다. 안전하게 중간 지점을 고른다.
        let mid = HOST.len() + HEADER_PREFIX_LEN + XNONCE_LEN + 1;
        assert!(mid < sealed.len() - FOOTER_LEN);
        sealed[mid] ^= 0x01;
        match open_with_key(&sealed, &key) {
            Opened::Broken { .. } => {}
            other => panic!("변조 → Broken 을 기대했으나 {other:?}"),
        }
    }

    #[test]
    fn plain_host_has_no_trailer() {
        let key = agent_keygen();
        // 우리 마법으로 끝나지 않는 평범한 문서.
        assert_eq!(open_with_key(HOST, &key), Opened::Plain);
    }

    #[test]
    fn malformed_bytes_never_panic() {
        let key = agent_keygen();
        // 여러 병적 입력: 빈 것, 짧은 것, 끝 마법만 있는 것, 끝 마법 + 쓰레기 길이.
        assert_eq!(open_with_key(&[], &key), Opened::Plain);
        assert_eq!(open_with_key(b"short", &key), Opened::Plain);

        // 끝 마법만 붙인 너무 짧은 입력 → Broken(패닉 아님).
        let mut only_end = vec![0u8; 4];
        only_end.extend_from_slice(&MAGIC_END);
        assert!(matches!(
            open_with_key(&only_end, &key),
            Opened::Broken { .. }
        ));

        // 끝 마법 + 말도 안 되는 trailer_len.
        let mut bad_len = vec![0xAAu8; 64];
        bad_len.extend_from_slice(&u64::MAX.to_le_bytes());
        bad_len.extend_from_slice(&MAGIC_END);
        assert!(matches!(
            open_with_key(&bad_len, &key),
            Opened::Broken { .. }
        ));

        // 유효한 봉인을 잘라 꼬리만 남기면 Broken 또는 Plain(패닉 금지)만 나온다.
        let sealed = seal_with_key(HOST, b"x", &key);
        for cut in 0..sealed.len() {
            let _ = open_with_key(&sealed[..cut], &key); // 패닉만 안 하면 통과.
            let _ = open_with_key(&sealed[cut..], &key);
        }
    }

    // ── 2번 모드 — 일회용 패드 ──

    #[test]
    fn otp_roundtrip_recovers_secret() {
        let secret = b"information-theoretic secret \x00\x01\x02";
        let pad = otp_generate_pad(secret.len());
        let sealed = otp_seal(secret, &pad).expect("패드가 충분히 길다");
        match otp_open(&sealed, &pad) {
            Opened::Sealed { plaintext } => assert_eq!(plaintext, secret),
            other => panic!("OTP Sealed 를 기대했으나 {other:?}"),
        }
    }

    #[test]
    fn otp_longer_pad_is_ok() {
        let secret = b"short";
        let pad = otp_generate_pad(secret.len() + 100);
        let sealed = otp_seal(secret, &pad).expect("긴 패드 허용");
        match otp_open(&sealed, &pad) {
            Opened::Sealed { plaintext } => assert_eq!(plaintext, secret),
            other => panic!("OTP Sealed 를 기대했으나 {other:?}"),
        }
    }

    #[test]
    fn otp_pad_too_short_is_error_not_truncated_seal() {
        let secret = b"this is longer than the pad";
        let pad = otp_generate_pad(4);
        match otp_seal(secret, &pad) {
            Err(AgentSealError::PadTooShort { needed, got }) => {
                assert_eq!(needed, secret.len());
                assert_eq!(got, 4);
            }
            Ok(_) => panic!("짧은 패드로 봉인이 성공하면 안 됩니다(OTP 규칙 위반)"),
        }
    }

    #[test]
    fn otp_open_with_short_pad_is_broken() {
        let secret = b"recover needs full pad";
        let pad = otp_generate_pad(secret.len());
        let sealed = otp_seal(secret, &pad).unwrap();
        // 복원 시 패드가 암호문보다 짧으면 Broken.
        match otp_open(&sealed, &pad[..secret.len() - 1]) {
            Opened::Broken { .. } => {}
            other => panic!("짧은 패드 복원 → Broken 을 기대했으나 {other:?}"),
        }
    }

    #[test]
    fn otp_ciphertext_is_secret_xor_pad() {
        // OTP 의 정의를 직접 검증: 트레일러에서 뽑은 암호문 = secret XOR pad.
        let secret = b"xor me";
        let pad = otp_generate_pad(secret.len());
        let sealed = otp_seal(secret, &pad).unwrap();
        // 암호문 영역만 추출해 확인.
        if let ParseResult::Found(p) = parse_trailer(&sealed) {
            let expect: Vec<u8> = secret.iter().zip(pad.iter()).map(|(s, x)| s ^ x).collect();
            assert_eq!(p.ciphertext, &expect[..]);
        } else {
            panic!("OTP 트레일러 파싱 실패");
        }
    }

    #[test]
    fn otp_malformed_never_panic() {
        let pad = otp_generate_pad(32);
        assert_eq!(otp_open(&[], &pad), Opened::Plain);
        assert_eq!(otp_open(b"nope", &pad), Opened::Plain);
        let sealed = otp_seal(b"hello", &otp_generate_pad(5)).unwrap();
        for cut in 0..sealed.len() {
            let _ = otp_open(&sealed[..cut], &pad); // 패닉 금지.
        }
    }

    // ── 교차 모드 자기서술 검증 ──

    #[test]
    fn open_with_key_rejects_otp_blob() {
        let key = agent_keygen();
        let sealed = otp_seal(b"otp payload", &otp_generate_pad(11)).unwrap();
        // OTP 봉인을 AEAD open 으로 열려 하면 Broken(algo 불일치).
        match open_with_key(&sealed, &key) {
            Opened::Broken { .. } => {}
            other => panic!("algo 불일치 → Broken 을 기대했으나 {other:?}"),
        }
    }

    #[test]
    fn otp_open_rejects_xchacha_blob() {
        let key = agent_keygen();
        let sealed = seal_with_key(HOST, b"aead payload", &key);
        let pad = otp_generate_pad(sealed.len());
        // AEAD 봉인을 OTP open 으로 열려 하면 Broken(algo 불일치).
        match otp_open(&sealed, &pad) {
            Opened::Broken { .. } => {}
            other => panic!("algo 불일치 → Broken 을 기대했으나 {other:?}"),
        }
    }
}
