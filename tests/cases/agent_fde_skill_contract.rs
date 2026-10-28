//! [#5333] rhwp-fde 스킬·픽스처 커버리지 가드.
//!
//! 새 CLI 를 넣지 않는다. 스킬 파일과 픽스처가 playbook 과
//! tools/fde/triage.py 를 가리키는지만 검사한다.
//! 엔진 로직을 재구현하거나 DocumentCore 를 고치지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_dir() -> PathBuf {
    repo_root().join(".claude/skills/rhwp-fde/fixtures")
}

fn skill_dir() -> PathBuf {
    repo_root().join(".claude/skills/rhwp-fde")
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
fn skill_frontmatter_names_fde() {
    let text = read_skill();
    assert!(text.starts_with("---\n"), "frontmatter 필요");
    assert!(text.contains("name: rhwp-fde"), "{text}");
    assert!(text.contains("fde_playbook.md"), "{text}");
    assert!(text.contains("tools/fde/triage.py"), "{text}");
    assert!(text.contains("invalid-input"), "{text}");
    assert!(text.contains("resolve-now"), "{text}");
    assert!(text.contains("escalate-bug"), "{text}");
    assert!(
        text.contains("gym 이 아니고") || text.contains("gym"),
        "gym 이 아님을 밝혀야 한다"
    );
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
        assert!(body.len() > 400, "{name} 가 너무 짧다");
    }
}

#[test]
fn fixtures_share_schema_and_issue() {
    for name in [
        "tree.json",
        "stop_rules.json",
        "envelope_keys.json",
        "journeys.json",
        "skill_index.json",
        "intent_matrix.json",
        "routes.json",
        "ticket_schema.json",
    ] {
        let v = read_json(name);
        assert_eq!(v["schemaVersion"], "1.0", "{name}");
        assert_eq!(v["issue"], 5333, "{name}");
    }
}

#[test]
fn tree_fixture_declares_not_gym_and_no_new_cli() {
    let tree = read_json("tree.json");
    assert_eq!(tree["notGym"], true);
    assert_eq!(tree["noNewCli"], true);
    assert_eq!(tree["noNewEngineLogic"], true);
    assert_eq!(tree["bugHunterRewriteForbidden"], true);
    assert_eq!(tree["symptomIsData"], true);
    let reuse = tree["coreReuse"].as_array().expect("coreReuse");
    let joined = reuse
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(joined.contains("tools/fde/triage.py"), "{joined}");
    assert!(joined.contains("capabilities --json"), "{joined}");
}

#[test]
fn stop_rule_ids_appear_in_docs() {
    let stop = read_ref("26_failure_signals.md") + &read_ref("22_pitfalls.md");
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
fn journeys_use_known_stop_ids() {
    let journeys = read_json("journeys.json");
    let stops = read_json("stop_rules.json");
    let mut ids = std::collections::HashSet::new();
    for r in stops["rules"].as_array().unwrap() {
        ids.insert(r["id"].as_str().unwrap().to_string());
    }
    let items = journeys["journeys"].as_array().unwrap();
    assert!(items.len() >= 70, "여정이 너무 적다");
    for j in items {
        let stop = j["stop"].as_str().unwrap();
        assert!(ids.contains(stop), "{} unknown stop {stop}", j["id"]);
        assert_eq!(j["notGym"], true);
        assert_eq!(j["liveCustomer"], true);
    }
}

#[test]
fn ticket_schema_forbids_prose_success() {
    let schema = read_json("ticket_schema.json");
    let prose = schema["forbiddenProse"].as_array().unwrap();
    assert!(prose.iter().any(|v| v == "it worked"));
    assert_eq!(schema["symptomFieldIsData"], true);
    let keys = schema["requiredKeys"].as_array().unwrap();
    for need in ["steps", "route", "symptom", "generatedBy"] {
        assert!(keys.iter().any(|v| v == need), "{need}");
    }
}

#[test]
fn routes_map_crash_alias_to_escalate_bug() {
    let routes = read_json("routes.json");
    let aliases = routes["aliases"].as_array().unwrap();
    let crash = aliases
        .iter()
        .find(|a| a["alias"] == "escalate-crash")
        .unwrap();
    assert_eq!(crash["mapsTo"], "escalate-bug");
    let corrupt = aliases
        .iter()
        .find(|a| a["alias"] == "escalate-corrupt")
        .unwrap();
    assert_eq!(corrupt["mapsTo"], "workaround");
}

#[test]
fn agent_definition_exists_for_link_only() {
    let path = repo_root().join(".claude/agents/rhwp-fde.md");
    assert!(path.is_file(), "{path:?}");
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("tools/fde/triage.py"), "{text}");
}

#[test]
fn playbook_file_is_the_authority() {
    let play = repo_root().join("mydocs/manual/fde_playbook.md");
    assert!(play.is_file(), "{play:?}");
    let idx = read_json("skill_index.json");
    assert_eq!(idx["authority"][0], "mydocs/manual/fde_playbook.md");
    assert_eq!(idx["authority"][1], "tools/fde/triage.py");
}

#[test]
fn working_doc_exists() {
    let path = repo_root().join("mydocs/working/archives/agent_fde.md");
    assert!(path.is_file(), "{path:?}");
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("#5333"), "{text}");
    assert!(text.contains("rhwp-fde"), "{text}");
}

#[test]
fn skill_index_forbids_gym_tree() {
    let idx = read_json("skill_index.json");
    let trees = idx["forbiddenTrees"].as_array().unwrap();
    assert!(trees.iter().any(|t| t == "gym/"), "{trees:?}");
}

#[test]
fn magic_byte_fixtures_exist() {
    for name in [
        "hwpx_head.bin",
        "hwp5_head.bin",
        "hwp3_head.bin",
        "pdf_disguise.bin",
        "empty.bin",
        "plain_text.bin",
    ] {
        let path = fixture_dir().join("binaries").join(name);
        assert!(path.is_file(), "{path:?}");
    }
    let hwpx = fs::read(fixture_dir().join("binaries/hwpx_head.bin")).unwrap();
    assert!(hwpx.starts_with(b"PK\x03\x04"), "{hwpx:?}");
    let hwp5 = fs::read(fixture_dir().join("binaries/hwp5_head.bin")).unwrap();
    assert!(
        hwp5.starts_with(b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1"),
        "{hwp5:?}"
    );
    let hwp3 = fs::read(fixture_dir().join("binaries/hwp3_head.bin")).unwrap();
    assert!(hwp3.starts_with(b"HWP Document File"), "{hwp3:?}");
}
