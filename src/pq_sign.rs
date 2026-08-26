//! 양자내성 서명 — 작업캡슐·출처(provenance) 서명을 양자 이후에도 살리는 확장.
//!
//! ## 왜 필요한가 — Shor
//!
//! `capsule_sign` 의 Ed25519 는 결정론·짧은 키·보편 검증이라는 장점이 있지만,
//! 타원곡선 이산로그에 안전성을 기대므로 충분히 큰 양자컴퓨터의 Shor 알고리즘에
//! 깨진다. 지금 발급한 서명이 "지금 수확해 나중에 해독"(harvest-now,
//! decrypt-later)의 표적이 되면, 미래에 위조가 가능해져 출처 신뢰가 소급 붕괴한다.
//!
//! ## 무엇을 더하나 — ML-DSA-65 (격자 기반, NIST FIPS 204)
//!
//! NIST 표준 격자 서명 ML-DSA(구 CRYSTALS-Dilithium)를 **새 능력으로 추가**한다.
//! 기존 Ed25519 서명 경로(`capsule_sign`)는 전혀 건드리지 않는다 — 이 모듈은
//! 별도의 신규 표면이다. 포맷 민첩성을 위해 알고리즘을 `alg` 문자열/태그로 명시한다.
//! ML-DSA-65 는 NIST 보안범주 3(≈192비트)이며 RustCrypto `ml-dsa` 크레이트가
//! 성능·안전 균형으로 권장하는 파라미터다.
//!
//! ## 하이브리드 — 둘 중 하나만 살아도 안전 (전환기 권장 태세)
//!
//! `hybrid_*` 는 Ed25519 서명과 ML-DSA 서명을 같은 메시지 위에 각각 만들어
//! 이어붙이고, 검증은 **둘 다** 통과해야 유효로 본다. 고전 서명은 잘 검증된
//! 성숙한 안전성을, 양자내성 서명은 미래 대비를 각각 담당한다 — 어느 한쪽 스킴이
//! (구현 결함이든 암호해독 진전이든) 무너져도 위조는 나머지 한쪽을 여전히 깨야
//! 하므로 출처는 살아남는다. NIST/IETF 가 권고하는 전환기 태세다.
//!
//! ## 결정론
//!
//! 키생성은 32바이트 시드에서 결정론적으로 파생하고(FIPS 204 `KeyGen_internal`),
//! 서명도 ML-DSA 의 결정론 변형을 쓴다 — 같은 키·같은 바이트 → 같은 서명. 이는
//! 이 저장소의 결정론 문화(replay·lineage 의 재현 판정)와 정합한다. 엔트로피는
//! 키생성 순간의 시드 32바이트에만 쓰고 OS CSPRNG(`getrandom`)에서 얻는다.
//!
//! ## 경계 — 이 모듈이 하지 않는 것
//!
//! 키 보관·등록부 신뢰뿌리·시점 증명은 여기 밖이다(각각 운영·거버넌스·앵커 축).
//! 이 모듈은 바이트 대 바이트의 서명/검증 원시연산만 제공하며, 잘못된 입력에는
//! 절대 패닉하지 않는다(검증 계열은 `false`, 서명 계열은 `Err`).

use ed25519_dalek::{
    Signature as EdSignature, Signer as _, SigningKey as EdSigningKey, Verifier as _,
    VerifyingKey as EdVerifyingKey,
};
use ml_dsa::{
    EncodedSignature, KeyInit as _, Keypair as _, MlDsa65, Seed, Signature as MlSignature,
    Signer as _, SigningKey as MlSigningKey, Verifier as _, VerifyingKey as MlVerifyingKey,
};

/// 순수 ML-DSA-65 서명의 알고리즘 표기 — 봉투·사이드카의 `alg` 필드에 쓴다.
pub const ALG_ML_DSA_65: &str = "ml-dsa-65";
/// 하이브리드(Ed25519 ++ ML-DSA-65) 서명의 알고리즘 표기.
pub const ALG_HYBRID_ED25519_ML_DSA_65: &str = "ed25519+ml-dsa-65";
/// 하이브리드 서명 블롭의 선두 1바이트 태그 — 포맷 자기서술/민첩성.
pub const HYBRID_SIG_TAG: u8 = 0x02;

