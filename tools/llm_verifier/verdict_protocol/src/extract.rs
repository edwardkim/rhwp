//! 기존 봉투에서 판정 필드만 꺼낸다. 모르는 키는 무시한다.

use crate::judgment::{JudgmentFields, Reproduced, VerifyBlock};
use serde_json::Value;

fn as_bool(v: Option<&Value>) -> Option<bool> {
    match v {
        Some(Value::Bool(b)) => Some(*b),
        Some(Value::String(s)) if s == "true" => Some(true),
        Some(Value::String(s)) if s == "false" => Some(false),
        _ => None,
    }
}

fn as_u64(v: Option<&Value>) -> Option<u64> {
    match v {
        Some(Value::Number(n)) => n
            .as_u64()
            .or_else(|| n.as_i64().and_then(|i| i.try_into().ok())),
        Some(Value::String(s)) => s.parse().ok(),
        _ => None,
    }
}

fn as_string(v: Option<&Value>) -> Option<String> {
    match v {
        Some(Value::String(s)) => Some(s.clone()),
        Some(Value::Null) | None => None,
        Some(other) => Some(other.to_string()),
    }
}

fn reproduced_of(v: Option<&Value>) -> Option<Reproduced> {
    match v {
        None => None,
        Some(Value::Null) => Some(Reproduced::Null),
        Some(Value::Bool(b)) => Some(Reproduced::Flag(*b)),
        Some(Value::Number(n)) => n
            .as_u64()
            .or_else(|| n.as_i64().and_then(|i| i.try_into().ok()))
            .map(Reproduced::Count),
        _ => None,
    }
}

fn verify_block(v: Option<&Value>) -> Option<VerifyBlock> {
    let obj = v.and_then(Value::as_object)?;
    let block = VerifyBlock {
        identical: as_bool(obj.get("identical")),
        diff_count: as_u64(obj.get("diffCount")),
    };
    if block.is_empty() {
        None
    } else {
        Some(block)
    }
}

/// rhwp `--json` 봉투에서 지식지도 판정 필드만 추출한다.
pub fn extract_judgment(envelope: &Value) -> JudgmentFields {
    let obj = match envelope.as_object() {
        Some(o) => o,
        None => return JudgmentFields::default(),
    };
    JudgmentFields {
        identical: as_bool(obj.get("identical")),
        has_signal: as_bool(obj.get("hasSignal")),
        reproduced: reproduced_of(obj.get("reproduced")),
        finding_count: as_u64(obj.get("findingCount")),
        verify: verify_block(obj.get("verify")),
        fail_count: as_u64(obj.get("failCount")),
        pass_count: as_u64(obj.get("passCount")),
        verdict: as_string(obj.get("verdict")),
        regression: as_bool(obj.get("regression")),
        status: as_string(obj.get("status")),
        clean: as_bool(obj.get("clean")),
        signal_count: as_u64(obj.get("signalCount")),
        valid: as_bool(obj.get("valid")),
        diff_count: as_u64(obj.get("diffCount")),
        strict: as_bool(obj.get("strict")),
        overflow_count: as_u64(obj.get("overflowCount")),
        overlap_count: as_u64(obj.get("overlapCount")),
        empty_page_count: as_u64(obj.get("emptyPageCount")),
        page_count_mismatch: as_bool(obj.get("pageCountMismatch")),
        over_pages: as_u64(obj.get("overPages")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn ir_diff_identical_false() {
        let env = json!({
            "schemaVersion": "1.0",
            "identical": false,
            "diffCount": 4,
            "categories": { "text": 3, "table": 1 }
        });
        let j = extract_judgment(&env);
        assert_eq!(j.identical, Some(false));
        assert_eq!(j.diff_count, Some(4));
        assert!(j.fail_signals().contains(&"identical=false"));
    }

    #[test]
    fn fill_fields_verify_nested() {
        let env = json!({
            "schemaVersion": "1.0",
            "filledCount": 2,
            "verify": { "identical": true, "diffCount": 0 }
        });
        let j = extract_judgment(&env);
        assert_eq!(j.verify.as_ref().unwrap().identical, Some(true));
        assert_eq!(j.verify.as_ref().unwrap().diff_count, Some(0));
        assert!(!j.has_any_fail_signal());
    }

    #[test]
    fn replay_attest_reproduced_null() {
        let env = json!({
            "mode": "attest",
            "reproduced": null,
            "outputSha256": "abc"
        });
        let j = extract_judgment(&env);
        assert!(matches!(j.reproduced, Some(Reproduced::Null)));
        assert!(!j.has_any_fail_signal());
    }
}
