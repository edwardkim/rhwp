//! 결속 판정. 산문이 아니라 닫힌 열거.

use crate::coords::DocumentCoords;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Pass,
    Fail,
}

impl Verdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "pass" | "PASS" => Some(Self::Pass),
            "fail" | "FAIL" => Some(Self::Fail),
            _ => None,
        }
    }

    pub fn is_pass(self) -> bool {
        matches!(self, Self::Pass)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailKind {
    /// 좌표가 전혀 없다.
    Unbound,
    /// 필수 4키 중 일부가 빠졌다.
    IncompleteCoords,
    /// 봉투에 없는 좌표 키를 지었다.
    InventedKey,
    /// 주장 문장이 비었다.
    EmptyClaim,
    /// 주장 좌표가 봉투 매치에 없다.
    EnvelopeMismatch,
    /// search / extract-data 가 아니다.
    UnknownEnvelopeKind,
}

impl FailKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unbound => "unbound",
            Self::IncompleteCoords => "incomplete_coords",
            Self::InventedKey => "invented_key",
            Self::EmptyClaim => "empty_claim",
            Self::EnvelopeMismatch => "envelope_mismatch",
            Self::UnknownEnvelopeKind => "unknown_envelope_kind",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "unbound" => Some(Self::Unbound),
            "incomplete_coords" => Some(Self::IncompleteCoords),
            "invented_key" => Some(Self::InventedKey),
            "empty_claim" => Some(Self::EmptyClaim),
            "envelope_mismatch" => Some(Self::EnvelopeMismatch),
            "unknown_envelope_kind" => Some(Self::UnknownEnvelopeKind),
            _ => None,
        }
    }
}

/// 한 주장의 결속 결과. `(claim_text, coords_present, field_set, pass/fail)`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BindDecision {
    pub claim_id: String,
    pub claim_text: String,
    pub coords_present: bool,
    pub field_set: Vec<String>,
    pub verdict: Verdict,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fail_kind: Option<FailKind>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing_fields: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub invented_keys: Vec<String>,
}

impl BindDecision {
    pub fn pass(id: impl Into<String>, text: impl Into<String>, coords: &DocumentCoords) -> Self {
        Self {
            claim_id: id.into(),
            claim_text: text.into(),
            coords_present: true,
            field_set: coords.field_set(),
            verdict: Verdict::Pass,
            fail_kind: None,
            missing_fields: Vec::new(),
            invented_keys: Vec::new(),
        }
    }

    pub fn fail(
        id: impl Into<String>,
        text: impl Into<String>,
        coords: Option<&DocumentCoords>,
        kind: FailKind,
    ) -> Self {
        let missing = coords
            .map(|c| {
                c.required_missing()
                    .into_iter()
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_else(|| {
                crate::REQUIRED_COORD_FIELDS
                    .iter()
                    .map(|s| (*s).to_string())
                    .collect()
            });
        Self {
            claim_id: id.into(),
            claim_text: text.into(),
            coords_present: coords.is_some_and(DocumentCoords::coords_present),
            field_set: coords.map(DocumentCoords::field_set).unwrap_or_default(),
            verdict: Verdict::Fail,
            fail_kind: Some(kind),
            missing_fields: missing,
            invented_keys: Vec::new(),
        }
    }

    pub fn is_pass(&self) -> bool {
        self.verdict.is_pass()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fail_kind_roundtrip() {
        for k in [
            FailKind::Unbound,
            FailKind::IncompleteCoords,
            FailKind::InventedKey,
            FailKind::EmptyClaim,
            FailKind::EnvelopeMismatch,
            FailKind::UnknownEnvelopeKind,
        ] {
            assert_eq!(FailKind::parse(k.as_str()), Some(k));
        }
    }
}
