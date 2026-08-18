//! [#5307] rhwp-security-sweep 스킬 — 실사용 에이전트 보안 스윕 계약.
//!
//! 새 CLI 를 만들지 않는다. 권위는 cli_commands.md 와 스킬 픽스처다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeSet;
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

#[test]
fn receive_ladder_order() {
    let doc = read_json("fixtures/receive_ladder.json");
    let ladder = doc["ladder"].as_array().unwrap();
    let cmds: Vec<&str> = ladder
        .iter()
        .map(|s| s["command"].as_str().unwrap())
        .collect();
    assert_eq!(cmds[0], "info");
    assert_eq!(cmds[1], "digest");
    assert_eq!(cmds[2], "fields");
    assert!(cmds[3].contains("injection"));
    assert!(cmds[4].contains("hidden-text"));
    assert!(cmds[5].contains("unicode"));
    assert_eq!(cmds[6], "export-text");
    assert_eq!(ladder[6]["afterGate"], true);
    let forbid = doc["forbidBeforeInspect"].as_array().unwrap();
    assert!(forbid.iter().any(|v| v == "export-text"));
}

#[test]
fn receive_docs_state_the_order() {
    let text = read_skill("SKILL.md") + &read_skill("references/09_receive_path.md");
    let info = text.find("info").expect("info");
    let digest = text.find("digest").expect("digest");
    let fields = text.find("fields").expect("fields");
    let inspect = text.find("inspect").expect("inspect");
    assert!(
        info < digest && digest < fields && fields < inspect,
        "사다리 순서"
    );
    assert!(text.contains("export-text"));
}

#[test]
fn receive_envelopes_mark_document_text_untrusted() {
    for name in [
        "receive_info.json",
        "receive_digest.json",
        "receive_fields_clean.json",
        "export_text_untrusted.json",
    ] {
        let env = read_json(&format!("fixtures/envelopes/{name}"));
        assert_eq!(env["untrustedContent"], true, "{name}");
        assert!(
            env["untrustedFields"].as_array().unwrap().len() > 0,
            "{name} fields"
        );
    }
    let dirty = read_json("fixtures/envelopes/receive_fields_dirty.json");
    assert_ne!(dirty["textSecurity"]["status"], "clean");
}

#[test]
fn untrusted_catalog_covers_sweep_fields() {
    let cat = read_json("fixtures/untrusted_fields.json");
    assert!(cat["principle"].as_str().unwrap().contains("DATA"));
    let paths: BTreeSet<&str> = cat["fields"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|f| f["path"].as_str())
        .collect();
    for p in [
        "hiddenText[].excerpt",
        "injectionSignals[].matched",
        "findings[].raw",
        "fields[].guide",
        "pages[].text",
    ] {
        assert!(paths.contains(p), "카탈로그에 {p}");
    }
    let slots: BTreeSet<&str> = cat["forbiddenSlots"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|s| s.as_str())
        .collect();
    for s in ["system_prompt", "shell_command", "tool_argument_path"] {
        assert!(slots.contains(s), "금지 자리 {s}");
    }
}

#[test]
fn untrusted_chapter_forbids_following_signals() {
    let text = read_skill("references/11_untrusted_content.md");
    assert!(text.contains("untrustedContent"));
    assert!(text.contains("untrustedFields"));
    assert!(text.contains("데이터"));
    assert!(text.contains("지시"));
    assert!(text.contains("export-provenance-map"));
}
