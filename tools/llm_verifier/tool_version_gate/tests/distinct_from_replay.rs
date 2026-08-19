//! V-replay(같은 버전 재실행) 와 필드를 섞지 않는다.

use llm_verifier_tool_version_gate::schema::{FORBIDDEN_KEYS, GATE_ROW_SCHEMA, TUPLE_FIELDS};
use llm_verifier_tool_version_gate::{blob_has_forbidden_key, gate};
use serde_json::json;

#[test]
fn tuple_is_version_pair_not_plan_hash() {
    assert_eq!(
        TUPLE_FIELDS,
        ["attestVersion", "verifyVersion", "reproduced", "accepted"]
    );
    assert!(!TUPLE_FIELDS.contains(&"plan"));
    assert!(!TUPLE_FIELDS.contains(&"expectSha"));
}

#[test]
fn row_schema_has_no_replay_labor_fields() {
    assert!(!GATE_ROW_SCHEMA.contains("planSha256"));
    assert!(!GATE_ROW_SCHEMA.contains("expectedOutputSha256"));
    assert!(!GATE_ROW_SCHEMA.contains("laborAccepted"));
    assert!(!FORBIDDEN_KEYS.is_empty());
}

#[test]
fn decision_json_rejects_replay_keys() {
    let d = gate("0.8.3", "0.8.4", Some(true));
    let raw = serde_json::to_value(&d).expect("json");
    assert!(blob_has_forbidden_key(&raw).is_none());
    assert!(raw.get("plan").is_none());
    assert!(raw.get("expectSha").is_none());
}

#[test]
fn injected_replay_key_is_detected() {
    let v = json!({
        "attestVersion": "0.8.3",
        "plan": "replay this plan"
    });
    assert_eq!(blob_has_forbidden_key(&v).as_deref(), Some("plan"));
}
