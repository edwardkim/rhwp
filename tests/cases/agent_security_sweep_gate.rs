//! [#5307] rhwp-security-sweep 스킬 — 실사용 에이전트 보안 스윕 계약.
//!
//! 새 CLI 를 만들지 않는다. 권위는 cli_commands.md 와 스킬 픽스처다.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::PathBuf;

use serde_json::Value;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn skill_dir() -> PathBuf {
    repo().join(".claude/skills/rhwp-security-sweep")
}

fn read_skill(rel: &str) -> String {
    let path = skill_dir().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{} 읽기 실패: {e}", path.display()))
}

fn read_json(rel: &str) -> Value {
    let text = read_skill(rel);
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{rel} JSON 파싱 실패: {e}"))
}

fn share_predicate(c: &Value) -> bool {
    let findings = c["redactFindingCount"].as_u64().unwrap_or(99);
    let hidden = c["hiddenClean"].as_bool().unwrap_or(false);
    let inj = c["injectionClean"].as_bool().unwrap_or(false);
    let uni = c["unicodeClean"].as_bool().unwrap_or(false);
    findings == 0 && hidden && inj && uni
}

#[test]
fn gate_cases_match_predicate() {
    let doc = read_json("fixtures/gate_cases.json");
    assert_eq!(doc["predicate"], "findingCount==0 AND clean==true");
    let cases = doc["cases"].as_array().unwrap();
    assert!(cases.len() >= 8);
    for c in cases {
        let id = c["id"].as_str().unwrap();
        if id == "G07-sanitize-missing-not-gate" {
            assert_eq!(c["share"], false, "{id} 는 절차상 금지");
            continue;
        }
        let expected = c["share"].as_bool().unwrap();
        let got = share_predicate(c);
        if id.starts_with("G01") {
            assert!(got && expected, "{id}");
        } else {
            assert_eq!(got, expected, "{id} predicate={got} share={expected}");
        }
    }
}

#[test]
fn resweep_envelopes_encode_pass_and_fail() {
    let pass = read_json("fixtures/envelopes/resweep_pass.json");
    assert_eq!(pass["gate"], true);
    assert_eq!(pass["redact"]["findingCount"], 0);
    assert_eq!(pass["hidden"]["clean"], true);
    assert_eq!(pass["injection"]["clean"], true);
    assert_eq!(pass["unicode"]["clean"], true);

    let pii = read_json("fixtures/envelopes/resweep_fail_pii.json");
    assert_eq!(pii["gate"], false);
    assert!(pii["redact"]["findingCount"].as_u64().unwrap() > 0);
    assert_eq!(pii["hidden"]["clean"], true);

    let hid = read_json("fixtures/envelopes/resweep_fail_hidden.json");
    assert_eq!(hid["gate"], false);
    assert_eq!(hid["hidden"]["clean"], false);
}

#[test]
fn skill_states_gate_predicate() {
    let text = read_skill("SKILL.md") + &read_skill("references/08_resweep_gate.md");
    assert!(text.contains("findingCount == 0") || text.contains("findingCount==0"));
    assert!(text.contains("clean == true") || text.contains("clean==true"));
    assert!(text.contains("재스윕"));
}

#[test]
fn pair_is_required_in_send_path() {
    let pair = read_skill("references/07_redact_sanitize_pair.md");
    assert!(pair.contains("짝"));
    assert!(pair.contains("미리보기"));
    assert!(pair.contains("removedCount"));
    let second = read_json("fixtures/envelopes/sanitize_second_zero.json");
    assert_eq!(second["removedCount"], 0);
    let first = read_json("fixtures/envelopes/sanitize_first.json");
    assert!(first["removedCount"].as_u64().unwrap() > 0);
}
