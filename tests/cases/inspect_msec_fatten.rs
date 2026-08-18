//! [#5476] inspect 3축 계약 픽스처 고도화 (M-sec).
//!
//! `tests/fixtures/inspect_msec/` 의 봉투·예외가 devel kind 집합과
//! stdout-empty 실패 규약을 지키는지 본다. 판정 코어를 부르지 않는다.
//! 새 탐지 규칙을 발명하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

const FIXTURE_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/inspect_msec");

fn fixture(rel: &str) -> PathBuf {
    Path::new(FIXTURE_ROOT).join(rel)
}

fn read_json(rel: &str) -> serde_json::Value {
    let raw = fs::read_to_string(fixture(rel))
        .unwrap_or_else(|e| panic!("{} 읽기 실패: {e}", fixture(rel).display()));
    serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("{} JSON 파싱 실패: {e}", fixture(rel).display()))
}

fn allowed_hidden() -> BTreeSet<&'static str> {
    [
        "same_as_background",
        "near_invisible",
        "zero_size",
        "off_page",
    ]
    .into_iter()
    .collect()
}

fn allowed_injection() -> BTreeSet<&'static str> {
    [
        "role_impersonation",
        "instruction_override",
        "tool_directive",
        "authority_claim",
        "exfiltration_hint",
        "delimiter_break",
    ]
    .into_iter()
    .collect()
}

fn allowed_unicode() -> BTreeSet<&'static str> {
    ["zero_width", "bidi_override", "tag_char", "confusable"]
        .into_iter()
        .collect()
}

