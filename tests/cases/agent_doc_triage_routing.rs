//! [#5296] 쪽수 밴드 라우팅 픽스처와 스킬 문구 정합.
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
fn each_band_is_described_in_skill_or_tree() {
    let text = read_skill() + &read_ref("00_tree.md");
    let payload = read_json("page_thresholds.json");
    for band in payload["bands"].as_array().unwrap() {
        let id = band["id"].as_str().unwrap();
        let label = band["label"].as_str().unwrap();
        assert!(
            text.contains(id) || text.contains(label),
            "밴드 {id}/{label} 문서화 누락"
        );
    }
}

#[test]
fn huge_band_forbids_full_digest_window() {
    let payload = read_json("page_thresholds.json");
    let bands = payload["bands"].as_array().unwrap();
    let huge = bands.iter().find(|b| b["id"] == "huge").unwrap();
    let forbid = huge["forbid"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(forbid.iter().any(|f| f.contains("export-text")));
    assert!(forbid
        .iter()
        .any(|f| f.contains("png") || f.contains("PNG")));
}

#[test]
fn routing_fixture_page_one_is_tiny() {
    let payload = read_json("sample_routing.json");
    let rows = payload["routing"].as_array().unwrap();
    let first = rows.iter().find(|r| r["pageCount"] == 1).unwrap();
    assert_eq!(first["band"], "tiny");
}

#[test]
fn routing_fixture_page_387_is_huge() {
    let payload = read_json("sample_routing.json");
    let rows = payload["routing"].as_array().unwrap();
    let row = rows.iter().find(|r| r["pageCount"] == 387).unwrap();
    assert_eq!(row["band"], "huge");
    assert_eq!(row["mustAnnounceTruncation"], true);
}
