//! [#5465] M04-f: 왕복 예외 카탈로그.
//!
//! 빈 치환·occurrence None/0·keepStyle·필드 순번·조건 거짓은
//! 기존 run step 의 경계이지 새 편집 API 가 아니다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/proptest_m04f");
const SAMPLES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/samples");

fn read_jsonl(rel: &str) -> Vec<serde_json::Value> {
    let path = Path::new(ROOT).join(rel);
    fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_str(line).unwrap_or_else(|e| panic!("{rel}: {e}")))
        .collect()
}

#[test]
fn exception_families_are_existing_step_edges() {
    let rows = read_jsonl("catalogs/exception_catalog.jsonl");
    assert!(rows.len() >= 200);
    let families: BTreeSet<_> = rows
        .iter()
        .map(|row| row["family"].as_str().unwrap())
        .collect();
    for family in [
        "empty_replace_deletion",
        "occurrence_none_vs_zero",
        "case_sensitive_flag",
        "keep_style_flag",
        "field_occurrence_address",
        "field_value_coercion",
        "condition_false_skips_precheck",
        "cell_grid_boundary",
        "needle_presence",
    ] {
        assert!(families.contains(family), "{family}");
    }
}

#[test]
fn empty_replace_is_deletion_not_a_new_api() {
    let rows = read_jsonl("catalogs/exception_catalog.jsonl");
    let mut n = 0usize;
    for row in &rows {
        if row["family"] != "empty_replace_deletion" {
            continue;
        }
        n += 1;
        assert_eq!(row["action"], "replace_text");
        assert_eq!(row["step"]["replace"], "");
        assert_eq!(row["schema"], "ok");
        let apply = row["apply"].as_str().unwrap();
        assert!(apply == "apply" || apply == "skip");
        assert!(row["note"].as_str().unwrap().contains("새 API 아님"));
    }
    assert!(n >= 14, "empty replace {n}");
}

#[test]
fn occurrence_none_and_zero_are_not_collapsed() {
    let rows = read_jsonl("catalogs/exception_catalog.jsonl");
    let mut none = 0;
    let mut zero = 0;
    for row in &rows {
        if row["family"] != "occurrence_none_vs_zero" {
            continue;
        }
        if row["occurrence"].is_null() {
            none += 1;
            assert_eq!(row["engine"], "replace_all_native");
            assert!(row["step"].get("occurrence").is_none());
        } else {
            zero += 1;
            assert_eq!(row["engine"], "replace_nth_native(0)");
            assert_eq!(row["step"]["occurrence"], 0);
        }
    }
    assert!(none >= 4 && zero >= 4, "none={none} zero={zero}");
}

#[test]
fn keep_style_default_is_false() {
    let rows = read_jsonl("catalogs/exception_catalog.jsonl");
    let mut omitted = 0;
    for row in &rows {
        if row["family"] != "keep_style_flag" {
            continue;
        }
        assert_eq!(row["default_when_omitted"], false);
        if row["keepStyle"].is_null() {
            omitted += 1;
            assert!(row["step"].get("keepStyle").is_none());
        }
    }
    assert!(omitted >= 1);
}

#[test]
fn field_values_include_number_and_bool() {
    let rows = read_jsonl("catalogs/exception_catalog.jsonl");
    let mut kinds = BTreeSet::new();
    for row in &rows {
        if row["family"] != "field_value_coercion" {
            continue;
        }
        kinds.insert(row["valueKind"].as_str().unwrap().to_string());
        assert_eq!(row["step"]["action"], "fill_fields");
    }
    assert!(kinds.contains("string"));
    assert!(kinds.contains("number"));
    assert!(kinds.contains("boolean"));
}

#[test]
fn cell_grid_boundary_is_inside_or_cell_missing() {
    let rows = read_jsonl("catalogs/exception_catalog.jsonl");
    let mut inside = 0;
    let mut outside = 0;
    for row in &rows {
        if row["family"] != "cell_grid_boundary" {
            continue;
        }
        match row["apply"].as_str().unwrap() {
            "apply" => {
                inside += 1;
                assert!(row["reason"].is_null());
            }
            "skip" => {
                outside += 1;
                assert_eq!(row["reason"], "cell_missing");
            }
            other => panic!("{other}"),
        }
    }
    assert!(inside >= 6 && outside >= 4, "in={inside} out={outside}");
}

#[test]
fn needle_presence_matches_claimed_fixtures() {
    let rows = read_jsonl("catalogs/exception_catalog.jsonl");
    for row in &rows {
        if row["family"] != "needle_presence" {
            continue;
        }
        let present = row["present"].as_bool().unwrap();
        if present {
            assert_eq!(row["apply"], "apply");
            assert!(row["reason"].is_null());
        } else {
            assert_eq!(row["apply"], "skip");
            assert_eq!(row["reason"], "no_hits");
        }
    }
}

#[test]
fn condition_catalog_rejects_multi_and_unknown_keys() {
    let rows = read_jsonl("catalogs/condition_catalog.jsonl");
    assert!(rows.len() >= 40);
    let mut ok = 0;
    let mut reject = 0;
    for row in &rows {
        match row["schema"].as_str().unwrap() {
            "ok" => {
                ok += 1;
                let cond = row["condition"].as_object().unwrap();
                assert_eq!(cond.len(), 1);
            }
            "reject" => reject += 1,
            other => panic!("{other}"),
        }
    }
    assert!(ok >= 20 && reject >= 10, "ok={ok} reject={reject}");
}

#[test]
fn claimed_needles_exist_in_real_text_fixtures() {
    let pairs = [
        ("hwpx/ref/ref_text.hwpx", ["안녕", "Hello", "123"]),
        ("para-001.hwp", ["오호라", "乾坤", "구궁산"]),
        ("table-001.hwp", ["품질", "5월", "평가"]),
    ];
    for (rel, needles) in pairs {
        let bytes = fs::read(Path::new(SAMPLES).join(rel)).unwrap();
        let core = DocumentCore::from_bytes(&bytes).unwrap();
        for needle in needles {
            let hits = core.grep(needle, true, None);
            assert!(!hits.is_empty(), "{rel} 에서 {needle} 없음");
        }
    }
}
