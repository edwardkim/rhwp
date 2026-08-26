//! [#4553] 정산 증빙 — 9년 축(workorder·claim·원장) 코어.
//!
//! ## 무엇을 잇나 (설계서 §1 — 연결 형식이 기여의 전부)
//!
//! (요구 명세 → 캡슐 → 게이트 판정 → 원장 기입)을 전부 검증 가능한 데이터로
//! 잇는다. 새 검증 메커니즘은 발명하지 않는다: 검수 기준은 6년 게이트 정책,
//! 납품 이력은 3년 계보, 원장은 5년 앵커 로그의 **동형 재사용**
//! (`anchor_log::load_kind`), 서명은 4년 사이드카다.
//!
//! ## 3해시 고정이 급소 (설계서 §2.2)
//!
//! claim 은 명세서·캡슐·게이트 봉투를 **파일 바이트 sha256** 으로 고정한다.
//! 청구 후 산출물 바꿔치기(P1)·다른 명세서에 갖다 붙이기(P4)·판정 위조(P2)가
//! 전부 해시 불일치라는 하나의 실패로 환원된다.
//!
//! ## 범위 경계 (설계서 첫 문단 그대로)
//!
//! 이 축은 돈을 움직이지 않는다. `unitPrice.amount` 는 문자열이고 도구는 그
//! 값을 계산하지 않고 운반만 한다. 산출물은 "지불 근거가 되는, 제3자 검증
//! 가능한 증빙"뿐이다 — 지불 실행을 넣지 않는 것은 기술 한계가 아니라 설계
//! 원칙이다(검증 도구와 금융 도구는 실패 시 피해 반경이 다르다).

use sha2::{Digest, Sha256};

/// 작업 명세서의 `kind`.
pub const WORKORDER_KIND: &str = "workorder";
/// 정산 증빙(청구)의 `kind`.
pub const CLAIM_KIND: &str = "settlementClaim";
/// 원장 항목의 `kind` — 5년 로그와 같은 체인, 다른 kind.
pub const LEDGER_KIND: &str = "settlementLedger";

/// 파일 바이트의 sha256 hex.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(64);
    for b in digest {
        use std::fmt::Write as _;
        let _ = write!(hex, "{b:02x}");
    }
    hex
}

/// 명세서 파싱 — kind·workorderId·acceptancePolicy 존재를 검사한다.
///
/// 검수 기준이 없는 명세서는 "요구 품질" 분쟁을 산문으로 되돌리므로 거부한다
/// (설계서 §2.1 — 검수 기준 = 정책 파일이 이 형식의 존재 이유).
pub fn parse_workorder(text: &str) -> Result<serde_json::Value, String> {
    let v: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("명세서 파싱 실패 - {e}"))?;
    if v["kind"] != WORKORDER_KIND {
        return Err(format!("kind 가 {WORKORDER_KIND} 가 아닙니다"));
    }
    if v["workorderId"].as_str().unwrap_or("").is_empty() {
        return Err("workorderId 가 필요합니다".to_string());
    }
    if v["acceptancePolicy"].is_null() {
        return Err(
            "acceptancePolicy 가 필요합니다 — 검수 기준 없는 명세서는 분쟁을 산문으로 되돌린다"
                .to_string(),
        );
    }
    Ok(v)
}

/// 원장에서 같은 캡슐의 accepted 기입을 찾는다 — 이중 청구(P3) 전역 검사.
///
/// 같은 캡슐을 같은 명세서에 다시 청구하든, **다른** 명세서에 갖다 붙여
/// 청구하든 똑같이 잡힌다 — 원장 전역에서 capsuleSha256 는 한 번만 accepted
/// 된다(설계서 §2.3).
pub fn find_accepted(ledger: &crate::anchor_log::AnchorLog, capsule_sha256: &str) -> Option<usize> {
    ledger.entries.iter().position(|e| {
        e["capsuleSha256"].as_str() == Some(capsule_sha256) && e["verdict"] == "accepted"
    })
}

/// 새 원장 기입 줄 — 5년 로그와 같은 체인 규약(직전 줄 바이트 해시·seq 연번).
pub fn make_ledger_line(
    ledger: &crate::anchor_log::AnchorLog,
    claim_sha256: &str,
    capsule_sha256: &str,
    verdict: &str,
    at: &str,
) -> String {
    serde_json::json!({
        "seq": ledger.entries.len(),
        "kind": LEDGER_KIND,
        "claimSha256": claim_sha256,
        // 이중 청구 검사의 키 — claim 파일 없이 원장만으로 전역 검사가 닫힌다.
        "capsuleSha256": capsule_sha256,
        "verdict": verdict,
        "prevEntryHash": ledger.line_hashes.last().cloned(),
        // 주장 필드 — 시점 증명은 체크포인트 공표(5년 축 동형)의 몫.
        "at": at,
    })
    .to_string()
}
