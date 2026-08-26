//! 코퍼스 한 행: (attest_version, verify_version, reproduced, accepted).

use crate::gate::{gate, reproduced_token};
use crate::reason::Reason;
use crate::schema::{CLAIM_ID, FORBIDDEN_KEYS, KIND, PROTOCOL_SCHEMA_VERSION};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 유일키. 패딩이 아니다.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UniquenessKey {
    pub attest_version: String,
    pub verify_version: String,
    pub reproduced: Option<bool>,
    pub accepted: bool,
}

impl UniquenessKey {
    pub fn as_string(&self) -> String {
        format!(
            "{}|{}|{}|{}",
            self.attest_version,
            self.verify_version,
            reproduced_token(self.reproduced),
            self.accepted
        )
    }
}

/// 한 게이트 레코드.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GateRow {
    pub schema_version: String,
    pub claim: String,
    pub kind: String,
    pub record_id: String,
    pub uniqueness_key: String,
    pub attest_version: String,
    pub verify_version: String,
    pub reproduced: Option<bool>,
    pub accepted: bool,
    pub reason: Reason,
    pub family: String,
}

impl GateRow {
    pub fn uniqueness(&self) -> UniquenessKey {
        UniquenessKey {
            attest_version: self.attest_version.clone(),
            verify_version: self.verify_version.clone(),
            reproduced: self.reproduced,
            accepted: self.accepted,
        }
    }

    pub fn recompute(&self) -> crate::gate::Decision {
        gate(&self.attest_version, &self.verify_version, self.reproduced)
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
        if self.record_id.trim().is_empty() {
            return Err("empty recordId".into());
        }
        if self.family.trim().is_empty() {
            return Err("empty family".into());
        }
        let got = self.recompute();
        if got.accepted != self.accepted {
            return Err(format!(
                "accepted {} vs recomputed {}",
                self.accepted, got.accepted
            ));
        }
        if got.reason != self.reason {
            return Err(format!(
                "reason {} vs recomputed {}",
                self.reason.as_str(),
                got.reason.as_str()
            ));
        }
        if self.uniqueness_key != self.uniqueness().as_string() {
            return Err(format!("uniquenessKey {}", self.uniqueness_key));
        }
        if self.reason == Reason::StaleTool && self.accepted {
            return Err("STALE_TOOL must not be accepted".into());
        }
        if self.accepted && self.reason != Reason::FreshReproduced {
            return Err("accepted without FRESH_REPRODUCED".into());
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
            attest_version: "0.8.3".into(),
            verify_version: "0.8.4".into(),
            reproduced: Some(true),
            accepted: false,
        };
        assert_eq!(k.as_string(), "0.8.3|0.8.4|true|false");
    }

    #[test]
    fn forbidden_plan_from_replay_axis() {
        let v = json!({"wrapper": [{"plan": "do it"}]});
        assert_eq!(blob_has_forbidden_key(&v).as_deref(), Some("plan"));
    }
}
