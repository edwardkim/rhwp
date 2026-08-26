//! 검증기 입력·판정 필드 JSON Schema. 기존 봉투 모양을 소비한다.

pub const PROTOCOL_SCHEMA_VERSION: &str = "v-proto.1.0";

pub const VERIFIER_INPUT_SCHEMA: &str = include_str!("../schema/verifier_input.schema.json");
pub const JUDGMENT_FIELDS_SCHEMA: &str = include_str!("../schema/judgment_fields.schema.json");
pub const MACHINE_VERDICT_SCHEMA: &str = include_str!("../schema/machine_verdict.schema.json");
pub const OBSERVATION_SCHEMA: &str = include_str!("../schema/observation.schema.json");

/// 닫힌 키 집합. 새 rhwp 필드를 여기서 발명하지 않는다.
pub const REQUIRED_OBSERVATION_KEYS: &[&str] = &["recordId", "sourceTag", "command", "exitClass"];

pub const JUDGMENT_KEY_NAMES: &[&str] = &[
    "identical",
    "hasSignal",
    "reproduced",
    "findingCount",
    "verify",
    "failCount",
    "passCount",
    "verdict",
    "regression",
    "status",
    "clean",
    "signalCount",
    "valid",
    "diffCount",
    "strict",
    "overflowCount",
    "overlapCount",
    "emptyPageCount",
    "pageCountMismatch",
    "overPages",
];

pub fn schema_parses(raw: &str) -> Result<serde_json::Value, String> {
    serde_json::from_str(raw).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_schemas_are_json_objects() {
        for raw in [
            VERIFIER_INPUT_SCHEMA,
            JUDGMENT_FIELDS_SCHEMA,
            MACHINE_VERDICT_SCHEMA,
            OBSERVATION_SCHEMA,
        ] {
            let v = schema_parses(raw).expect("schema json");
            assert!(v.get("$schema").is_some() || v.get("title").is_some());
            assert_eq!(v.get("type").and_then(|t| t.as_str()), Some("object"));
        }
    }
}
