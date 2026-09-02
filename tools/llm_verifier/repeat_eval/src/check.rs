//! 같은 기계 검사의 종류. 기준 분해(V-decomp)가 아니다.

use serde::{Deserialize, Serialize};

/// 관측을 어떻게 줄일지.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ValueKind {
    Exit,
    Bool,
    U64,
    Text,
    PassFail,
}

impl ValueKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Exit => "exit",
            Self::Bool => "bool",
            Self::U64 => "u64",
            Self::Text => "text",
            Self::PassFail => "passFail",
        }
    }

    pub fn is_numeric(self) -> bool {
        matches!(self, Self::U64)
    }

    pub fn reduce_kind(self) -> crate::report::ReduceKind {
        if self.is_numeric() {
            crate::report::ReduceKind::Mean
        } else {
            crate::report::ReduceKind::Majority
        }
    }
}

/// 한 종류의 기계 검사.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CheckKind {
    ExitClass,
    EnvelopeField,
    PassFail,
}

impl CheckKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ExitClass => "exitClass",
            Self::EnvelopeField => "envelopeField",
            Self::PassFail => "passFail",
        }
    }
}

/// 반복할 검사 명세. 원자 기준 묶음이 아니다.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckSpec {
    pub name: String,
    pub kind: CheckKind,
    pub value_kind: ValueKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

impl CheckSpec {
    pub fn exit_class() -> Self {
        Self {
            name: "exitClass".into(),
            kind: CheckKind::ExitClass,
            value_kind: ValueKind::Exit,
            path: None,
        }
    }

    pub fn pass_fail() -> Self {
        Self {
            name: "passFail".into(),
            kind: CheckKind::PassFail,
            value_kind: ValueKind::PassFail,
            path: None,
        }
    }

    pub fn envelope_bool(path: &str) -> Self {
        Self {
            name: path.into(),
            kind: CheckKind::EnvelopeField,
            value_kind: ValueKind::Bool,
            path: Some(path.into()),
        }
    }

    pub fn envelope_u64(path: &str) -> Self {
        Self {
            name: path.into(),
            kind: CheckKind::EnvelopeField,
            value_kind: ValueKind::U64,
            path: Some(path.into()),
        }
    }

    pub fn envelope_text(path: &str) -> Self {
        Self {
            name: path.into(),
            kind: CheckKind::EnvelopeField,
            value_kind: ValueKind::Text,
            path: Some(path.into()),
        }
    }

    pub fn parse_name(name: &str) -> Option<Self> {
        match name {
            "exitClass" | "exit_class" => Some(Self::exit_class()),
            "passFail" | "pass_fail" => Some(Self::pass_fail()),
            "identical" | "verify.identical" | "hasSignal" | "regression" | "pageCountMismatch"
            | "untrustedContent" | "strict" | "wasDistribution" | "reproduced" => {
                Some(Self::envelope_bool(name))
            }
            "filledCount" | "changedCount" | "diffCount" | "verify.diffCount" | "failCount"
            | "passCount" | "overflowCount" | "overlapCount" | "emptyPageCount" | "overPages"
            | "replacedCount" | "redactedCount" | "removedCount" | "pageCount" | "paraCount" => {
                Some(Self::envelope_u64(name))
            }
            "verdict" | "status" => Some(Self::envelope_text(name)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_fields_parse() {
        assert_eq!(
            CheckSpec::parse_name("verify.identical")
                .unwrap()
                .value_kind,
            ValueKind::Bool
        );
        assert_eq!(
            CheckSpec::parse_name("filledCount").unwrap().value_kind,
            ValueKind::U64
        );
        assert!(CheckSpec::parse_name("holisticScore").is_none());
        assert!(CheckSpec::parse_name("bestOfN").is_none());
        assert!(CheckSpec::parse_name("atomPass").is_none());
    }
}
