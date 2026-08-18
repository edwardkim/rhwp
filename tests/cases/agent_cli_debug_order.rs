//! [#5316] 레이아웃 디버그 6단 순서 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::PathBuf;

use serde_json::Value;

fn skill_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".claude/skills/rhwp-cli")
}

fn read_text(rel: &str) -> String {
    let path = skill_dir().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn read_json(rel: &str) -> Value {
    serde_json::from_str(&read_text(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

const ORDER: &[&str] = &[
    "export-svg",
    "dump-pages",
    "dump",
    "ir-diff",
    "export-render-tree",
    "hwp5-inventory-diff",
];

#[test]
fn fixture_order_is_six_steps() {
    let doc = read_json("fixtures/debug_order.json");
    let steps = doc["order"].as_array().unwrap();
    assert_eq!(steps.len(), 6);
    for (i, name) in ORDER.iter().enumerate() {
        assert_eq!(steps[i]["step"], i + 1);
        assert_eq!(steps[i]["command"], *name);
    }
    assert_eq!(doc["pageZeroBased"], true);
}

#[test]
fn skill_and_reference_keep_same_order() {
    let skill = read_text("SKILL.md");
    let refer = read_text("references/17_layout_debug_order.md");
    let idx = read_json("fixtures/skill_index.json");
    let listed: Vec<&str> = idx["debugOrder"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert_eq!(listed, ORDER);

    let start = skill.find("레이아웃·간격·겹침").expect("디버그 절");
    let section = &skill[start..];
    let needles = [
        "export-svg <파일> --debug-overlay",
        "dump-pages <파일> -p N",
        "dump <파일> -s N -p M",
        "ir-diff a.hwpx b.hwp",
        "export-render-tree <파일> -p N",
        "hwp5-inventory-diff oracle.hwp generated.hwp",
    ];
    let mut prev = 0usize;
    for needle in needles {
        let pos = section.find(needle).unwrap_or_else(|| panic!("{needle}"));
        assert!(pos >= prev, "SKILL 순서 뒤집힘: {needle}");
        prev = pos;
    }
    let ref_needles = [
        "1. `export-svg --debug-overlay`",
        "2. `dump-pages`",
        "3. `dump`",
        "4. `ir-diff`",
        "5. `export-render-tree`",
        "6. `hwp5-inventory-diff`",
    ];
    let mut prev_ref = 0usize;
    for needle in ref_needles {
        let pos_r = refer.find(needle).unwrap_or_else(|| panic!("{needle}"));
        assert!(pos_r >= prev_ref, "17장 순서 뒤집힘: {needle}");
        prev_ref = pos_r;
    }
}

#[test]
fn first_step_is_overlay() {
    let skill = read_text("SKILL.md");
    assert!(skill.contains("--debug-overlay"));
    let step1 = &read_json("fixtures/debug_order.json")["order"][0];
    let flags = step1["flags"].as_array().unwrap();
    let joined: Vec<&str> = flags.iter().filter_map(|v| v.as_str()).collect();
    assert!(joined.contains(&"--debug-overlay"));
}

#[test]
fn last_step_is_inventory_diff() {
    let last = &read_json("fixtures/debug_order.json")["order"][5];
    assert_eq!(last["command"], "hwp5-inventory-diff");
    let skill = read_text("SKILL.md");
    assert!(skill.contains("oracle.hwp"));
    assert!(skill.contains("generated.hwp"));
}
