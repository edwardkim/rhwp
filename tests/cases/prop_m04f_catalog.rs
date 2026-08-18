//! [#5465] M04-f: 왕복 카탈로그 구조·수량·금지 범위.
//!
//! `tests/fixtures/proptest_m04f/` 가 기존 run step 4종만 담고,
//! DocumentCore mutation 을 발명하지 않았는지 확인한다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/proptest_m04f");
const ACTIONS: [&str; 4] = ["fill_fields", "replace_text", "set_cell", "set_checkbox"];
const FORBIDDEN: [&str; 10] = [
    "insert_text",
    "delete_text",
    "merge_cells",
    "split_cell",
    "insert_table",
    "delete_row",
    "apply_style",
    "mutate_core",
    "set_field",
    "insert_paragraph",
];

fn fixture(rel: &str) -> PathBuf {
    Path::new(ROOT).join(rel)
}

fn read_text(rel: &str) -> String {
    fs::read_to_string(fixture(rel)).unwrap_or_else(|e| panic!("{}: {e}", fixture(rel).display()))
}

fn read_json(rel: &str) -> serde_json::Value {
    serde_json::from_str(&read_text(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

fn read_jsonl(rel: &str) -> Vec<serde_json::Value> {
    read_text(rel)
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_str(line).unwrap_or_else(|e| panic!("{rel}: {e}: {line}")))
        .collect()
}

#[test]
fn catalog_readme_and_schema_exist() {
    for rel in [
        "README.md",
        "schema/catalog.v1.json",
        "reports/fatten_summary.json",
        "reports/fatten_summary.md",
        "reports/skip_honesty.md",
        "reports/ci.md",
        "catalogs/fixtures.json",
        "catalogs/skip_reasons.json",
        "matrices/fixture_x_step.tsv",
    ] {
        assert!(fixture(rel).is_file(), "{rel}");
    }
}

#[test]
fn summary_counts_meet_fatten_floor() {
    let summary = read_json("reports/fatten_summary.json");
    assert_eq!(summary["issue"], 5465);
    assert_eq!(summary["seat"], "M04-f");
    let counts = &summary["counts"];
    assert!(counts["validPlans"].as_u64().unwrap() >= 800);
    assert!(counts["invalidPlans"].as_u64().unwrap() >= 80);
    assert!(counts["skipCatalog"].as_u64().unwrap() >= 200);
    assert!(counts["exceptions"].as_u64().unwrap() >= 200);
    assert!(counts["mutations"].as_u64().unwrap() >= 40);
    assert_eq!(counts["fixtures"], 6);
}

#[test]
fn actions_are_exactly_the_existing_run_steps() {
    let summary = read_json("reports/fatten_summary.json");
    let actions: Vec<String> = summary["actions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert_eq!(actions, ACTIONS);
    let schema = read_json("schema/catalog.v1.json");
    assert_eq!(schema["version"], "m04f.v1");
}

#[test]
fn out_of_scope_lists_other_seats() {
    let summary = read_json("reports/fatten_summary.json");
    let out: BTreeSet<String> = summary["outOfScope"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    for seat in [
        "DocumentCore 새 mutation API",
        "canvaskit_policy",
        "pdf renderer",
        "page-count serializer",
        "layout-anomaly",
        "oracle_public",
        "render_backend",
        "gym",
    ] {
        assert!(out.iter().any(|s| s.contains(seat) || s == seat), "{seat}");
    }
}

#[test]
fn fixtures_claim_only_established_capabilities() {
    let fixtures = read_json("catalogs/fixtures.json");
    let arr = fixtures.as_array().unwrap();
    assert_eq!(arr.len(), 6);
    let mut seen = BTreeSet::new();
    for fx in arr {
        let id = fx["id"].as_str().unwrap();
        assert!(seen.insert(id.to_string()), "중복 fixture {id}");
        assert!(fx["path"].as_str().unwrap().starts_with("samples/"));
        let apply = fx["applyable"].as_object().unwrap();
        assert_eq!(apply.len(), 4);
        for action in ACTIONS {
            assert!(apply.contains_key(action), "{id} {action}");
        }
        if !fx["claimed"].as_bool().unwrap() {
            for action in ACTIONS {
                assert!(
                    !apply[action].as_bool().unwrap(),
                    "미주장 픽스처 {id} 가 {action} apply 를 주장함"
                );
            }
        }
    }
    assert!(seen.contains("ref_text_hwpx"));
    assert!(seen.contains("ref_table_hwpx"));
    assert!(seen.contains("para001_hwp5"));
    assert!(seen.contains("table001_hwp5"));
    assert!(seen.contains("ref_empty_hwpx"));
    assert!(seen.contains("ref_mixed_hwpx"));
}

#[test]
fn valid_plans_never_invent_document_core_actions() {
    let rows = read_jsonl("catalogs/valid_plans.jsonl");
    assert!(rows.len() >= 800);
    let mut ids = BTreeSet::new();
    for row in &rows {
        let id = row["id"].as_str().unwrap();
        assert!(ids.insert(id.to_string()), "중복 {id}");
        assert_eq!(row["expected"], "schema_ok");
        let plan = &row["plan"];
        assert_eq!(plan["planVersion"], "1.0");
        let steps = plan["steps"].as_array().unwrap();
        assert!(!steps.is_empty());
        for step in steps {
            let action = step["action"].as_str().unwrap();
            assert!(ACTIONS.contains(&action), "허용되지 않은 action {action}");
            assert!(
                !FORBIDDEN.contains(&action),
                "DocumentCore mutation 발명: {action}"
            );
        }
    }
}

#[test]
fn action_variant_files_match_valid_plan_families() {
    let valid = read_jsonl("catalogs/valid_plans.jsonl");
    for action in ACTIONS {
        let rows = read_jsonl(&format!("cases/{action}_variants.jsonl"));
        assert!(!rows.is_empty(), "{action} variants");
        for row in &rows {
            assert_eq!(row["action"], action);
            assert_eq!(row["plan"]["steps"][0]["action"], action);
        }
        let from_valid = valid.iter().filter(|row| row["action"] == action).count();
        assert_eq!(from_valid, rows.len(), "{action} 슬라이스 길이");
    }
}

#[test]
fn fixture_matrix_covers_every_fixture_action() {
    let text = read_text("matrices/fixture_x_step.tsv");
    let mut lines = text.lines();
    let header = lines.next().unwrap();
    assert!(header.starts_with("fixture\t"));
    let mut pairs = BTreeSet::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        assert!(cols.len() >= 5, "{line}");
        pairs.insert((cols[0].to_string(), cols[3].to_string()));
        assert!(
            cols[4] == "apply_possible" || cols[4] == "skip_only",
            "{}",
            cols[4]
        );
    }
    let fixtures = read_json("catalogs/fixtures.json");
    for fx in fixtures.as_array().unwrap() {
        for action in ACTIONS {
            assert!(
                pairs.contains(&(fx["id"].as_str().unwrap().to_string(), action.to_string())),
                "{} × {action}",
                fx["id"]
            );
        }
    }
}

#[test]
fn ci_doc_does_not_add_fifth_nextest_shard() {
    let ci = read_text("reports/ci.md");
    assert!(ci.contains("run-prop-roundtrip.mjs"));
    assert!(ci.contains("5번째 shard"));
    assert!(ci.contains("prop_m04f_catalog"));
    assert!(!ci.contains("cargo-fuzz"));
}

#[test]
fn jsonl_files_are_unique_objects() {
    for rel in [
        "catalogs/skip_catalog.jsonl",
        "catalogs/invalid_plans.jsonl",
        "catalogs/exception_catalog.jsonl",
        "catalogs/mutation_sequences.jsonl",
        "catalogs/condition_catalog.jsonl",
    ] {
        let rows = read_jsonl(rel);
        assert!(!rows.is_empty(), "{rel}");
        let mut ids = BTreeSet::new();
        for row in &rows {
            let id = row["id"].as_str().unwrap();
            assert!(ids.insert(id.to_string()), "{rel} 중복 {id}");
        }
    }
}
