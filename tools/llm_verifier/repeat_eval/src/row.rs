//! 코퍼스 한 행: (산출, k, 검사, 표, 분산, 최종).

use crate::artifact::Artifact;
use crate::check::CheckSpec;
use crate::report::FinalValue;
use crate::schema::{CLAIM_ID, FORBIDDEN_KEYS, KIND, PROTOCOL_SCHEMA_VERSION};
use crate::trial::Trial;
use crate::variance::VarianceStats;
use crate::vote::VoteTally;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 유일키. 패딩이 아니다.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UniquenessKey {
    pub artifact_id: String,
    pub k: u32,
    pub check: String,
}

impl UniquenessKey {
    pub fn as_string(&self) -> String {
        format!("{}|k={}|{}", self.artifact_id, self.k, self.check)
    }
}

/// 한 반복 평가 레코드.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepeatRow {
    pub schema_version: String,
    pub claim: String,
    pub kind: String,
    pub record_id: String,
    pub uniqueness_key: String,
    pub artifact: Artifact,
    pub k: u32,
    pub check: CheckSpec,
    pub trials: Vec<Trial>,
    pub votes: VoteTally,
    pub variance: VarianceStats,
    pub final_value: FinalValue,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
}

impl RepeatRow {
    pub fn uniqueness(&self) -> UniquenessKey {
        UniquenessKey {
            artifact_id: self.artifact.artifact_id.clone(),
            k: self.k,
            check: self.check.name.clone(),
        }
    }

    pub fn validate_shape(&self) -> Result<(), String> {
        if self.schema_version != PROTOCOL_SCHEMA_VERSION {
            return Err(format!("schemaVersion {}", self.schema_version));
        }
        if self.claim != CLAIM_ID {
            return Err(format!("claim {}", self.claim));
        }
        if self.kind != KIND {
            return Err(format!("kind {}", self.kind));
        }
        if self.k as usize != self.trials.len() {
            return Err(format!("k={} trials={}", self.k, self.trials.len()));
        }
        if self.k < 2 {
            return Err("k must be >= 2".into());
        }
        let mut seeds = self.trials.iter().map(|t| t.seed).collect::<Vec<_>>();
        seeds.sort_unstable();
        seeds.dedup();
        if seeds.len() != self.trials.len() {
            return Err("duplicate trial seed".into());
        }
        if self.check.name.trim().is_empty() {
            return Err("empty check".into());
        }
        if self.artifact.artifact_id.trim().is_empty() {
            return Err("empty artifactId".into());
        }
        Ok(())
    }
}

pub fn blob_has_forbidden_key(value: &Value) -> Option<String> {
    walk(value, "")
}

fn walk(value: &Value, path: &str) -> Option<String> {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                if FORBIDDEN_KEYS.contains(&k.as_str()) {
                    return Some(k.clone());
                }
                let child = if path.is_empty() {
                    k.clone()
                } else {
                    format!("{path}.{k}")
                };
                if let Some(hit) = walk(v, &child) {
                    return Some(hit);
                }
            }
            None
        }
        Value::Array(items) => {
            for (i, v) in items.iter().enumerate() {
                let child = format!("{path}[{i}]");
                if let Some(hit) = walk(v, &child) {
                    return Some(hit);
                }
            }
            None
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn uniqueness_string() {
        let k = UniquenessKey {
            artifact_id: "a1".into(),
            k: 7,
            check: "verify.identical".into(),
        };
        assert_eq!(k.as_string(), "a1|k=7|verify.identical");
    }

    #[test]
    fn forbidden_best_of_n() {
        let v = json!({"wrapper": [{"bestOfN": 3}]});
        assert_eq!(blob_has_forbidden_key(&v).as_deref(), Some("bestOfN"));
    }
}
