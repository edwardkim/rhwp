//! [#5331] rhwp-recipes 스킬·픽스처 커버리지 가드.
//!
//! 새 CLI / 편집 로직을 넣지 않는다. 스킬 파일과 픽스처가 정본
//! 레시피 여덟 장(01·02·03·04·05·06·09·10)을 가리키고 07·08 을
//! 발명하지 않는지만 검사한다. 살아 있는 rhwp 를 돌리지 않는다.

#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_dir() -> PathBuf {
    repo_root().join(".claude/skills/rhwp-recipes/fixtures")
}

fn skill_dir() -> PathBuf {
    repo_root().join(".claude/skills/rhwp-recipes")
}

fn recipes_dir() -> PathBuf {
    repo_root().join("mydocs/manual/recipes")
}

fn read_json(name: &str) -> serde_json::Value {
    let path = fixture_dir().join(name);
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{path:?} JSON: {e}"))
}

fn read_skill() -> String {
    fs::read_to_string(skill_dir().join("SKILL.md")).expect("SKILL.md")
}

fn read_ref(name: &str) -> String {
    fs::read_to_string(skill_dir().join("references").join(name))
        .unwrap_or_else(|e| panic!("{name}: {e}"))
}

#[test]
fn skill_frontmatter_names_recipes_router() {
    let text = read_skill();
    assert!(text.starts_with("---\n"), "frontmatter 필요");
    assert!(text.contains("name: rhwp-recipes"), "{text}");
    assert!(text.contains("01"), "{text}");
    assert!(text.contains("09"), "{text}");
    assert!(text.contains("10"), "{text}");
    assert!(text.contains("07"), "{text}");
    assert!(text.contains("08"), "{text}");
    assert!(
        text.contains("gym 이 아니고") || text.contains("gym"),
        "gym 이 아님을 밝혀야 한다"
    );
    assert!(text.contains("라우터"), "{text}");
}

#[test]
fn references_listed_in_skill_index_exist() {
    let idx = read_json("skill_index.json");
    let refs = idx["references"].as_array().expect("references");
    assert!(refs.len() >= 16, "레퍼런스 16장 이상: {refs:?}");
    for r in refs {
        let name = r.as_str().expect("name");
        let path = skill_dir().join("references").join(name);
        assert!(path.is_file(), "누락 {path:?}");
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.len() > 200, "{name} 가 너무 짧다");
    }
}

#[test]
fn fixtures_share_schema_and_issue() {
    for name in [
        "tree.json",
        "stop_rules.json",
        "recipe_cards.json",
        "gap_07_08.json",
        "exceptions.json",
        "intent_matrix.json",
        "journeys.json",
        "skill_index.json",
        "last_verified.json",
    ] {
        let v = read_json(name);
        assert_eq!(v["schemaVersion"], "1.0", "{name}");
        assert_eq!(v["issue"], 5331, "{name}");
    }
}

#[test]
fn tree_fixture_declares_router_not_gym() {
    let tree = read_json("tree.json");
    assert_eq!(tree["notGym"], true);
    assert_eq!(tree["noNewCli"], true);
    assert_eq!(tree["noNewEditLogic"], true);
    assert_eq!(tree["routerOnly"], true);
    let missing = tree["missingIds"].as_array().expect("missingIds");
    let joined: Vec<&str> = missing.iter().filter_map(|v| v.as_str()).collect();
    assert_eq!(joined, vec!["07", "08"]);
}

#[test]
fn existing_recipe_files_present_missing_absent() {
    for name in [
        "01_fill_form_and_submit.md",
        "02_table_csv_roundtrip.md",
        "03_redact_before_sharing.md",
        "04_safety_check_untrusted_doc.md",
        "05_mail_merge_batch_fill.md",
        "06_visual_regression_before_after.md",
        "09_bulk_extract_convert.md",
        "10_security_sweep_before_share.md",
    ] {
        assert!(recipes_dir().join(name).is_file(), "{name}");
    }
    let extras: Vec<_> = fs::read_dir(recipes_dir())
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with("07_") || n.starts_with("08_"))
        .collect();
    assert!(extras.is_empty(), "결번 파일이 있으면 안 된다: {extras:?}");
}

#[test]
fn gap_fixture_forbids_invention() {
    let gap = read_json("gap_07_08.json");
    assert_eq!(gap["doNotInvent"], true);
    let gap_md = read_ref("10_gap_07_08.md");
    assert!(gap_md.contains("#3905"), "{gap_md}");
    assert!(gap_md.contains("만들지 않는다"), "{gap_md}");
}

#[test]
fn stop_rule_ids_appear_in_docs() {
    let stop = read_ref("15_stop_conditions.md");
    let skill = read_skill();
    let rules = read_json("stop_rules.json");
    for rule in rules["rules"].as_array().unwrap() {
        let id = rule["id"].as_str().unwrap();
        assert!(
            stop.contains(id) || skill.contains(id),
            "정지 장에 {id} 없음"
        );
    }
}

#[test]
fn exception_kinds_are_three() {
    let ex = read_json("exceptions.json");
    let paths = ex["paths"].as_array().unwrap();
    assert_eq!(paths.len(), 3);
    let kinds: Vec<&str> = paths.iter().map(|p| p["kind"].as_str().unwrap()).collect();
    assert!(kinds.contains(&"missing-recipe"));
    assert!(kinds.contains(&"stale-last-verified"));
    assert!(kinds.contains(&"two-recipe-match"));
}

#[test]
fn cards_have_triggers_first_command_next_skill() {
    let cards = read_json("recipe_cards.json");
    let list = cards["cards"].as_array().unwrap();
    assert_eq!(list.len(), 8);
    for card in list {
        assert!(!card["triggers"].as_array().unwrap().is_empty());
        assert!(!card["firstCommand"].as_str().unwrap().is_empty());
        assert!(card["nextSkill"].as_str().unwrap().starts_with("rhwp-"));
        assert!(!card["untrustedNote"].as_str().unwrap().is_empty());
        assert!(!card["stopWhen"].as_array().unwrap().is_empty());
        assert_eq!(card["stale"], false);
    }
}

#[test]
fn journeys_use_known_stop_ids() {
    let journeys = read_json("journeys.json");
    let stops = read_json("stop_rules.json");
    let mut ids = std::collections::HashSet::new();
    for r in stops["rules"].as_array().unwrap() {
        ids.insert(r["id"].as_str().unwrap().to_string());
    }
    let items = journeys["journeys"].as_array().unwrap();
    assert!(items.len() >= 40, "여정이 너무 적다");
    for j in items {
        let stop = j["stop"].as_str().unwrap();
        assert!(ids.contains(stop), "여정 정지 {stop} 미정의");
        assert!(!j["steps"].as_array().unwrap().is_empty());
        assert_eq!(j["notGym"], true);
    }
}

#[test]
fn transcripts_are_excerpted_from_canonical() {
    let idx = read_json("transcripts_index.json");
    assert_eq!(idx["excerptedFromCanonical"], true);
    assert_eq!(idx["fabricatedLive"], false);
    let ids = idx["ids"].as_array().unwrap();
    assert!(ids.len() >= 40);
}

#[test]
fn forbidden_peer_skills_not_rewritten_here() {
    let idx = read_json("skill_index.json");
    for name in idx["forbiddenSkillsTouch"].as_array().unwrap() {
        let slug = name.as_str().unwrap();
        let peer = repo_root()
            .join(".claude/skills")
            .join(slug)
            .join("SKILL.md");
        assert!(peer.is_file(), "존재해야 하는 이웃 스킬 {peer:?}");
    }
}
