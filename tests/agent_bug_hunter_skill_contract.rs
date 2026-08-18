//! [#5324] bug-hunter 스킬·픽스처 커버리지 가드.
//!
//! 새 CLI 를 넣지 않는다. 스킬 파일과 픽스처가 playbook 과
//! 기존 CLI · tools/fidelity_compare 를 가리키는지만 검사한다.
//! 엔진을 실행하거나 DocumentCore 를 고치지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_dir() -> PathBuf {
    repo_root().join(".agents/skills/bug-hunter/fixtures")
}

fn skill_dir() -> PathBuf {
    repo_root().join(".agents/skills/bug-hunter")
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
fn skill_frontmatter_names_bug_hunter() {
    let text = read_skill();
    assert!(text.starts_with("---\n"), "frontmatter 필요");
    assert!(text.contains("name: bug-hunter"), "{text}");
    assert!(text.contains("bug_hunting_playbook.md"), "{text}");
    assert!(text.contains("fidelity_compare"), "{text}");
    assert!(text.contains("소실"), "{text}");
    assert!(text.contains("과잉"), "{text}");
    assert!(text.contains("치환"), "{text}");
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
        "classification.json",
        "issue_templates.json",
    ] {
        let v = read_json(name);
        assert_eq!(v["schemaVersion"], "1.0", "{name}");
        assert_eq!(v["issue"], 5324, "{name}");
    }
}

#[test]
fn tree_fixture_declares_not_gym_and_no_new_cli() {
    let tree = read_json("tree.json");
    assert_eq!(tree["notGym"], true);
    assert_eq!(tree["noNewCli"], true);
    assert_eq!(tree["secondRubricForbidden"], true);
    assert_eq!(tree["huntingNotFix"], true);
    assert_eq!(tree["aaIsNotHangulFidelity"], true);
    let reuse = tree["coreReuse"].as_array().expect("coreReuse");
    let joined = reuse
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(joined.contains("fidelity_compare"), "{joined}");
    assert!(joined.contains("export-svg"), "{joined}");
}

#[test]
fn stop_rule_ids_appear_in_docs() {
    let stop = read_ref("22_failure_signals.md");
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
    assert!(items.len() >= 80, "여정이 너무 적다");
    for j in items {
        let stop = j["stop"].as_str().unwrap();
        assert!(ids.contains(stop), "{} unknown stop {stop}", j["id"]);
        assert_eq!(j["notGym"], true);
    }
}

#[test]
fn classification_contract_is_playbook_multiset() {
    let klass = read_json("classification.json");
    assert_eq!(klass["missing"], "loss");
    assert_eq!(klass["extra"], "excess");
    assert_eq!(klass["both"], "substitution");
}

#[test]
fn issue_template_requires_repro_path_and_truth() {
    let tmpl = read_json("issue_templates.json");
    let req = tmpl["requiredFields"].as_array().unwrap();
    let got: Vec<&str> = req.iter().filter_map(|v| v.as_str()).collect();
    assert_eq!(got, ["repro", "codePath", "groundTruth"]);
}

#[test]
fn claude_pointer_exists() {
    let path = repo_root().join(".claude/skills/rhwp-bug-hunter/SKILL.md");
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("얇은 포인터"), "{text}");
    assert!(text.contains(".agents/skills/bug-hunter"), "{text}");
}

#[test]
fn playbook_file_is_the_authority() {
    let play = repo_root().join("mydocs/manual/bug_hunting_playbook.md");
    assert!(play.is_file(), "{play:?}");
    let idx = read_json("skill_index.json");
    assert_eq!(idx["authority"][0], "mydocs/manual/bug_hunting_playbook.md");
}
