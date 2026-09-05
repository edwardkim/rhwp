//! [#5296] 실사용 여정이 레퍼런스와 픽스처에 같이 있다.
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
    repo_root().join(".agents/skills/rhwp-doc-triage")
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
fn every_journey_id_appears_in_journeys_reference() {
    let text = read_ref("12_journeys.md");
    let payload = read_json("journeys.json");
    for j in payload["journeys"].as_array().unwrap() {
        let id = j["id"].as_str().unwrap();
        assert!(text.contains(id), "여정 장에 {id} 없음");
        assert!(j["ask"].as_str().unwrap().chars().count() > 2);
    }
}

#[test]
fn journeys_do_not_start_with_unlimited_export_text_except_tiny() {
    let payload = read_json("journeys.json");
    for j in payload["journeys"].as_array().unwrap() {
        let steps = j["steps"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap())
            .collect::<Vec<_>>();
        let pages = j["pages"].as_str().unwrap();
        if pages != "tiny" {
            assert!(
                !steps.contains(&"export-text"),
                "tiny 가 아닌 여정이 전문으로 시작: {j}"
            );
        }
    }
}

#[test]
fn form_and_table_journeys_handoff() {
    let js = read_json("journeys.json");
    let form = js["journeys"]
        .as_array()
        .unwrap()
        .iter()
        .find(|j| j["id"] == "J07")
        .unwrap();
    assert_eq!(form["stop"], "S11");
    let table = js["journeys"]
        .as_array()
        .unwrap()
        .iter()
        .find(|j| j["id"] == "J08")
        .unwrap();
    assert_eq!(table["stop"], "S11");
}
