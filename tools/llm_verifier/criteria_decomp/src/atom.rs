//! 원자 기준 한 줄. 총점이 아니다.

use crate::envelope::Observed;
use crate::field::is_allowed_envelope_field;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtomSpec {
    pub criterion_id: String,
    pub task: String,
    pub envelope_field: String,
    pub expected: Expected,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
}

/// 한 봉투 필드에 대한 닫힌 기대. 자유 산문 조건이 아니다.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Expected {
    Bool { value: bool },
    U64 { value: u64 },
    EmptySeq,
    Absent,
    Present,
    StrEq { value: String },
    Null,
}

impl Expected {
    pub fn matches(&self, observed: &Observed) -> bool {
        match (self, observed) {
            (Self::Bool { value }, Observed::Bool(b)) => *value == *b,
            (Self::U64 { value }, Observed::U64(n)) => *value == *n,
            (Self::U64 { value }, Observed::I64(n)) if *n >= 0 => *value == *n as u64,
            (Self::EmptySeq, Observed::Seq(v)) => v.is_empty(),
            (Self::EmptySeq, Observed::Missing) => true,
            (Self::Absent, Observed::Missing) | (Self::Absent, Observed::Null) => true,
            (Self::Present, Observed::Missing) | (Self::Present, Observed::Null) => false,
            (Self::Present, _) => true,
            (Self::StrEq { value }, Observed::Text(s)) => value == s,
            (Self::Null, Observed::Null) => true,
            _ => false,
        }
    }

    pub fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

impl AtomSpec {
    pub fn field_is_allowed(&self) -> bool {
        is_allowed_envelope_field(&self.envelope_field)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_not_found_matches_empty_or_missing() {
        let e = Expected::EmptySeq;
        assert!(e.matches(&Observed::Seq(Vec::new())));
        assert!(e.matches(&Observed::Missing));
        assert!(!e.matches(&Observed::Seq(vec![Value::String("성명".into())])));
    }

    #[test]
    fn bool_expected_does_not_cross_types() {
        let e = Expected::Bool { value: true };
        assert!(e.matches(&Observed::Bool(true)));
        assert!(!e.matches(&Observed::U64(1)));
        assert!(!e.matches(&Observed::Text("true".into())));
    }
}
