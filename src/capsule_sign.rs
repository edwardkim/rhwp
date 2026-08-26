//! [#4509] 캡슐 서명 — 귀속(4년 축)의 발급·검증 코어.
//!
//! ## 무엇을 봉인하나 — 파일 바이트, 정규화 아님
//!
//! 서명 대상은 캡슐 **파일 바이트 그대로**다. 정규화(canonical JSON) 서명은
//! 재직렬화 내성을 주지만 정규화 규칙 자체가 공격면이고 구현 분기점이다 —
//! 계보 축 전체가 이미 파일 바이트 해시(부모 링크) 위에 서 있으므로, 서명도
//! 같은 대상을 봉인해야 두 체계가 어긋나지 않는다. "캡슐 파일은 발급 후
//! 불변"은 계보의 기존 전제라 새 제약이 아니다 (설계서 §2.1, devel
//! horizon_year4_signing.md).
//!
//! ## 왜 분리 서명(sidecar)인가
//!
//! 서명을 캡슐 안에 넣으면 "서명 필드를 제외한 바이트"라는 정규화 문제가
//! 되돌아온다. `<캡슐>.sig.json` 분리 파일이면 캡슐 바이트는 그대로이고,
//! 계보(부모 해시)·감사(파일 해시)와의 정합이 공짜다.
//!
//! ## 왜 Ed25519 인가
//!
//! 결정론 서명(같은 키·같은 바이트 → 같은 서명)이라 이 저장소의 결정론
//! 문화(replay·lineage 의 재현 판정)와 정합하고, 키·서명이 짧으며(32B/64B),
//! 검증 구현이 보편적이다. `alg` 필드로 후속 알고리즘 여지는 남긴다.
//!
//! ## 이 모듈이 하지 않는 것 (경계)
//!
//! - **서명 시점 증명** — `signedAt` 은 주장 필드다. 시점 증명은 5년 축(앵커).
//! - **키 유출 소급 판정** — 폐기 기록 이후의 신규 검증만 잡는다 (S3).
//! - **키 등록부의 신뢰 뿌리** — 어떤 keyring 을 믿을지는 도구 밖 거버넌스다.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine as _;
use ed25519_dalek::{Signature, Signer as _, SigningKey, Verifier as _, VerifyingKey};

use rhwp::schema_registry::SIGNING_SCHEMA_VERSION;

/// 재수출 — 하네스 init 이 키링 골격에 쓴다 (레지스트리 단일 출처 유지).
pub const SIGNING_SCHEMA_VERSION_STR: &str = SIGNING_SCHEMA_VERSION;

/// 키 파일의 `kind` — 비밀키를 담으므로 절대 캡슐·봉투에 인라인하지 않는다.
pub const KEY_KIND: &str = "ed25519Key";
/// 분리 서명 파일의 `kind`.
pub const SIG_KIND: &str = "capsuleSignature";
/// 키 등록부의 `kind`.
pub const KEYRING_KIND: &str = "keyring";
/// 서명 알고리즘 표기 — 후속 알고리즘 추가 시 이 값으로 분기한다.
pub const ALG: &str = "ed25519";

/// `<캡슐>.sig.json` — 분리 서명의 기본 위치 규약.
pub fn sidecar_path(capsule_path: &str) -> String {
    format!("{capsule_path}.sig.json")
}

/// 32바이트 OS 엔트로피로 새 서명키를 만들어 키 파일 JSON 을 돌려준다.
///
/// 반환 JSON 은 **비밀키를 담는다** — 호출자는 파일 권한·보관 책임을 진다
/// (Windows 에는 0600 이 없다 — 운영 수칙은 기술 문서에).
pub fn generate_key_json(key_id: &str) -> Result<serde_json::Value, String> {
    let mut secret = [0u8; 32];
    getrandom::fill(&mut secret).map_err(|e| format!("OS 엔트로피 획득 실패: {e}"))?;
    let signing = SigningKey::from_bytes(&secret);
    let public = signing.verifying_key();
    Ok(serde_json::json!({
        "schemaVersion": SIGNING_SCHEMA_VERSION,
        "kind": KEY_KIND,
        "keyId": key_id,
        "alg": ALG,
        "secret": B64.encode(secret),
        "publicKey": B64.encode(public.to_bytes()),
    }))
}

