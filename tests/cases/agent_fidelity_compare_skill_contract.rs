//! [#5329] rhwp-fidelity-compare 스킬·픽스처 커버리지 가드.
//!
//! 새 CLI 를 넣지 않는다. 스킬 파일과 픽스처가 기존 도구 표면
//! (`tools/fidelity_compare`, `rhwp export-svg`)을 가리키는지만
//! 검사한다. Chrome·한컴 PDF 가 없는 CI 에서도 파일만으로 닫힌다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn skill_dir() -> PathBuf {
    repo_root().join(".claude/skills/rhwp-fidelity-compare")
}

fn fixture_dir() -> PathBuf {
    skill_dir().join("fixtures")
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
fn skill_frontmatter_names_fidelity_compare() {
    let text = read_skill();
    assert!(text.starts_with("---\n"), "frontmatter 필요");
    assert!(text.contains("name: rhwp-fidelity-compare"), "{text}");
    assert!(text.contains("fidelity_compare"), "{text}");
    assert!(text.contains("export-svg"), "{text}");
    assert!(
        text.contains("gym 이 아니고") || text.contains("gym"),
        "gym 이 아님을 밝혀야 한다"
    );
    assert!(text.contains("새 CLI"), "{text}");
    assert!(
        text.contains("venv\\Scripts\\python.exe") || text.contains(r"venv\Scripts\python.exe")
    );
    assert!(text.contains("--break-system-packages"), "{text}");
    assert!(text.contains("visual_verification_governance"), "{text}");
}

#[test]
fn references_listed_in_skill_index_exist() {
    let idx = read_json("skill_index.json");
    let refs = idx["references"].as_array().expect("references");
    assert!(refs.len() >= 16, "레퍼런스 16장 이상: {refs:?}");
    for r in refs {
        let name = r.as_str().expect("name");
        if name.starts_with('_') {
            continue;
        }
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
        "skill_index.json",
        "journeys.json",
        "exception_catalog.json",
        "provenance_schema.json",
        "outputs.json",
        "intent_matrix.json",
    ] {
        let v = read_json(name);
        assert_eq!(v["schemaVersion"], "1.0", "{name}");
        assert_eq!(v["issue"], 5329, "{name}");
        assert_eq!(v["notGym"], true, "{name}");
        assert_eq!(v["noNewCli"], true, "{name}");
    }
}

#[test]
fn tree_fixture_declares_not_gym_and_no_new_cli() {
    let tree = read_json("tree.json");
    assert_eq!(tree["notGym"], true);
    assert_eq!(tree["noNewCli"], true);
    assert_eq!(tree["textOnlySkipsChrome"], true);
    assert_eq!(tree["rankingIsCandidate"], true);
    assert_eq!(tree["verdictIsMaintainer"], true);
    assert_eq!(tree["breakSystemPackages"], false);
    assert_eq!(tree["windowsPython"], r"venv\Scripts\python.exe");
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
    let catalog = read_ref("27_exception_catalog.md");
    let skill = read_skill();
    let rules = read_json("stop_rules.json");
    for rule in rules["rules"].as_array().unwrap() {
        let id = rule["id"].as_str().unwrap();
        assert!(
            catalog.contains(id) || skill.contains(id),
            "정지 장에 {id} 없음"
        );
    }
}

#[test]
fn journeys_use_known_stop_ids() {
    let journeys = read_json("journeys.json");
    let stops = read_json("stop_rules.json");
    let mut ids = HashSet::new();
    for r in stops["rules"].as_array().unwrap() {
        ids.insert(r["id"].as_str().unwrap().to_string());
    }
    let items = journeys["journeys"].as_array().unwrap();
    assert!(items.len() >= 80, "여정이 너무 적다");
    for j in items {
        let stop = j["stop"].as_str().unwrap();
        assert!(ids.contains(stop), "여정 정지 {stop} 미정의");
        assert!(!j["steps"].as_array().unwrap().is_empty());
        assert_eq!(j["notGym"], true);
    }
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

#[test]
fn invented_commands_are_forbidden() {
    let idx = read_json("skill_index.json");
    let skill = read_skill();
    for cmd in idx["inventedCommandsForbidden"].as_array().unwrap() {
        let name = cmd.as_str().unwrap();
        assert!(
            !skill.contains(&format!("rhwp {name}")),
            "발명 명령 {name} 이 SKILL 에 호출로 등장"
        );
    }
    let trees = idx["forbiddenTrees"].as_array().unwrap();
    assert!(trees.iter().any(|t| t == "gym/"), "{trees:?}");
}

#[test]
fn text_report_fixture_has_contract_columns() {
    let path = fixture_dir().join("tsv/text_report_mixed.tsv");
    let text = fs::read_to_string(&path).unwrap();
    let header = text.lines().next().unwrap();
    assert_eq!(
        header,
        "page\treference_only\tsvg_only\treference_only_chars\tsvg_only_chars\tnote"
    );
    assert!(text.contains("substitution-candidate"), "{text}");
}

#[test]
fn report_tsv_is_worst_first() {
    let path = fixture_dir().join("tsv/report_ranked.tsv");
    let text = fs::read_to_string(&path).unwrap();
    let header = text.lines().next().unwrap();
    assert_eq!(header, "page\tdiff%\tnote");
    let mut scores = Vec::new();
    for line in text.lines().skip(1) {
        if line.is_empty() {
            continue;
        }
        let score: f64 = line.split('\t').nth(1).unwrap().parse().unwrap();
        scores.push(score);
    }
    let mut sorted = scores.clone();
    sorted.sort_by(|a, b| b.partial_cmp(a).unwrap());
    assert_eq!(scores, sorted, "최악 쪽 우선이 아님: {scores:?}");
}

#[test]
fn provenance_and_run_state_headers() {
    let prov = fs::read_to_string(fixture_dir().join("tsv/provenance_plan.tsv")).unwrap();
    assert_eq!(prov.lines().next().unwrap(), "role\tpath\tgrade");
    assert!(prov.contains("reference_pdf"));
    let run = fs::read_to_string(fixture_dir().join("tsv/run_state_complete.tsv")).unwrap();
    assert_eq!(run.lines().next().unwrap(), "field\tvalue");
    assert!(run.contains("run_state\tcomplete"));
}

#[test]
fn exception_catalog_covers_required_paths() {
    let cat = read_json("exception_catalog.json");
    let ids: HashSet<_> = cat["exceptions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["id"].as_str().unwrap().to_string())
        .collect();
    for need in ["E-CHROME", "E-VENV", "E-PAGECOUNT", "E-ENCRYPT", "E-TOFU"] {
        assert!(ids.contains(need), "예외 {need} 없음: {ids:?}");
    }
}

#[test]
fn working_doc_exists() {
    let path = repo_root().join("mydocs/working/archives/agent_fidelity_compare.md");
    assert!(path.is_file(), "{path:?}");
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("#5329"), "{text}");
    assert!(text.contains("rhwp-fidelity-compare"), "{text}");
    assert!(text.contains("fidelity_compare"), "{text}");
}

#[test]
fn skill_index_forbids_gym_tree() {
    let idx = read_json("skill_index.json");
    let trees = idx["forbiddenTrees"].as_array().unwrap();
    assert!(trees.iter().any(|t| t == "gym/"), "{trees:?}");
}

#[test]
fn examples_listed_exist() {
    let idx = read_json("skill_index.json");
    let examples = idx["examples"].as_array().expect("examples");
    assert!(examples.len() >= 16, "{examples:?}");
    for name in examples {
        let path = skill_dir().join("examples").join(name.as_str().unwrap());
        assert!(path.is_file(), "누락 {path:?}");
    }
}

#[test]
fn tool_readme_is_authority() {
    let readme = repo_root().join("tools/fidelity_compare/README.md");
    assert!(readme.is_file(), "{readme:?}");
    let text = fs::read_to_string(&readme).unwrap();
    assert!(text.contains("--text-only"), "{text}");
    assert!(text.contains("venv\\Scripts\\python.exe") || text.contains("Scripts"));
    assert!(text.contains("visual_verification_governance"), "{text}");
}

#[test]
fn no_new_rhwp_cli_in_skill_tree() {
    let skill = read_skill();
    for banned in [
        "rhwp fidelity-diff",
        "rhwp pdf-compare",
        "rhwp hangul-diff",
        "rhwp oracle-diff",
        "rhwp hancom-compare",
    ] {
        assert!(!skill.contains(banned), "발명 CLI {banned}");
    }
}
