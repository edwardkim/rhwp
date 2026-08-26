//! [#5316] 요청→명령 매핑이 기존 CLI 이름만 쓰는지.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use serde_json::Value;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn skill_dir() -> PathBuf {
    repo().join(".claude/skills/rhwp-cli")
}

fn read_json(rel: &str) -> Value {
    let path = skill_dir().join(rel);
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

fn read_skill() -> String {
    fs::read_to_string(skill_dir().join("SKILL.md")).expect("SKILL.md")
}

const CORE: &[&str] = &[
    "export-svg",
    "export-png",
    "export-pdf",
    "export-text",
    "export-markdown",
    "dump-pages",
    "dump",
    "dump-records",
    "diag",
    "info",
    "export-render-tree",
    "ir-diff",
    "thumbnail",
    "convert",
    "hwp5-inventory-diff",
];

#[test]
fn command_map_lists_core_surface() {
    let map = read_json("fixtures/command_map.json");
    assert_eq!(map["skill"], "rhwp-cli");
    let ids: BTreeSet<&str> = map["commands"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|c| c["id"].as_str())
        .collect();
    for name in CORE {
        assert!(ids.contains(name), "command_map 에 {name} 없음");
        assert!(read_skill().contains(name), "SKILL 에 {name} 없음");
    }
}

#[test]
fn mapped_commands_have_request_phrases() {
    let map = read_json("fixtures/command_map.json");
    for c in map["commands"].as_array().unwrap() {
        let id = c["id"].as_str().unwrap();
        let req = c["request"].as_array().unwrap();
        assert!(!req.is_empty(), "{id} request 비어 있음");
        assert!(c["argv"].as_str().unwrap().contains(id), "{id} argv");
    }
}

#[test]
fn intents_only_use_known_commands() {
    let map = read_json("fixtures/command_map.json");
    let mut known: BTreeSet<String> = map["commands"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|c| c["id"].as_str().map(str::to_string))
        .collect();
    let hwp5 = read_json("fixtures/hwp5_family.json");
    for c in hwp5["commands"].as_array().unwrap() {
        known.insert(c["id"].as_str().unwrap().to_string());
    }
    let intents = read_json("fixtures/intents.json");
    for it in intents["intents"].as_array().unwrap() {
        let cmd = it["command"].as_str().unwrap();
        assert!(known.contains(cmd), "발명된 명령: {cmd}");
    }
}

#[test]
fn scenario_catalog_never_invents_cli() {
    let cat = read_json("fixtures/scenario_catalog.json");
    assert_eq!(cat["issue"], 5316);
    assert!(cat["count"].as_u64().unwrap() >= 100);
    for s in cat["scenarios"].as_array().unwrap() {
        assert_eq!(s["newCli"], false, "{}", s["id"]);
        assert_eq!(s["gym"], false, "{}", s["id"]);
        assert_eq!(s["selfRoundTripIsHangul"], false, "{}", s["id"]);
    }
}

#[test]
fn index_commands_match_map() {
    let idx = read_json("fixtures/skill_index.json");
    let map = read_json("fixtures/command_map.json");
    let from_idx: Vec<&str> = idx["commands"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    let from_map: Vec<&str> = map["commands"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|c| c["id"].as_str())
        .collect();
    assert_eq!(from_idx, from_map);
}
