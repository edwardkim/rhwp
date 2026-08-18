//! [#5342] rhwp-knowledge-map 스킬·픽스처 커버리지 가드.
//!
//! 새 CLI / 편집 로직을 넣지 않는다. 스킬 파일과 픽스처가
//! llms.txt → agent_knowledge_map.md → canonical 하나 순서를
//! 가리키고, 지도 §2 바깥의 필드 이름을 발명하지 않는지만
//! 검사한다. 살아 있는 rhwp 를 돌리지 않는다.

#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_dir() -> PathBuf {
    repo_root().join(".claude/skills/rhwp-knowledge-map/fixtures")
}

fn skill_dir() -> PathBuf {
    repo_root().join(".claude/skills/rhwp-knowledge-map")
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
fn skill_frontmatter_names_knowledge_map() {
    let text = read_skill();
    assert!(text.starts_with("---\n"), "frontmatter 필요");
    assert!(text.contains("name: rhwp-knowledge-map"), "{text}");
    assert!(text.contains("llms.txt"), "{text}");
    assert!(text.contains("agent_knowledge_map.md"), "{text}");
    assert!(
        text.contains("gym 이 아니고") || text.contains("gym"),
        "gym 이 아님을 밝혀야 한다"
    );
    assert!(text.contains("진입점"), "{text}");
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
        "exceptions.json",
        "intent_matrix.json",
        "journeys.json",
        "skill_index.json",
        "last_verified.json",
        "first_read.json",
        "remeasure.json",
        "envelope_fields.json",
        "version_mismatch.json",
    ] {
        let v = read_json(name);
        assert_eq!(v["schemaVersion"], "1.0", "{name}");
        assert_eq!(v["issue"], 5342, "{name}");
    }
}

#[test]
fn tree_fixture_declares_entry_not_gym() {
    let tree = read_json("tree.json");
    assert_eq!(tree["notGym"], true);
    assert_eq!(tree["noNewCli"], true);
    assert_eq!(tree["noNewEditLogic"], true);
    assert_eq!(tree["routerOnly"], true);
    assert_eq!(tree["entryPoint"], true);
    assert_eq!(tree["doNotRenarrateMapRows"], true);
    assert_eq!(tree["canonicalWins"], true);
}

#[test]
fn first_read_is_llms_then_map() {
    let first = read_json("first_read.json");
    let order = first["order"].as_array().expect("order");
    assert_eq!(order[0]["path"], "llms.txt");
    assert_eq!(order[1]["path"], "mydocs/manual/agent_knowledge_map.md");
    assert_eq!(order.len(), 3);
    assert!(repo_root().join("llms.txt").is_file());
    assert!(repo_root()
        .join("mydocs/manual/agent_knowledge_map.md")
        .is_file());
}

#[test]
fn remesure_has_three_commands() {
    let rm = read_json("remeasure.json");
    let cmds = rm["commands"].as_array().expect("commands");
    assert_eq!(cmds.len(), 3);
    assert_eq!(cmds[0]["id"], "RM01");
    assert_eq!(cmds[2]["method"], "tools/list");
}

#[test]
fn stop_rule_ids_appear_in_docs() {
    let stop = read_ref("13_stop_conditions.md");
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
fn exception_kinds_are_four() {
    let ex = read_json("exceptions.json");
    let paths = ex["paths"].as_array().unwrap();
    assert_eq!(paths.len(), 4);
    let kinds: Vec<&str> = paths.iter().map(|p| p["kind"].as_str().unwrap()).collect();
    assert!(kinds.contains(&"stale-last-verified"));
    assert!(kinds.contains(&"binary-version-mismatch"));
    assert!(kinds.contains(&"map-vs-canonical"));
    assert!(kinds.contains(&"invented-field-name"));
}

#[test]
fn field_names_live_in_map() {
    let fields = read_json("envelope_fields.json");
    assert_eq!(fields["invented"], false);
    assert_eq!(fields["definitionsCopied"], false);
    let names = fields["names"].as_array().unwrap();
    assert!(names.len() >= 100);
    let map =
        fs::read_to_string(repo_root().join("mydocs/manual/agent_knowledge_map.md")).expect("map");
    for name in names {
        let n = name.as_str().unwrap();
        let needle = format!("`{n}`");
        assert!(map.contains(&needle), "지도에 없는 필드 {n}");
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
    }
}

#[test]
fn transcripts_are_excerpted_from_canonical() {
    let idx = read_json("transcripts.json");
    assert_eq!(idx["excerptedFromCanonical"], true);
    assert_eq!(idx["fabricatedLive"], false);
    let ids = idx["ids"].as_array().unwrap();
    assert!(ids.len() >= 20);
}

#[test]
fn version_mismatch_prefers_binary() {
    let v = read_json("version_mismatch.json");
    assert_eq!(v["winner"], "binary");
    assert_eq!(v["mismatch"], true);
}

#[test]
fn forbidden_peer_skills_named() {
    let idx = read_json("skill_index.json");
    let names = idx["forbiddenSkillsTouch"].as_array().unwrap();
    let joined: Vec<&str> = names.iter().filter_map(|v| v.as_str()).collect();
    assert!(joined.contains(&"rhwp-codex"));
    assert!(joined.contains(&"rhwp-agent-surface"));
    assert!(repo_root()
        .join(".claude/skills/rhwp-codex/SKILL.md")
        .is_file());
    assert!(repo_root()
        .join(".claude/skills/rhwp-agent-surface/SKILL.md")
        .is_file());
}
