use llm_verifier_verdict_protocol::{extract_judgment, Reproduced};
use serde_json::json;

#[test]
fn layout_anomaly_fields() {
    let j = extract_judgment(&json!({
        "schemaVersion": "1.0",
        "hasSignal": true,
        "strict": true,
        "overflowCount": 2,
        "overlapCount": 1,
        "emptyPageCount": 0
    }));
    assert_eq!(j.has_signal, Some(true));
    assert!(j.layout_strict_fail());
}

#[test]
fn replay_count_reproduced() {
    let j = extract_judgment(&json!({"reproduced": 4}));
    assert!(matches!(j.reproduced, Some(Reproduced::Count(4))));
}

#[test]
fn unknown_keys_are_ignored() {
    let j = extract_judgment(&json!({
        "schemaVersion": "1.0",
        "title": "무시",
        "identical": true
    }));
    assert_eq!(j.identical, Some(true));
    assert!(j.finding_count.is_none());
}
