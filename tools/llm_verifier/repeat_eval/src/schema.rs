//! 스키마 버전과 동봉 JSON Schema 본문.

pub const PROTOCOL_SCHEMA_VERSION: &str = "v-repeat.1.0";
pub const CLAIM_ID: &str = "V-repeat";
pub const KIND: &str = "repeatEvaluation";

/// 다른 LLM-as-verifier 축의 필드. 이 크레이트는 읽지도 쓰지도 않는다.
pub const FORBIDDEN_KEYS: &[&str] = &[
    "bestOfN",
    "expectedRank",
    "winnerId",
    "rankFields",
    "ranking",
    "holisticScore",
    "atomPass",
    "holisticWouldHide",
    "criterionId",
    "processReward",
    "process_steps",
    "processSteps",
    "stepReward",
    "proseScore",
    "llmScore",
    "rubricScore",
];

pub const REPEAT_ROW_SCHEMA: &str = include_str!("../schema/repeat_row.schema.json");
pub const TRIAL_SCHEMA: &str = include_str!("../schema/trial.schema.json");
pub const REDUCE_REPORT_SCHEMA: &str = include_str!("../schema/reduce_report.schema.json");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_schemas_are_json_objects() {
        for raw in [REPEAT_ROW_SCHEMA, TRIAL_SCHEMA, REDUCE_REPORT_SCHEMA] {
            let v: serde_json::Value = serde_json::from_str(raw).expect("schema json");
            assert!(v.get("$schema").is_some());
            assert!(v.get("title").is_some());
        }
    }
}