#[test]
fn catalog_lists_only_devel_axes_and_kinds() {
    let cat = read_json("catalog.json");
    assert_eq!(cat["issue"], 5476);
    assert_eq!(cat["inventedRule"], false);
    let axes: Vec<&str> = cat["axes"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert_eq!(axes, ["hidden-text", "injection", "unicode"]);
    let ht: BTreeSet<&str> = cat["kinds"]["hidden-text"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert_eq!(ht, allowed_hidden());
    let inj: BTreeSet<&str> = cat["kinds"]["injection"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert_eq!(inj, allowed_injection());
    let uni: BTreeSet<&str> = cat["kinds"]["unicode"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert_eq!(uni, allowed_unicode());
}

fn load_index() -> Vec<(String, String)> {
    let text = fs::read_to_string(fixture("matrices/catalog.tsv")).expect("catalog.tsv");
    let mut rows = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if i == 0 || line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        assert!(cols.len() >= 5, "catalog.tsv 열 부족: {line}");
        rows.push((cols[0].to_string(), cols[4].to_string()));
    }
    rows
}

#[test]
fn every_case_file_matches_catalog_and_source_rule() {
    let mut seen = HashMap::new();
    for (id, path) in load_index() {
        let rec = read_json(&path);
        assert_eq!(rec["id"], id, "{path}");
        assert_eq!(rec["inventedRule"], false, "{id}");
        assert_eq!(rec["issue"], 5476, "{id}");
        assert!(
            rec["sourceRule"]["file"].as_str().unwrap().ends_with(".rs"),
            "{id} sourceRule.file"
        );
        seen.insert(id, rec);
    }
    assert!(seen.len() >= 200, "픽스처가 너무 적다: {}", seen.len());
}

#[test]
fn hidden_text_envelopes_keep_exit_zero_and_known_kinds() {
    let cat = read_json("catalog.json");
    let allow = allowed_hidden();
    let keys = cat["requiredEnvelopeKeys"]["hidden-text"]
        .as_array()
        .unwrap();
    let mut n = 0usize;
    for (_id, path) in load_index() {
        let rec = read_json(&path);
        if rec["axis"] != "hidden-text" || rec["polarity"] == "exception" {
            continue;
        }
        n += 1;
        assert_eq!(rec["cli"]["exitCode"], 0, "{}", rec["id"]);
        let env = &rec["envelope"];
        for key in keys {
            assert!(
                env.get(key.as_str().unwrap()).is_some(),
                "{} {key}",
                rec["id"]
            );
        }
        for hit in env["hiddenText"].as_array().unwrap() {
            let kind = hit["kind"].as_str().unwrap();
            assert!(
                allow.contains(kind),
                "invented kind {kind} in {}",
                rec["id"]
            );
        }
        if env["clean"] == true {
            assert_eq!(env["hiddenText"].as_array().unwrap().len(), 0);
            assert_eq!(env["hiddenCharCount"], 0);
        }
    }
    assert!(n >= 40, "hidden-text 성공 봉투가 적다: {n}");
}

#[test]
fn injection_envelopes_mark_matched_as_data() {
    let allow = allowed_injection();
    let mut n = 0usize;
    for (_id, path) in load_index() {
        let rec = read_json(&path);
        if rec["axis"] != "injection" || rec.get("envelope").is_none() {
            continue;
        }
        n += 1;
        assert_eq!(rec["cli"]["exitCode"], 0, "{}", rec["id"]);
        for sig in rec["envelope"]["injectionSignals"].as_array().unwrap() {
            let kind = sig["kind"].as_str().unwrap();
            assert!(
                allow.contains(kind),
                "invented kind {kind} in {}",
                rec["id"]
            );
        }
        if rec["consume"]["branch"] == "minConfidence" {
            let kept: Vec<_> = rec["consume"]["kept"]
                .as_array()
                .unwrap()
                .iter()
                .map(|kind| kind.as_str().unwrap())
                .collect();
            let observed: Vec<_> = rec["envelope"]["injectionSignals"]
                .as_array()
                .unwrap()
                .iter()
                .map(|signal| signal["kind"].as_str().unwrap())
                .collect();
            assert_eq!(kept, observed, "{}", rec["id"]);
        } else if rec["envelope"]["clean"] == false {
            assert_eq!(rec["consume"]["matchedIs"], "DATA", "{}", rec["id"]);
            assert_eq!(rec["consume"]["doNotExecuteMatched"], true, "{}", rec["id"]);
        }
    }
    assert!(n >= 80, "injection 성공 봉투가 적다: {n}");
}

#[test]
fn unicode_envelopes_keep_rendered_and_raw() {
    let allow = allowed_unicode();
    let mut n = 0usize;
    for (_id, path) in load_index() {
        let rec = read_json(&path);
        if rec["axis"] != "unicode" || rec.get("envelope").is_none() {
            continue;
        }
        n += 1;
        let env = &rec["envelope"];
        assert!(env["kindCounts"].is_object(), "{}", rec["id"]);
        for k in allow.iter() {
            assert!(env["kindCounts"][k].is_number(), "{} {k}", rec["id"]);
        }
        for hit in env["findings"].as_array().unwrap() {
            let kind = hit["kind"].as_str().unwrap();
            assert!(
                allow.contains(kind),
                "invented kind {kind} in {}",
                rec["id"]
            );
            assert!(hit["rendered"].is_string(), "{}", rec["id"]);
            assert!(hit["raw"].is_string(), "{}", rec["id"]);
        }
        if env["clean"] == true {
            assert_eq!(env["findingCount"], 0);
            assert_eq!(env["findings"].as_array().unwrap().len(), 0);
        }
    }
    assert!(n >= 40, "unicode 성공 봉투가 적다: {n}");
}

#[test]
fn exception_envelopes_keep_stdout_empty() {
    let mut n = 0usize;
    for (_id, path) in load_index() {
        let rec = read_json(&path);
        if rec["family"] != "exception" {
            continue;
        }
        n += 1;
        assert!(
            rec.get("envelope").is_none() || rec["envelope"].is_null(),
            "{}",
            rec["id"]
        );
        assert_eq!(rec["cli"]["stdoutBytes"], 0, "{}", rec["id"]);
        let code = rec["cli"]["exitCode"].as_i64().unwrap();
        assert!(code == 1 || code == 2, "{} exit {code}", rec["id"]);
        assert!(
            rec["cli"]["stderrContains"].as_str().unwrap().len() > 3,
            "{}",
            rec["id"]
        );
    }
    assert!(n >= 20, "예외 봉투가 적다: {n}");
}

#[test]
fn resweep_matrix_requires_clean_true() {
    let text = fs::read_to_string(fixture("matrices/resweep_gate.tsv")).expect("resweep_gate.tsv");
    assert!(text.contains("hidden-text\tclean\tTrue"));
    assert!(text.contains("injection\tclean\tTrue"));
    assert!(text.contains("unicode\tclean\tTrue"));
    assert!(text.contains("exitOnDetection"));
}
