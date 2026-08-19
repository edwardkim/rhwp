//! V-fresh 스키마 상수. V-replay(같은 버전 재실행) 필드를 들이지 않는다.

pub const PROTOCOL_SCHEMA_VERSION: &str = "v-fresh.1.0";
pub const CLAIM_ID: &str = "V-fresh";
pub const KIND: &str = "toolVersionGate";

/// 유일키. 패딩이 아니다.
pub const UNIQUENESS: &str = "attestVersion|verifyVersion|reproduced|accepted";

/// 이 축의 4열. 코퍼스 한 행이 이 튜플이다.
pub const TUPLE_FIELDS: [&str; 4] = ["attestVersion", "verifyVersion", "reproduced", "accepted"];

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
    "plan",
    "planText",
    "planSha256",
    "expectSha",
    "expectedOutputSha256",
    "laborAccepted",
];

pub const GATE_ROW_SCHEMA: &str = include_str!("../schema/gate_row.schema.json");
pub const DECISION_SCHEMA: &str = include_str!("../schema/decision.schema.json");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_version_is_fresh_axis() {
        assert_eq!(PROTOCOL_SCHEMA_VERSION, "v-fresh.1.0");
        assert_eq!(CLAIM_ID, "V-fresh");
        assert_eq!(KIND, "toolVersionGate");
    }

    #[test]
    fn forbidden_keys_block_replay_and_other_axes() {
        assert!(FORBIDDEN_KEYS.contains(&"plan"));
        assert!(FORBIDDEN_KEYS.contains(&"expectSha"));
        assert!(FORBIDDEN_KEYS.contains(&"bestOfN"));
        assert!(FORBIDDEN_KEYS.contains(&"holisticScore"));
        assert!(FORBIDDEN_KEYS.contains(&"processReward"));
    }

    #[test]
    fn bundled_schemas_are_json_objects() {
        for raw in [GATE_ROW_SCHEMA, DECISION_SCHEMA] {
            let v: serde_json::Value = serde_json::from_str(raw).expect("schema json");
            assert!(v.get("$schema").is_some());
            assert!(v.get("title").is_some());
        }
    }
}
