//! [#5296] rhwp-doc-triage 스킬 파일·사다리·정지 규칙 커버리지 가드.
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
fn skill_frontmatter_names_doc_triage() {
    let text = read_skill();
    assert!(text.starts_with("---\n"), "frontmatter 필요");
    assert!(text.contains("name: rhwp-doc-triage"), "{text}");
    assert!(text.contains("info"), "{text}");
    assert!(text.contains("explain"), "{text}");
    assert!(text.contains("export-structure"), "{text}");
    assert!(text.contains("digest"), "{text}");
    assert!(text.contains("search"), "{text}");
    assert!(text.contains("extract-data"), "{text}");
}

#[test]
fn references_listed_in_skill_index_exist() {
    let idx = read_json("skill_index.json");
    let refs = idx["references"].as_array().expect("references");
    assert!(refs.len() >= 16, "레퍼런스 16장 이상: {refs:?}");
    for r in refs {
        let name = r.as_str().expect("name");
        let path = skill_dir().join("references").join(name);
        assert!(path.is_file(), "누락 {path:?}");
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.len() > 400, "{name} 가 너무 짧다");
    }
}

#[test]
fn skill_forbids_full_dump_and_gym() {
    let text = read_skill();
    assert!(text.contains("전문"), "{text}");
    assert!(text.contains("덤프"), "{text}");
    assert!(
        text.contains("gym 이 아니고") || text.contains("gym"),
        "gym 이 아님을 밝혀야 한다"
    );
    assert!(text.contains("07_when_to_stop"), "{text}");
}

#[test]
fn ladder_order_is_documented_in_tree_and_skill() {
    let tree = read_ref("00_tree.md");
    let skill = read_skill();
    let ladder = read_json("command_ladder.json");
    let steps = ladder["ladder"].as_array().unwrap();
    let mut prev_pos = 0usize;
    for step in steps {
        let cmd = step["command"].as_str().unwrap();
        let pos = tree.find(cmd).expect(cmd);
        assert!(pos >= prev_pos, "트리 순서가 뒤집힘: {cmd}");
        prev_pos = pos;
        assert!(skill.contains(cmd), "SKILL 에 {cmd} 없음");
    }
}

#[test]
fn stop_rule_ids_appear_in_stop_reference() {
    let stop = read_ref("07_when_to_stop.md");
    let rules = read_json("stop_rules.json");
    for rule in rules["rules"].as_array().unwrap() {
        let id = rule["id"].as_str().unwrap();
        assert!(stop.contains(id), "정지 장에 {id} 없음");
        assert!(read_skill().contains(id) || stop.contains(id));
    }
}

#[test]
fn handoff_skills_are_named() {
    let text = read_ref("09_handoff.md") + &read_skill();
    for name in [
        "rhwp-table-exchange",
        "rhwp-form-fill",
        "rhwp-security-sweep",
        "rhwp-provenance",
        "rhwp-safe-edit",
        "rhwp-bulk-pipeline",
    ] {
        assert!(text.contains(name), "인계 {name} 누락");
    }
}

#[test]
fn forbidden_peer_skills_not_rewritten_here() {
    let idx = read_json("skill_index.json");
    for name in idx["forbiddenSkillsTouch"].as_array().unwrap() {
        let slug = name.as_str().unwrap();
        let peer = repo_root()
            .join(".claude/skills")
            .join(slug)
            .join("SKILL.md");
        assert!(peer.is_file(), "존재해야 하는 이웃 스킬 {peer:?}");
    }
}
