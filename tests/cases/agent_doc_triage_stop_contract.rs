//! [#5296] 정지 규칙·반덤프 문구가 스킬과 픽스처에서 일치한다.
#![cfg(not(target_arch = "wasm32"))]
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_dir() -> PathBuf {
    repo_root().join("tests/fixtures/agent_doc_triage")
}

fn skill_dir() -> PathBuf {
    repo_root().join(".claude/skills/rhwp-doc-triage")
}

fn read_json(name: &str) -> serde_json::Value {
    let path = fixture_dir().join(name);
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{path:?} JSON: {e}"))
}

fn read_skill() -> String {
    fs::read_to_string(skill_dir().join("SKILL.md")).expect("SKILL.md")
}

fn read_ref(name: &str) -> String {
    fs::read_to_string(skill_dir().join("references").join(name))
        .unwrap_or_else(|e| panic!("{name}: {e}"))
}

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample(rel: &str) -> PathBuf {
    repo_root().join(rel)
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn describe(args: &[&str], output: &Output) -> String {
    format!(
        "명령: rhwp {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn run_ok_json(args: &[&str]) -> serde_json::Value {
    let output = run(args);
    assert_eq!(output.status.code(), Some(0), "{}", describe(args, &output));
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|e| panic!("stdout JSON 아님 ({e}).\n{}", describe(args, &output)))
}

#[test]
fn anti_dump_reference_lists_forbidden_patterns() {
    let text = read_ref("15_anti_dump.md");
    for pat in [
        "export-text",
        "--max-chars",
        "export-png",
        "digest --pages",
        "전문",
    ] {
        assert!(text.contains(pat), "anti-dump 에 {pat} 없음");
    }
}

#[test]
fn context_budget_reference_mentions_limits() {
    let text = read_ref("10_context_budget.md");
    assert!(text.contains("--limit"));
    assert!(text.contains("--max-chars"));
    assert!(text.contains("truncated"));
}

#[test]
fn page_address_reference_states_zero_based() {
    let text = read_ref("14_page_address.md");
    assert!(text.contains("0"));
    assert!(text.contains("page+1") || text.contains("page + 1") || text.contains("page+1"));
    assert!(text.contains("extract-pages"));
}

#[test]
fn stop_rules_never_recommend_unlimited_export_text() {
    let rules = read_json("stop_rules.json");
    for rule in rules["rules"].as_array().unwrap() {
        let never = rule["never"].as_str().unwrap();
        let action = rule["action"].as_str().unwrap();
        assert!(
            !action.contains("export-text 무제한"),
            "정지가 무제한 덤프를 시키면 안 된다: {rule}"
        );
        let _ = never;
    }
}

#[test]
fn skill_stop_table_has_all_fixture_ids() {
    let skill = read_skill();
    let stop = read_ref("07_when_to_stop.md");
    let joined = skill + &stop;
    let payload = read_json("stop_rules.json");
    for rule in payload["rules"].as_array().unwrap() {
        let id = rule["id"].as_str().unwrap();
        assert!(joined.contains(id), "{id}");
    }
}
