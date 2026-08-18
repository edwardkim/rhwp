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
    repo().join(".claude/skills/rhwp-security-sweep")
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
fn pii_rules_are_conservative_four_kinds() {
    let rules = read_json("fixtures/pii_rules.json");
    assert_eq!(rules["conservative"], true);
    assert_eq!(rules["falsePositiveIsFailure"], true);
    let kinds: BTreeSet<&str> = rules["kinds"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|k| k["id"].as_str())
        .collect();
    for id in ["ssn", "card", "phone", "email"] {
        assert!(kinds.contains(id), "규칙에 {id}");
    }
    let oos: BTreeSet<&str> = rules["outOfScope"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|s| s.as_str())
        .collect();
    for id in ["account", "passport", "non-02-area-codes", "card-13"] {
        assert!(oos.contains(id), "범위 밖 {id}");
    }
}

#[test]
fn pii_cases_match_documented_recipe_examples() {
    let cases = read_json("fixtures/pii_cases.json");
    let list = cases["cases"].as_array().unwrap();
    assert!(list.len() >= 30, "사례가 너무 적다: {}", list.len());
    let mut detect = 0;
    let mut skip = 0;
    for c in list {
        let id = c["id"].as_str().unwrap();
        assert!(c["kind"].as_str().is_some(), "{id} kind");
        assert!(c["reason"].as_str().unwrap().len() > 4, "{id} reason");
        if c["detect"].as_bool().unwrap() {
            detect += 1;
        } else {
            skip += 1;
        }
    }
    assert!(detect >= 8, "양성 {detect}");
    assert!(skip >= 12, "음성 {skip}");

    let by_id: std::collections::BTreeMap<_, _> = list
        .iter()
        .map(|c| (c["id"].as_str().unwrap(), c))
        .collect();
    assert_eq!(by_id["ssn-pass-recipe3"]["detect"], true);
    assert_eq!(by_id["ssn-bait-recipe3"]["detect"], false);
    assert_eq!(by_id["card-visa-test"]["detect"], true);
    assert_eq!(by_id["card-luhn-fail"]["detect"], false);
    assert_eq!(by_id["phone-010"]["detect"], true);
    assert_eq!(by_id["phone-no-hyphen"]["detect"], false);
    assert_eq!(by_id["phone-031-oos"]["detect"], false);
    assert_eq!(by_id["email-recipe3"]["detect"], true);
    assert_eq!(by_id["email-one-label"]["detect"], false);
}

#[test]
fn skill_and_pii_chapter_agree_on_rules() {
    let skill = read_skill("SKILL.md") + &read_skill("references/05_pii_rules.md");
    for token in [
        "######-#######",
        "mod 11",
        "Luhn",
        "01[016789]",
        "하이픈 필수",
        "라벨 2개",
        "02 외 지역번호",
        "여권번호",
        "계좌번호",
    ] {
        assert!(skill.contains(token), "규칙 문서에 {token}");
    }
}

#[test]
fn tests_do_not_invent_new_detectors() {
    let pii = read_skill("references/05_pii_rules.md");
    assert!(pii.contains("규칙을 넓히지 않는다") || pii.contains("발명하지 않는다"));
    let surface = read_json("fixtures/cli_surface.json");
    assert_eq!(surface["newCommandsForbidden"], true);
    for name in surface["doNotInvent"].as_array().unwrap() {
        let n = name.as_str().unwrap();
        assert!(!n.is_empty(), "금지 목록이 비면 안 된다");
    }
}