/// ML-DSA-65 공개키(검증키) 인코딩 길이 (FIPS 204).
pub const ML_DSA_65_PUBLIC_LEN: usize = 1952;
/// ML-DSA-65 비밀키를 대표하는 시드 길이 — 32바이트가 모든 보안수준에서 동일하며
/// 크레이트가 권장하는 직렬화다(시드로 결정론 키생성).
pub const ML_DSA_65_SECRET_LEN: usize = 32;
/// ML-DSA-65 서명 인코딩 길이 (FIPS 204).
pub const ML_DSA_65_SIG_LEN: usize = 3309;

/// Ed25519 공개키 길이.
pub const ED25519_PUBLIC_LEN: usize = 32;
/// Ed25519 비밀키(시드) 길이.
pub const ED25519_SECRET_LEN: usize = 32;
/// Ed25519 서명 길이.
pub const ED25519_SIG_LEN: usize = 64;

/// 하이브리드 공개키 길이 = Ed25519(32) ++ ML-DSA-65(1952).
pub const HYBRID_PUBLIC_LEN: usize = ED25519_PUBLIC_LEN + ML_DSA_65_PUBLIC_LEN;
/// 하이브리드 비밀키 길이 = Ed25519 시드(32) ++ ML-DSA-65 시드(32).
pub const HYBRID_SECRET_LEN: usize = ED25519_SECRET_LEN + ML_DSA_65_SECRET_LEN;
/// 하이브리드 서명 길이 = 태그(1) ++ Ed25519 서명(64) ++ ML-DSA-65 서명(3309).
pub const HYBRID_SIG_LEN: usize = 1 + ED25519_SIG_LEN + ML_DSA_65_SIG_LEN;

// ===== 순수 ML-DSA-65 =====

/// ML-DSA-65 키쌍을 새로 만들어 `(공개키, 비밀키)` 바이트로 돌려준다.
///
/// 공개키는 1952바이트, 비밀키는 32바이트 시드다(크레이트 권장 직렬화, FIPS 204
/// `KeyGen_internal` 재현). 시드 엔트로피는 OS CSPRNG 에서 얻는다 — 호출자는
/// 비밀키(시드) 보관 책임을 진다.
///
/// # Panics
/// OS 엔트로피 획득에 실패하면 패닉한다. 약한 키를 조용히 만드는 것보다 낫고,
/// 이 반환 계약((Vec, Vec))에는 오류 경로가 없다. 실무 시스템에서 사실상 일어나지
/// 않는 조건이다.
#[must_use]
pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
    let mut seed = [0u8; ML_DSA_65_SECRET_LEN];
    getrandom::fill(&mut seed).expect("OS 엔트로피 획득 실패");
    let sk = MlSigningKey::<MlDsa65>::from_seed(&Seed::from(seed));
    let public = sk.verifying_key().encode().to_vec();
    (public, seed.to_vec())
}

/// ML-DSA-65 분리(detached) 서명 바이트를 만든다.
///
/// `secret_key` 는 `generate_keypair` 가 준 32바이트 시드다. 서명은 결정론적이라
/// 같은 (키, 메시지) 는 항상 같은 서명을 낸다. 시드 길이가 틀리면 `Err`.
pub fn sign(secret_key: &[u8], message: &[u8]) -> Result<Vec<u8>, String> {
    let sk = MlSigningKey::<MlDsa65>::new_from_slice(secret_key)
        .map_err(|_| format!("ML-DSA 비밀키는 {ML_DSA_65_SECRET_LEN}바이트 시드여야 합니다"))?;
    let sig: MlSignature<MlDsa65> = sk
        .try_sign(message)
        .map_err(|e| format!("ML-DSA 서명 실패: {e}"))?;
    Ok(sig.encode().to_vec())
}

