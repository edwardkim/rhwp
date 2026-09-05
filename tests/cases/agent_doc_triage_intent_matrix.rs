//! [#5296] 발화 행렬·질의 카탈로그·쪽수 표가 스킬 장과 일치한다.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_dir() -> PathBuf {
    repo_root().join("tests/fixtures/agent_doc_triage")
}

fn read_json(name: &str) -> serde_json::Value {
    let path = fixture_dir().join(name);
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{path:?} JSON: {e}"))
}

fn read_ref(name: &str) -> String {
    fs::read_to_string(
        repo_root()
            .join(".agents/skills/rhwp-doc-triage/references")
            .join(name),
    )
    .unwrap_or_else(|e| panic!("{name}: {e}"))
}

#[test]
fn intent_ids_are_unique_and_documented() {
    let intents = read_json("intent_matrix.json")["intents"]
        .as_array()
        .unwrap()
        .clone();
    assert!(intents.len() >= 100, "발화 행렬이 너무 짧다");
    let text = read_ref("16_intent_matrix.md");
    let mut seen = std::collections::HashSet::new();
    for it in &intents {
        let id = it["id"].as_str().unwrap();
        assert!(seen.insert(id.to_string()), "중복 {id}");
        assert!(text.contains(id), "장에 {id} 없음");
        assert!(it["utterance"].as_str().unwrap().chars().count() >= 2);
        assert!(it["first"].as_str().unwrap().chars().count() >= 2);
    }
}

#[test]
fn reject_intents_never_choose_unlimited_export_text() {
    let payload = read_json("intent_matrix.json");
    for it in payload["intents"].as_array().unwrap() {
        let first = it["first"].as_str().unwrap();
        let id = it["id"].as_str().unwrap();
        if first.contains("거절") {
            assert!(
                !first.contains("export-text 무제한"),
                "{id} 거절이 덤프를 시킨다"
            );
        }
    }
}

#[test]
fn query_catalog_has_limits_for_search_like_tools() {
    let payload = read_json("query_catalog.json");
    let qs = payload["queries"].as_array().unwrap();
    assert!(qs.len() >= 40, "질의 카탈로그가 너무 짧다");
    let text = read_ref("17_query_catalog.md");
    for q in qs {
        let needle = q["q"].as_str().unwrap();
        assert!(text.contains(needle), "질의 장에 {needle} 없음");
        let tool = q["tool"].as_str().unwrap();
        if tool.starts_with("search") {
            assert!(q["limit"].as_u64().unwrap() > 0, "{needle} search limit");
        }
    }
}

#[test]
fn pagecount_table_covers_1_to_220() {
    let payload = read_json("pagecount_1_220.json");
    let rows = payload["routing"].as_array().unwrap();
    assert_eq!(rows.len(), 220);
    let text = read_ref("18_pagecount_routing.md");
    assert!(text.contains("| 1 |"));
    assert!(text.contains("| 220 |"));
    assert_eq!(rows[0]["pageCount"], 1);
    assert_eq!(rows[0]["band"], "tiny");
    assert_eq!(rows[219]["pageCount"], 220);
    assert_eq!(rows[219]["band"], "huge");
    for row in rows {
        let p = row["pageCount"].as_u64().unwrap();
        let band = row["band"].as_str().unwrap();
        let expect = if p <= 3 {
            "tiny"
        } else if p <= 8 {
            "small"
        } else if p <= 30 {
            "medium"
        } else if p <= 100 {
            "large"
        } else {
            "huge"
        };
        assert_eq!(band, expect, "page {p}");
    }
}

#[test]
fn field_catalog_mentions_key_envelope_fields() {
    let text = read_ref("20_field_catalog.md");
    for key in [
        "pageCount",
        "paragraphCount",
        "excerpt",
        "nextStep",
        "totalMatchCount",
        "normalized",
        "untrusted",
    ] {
        assert!(text.contains(key), "{key}");
    }
}

#[test]
fn worked_traces_are_not_gym() {
    let text = read_ref("19_worked_traces.md");
    assert!(text.contains("gym 없음") || text.contains("gym"));
    assert!(text.contains("samples/hwp3-sample.hwp"));
    assert!(text.contains("form-fill") || text.contains("fields"));
}
