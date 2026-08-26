//! 반복 평가의 축소 결과. 후보 순위표가 아니다.

use crate::check::ValueKind;
use crate::variance::VarianceStats;
use crate::vote::VoteTally;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ReduceKind {
    Majority,
    Mean,
}

impl ReduceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Majority => "majority",
            Self::Mean => "mean",
        }
    }
}

/// 최종 값. 산문 점수가 아니다.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinalValue {
    pub reduce: ReduceKind,
    pub value: String,
    pub tie: bool,
    pub pass: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numeric: Option<f64>,
}

impl FinalValue {
    pub fn from_votes(tally: &VoteTally, kind: ValueKind) -> Self {
        let value = tally
            .majority
            .clone()
            .unwrap_or_else(|| tally.plurality.clone());
        let pass = match kind {
            ValueKind::Exit => value == "0",
            ValueKind::Bool => value == "true",
            ValueKind::PassFail => value == "pass",
            ValueKind::Text => value == "pass" || value.eq_ignore_ascii_case("OK"),
            ValueKind::U64 => false,
        };
        Self {
            reduce: ReduceKind::Majority,
            value,
            tie: tally.is_tie,
            pass: pass && !tally.is_tie,
            numeric: None,
        }
    }

    pub fn from_mean(mean: f64, intended: Option<f64>) -> Self {
        let rounded = if mean.fract().abs() < 1e-9 {
            format!("{:.0}", mean)
        } else {
            let s = format!("{:.6}", mean);
            s.trim_end_matches('0').trim_end_matches('.').to_string()
        };
        let pass = match intended {
            Some(want) => (mean - want).abs() < 0.5,
            None => true,
        };
        Self {
            reduce: ReduceKind::Mean,
            value: rounded,
            tie: false,
            pass,
            numeric: Some(mean),
        }
    }
}

/// 한 (산출, 검사, K) 의 축소 보고.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReduceReport {
    pub artifact_id: String,
    pub k: u32,
    pub check: String,
    pub votes: VoteTally,
    pub variance: VarianceStats,
    pub final_value: FinalValue,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn majority_pass_true() {
        let vals = ["true", "true", "false"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();
        let t = VoteTally::from_values(&vals, ValueKind::Bool);
        let f = FinalValue::from_votes(&t, ValueKind::Bool);
        assert_eq!(f.value, "true");
        assert!(f.pass);
        assert!(!f.tie);
    }
}
