//! 한 과업의 원자 묶음. 총점 필드가 없다.

use crate::atom::AtomSpec;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskBundle {
    pub task: String,
    pub atoms: Vec<AtomSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub envelope: Option<Value>,
    /// 있으면 원자 없이 총점만 주려는 잘못된 입력이다.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub holistic_score: Option<f64>,
}

impl TaskBundle {
    pub fn is_holistic_only(&self) -> bool {
        self.atoms.is_empty() && self.holistic_score.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_without_atoms_is_holistic_only() {
        let b = TaskBundle {
            task: "한 덩어리로만 채점한다".into(),
            atoms: Vec::new(),
            envelope: None,
            holistic_score: Some(0.87),
        };
        assert!(b.is_holistic_only());
    }
}