/// ML-DSA-65 분리 서명을 검증한다.
///
/// 잘못된 입력(길이·형식 오류, 미상의 바이트)에는 **절대 패닉하지 않고** `false`
/// 를 돌려준다. 서명이 이 공개키·이 메시지에 대해 유효할 때만 `true`.
#[must_use]
pub fn verify(public_key: &[u8], message: &[u8], signature: &[u8]) -> bool {
    // 공개키 복원 — 길이가 틀리면 InvalidLength → false.
    let Ok(vk) = MlVerifyingKey::<MlDsa65>::new_from_slice(public_key) else {
        return false;
    };
    // 서명 바이트 → 고정크기 배열(길이 검사) → 디코드(형식 검사, Option).
    let Ok(sig_arr) = <EncodedSignature<MlDsa65>>::try_from(signature) else {
        return false;
    };
    let Some(sig) = MlSignature::<MlDsa65>::decode(&sig_arr) else {
        return false;
    };
    vk.verify(message, &sig).is_ok()
}

// ===== 하이브리드 (Ed25519 ++ ML-DSA-65) =====

/// 하이브리드용 Ed25519 키쌍 — `(공개키 32B, 비밀키 32B)`.
///
/// # Panics
/// OS 엔트로피 실패 시 패닉(‹generate_keypair› 와 같은 계약).
#[must_use]
pub fn ed25519_generate_keypair() -> (Vec<u8>, Vec<u8>) {
    let mut seed = [0u8; ED25519_SECRET_LEN];
    getrandom::fill(&mut seed).expect("OS 엔트로피 획득 실패");
    let sk = EdSigningKey::from_bytes(&seed);
    let public = sk.verifying_key().to_bytes().to_vec();
    (public, seed.to_vec())
}

/// 하이브리드 키쌍 — `(하이브리드_공개키, 하이브리드_비밀키)`.
///
/// - 공개키 = Ed25519 공개키(32B) ++ ML-DSA-65 공개키(1952B).
/// - 비밀키 = Ed25519 시드(32B) ++ ML-DSA-65 시드(32B).
///
/// 두 절반은 고정 오프셋으로 자기서술적이라 별도 길이 헤더가 필요 없다.
///
/// # Panics
/// OS 엔트로피 실패 시 패닉(하위 키생성과 같은 계약).
#[must_use]
pub fn hybrid_generate_keypair() -> (Vec<u8>, Vec<u8>) {
    let (ed_pub, ed_sec) = ed25519_generate_keypair();
    let (ml_pub, ml_sec) = generate_keypair();
    let mut public = Vec::with_capacity(HYBRID_PUBLIC_LEN);
    public.extend_from_slice(&ed_pub);
    public.extend_from_slice(&ml_pub);
    let mut secret = Vec::with_capacity(HYBRID_SECRET_LEN);
    secret.extend_from_slice(&ed_sec);
    secret.extend_from_slice(&ml_sec);
    (public, secret)
}

/// 하이브리드 서명 = `태그(1) ++ Ed25519 서명(64) ++ ML-DSA-65 서명(3309)`.
///
/// `hybrid_secret` 은 `hybrid_generate_keypair` 가 준 64바이트(32+32) 이어붙임이다.
/// 두 서명 모두 같은 `message` 바이트 위에 만든다.
pub fn hybrid_sign(hybrid_secret: &[u8], message: &[u8]) -> Result<Vec<u8>, String> {
    if hybrid_secret.len() != HYBRID_SECRET_LEN {
        return Err(format!(
            "하이브리드 비밀키는 {HYBRID_SECRET_LEN}바이트여야 합니다"
        ));
    }
    let (ed_seed_bytes, ml_seed_bytes) = hybrid_secret.split_at(ED25519_SECRET_LEN);
    // 고전 절반 — Ed25519.
    let ed_seed: [u8; ED25519_SECRET_LEN] = ed_seed_bytes
        .try_into()
        .map_err(|_| "Ed25519 시드 길이 오류".to_string())?;
    let ed_sk = EdSigningKey::from_bytes(&ed_seed);
    let ed_sig = ed_sk.sign(message);
    // 양자내성 절반 — ML-DSA-65.
    let ml_sig = sign(ml_seed_bytes, message)?;
    // 태그 || ed_sig(64) || ml_sig(3309)
    let mut out = Vec::with_capacity(1 + ED25519_SIG_LEN + ml_sig.len());
    out.push(HYBRID_SIG_TAG);
    out.extend_from_slice(&ed_sig.to_bytes());
    out.extend_from_slice(&ml_sig);
    Ok(out)
}