/// 키 파일에서 서명키를 복원한다. 반환: (서명키, keyId, publicKey b64).
pub fn load_signing_key(path: &str) -> Result<(SigningKey, String, String), String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("키 파일을 읽을 수 없습니다 - {path}: {e}"))?;
    let v: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("키 파일 JSON 파싱 실패 - {path}: {e}"))?;
    if v["kind"] != KEY_KIND {
        return Err(format!("키 파일 kind 가 {KEY_KIND} 가 아닙니다 - {path}"));
    }
    let key_id = v["keyId"]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("키 파일에 keyId 가 없습니다 - {path}"))?
        .to_string();
    let secret_b64 = v["secret"]
        .as_str()
        .ok_or_else(|| format!("키 파일에 secret 이 없습니다 - {path}"))?;
    let secret = B64
        .decode(secret_b64)
        .map_err(|e| format!("secret base64 해독 실패 - {path}: {e}"))?;
    let secret: [u8; 32] = secret
        .try_into()
        .map_err(|_| format!("secret 은 32바이트여야 합니다 - {path}"))?;
    let signing = SigningKey::from_bytes(&secret);
    let public_b64 = B64.encode(signing.verifying_key().to_bytes());
    Ok((signing, key_id, public_b64))
}

/// 캡슐 바이트를 서명해 분리 서명 파일의 JSON 을 만든다.
pub fn make_sidecar_json(
    signing: &SigningKey,
    key_id: &str,
    capsule_sha256: &str,
    capsule_bytes: &[u8],
) -> serde_json::Value {
    let sig: Signature = signing.sign(capsule_bytes);
    serde_json::json!({
        "schemaVersion": SIGNING_SCHEMA_VERSION,
        "kind": SIG_KIND,
        "capsuleSha256": capsule_sha256,
        "alg": ALG,
        "keyId": key_id,
        "signature": B64.encode(sig.to_bytes()),
        // 주장 필드 — 시점 증명이 아니다. 증명은 5년 축(앵커)의 몫 (설계서 §2.2).
        "signedAt": rfc3339_utc_now(),
    })
}

/// 키 등록부 항목 — 공개키와 폐기 기록.
pub struct KeyEntry {
    pub public: VerifyingKey,
    pub revoked: Option<serde_json::Value>,
}

/// keyring.json 을 keyId → 항목 지도로 읽는다.
pub fn load_keyring(path: &str) -> Result<BTreeMap<String, KeyEntry>, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("키 등록부를 읽을 수 없습니다 - {path}: {e}"))?;
    let v: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("키 등록부 JSON 파싱 실패 - {path}: {e}"))?;
    keyring_from_value(&v, path)
}

/// 값으로 받은 keyring — 연합(trust-domain 인라인 keyring, #4549)이 쓴다.
pub fn keyring_from_value(
    v: &serde_json::Value,
    origin: &str,
) -> Result<BTreeMap<String, KeyEntry>, String> {
    if v["kind"] != KEYRING_KIND {
        return Err(format!(
            "키 등록부 kind 가 {KEYRING_KIND} 가 아닙니다 - {origin}"
        ));
    }
    let mut map = BTreeMap::new();
    for (idx, key) in v["keys"]
        .as_array()
        .ok_or_else(|| format!("키 등록부에 keys 배열이 없습니다 - {origin}"))?
        .iter()
        .enumerate()
    {
        let key_id = key["keyId"]
            .as_str()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| format!("keys[{idx}].keyId 가 없습니다 - {origin}"))?;
        let public_b64 = key["publicKey"]
            .as_str()
            .ok_or_else(|| format!("keys[{idx}].publicKey 가 없습니다 - {origin}"))?;
        let public = B64
            .decode(public_b64)
            .ok()
            .and_then(|b| <[u8; 32]>::try_from(b).ok())
            .and_then(|b| VerifyingKey::from_bytes(&b).ok())
            .ok_or_else(|| {
                format!("keys[{idx}].publicKey 가 유효한 Ed25519 공개키가 아닙니다 - {origin}")
            })?;
        let revoked = match key.get("revoked") {
            None | Some(serde_json::Value::Null) => None,
            Some(r) => Some(r.clone()),
        };
        map.insert(key_id.to_string(), KeyEntry { public, revoked });
    }
    Ok(map)
}

