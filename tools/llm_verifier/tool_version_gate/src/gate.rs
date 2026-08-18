//! toolVersion 게이트. V-replay(같은 버전 재실행)가 아니다.
//!
//! `receipt.toolVersion !=` 검증기 바이너리 버전이면 `reproduced:true` 를
//! 합격으로 받지 않는다 (낡은 도구).

use crate::reason::Reason;
use crate::schema::{CLAIM_ID, KIND, PROTOCOL_SCHEMA_VERSION};
use crate::version::ToolVersion;
use serde::{Deserialize, Serialize};

/// 한 쌍의 버전과 재현 주장에 대한 닫힌 판정.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Decision {
    pub schema_version: String,
    pub claim: String,
    pub kind: String,
    pub attest_version: String,
    pub verify_version: String,
    pub reproduced: Option<bool>,
    pub accepted: bool,
    pub reason: Reason,
}

impl Decision {
    pub fn uniqueness_key(&self) -> String {
        format!(
            "{}|{}|{}|{}",
            self.attest_version,
            self.verify_version,
            reproduced_token(self.reproduced),
            self.accepted
        )
    }
}

pub fn reproduced_token(reproduced: Option<bool>) -> &'static str {
    match reproduced {
        Some(true) => "true",
        Some(false) => "false",
        None => "null",
    }
}

pub fn parse_reproduced(raw: &str) -> Result<Option<bool>, String> {
    match raw.trim() {
        "true" => Ok(Some(true)),
        "false" => Ok(Some(false)),
        "null" | "" => Ok(None),
        other => Err(format!("reproduced {other}")),
    }
}

fn classify(attest: &ToolVersion, verify: &ToolVersion, reproduced: Option<bool>) -> Reason {
    if attest.is_empty() {
        return Reason::AttestVersionMissing;
    }
    if verify.is_empty() {
        return Reason::VerifyVersionMissing;
    }
    if !attest.same_identity(verify) {
        return match reproduced {
            Some(true) => Reason::StaleTool,
            Some(false) => Reason::StaleAndNotReproduced,
            None => Reason::StaleAndAbsent,
        };
    }
    match reproduced {
        Some(true) => Reason::FreshReproduced,
        Some(false) => Reason::FreshNotReproduced,
        None => Reason::FreshAbsent,
    }
}

/// `(attest_version, verify_version, reproduced) -> accepted`.
pub fn gate(attest_version: &str, verify_version: &str, reproduced: Option<bool>) -> Decision {
    let attest = ToolVersion::parse(attest_version);
    let verify = ToolVersion::parse(verify_version);
    let reason = classify(&attest, &verify, reproduced);
    Decision {
        schema_version: PROTOCOL_SCHEMA_VERSION.to_string(),
        claim: CLAIM_ID.to_string(),
        kind: KIND.to_string(),
        attest_version: attest_version.to_string(),
        verify_version: verify_version.to_string(),
        reproduced,
        accepted: reason.accepts(),
        reason,
    }
}

/// `reproduced:true` 주장만 본다. 버전 불일치면 거부.
pub fn gate_reproduced_true(attest_version: &str, verify_version: &str) -> Decision {
    gate(attest_version, verify_version, Some(true))
}

/// 이 크레이트 바이너리 버전과 영수증을 대조한다. 새 CLI 가 아니다.
pub fn gate_against_this_binary(receipt_tool_version: &str, reproduced: Option<bool>) -> Decision {
    gate(receipt_tool_version, env!("CARGO_PKG_VERSION"), reproduced)
}

pub fn accept_reproduced(attest_version: &str, verify_version: &str, reproduced: bool) -> bool {
    gate(attest_version, verify_version, Some(reproduced)).accepted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_reproduced_true_is_not_accepted() {
        let d = gate_reproduced_true("0.8.3", "0.8.4");
        assert_eq!(d.reason, Reason::StaleTool);
        assert!(!d.accepted);
    }

    #[test]
    fn matching_reproduced_true_is_accepted() {
        let d = gate("0.8.4", "0.8.4", Some(true));
        assert_eq!(d.reason, Reason::FreshReproduced);
        assert!(d.accepted);
    }

    #[test]
    fn matching_reproduced_false_is_not_accepted() {
        let d = gate("0.8.4", "0.8.4", Some(false));
        assert_eq!(d.reason, Reason::FreshNotReproduced);
        assert!(!d.accepted);
    }

    #[test]
    fn this_binary_rejects_foreign_receipt_version() {
        let d = gate_against_this_binary("0.8.4", Some(true));
        assert!(!d.accepted);
        assert_eq!(d.reason, Reason::StaleTool);
        assert_eq!(d.verify_version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn this_binary_accepts_own_crate_version() {
        let d = gate_against_this_binary(env!("CARGO_PKG_VERSION"), Some(true));
        assert!(d.accepted);
    }
}