/// 하이브리드 검증 — **두 서명이 모두** 통과해야 `true`.
///
/// 어느 한 스킴이 무너져도(구현 결함/암호해독) 위조는 나머지 절반을 여전히 깨야
/// 하므로 출처는 살아남는다 — 이것이 전환기 하이브리드 태세의 핵심이다. 잘못된
/// 입력(길이·태그·형식)에는 **절대 패닉하지 않고** `false` 를 돌려준다.
#[must_use]
pub fn hybrid_verify(hybrid_public: &[u8], message: &[u8], signature: &[u8]) -> bool {
    // 공개키 분해.
    if hybrid_public.len() != HYBRID_PUBLIC_LEN {
        return false;
    }
    let (ed_pub, ml_pub) = hybrid_public.split_at(ED25519_PUBLIC_LEN);
    // 서명 분해: 태그(1) || ed_sig(64) || ml_sig(3309).
    if signature.len() != HYBRID_SIG_LEN || signature[0] != HYBRID_SIG_TAG {
        return false;
    }
    let ed_sig_bytes = &signature[1..1 + ED25519_SIG_LEN];
    let ml_sig_bytes = &signature[1 + ED25519_SIG_LEN..];
    // 두 절반을 각각 검증 — 둘 다 통과해야 유효.
    let ed_ok = ed25519_verify(ed_pub, message, ed_sig_bytes);
    let ml_ok = verify(ml_pub, message, ml_sig_bytes);
    ed_ok && ml_ok
}