/// 서명 판정 — 봉투에 그대로 실리는 필드 묶음.
pub struct SigVerdict {
    /// 암호학적 검증 결과. 키를 몰라 검증 자체가 불가능하면 `None`.
    pub signature_ok: Option<bool>,
    pub key_id: Option<String>,
    pub key_known: bool,
    pub revoked: Option<serde_json::Value>,
    /// valid | invalid | unknownKey | revoked | malformed
    pub verdict: &'static str,
}

/// 분리 서명 JSON 을 캡슐 바이트·키 등록부와 대조한다.
///
/// 판정 우선순위: 형식 오류(malformed) → 미등록 키(unknownKey) → 폐기
/// 키(revoked — 서명 검증은 수행하되 판정은 폐기가 우선) → 서명
/// 불일치(invalid) → valid. 폐기 키의 과거 서명 소급 판정은 이 축 단독으로
/// 불가능하다(S3) — 시점 증명(5년 축) 결합 전까지 "현재 폐기됨"까지만 말한다.
pub fn verify_sidecar(
    sidecar: &serde_json::Value,
    capsule_bytes: &[u8],
    keyring: &BTreeMap<String, KeyEntry>,
) -> SigVerdict {
    if sidecar["kind"] != SIG_KIND || sidecar["alg"] != ALG {
        return SigVerdict {
            signature_ok: None,
            key_id: sidecar["keyId"].as_str().map(str::to_string),
            key_known: false,
            revoked: None,
            verdict: "malformed",
        };
    }
    let Some(key_id) = sidecar["keyId"].as_str().filter(|s| !s.is_empty()) else {
        return SigVerdict {
            signature_ok: None,
            key_id: None,
            key_known: false,
            revoked: None,
            verdict: "malformed",
        };
    };
    let Some(entry) = keyring.get(key_id) else {
        return SigVerdict {
            signature_ok: None,
            key_id: Some(key_id.to_string()),
            key_known: false,
            revoked: None,
            verdict: "unknownKey",
        };
    };
    let sig_ok = sidecar["signature"]
        .as_str()
        .and_then(|s| B64.decode(s).ok())
        .and_then(|b| <[u8; 64]>::try_from(b).ok())
        .map(|b| {
            entry
                .public
                .verify(capsule_bytes, &Signature::from_bytes(&b))
                .is_ok()
        });
    let verdict = match (&entry.revoked, sig_ok) {
        (Some(_), _) => "revoked",
        (None, Some(true)) => "valid",
        (None, Some(false)) => "invalid",
        (None, None) => "malformed",
    };
    SigVerdict {
        signature_ok: sig_ok,
        key_id: Some(key_id.to_string()),
        key_known: true,
        revoked: entry.revoked.clone(),
        verdict,
    }
}

/// 현재 UTC 를 RFC 3339 로 — 외부 시간 크레이트 없이 (그레고리력 민간 산법).
///
/// `signedAt` 주장 필드 전용이다. 초 단위면 충분하고, 판정 어디에도 쓰이지
/// 않으므로 윤초는 고려하지 않는다.
pub fn rfc3339_utc_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (days, rem) = (secs / 86_400, secs % 86_400);
    let (hh, mm, ss) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    // civil-from-days (Howard Hinnant 산법, 1970-01-01 = day 0).
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    let mut out = String::with_capacity(20);
    let _ = write!(out, "{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}Z");
    out
}
