//! [#5296] 트리아지 픽스처 스키마·내부 정합.
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
fn fixtures_share_schema_version() {
    for name in [
        "tree.json",
        "stop_rules.json",
        "envelope_keys.json",
        "journeys.json",
        "handoff.json",
        "pitfalls.json",
        "page_thresholds.json",
        "skill_index.json",
        "sample_routing.json",
        "command_ladder.json",
    ] {
        let v = read_json(name);
        assert_eq!(v["schemaVersion"], "1.0", "{name}");
    }
}

#[test]
fn envelope_keys_cover_ladder_commands() {
    let env = read_json("envelope_keys.json");
    let tree = read_json("tree.json");
    for cmd in tree["ladder"].as_array().unwrap() {
        let name = cmd.as_str().unwrap();
        let keys = env["commands"][name]["keys"].as_array();
        assert!(keys.is_some(), "{name} keys 없음");
        assert!(
            keys.unwrap().iter().any(|k| k == "schemaVersion"),
            "{name} schemaVersion"
        );
    }
}

#[test]
fn routing_bands_are_contiguous_from_page_one() {
    let routing = read_json("sample_routing.json");
    let rows = routing["routing"].as_array().unwrap();
    assert!(rows.len() >= 40, "쪽수 라우팅 표본이 너무 적다");
    let mut seen_tiny = false;
    let mut seen_huge = false;
    for row in rows {
        let pages = row["pageCount"].as_u64().unwrap();
        let band = row["band"].as_str().unwrap();
        if pages <= 3 {
            assert_eq!(band, "tiny", "{row}");
            seen_tiny = true;
        } else if pages >= 101 {
            assert_eq!(band, "huge", "{row}");
            seen_huge = true;
            let forbid = row["forbid"].as_array().unwrap();
            assert!(
                forbid
                    .iter()
                    .any(|f| f.as_str() == Some("export-text-unlimited")),
                "{row}"
            );
        }
        if pages >= 9 {
            assert_eq!(row["mustAnnounceTruncation"], true, "{row}");
        }
    }
    assert!(seen_tiny && seen_huge);
}

#[test]
fn journeys_point_at_known_stop_ids_or_bands() {
    let journeys = read_json("journeys.json");
    let stops = read_json("stop_rules.json");
    let mut ids = std::collections::HashSet::new();
    for r in stops["rules"].as_array().unwrap() {
        ids.insert(r["id"].as_str().unwrap().to_string());
    }
    for j in journeys["journeys"].as_array().unwrap() {
        let stop = j["stop"].as_str().unwrap();
        if stop.starts_with('S') {
            assert!(ids.contains(stop), "여정 정지 {stop} 미정의");
        }
        assert!(!j["steps"].as_array().unwrap().is_empty());
    }
}

#[test]
fn tree_fixture_declares_not_gym_and_no_new_cli() {
    let tree = read_json("tree.json");
    assert_eq!(tree["notGym"], true);
    assert_eq!(tree["noNewCli"], true);
    assert_eq!(tree["issue"], 5296);
}