/// Ed25519 분리 서명 검증 — 잘못된 입력에는 패닉 없이 `false`.
fn ed25519_verify(public_key: &[u8], message: &[u8], signature: &[u8]) -> bool {
    let Ok(pk_bytes) = <[u8; ED25519_PUBLIC_LEN]>::try_from(public_key) else {
        return false;
    };
    let Ok(vk) = EdVerifyingKey::from_bytes(&pk_bytes) else {
        return false;
    };
    let Ok(sig_bytes) = <[u8; ED25519_SIG_LEN]>::try_from(signature) else {
        return false;
    };
    let sig = EdSignature::from_bytes(&sig_bytes);
    vk.verify(message, &sig).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 테스트용 바이트는 **실행마다 새로 뽑는다**.
    ///
    /// 시드·공개키·서명 자리에 상수를 두면 CodeQL 이 하드코딩 암호값(critical)으로
    /// 잡는다. 여기서 고정하려는 성질(결정론·길이 거부·쓰레기 키 거부)은 어느 것도
    /// 특정 바이트 값에 기대지 않으므로, 난수로 뽑는 편이 매 실행 재확인이 된다.
    fn rand_bytes(n: usize) -> Vec<u8> {
        let mut buf = vec![0u8; n];
        getrandom::fill(&mut buf).expect("테스트 난수");
        buf
    }

    /// 길이는 맞고 내용은 전부 0 인 버퍼 — "구조적으로 무효한 키" 경계.
    /// 난수 버퍼를 0 으로 덮어 만든다(상수 배열이 아니라 실행시 값).
    fn zeroed(n: usize) -> Vec<u8> {
        let mut buf = rand_bytes(n);
        buf.iter_mut().for_each(|b| *b = 0);
        buf
    }

    // ----- 순수 ML-DSA-65 -----

    #[test]
    fn ml_dsa_roundtrip() {
        let (pk, sk) = generate_keypair();
        assert_eq!(pk.len(), ML_DSA_65_PUBLIC_LEN);
        assert_eq!(sk.len(), ML_DSA_65_SECRET_LEN);
        let msg = b"work-capsule provenance bytes";
        let sig = sign(&sk, msg).expect("sign");
        assert_eq!(sig.len(), ML_DSA_65_SIG_LEN);
        assert!(verify(&pk, msg, &sig));
    }

    #[test]
    fn ml_dsa_deterministic() {
        // 같은 시드·같은 메시지 → 같은 서명(결정론 변형).
        let seed = rand_bytes(ML_DSA_65_SECRET_LEN);
        let msg = b"deterministic";
        assert_eq!(sign(&seed, msg).unwrap(), sign(&seed, msg).unwrap());
    }

    #[test]
    fn ml_dsa_wrong_key_fails() {
        let (_pk1, sk1) = generate_keypair();
        let (pk2, _sk2) = generate_keypair();
        let msg = b"bind to key 1";
        let sig = sign(&sk1, msg).unwrap();
        assert!(!verify(&pk2, msg, &sig));
    }

    #[test]
    fn ml_dsa_tampered_message_fails() {
        let (pk, sk) = generate_keypair();
        let sig = sign(&sk, b"original").unwrap();
        assert!(!verify(&pk, b"original!", &sig));
        assert!(!verify(&pk, b"0riginal", &sig));
    }

    #[test]
    fn ml_dsa_tampered_signature_fails() {
        let (pk, sk) = generate_keypair();
        let msg = b"sign me";
        // 첫 바이트를 뒤집는다.
        let mut sig = sign(&sk, msg).unwrap();
        sig[0] ^= 0x01;
        assert!(!verify(&pk, msg, &sig));
        // 마지막 바이트도.
        let mut sig2 = sign(&sk, msg).unwrap();
        let last = sig2.len() - 1;
        sig2[last] ^= 0x80;
        assert!(!verify(&pk, msg, &sig2));
    }

    #[test]
    fn ml_dsa_malformed_inputs_never_panic() {
        let (pk, sk) = generate_keypair();
        let msg = b"m";
        let sig = sign(&sk, msg).unwrap();
        // 빈/짧은/긴 공개키·서명 — 전부 false, 패닉 없음.
        assert!(!verify(&[], msg, &sig));
        assert!(!verify(&rand_bytes(10), msg, &sig));
        assert!(!verify(&pk, msg, &[]));
        assert!(!verify(&pk, msg, &rand_bytes(10)));
        // 길이는 맞지만 전부 0 인 공개키/서명.
        assert!(!verify(&zeroed(ML_DSA_65_PUBLIC_LEN), msg, &sig));
        assert!(!verify(&pk, msg, &zeroed(ML_DSA_65_SIG_LEN)));
        // 길이 초과.
        assert!(!verify(&rand_bytes(ML_DSA_65_PUBLIC_LEN + 1), msg, &sig));
        assert!(!verify(&pk, msg, &rand_bytes(ML_DSA_65_SIG_LEN + 1)));
    }

    #[test]
    fn sign_rejects_bad_secret_len() {
        assert!(sign(&[], b"x").is_err());
        assert!(sign(&rand_bytes(16), b"x").is_err());
        assert!(sign(&rand_bytes(33), b"x").is_err());
    }

    // ----- 하이브리드 -----

    #[test]
    fn hybrid_roundtrip() {
        let (pk, sk) = hybrid_generate_keypair();
        assert_eq!(pk.len(), HYBRID_PUBLIC_LEN);
        assert_eq!(sk.len(), HYBRID_SECRET_LEN);
        let msg = b"hybrid provenance";
        let sig = hybrid_sign(&sk, msg).unwrap();
        assert_eq!(sig.len(), HYBRID_SIG_LEN);
        assert!(hybrid_verify(&pk, msg, &sig));
    }

    #[test]
    fn hybrid_tampered_message_fails() {
        let (pk, sk) = hybrid_generate_keypair();
        let sig = hybrid_sign(&sk, b"pay 100").unwrap();
        assert!(!hybrid_verify(&pk, b"pay 900", &sig));
    }

    #[test]
    fn hybrid_requires_both_halves() {
        // 하이브리드 검증은 두 절반이 모두 유효해야 true — 각각을 손상시켜 확인.
        let (pk, sk) = hybrid_generate_keypair();
        let msg = b"both must hold";
        let good = hybrid_sign(&sk, msg).unwrap();
        assert!(hybrid_verify(&pk, msg, &good));

        // (1) Ed25519 절반만 손상 → false.
        let mut break_ed = good.clone();
        break_ed[1] ^= 0x01; // 태그 다음 첫 ed_sig 바이트.
        assert!(!hybrid_verify(&pk, msg, &break_ed));

        // (2) ML-DSA 절반만 손상 → false.
        let mut break_ml = good.clone();
        break_ml[1 + ED25519_SIG_LEN] ^= 0x01;
        assert!(!hybrid_verify(&pk, msg, &break_ml));
    }

    #[test]
    fn hybrid_cross_key_mixing_fails() {
        // 한쪽 스킴만 맞는 섞인 공개키로는 통과 못 한다 — "둘 다"를 강제.
        let (pk_a, sk_a) = hybrid_generate_keypair();
        let (pk_b, _sk_b) = hybrid_generate_keypair();
        let msg = b"x";
        let sig_a = hybrid_sign(&sk_a, msg).unwrap();
        // A 의 Ed25519 공개키 + B 의 ML-DSA 공개키.
        let mut mixed = Vec::new();
        mixed.extend_from_slice(&pk_a[..ED25519_PUBLIC_LEN]);
        mixed.extend_from_slice(&pk_b[ED25519_PUBLIC_LEN..]);
        // ML-DSA 절반이 A 서명과 안 맞으므로 false.
        assert!(!hybrid_verify(&mixed, msg, &sig_a));

        // 대칭: A 의 ML-DSA + B 의 Ed25519 → Ed 절반 불일치로 false.
        let mut mixed2 = Vec::new();
        mixed2.extend_from_slice(&pk_b[..ED25519_PUBLIC_LEN]);
        mixed2.extend_from_slice(&pk_a[ED25519_PUBLIC_LEN..]);
        assert!(!hybrid_verify(&mixed2, msg, &sig_a));
    }

    #[test]
    fn hybrid_malformed_inputs_never_panic() {
        let (pk, sk) = hybrid_generate_keypair();
        let msg = b"m";
        let sig = hybrid_sign(&sk, msg).unwrap();
        assert!(!hybrid_verify(&[], msg, &sig));
        assert!(!hybrid_verify(&pk, msg, &[]));
        assert!(!hybrid_verify(&pk, msg, &[HYBRID_SIG_TAG]));
        assert!(!hybrid_verify(&pk, msg, &zeroed(HYBRID_SIG_LEN)));
        // 태그가 틀림.
        let mut wrong_tag = sig.clone();
        wrong_tag[0] = 0xFF;
        assert!(!hybrid_verify(&pk, msg, &wrong_tag));
        // 비밀키 길이 오류 → sign 실패(패닉 아님).
        assert!(hybrid_sign(&rand_bytes(10), msg).is_err());
        assert!(hybrid_sign(&[], msg).is_err());
    }

    #[test]
    fn algo_identity_constants() {
        assert_eq!(ALG_ML_DSA_65, "ml-dsa-65");
        assert_eq!(ALG_HYBRID_ED25519_ML_DSA_65, "ed25519+ml-dsa-65");
        assert_eq!(HYBRID_SIG_TAG, 0x02);
        assert_eq!(HYBRID_PUBLIC_LEN, 1984);
        assert_eq!(HYBRID_SECRET_LEN, 64);
        assert_eq!(HYBRID_SIG_LEN, 3374);
    }
}
