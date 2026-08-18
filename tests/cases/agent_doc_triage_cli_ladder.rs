//! [#5296] 트리아지 사다리를 실 CLI 봉투에 대조한다. 새 명령은 만들지 않는다.
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

const SMALL: &str = "samples/para-001.hwp";
const FIELDS: &str = "samples/field-01.hwp";
const TABLE: &str = "samples/table-001.hwp";
const DIGEST_SAMPLE: &str = "samples/hwp3-sample.hwp";
const STRUCT_SAMPLE: &str = "samples/hwp3-sample16.hwp";

fn required_keys(command: &str) -> Vec<String> {
    let env = read_json("envelope_keys.json");
    env["commands"][command]["keys"]
        .as_array()
        .unwrap()
        .iter()
        .map(|k| k.as_str().unwrap().to_string())
        .collect()
}

fn assert_keys(command: &str, v: &serde_json::Value) {
    for key in required_keys(command) {
        assert!(v.get(&key).is_some(), "{command} 봉투에 {key} 없음: {v}");
    }
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
}

#[test]
fn info_envelope_matches_fixture_keys() {
    let p = sample(SMALL);
    let v = run_ok_json(&["info", p.to_str().unwrap(), "--json"]);
    assert_keys("info", &v);
    assert!(v["pageCount"].as_u64().unwrap() >= 1);
}

#[test]
fn explain_envelope_matches_fixture_keys_on_fields_and_tables() {
    for rel in [FIELDS, TABLE, SMALL] {
        let p = sample(rel);
        let v = run_ok_json(&["explain", p.to_str().unwrap(), "--json"]);
        assert_keys("explain", &v);
        assert!(v["summary"].as_str().unwrap().chars().count() > 0);
        assert!(v["paragraphCount"].as_u64().is_some());
    }
}

#[test]
fn export_structure_envelope_matches_fixture_keys() {
    let p = sample(STRUCT_SAMPLE);
    let v = run_ok_json(&["export-structure", p.to_str().unwrap(), "--json"]);
    assert_keys("export-structure", &v);
    assert!(v["nodeCount"].as_u64().is_some());
    assert!(v["structure"].is_object());
}

#[test]
fn digest_envelope_matches_fixture_keys_and_is_one_line() {
    let p = sample(DIGEST_SAMPLE);
    let args = [
        "digest",
        p.to_str().unwrap(),
        "--json",
        "--max-chars",
        "400",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout.lines().filter(|l| !l.trim().is_empty()).count(),
        1,
        "digest 는 한 줄: {}",
        describe(&args, &output)
    );
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_keys("digest", &v);
    assert!(v["excerpt"].is_string());
    assert!(v["nextStep"].is_string());
}

#[test]
fn search_zero_matches_is_success_not_error() {
    let p = sample(SMALL);
    let v = run_ok_json(&[
        "search",
        p.to_str().unwrap(),
        "--json",
        "--limit",
        "5",
        "--",
        "___no_such_token_5296___",
    ]);
    assert_keys("search", &v);
    assert_eq!(v["matchCount"], 0);
    assert_eq!(v["totalMatchCount"], 0);
}

#[test]
fn extract_data_envelope_matches_fixture_keys() {
    let p = sample(DIGEST_SAMPLE);
    let v = run_ok_json(&[
        "extract-data",
        p.to_str().unwrap(),
        "--json",
        "--kind",
        "all",
        "--limit",
        "20",
    ]);
    assert_keys("extract-data", &v);
    assert!(v["items"].is_array());
    assert!(v.get("normalized").is_none() || v["items"].as_array().unwrap().is_empty());
    for item in v["items"].as_array().unwrap() {
        assert!(item.get("normalized").is_some(), "{item}");
        assert!(item.get("raw").is_some(), "{item}");
    }
}

#[test]
fn tiny_document_allows_export_text_json() {
    let p = sample(SMALL);
    let info = run_ok_json(&["info", p.to_str().unwrap(), "--json"]);
    let pages = info["pageCount"].as_u64().unwrap();
    if pages <= 3 {
        let text = run_ok_json(&["export-text", p.to_str().unwrap(), "--json"]);
        assert_eq!(text["schemaVersion"], "1.0");
        assert!(text["pages"].is_array());
    }
}

#[test]
fn usage_errors_stay_silent_on_stdout() {
    let p = sample(SMALL);
    let path = p.to_str().unwrap();
    let cases: [&[&str]; 2] = [
        &["search", path, "--json", "--limit", "0", "x"],
        &["digest", path, "--json", "--max-chars", "0"],
    ];
    for args in cases {
        let output = run(args);
        assert_eq!(output.status.code(), Some(2), "{}", describe(args, &output));
        assert!(output.stdout.is_empty(), "{}", describe(args, &output));
    }
}
