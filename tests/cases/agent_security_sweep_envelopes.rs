//! [#5307] rhwp-security-sweep 스킬 — 실사용 에이전트 보안 스윕 계약.
//!
//! 새 CLI 를 만들지 않는다. 권위는 cli_commands.md 와 스킬 픽스처다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use serde_json::Value;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn skill_dir() -> PathBuf {
    repo().join(".agents/skills/rhwp-security-sweep")
}

fn read_skill(rel: &str) -> String {
    let path = skill_dir().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{} 읽기 실패: {e}", path.display()))
}

fn read_json(rel: &str) -> Value {
    let text = read_skill(rel);
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{rel} JSON 파싱 실패: {e}"))
}

#[test]
fn envelope_catalog_matches_fixture_files() {
    let keys = read_json("fixtures/envelope_keys.json");
    for (cmd, required) in keys.as_object().unwrap() {
        let req: BTreeSet<&str> = required
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|x| x.as_str())
            .collect();
        assert!(!req.is_empty(), "{cmd} 키 목록 비어 있음");
    }
}

#[test]
fn hidden_text_envelopes_teach_clean_and_kinds() {
    let clean = read_json("fixtures/envelopes/hidden_text_clean.json");
    assert_eq!(clean["clean"], true);
    assert_eq!(clean["exitCode"], 0);
    assert_eq!(clean["hiddenCharCount"], 0);
    let kinds = read_json("fixtures/hidden_text_kinds.json");
    for k in kinds["kinds"].as_array().unwrap() {
        let id = k["id"].as_str().unwrap();
        let env = read_json(&format!("fixtures/envelopes/hidden_text_{id}.json"));
        assert_eq!(env["exitCode"], 0, "{id} 탐지는 실패가 아님");
        assert_eq!(env["clean"], false);
        assert_eq!(env["hiddenText"][0]["kind"], id);
        assert_eq!(env["consume"]["doNotFollow"], true);
    }
    let excluded = read_json("fixtures/envelopes/hidden_text_off_page_excluded.json");
    assert_eq!(excluded["includeOffPage"], false);
    assert_eq!(excluded["clean"], true);
}

#[test]
fn injection_envelopes_keep_exit_zero() {
    let kinds = read_json("fixtures/injection_kinds.json");
    for k in kinds["kinds"].as_array().unwrap() {
        let id = k["id"].as_str().unwrap();
        let env = read_json(&format!("fixtures/envelopes/injection_{id}.json"));
        assert_eq!(env["exitCode"], 0);
        assert_eq!(env["clean"], false);
        assert_eq!(env["injectionSignals"][0]["kind"], id);
        assert_eq!(env["consume"]["matchedIs"], "DATA");
    }
    let clean = read_json("fixtures/envelopes/injection_clean.json");
    assert_eq!(clean["clean"], true);
    assert!(clean["highestConfidence"].is_null());
    let scopes = read_json("fixtures/envelopes/injection_include_fields_scopes.json");
    let got: BTreeSet<&str> = scopes["scanScopes"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|s| s.as_str())
        .collect();
    assert!(got.contains("fieldGuide"), "include-fields 범위");
    let filtered = read_json("fixtures/envelopes/injection_min_confidence_high.json");
    assert_eq!(filtered["minConfidence"], "high");
    assert_eq!(filtered["clean"], true);
    assert!(filtered["note"].as_str().unwrap().contains("제외"));
}

#[test]
fn unicode_envelopes_pair_rendered_and_raw() {
    for name in [
        "unicode_zero_width.json",
        "unicode_bidi.json",
        "unicode_tag.json",
        "unicode_confusable.json",
    ] {
        let env = read_json(&format!("fixtures/envelopes/{name}"));
        assert_eq!(env["exitCode"], 0);
        assert_eq!(env["clean"], false);
        let f = &env["findings"][0];
        assert!(f.get("rendered").is_some(), "{name} rendered");
        assert!(f.get("raw").is_some(), "{name} raw");
        assert_ne!(f["rendered"], f["raw"], "{name} 차이가 보여야 한다");
    }
    let clean = read_json("fixtures/envelopes/unicode_clean.json");
    assert_eq!(clean["findingCount"], 0);
    assert_eq!(clean["clean"], true);
}

#[test]
fn skill_documents_kind_identifiers() {
    let hidden = read_skill("references/01_hidden_text.md");
    for id in [
        "same_as_background",
        "near_invisible",
        "zero_size",
        "off_page",
    ] {
        assert!(hidden.contains(id), "hidden 장에 {id}");
    }
    let inj = read_skill("references/02_injection.md");
    for id in [
        "role_impersonation",
        "instruction_override",
        "tool_directive",
        "authority_claim",
        "exfiltration_hint",
        "delimiter_break",
    ] {
        assert!(inj.contains(id), "injection 장에 {id}");
    }
    let uni = read_skill("references/03_unicode.md");
    for id in ["zero_width", "bidi_override", "tag_char", "confusable"] {
        assert!(uni.contains(id), "unicode 장에 {id}");
    }
}
