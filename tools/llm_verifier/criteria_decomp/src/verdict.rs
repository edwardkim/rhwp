//! 원자 판정. 산문 점수나 Best-of-N 순위가 아니다.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailKind {
    /// 지식지도에 없는 봉투 키를 지었다.
    InventedField,
    /// 원자 없이 총점만 주려 했다.
    HolisticOnly,
    /// 과업 문장이 비었다.
    EmptyTask,
    /// 기준 식별자가 비었다.
    EmptyCriterion,
    /// 봉투에 해당 필드가 없다.
    MissingField,
    /// 관측 값이 기대와 다르다.
    AtomMismatch,
    /// 형제 원자 개수가 모순이다.
    BundleShape,
}

impl FailKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InventedField => "invented_field",
            Self::HolisticOnly => "holistic_only",
            Self::EmptyTask => "empty_task",
            Self::EmptyCriterion => "empty_criterion",
            Self::MissingField => "missing_field",
            Self::AtomMismatch => "atom_mismatch",
            Self::BundleShape => "bundle_shape",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "invented_field" => Some(Self::InventedField),
            "holistic_only" => Some(Self::HolisticOnly),
            "empty_task" => Some(Self::EmptyTask),
            "empty_criterion" => Some(Self::EmptyCriterion),
            "missing_field" => Some(Self::MissingField),
            "atom_mismatch" => Some(Self::AtomMismatch),
            "bundle_shape" => Some(Self::BundleShape),
            _ => None,
        }
    }
}

/// 한 원자 기준의 판정. 총점을 대체하지 않는다.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtomVerdict {
    pub criterion_id: String,
    pub task: String,
    pub envelope_field: String,
    pub atom_pass: bool,
    pub holistic_would_hide: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fail_kind: Option<FailKind>,
}

impl AtomVerdict {
    pub fn is_pass(&self) -> bool {
        self.atom_pass && self.fail_kind.is_none()
    }
}

/// 한 과업의 원자 묶음. 총점 필드를 갖지 않는다.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecompReport {
    pub task: String,
    pub atoms: Vec<AtomVerdict>,
    pub atom_pass_count: u64,
    pub atom_total: u64,
    pub hidden_fail_count: u64,
}

impl DecompReport {
    pub fn all_atoms_pass(&self) -> bool {
        self.atom_total > 0 && self.atom_pass_count == self.atom_total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fail_kind_roundtrip() {
        for k in [
            FailKind::InventedField,
            FailKind::HolisticOnly,
            FailKind::EmptyTask,
            FailKind::EmptyCriterion,
            FailKind::MissingField,
            FailKind::AtomMismatch,
            FailKind::BundleShape,
        ] {
            assert_eq!(FailKind::parse(k.as_str()), Some(k));
        }
    }
}
