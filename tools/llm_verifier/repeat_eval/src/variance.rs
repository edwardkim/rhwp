//! K회 관측의 분산. 순위 점수가 아니다.

use serde::{Deserialize, Serialize};

/// 범주 불일치와 수치 표본분산.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VarianceStats {
    pub n: u64,
    pub distinct: u64,
    pub disagreement: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample_variance: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mean: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
}

impl VarianceStats {
    pub fn categorical(values: &[String], majority_frac: f64) -> Self {
        let mut uniq = values.to_vec();
        uniq.sort();
        uniq.dedup();
        Self {
            n: values.len() as u64,
            distinct: uniq.len() as u64,
            disagreement: (1.0 - majority_frac).clamp(0.0, 1.0),
            sample_variance: None,
            mean: None,
            min: None,
            max: None,
        }
    }

    pub fn numeric(xs: &[f64]) -> Self {
        let n = xs.len() as u64;
        if xs.is_empty() {
            return Self {
                n: 0,
                distinct: 0,
                disagreement: 0.0,
                sample_variance: None,
                mean: None,
                min: None,
                max: None,
            };
        }
        let mean = xs.iter().sum::<f64>() / xs.len() as f64;
        let var = if xs.len() >= 2 {
            let ss = xs
                .iter()
                .map(|x| {
                    let d = x - mean;
                    d * d
                })
                .sum::<f64>();
            Some(ss / (xs.len() as f64 - 1.0))
        } else {
            Some(0.0)
        };
        let mut sorted = xs.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        sorted.dedup_by(|a, b| (*a - *b).abs() < 1e-12);
        let disagreement = if xs.is_empty() {
            0.0
        } else {
            let mut counts: Vec<u64> = Vec::new();
            for x in xs {
                match counts.iter_mut().next() {
                    _ => {}
                }
                let _ = x;
            }
            let mut freq: std::collections::BTreeMap<i64, u64> = std::collections::BTreeMap::new();
            for x in xs {
                let key = (x * 1000.0).round() as i64;
                *freq.entry(key).or_insert(0) += 1;
            }
            let max = freq.values().copied().max().unwrap_or(0);
            1.0 - (max as f64 / n as f64)
        };
        Self {
            n,
            distinct: sorted.len() as u64,
            disagreement,
            sample_variance: var,
            mean: Some(mean),
            min: xs.iter().copied().reduce(f64::min),
            max: xs.iter().copied().reduce(f64::max),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unanimous_zero_disagreement() {
        let vals = vec!["0".into(), "0".into(), "0".into()];
        let v = VarianceStats::categorical(&vals, 1.0);
        assert_eq!(v.distinct, 1);
        assert_eq!(v.disagreement, 0.0);
    }

    #[test]
    fn numeric_sample_variance() {
        let v = VarianceStats::numeric(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]);
        assert!((v.mean.unwrap() - 5.0).abs() < 1e-9);
        assert!((v.sample_variance.unwrap() - 4.5714285714).abs() < 1e-6);
    }
}
