//! 범주 표. 후보 순위가 아니다.

use crate::check::ValueKind;
use crate::exit_class::ExitClass;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// 값 → 득표. 키는 안정 정렬을 위해 BTree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoteTally {
    pub counts: BTreeMap<String, u64>,
    pub majority: Option<String>,
    pub plurality: String,
    pub majority_count: u64,
    pub is_tie: bool,
    pub majority_frac: f64,
}

impl VoteTally {
    pub fn from_values(values: &[String], kind: ValueKind) -> Self {
        let mut counts: BTreeMap<String, u64> = BTreeMap::new();
        for v in values {
            *counts.entry(v.clone()).or_insert(0) += 1;
        }
        let total = values.len() as u64;
        let max = counts.values().copied().max().unwrap_or(0);
        let mut winners: Vec<String> = counts
            .iter()
            .filter(|(_, c)| **c == max)
            .map(|(k, _)| k.clone())
            .collect();
        winners.sort();
        let is_tie = winners.len() > 1;
        let plurality = if is_tie {
            conservative_pick(&winners, kind)
        } else {
            winners.first().cloned().unwrap_or_else(|| "missing".into())
        };
        let majority = if !is_tie && max * 2 > total {
            Some(plurality.clone())
        } else {
            None
        };
        let majority_frac = if total == 0 {
            0.0
        } else {
            max as f64 / total as f64
        };
        Self {
            counts,
            majority,
            plurality,
            majority_count: max,
            is_tie,
            majority_frac,
        }
    }
}

/// 동률이면 fail-closed. 순위 키가 아니다.
pub fn conservative_pick(winners: &[String], kind: ValueKind) -> String {
    match kind {
        ValueKind::Exit => winners
            .iter()
            .filter_map(|s| s.parse::<i32>().ok().and_then(ExitClass::from_code))
            .reduce(ExitClass::worse)
            .map(|e| e.code().to_string())
            .unwrap_or_else(|| winners.last().cloned().unwrap_or_else(|| "3".into())),
        ValueKind::Bool => {
            if winners.iter().any(|w| w == "false") {
                "false".into()
            } else {
                winners.last().cloned().unwrap_or_else(|| "false".into())
            }
        }
        ValueKind::PassFail => {
            if winners.iter().any(|w| w == "fail") {
                "fail".into()
            } else {
                winners.last().cloned().unwrap_or_else(|| "fail".into())
            }
        }
        ValueKind::Text => {
            for bad in ["fail", "FAIL", "judgment_fail", "mismatch"] {
                if winners.iter().any(|w| w == bad) {
                    return bad.into();
                }
            }
            winners.last().cloned().unwrap_or_else(|| "fail".into())
        }
        ValueKind::U64 => winners
            .iter()
            .filter_map(|s| s.parse::<u64>().ok())
            .max()
            .map(|n| n.to_string())
            .unwrap_or_else(|| winners.last().cloned().unwrap_or_else(|| "0".into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn majority_true() {
        let vals = ["true", "true", "true", "false", "true"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();
        let t = VoteTally::from_values(&vals, ValueKind::Bool);
        assert_eq!(t.majority.as_deref(), Some("true"));
        assert!(!t.is_tie);
        assert!((t.majority_frac - 0.8).abs() < 1e-9);
    }

    #[test]
    fn tie_bool_is_false() {
        let vals = ["true", "false", "true", "false"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();
        let t = VoteTally::from_values(&vals, ValueKind::Bool);
        assert!(t.is_tie);
        assert_eq!(t.plurality, "false");
        assert!(t.majority.is_none());
    }

    #[test]
    fn tie_exit_picks_worse() {
        let vals = ["0", "3", "0", "3"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();
        let t = VoteTally::from_values(&vals, ValueKind::Exit);
        assert_eq!(t.plurality, "3");
    }
}
