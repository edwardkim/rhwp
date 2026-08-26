//! [#5316] HWPX→HWP 저장 계약과 자기 왕복 ≠ 한컴.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::PathBuf;

use serde_json::Value;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn skill_dir() -> PathBuf {
    repo().join(".claude/skills/rhwp-cli")
}

fn read_text(rel: &str) -> String {
    let path = skill_dir().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn read_json(rel: &str) -> Value {
    serde_json::from_str(&read_text(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

#[test]
fn oracle_precedes_generated() {
    let fam = read_json("fixtures/hwp5_family.json");
    let order: Vec<&str> = fam["argumentOrder"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert_eq!(order, ["oracle", "generated"]);
    let skill = read_text("SKILL.md");
    let oracle_at = skill.find("oracle.hwp").expect("oracle.hwp");
    let generated_at = skill.find("generated.hwp").expect("generated.hwp");
    assert!(oracle_at < generated_at, "인자 순서가 뒤집힘");
}

#[test]
fn hwp5_family_names_are_real() {
    let fam = read_json("fixtures/hwp5_family.json");
    let names: Vec<&str> = fam["commands"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|c| c["id"].as_str())
        .collect();
    for must in [
        "hwp5-inventory",
        "hwp5-inventory-diff",
        "hwp5-table-probe",
        "hwp5-anchor-trace",
        "hwp5-char-shape-audit",
        "hwp5-roundtrip",
    ] {
        assert!(names.contains(&must), "{must}");
    }
    assert!(names.iter().all(|n| n.starts_with("hwp5-")), "{names:?}");
}

#[test]
fn self_roundtrip_is_not_hangul() {
    let idx = read_json("fixtures/skill_index.json");
    assert_eq!(idx["selfRoundTripIsNotHangul"], true);
    let text = read_text("SKILL.md") + &read_text("references/19_roundtrip_vs_hangul.md");
    assert!(text.contains("한컴"));
    assert!(text.contains("round-trip") || text.contains("라운드트립") || text.contains("왕복"));
    assert!(text.contains("자기") || text.contains("자기 직렬화") || text.contains("자기 round"));
}

#[test]
fn working_doc_forbids_new_cli_and_gym() {
    let text = fs::read_to_string(repo().join("mydocs/working/agent_cli.md")).unwrap();
    assert!(text.contains("새 CLI") || text.contains("새 rhwp CLI"));
    assert!(text.contains("gym"));
    assert!(text.contains("DocumentCore"));
    assert!(text.contains("5316"));
}

#[test]
fn no_new_cli_names_in_surface() {
    let surface = read_text("references/26_cli_surface.md");
    assert!(!surface.contains("layout-debug"));
    assert!(!surface.contains("export-layout"));
    assert!(surface.contains("새 rhwp CLI"));
}
