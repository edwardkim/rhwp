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

fn finding_has_raw(f: &Value) -> bool {
    f.as_object()
        .map(|o| o.contains_key("raw"))
        .unwrap_or(false)
}

#[test]
fn automation_default_is_no_raw() {
    let auto = read_json("fixtures/automation_no_raw.json");
    assert_eq!(auto["cliDefaultIncludesRaw"], true);
    assert_eq!(auto["agentAutomationDefault"], "--no-raw");
    let flags = auto["requiredFlags"].as_array().unwrap();
    assert!(flags.iter().any(|v| v == "--no-raw"));
    assert!(flags.iter().any(|v| v == "--dry-run"));
}

#[test]
fn no_raw_envelope_omits_raw_key() {
    let env = read_json("fixtures/envelopes/redact_dry_run_no_raw.json");
    assert_eq!(env["noRaw"], true);
    assert_eq!(env["dryRun"], true);
    assert_eq!(env["findingCount"], 4);
    for f in env["findings"].as_array().unwrap() {
        assert!(!finding_has_raw(f), "no-raw 인데 raw 키가 있다: {f}");
        assert!(f.get("masked").is_some());
        assert!(f.get("kind").is_some());
    }
}

#[test]
fn with_raw_envelope_is_not_for_logs() {
    let env = read_json("fixtures/envelopes/redact_dry_run_with_raw.json");
    assert_eq!(env["noRaw"], false);
    let mut raws = 0;
    for f in env["findings"].as_array().unwrap() {
        if finding_has_raw(f) {
            raws += 1;
        }
    }
    assert_eq!(raws, 4);
    assert!(env["warning"].as_str().unwrap().contains("로그"));
}

#[test]
fn skill_requires_no_raw_for_automation() {
    let text = read_skill("SKILL.md") + &read_skill("references/06_no_raw.md");
    assert!(text.contains("--no-raw"));
    assert!(text.contains("자동화"));
    assert!(text.contains("findings[].raw") || text.contains("raw"));
}

#[test]
fn missing_output_and_exit2_are_documented() {
    let miss = read_json("fixtures/envelopes/redact_missing_output.json");
    assert_eq!(miss["findingCount"], 0);
    assert!(miss.get("output").is_none());
    let e2 = read_json("fixtures/envelopes/redact_exit2_no_output.json");
    assert_eq!(e2["exitCode"], 2);
    assert_eq!(e2["stdoutBytes"], 0);
}

#[test]
fn detection_is_not_failure_in_exit_catalog() {
    let exits = read_json("fixtures/exit_codes.json");
    assert_eq!(exits["inspect"]["signalPresent"], 0);
    assert_eq!(exits["inspect"]["detectionIsFailure"], false);
    let skill = read_skill("references/10_exit_codes.md");
    assert!(skill.contains("탐지") && skill.contains("실패"));
    assert!(skill.contains("판정은 데이터"));
}
