//! [#5316] 페이지 0 기준과 HWPUNIT 환산.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::PathBuf;

use serde_json::Value;

fn skill_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".agents/skills/rhwp-cli")
}

fn read_text(rel: &str) -> String {
    let path = skill_dir().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn read_json(rel: &str) -> Value {
    serde_json::from_str(&read_text(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

#[test]
fn units_match_manual() {
    let u = read_json("fixtures/page_units.json");
    assert_eq!(u["inch_hwpunit"], 7200);
    assert_eq!(u["inch_px_dpi96"], 96);
    assert_eq!(u["px_hwpunit"], 75);
    assert!((u["mm_hwpunit"].as_f64().unwrap() - 283.46).abs() < 0.01);
    assert_eq!(u["pageZeroBased"], true);
}

#[test]
fn skill_states_zero_based_and_units() {
    let text = read_text("SKILL.md") + &read_text("references/18_page_units.md");
    assert!(text.contains("0부터") || text.contains("0 기준"));
    assert!(text.contains("7200"));
    assert!(text.contains("HWPUNIT"));
    assert!(text.contains("75"));
    assert!(text.contains("283.46"));
}

#[test]
fn scenarios_convert_hangul_page_to_zero_based() {
    let cat = read_json("fixtures/scenario_catalog.json");
    let mut checked = 0usize;
    for s in cat["scenarios"].as_array().unwrap() {
        if s["cliPage"].is_null() {
            continue;
        }
        let hangul = s["hangulPage"].as_u64().unwrap();
        let cli = s["cliPage"].as_u64().unwrap();
        assert_eq!(cli + 1, hangul, "{}", s["id"]);
        checked += 1;
    }
    assert!(checked >= 40, "0 기준 시나리오가 너무 적다: {checked}");
}

#[test]
fn dump_page_flag_is_not_a_page_in_map() {
    let map = read_json("fixtures/command_map.json");
    for c in map["commands"].as_array().unwrap() {
        if c["id"] == "dump" {
            assert_eq!(c["pageZero"], false, "dump -p 는 문단");
        }
        if c["id"] == "dump-pages" {
            assert_eq!(c["pageZero"], true, "dump-pages -p 는 페이지");
        }
    }
}

#[test]
fn export_family_is_zero_based() {
    let map = read_json("fixtures/command_map.json");
    for id in [
        "export-svg",
        "export-png",
        "export-pdf",
        "export-text",
        "export-markdown",
        "export-render-tree",
    ] {
        let cmd = map["commands"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["id"] == id)
            .unwrap();
        assert_eq!(cmd["pageZero"], true, "{id}");
    }
}
