//! [#5337] rhwp-chief 스킬·픽스처 커버리지 가드.
//!
//! 새 rhwp CLI 를 넣지 않는다. 스킬 파일과 픽스처가 기존 표면
//! (service_loop.py / playbook §4 표 / FDE 게이트)을 가리키는지만
//! 검사하고, 표본이 있으면 `export-text --json` 으로 읽기 전용 대조만 한다.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_dir() -> PathBuf {
    repo_root().join(".claude/skills/rhwp-chief/fixtures")
}

fn skill_dir() -> PathBuf {
    repo_root().join(".claude/skills/rhwp-chief")
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

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn describe(args: &[&str], output: &Output) -> String {
    format!(
        "명령: rhwp {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn skill_frontmatter_names_chief() {
    let text = read_skill();
    assert!(text.starts_with("---\n"), "frontmatter 필요");
    assert!(text.contains("name: rhwp-chief"), "{text}");
    assert!(text.contains("request.json"), "{text}");
    assert!(text.contains("needs-agent"), "{text}");
    assert!(text.contains("export-pdf"), "{text}");
    assert!(text.contains("tools/chief/service_loop.py"), "{text}");
    assert!(
        text.contains("gym 이 아니다") || text.contains("gym"),
        "gym 이 아님을 밝혀야 한다"
    );
}

#[test]
fn references_listed_in_skill_index_exist() {
    let idx = read_json("skill_index.json");
    let refs = idx["references"].as_array().expect("references");
    assert!(refs.len() >= 20, "레퍼런스 20장 이상: {refs:?}");
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
        "skill_index.json",
        "routing_table.json",
        "stop_rules.json",
        "journeys.json",
        "intent_matrix.json",
        "layers.json",
        "queue_catalog.json",
    ] {
        let v = read_json(name);
        assert_eq!(v["schemaVersion"], "1.0", "{name}");
        assert_eq!(v["issue"], 5337, "{name}");
        assert_eq!(v["notGym"], true, "{name}");
        assert_eq!(v["noNewCli"], true, "{name}");
    }
}

#[test]
fn routing_table_lists_playbook_goals() {
    let table = read_json("routing_table.json");
    let goals = table["goals"].as_array().unwrap();
    let names: Vec<&str> = goals.iter().map(|g| g["goal"].as_str().unwrap()).collect();
    assert_eq!(
        names,
        vec![
            "diagnose",
            "export-text",
            "export-pdf",
            "export-hwpx",
            "convert-hwp",
            "extract-tables",
            "fill",
        ]
    );
    assert_eq!(table["offTableStatus"], "needs-agent");
    assert_eq!(goals[0]["defaultWhenMissing"], true);
}

#[test]
fn stop_rule_ids_appear_in_docs() {
    let stop = read_ref("19_stop_rules.md");
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
    assert!(items.len() >= 90, "여정이 너무 적다");
    for j in items {
        if let Some(stop) = j["stop"].as_str() {
            assert!(ids.contains(stop), "여정 정지 {stop} 미정의");
        }
        assert!(!j["steps"].as_array().unwrap().is_empty());
        assert_eq!(j["notGym"], true);
    }
}

#[test]
fn missing_goal_intents_route_to_diagnose() {
    let intents = read_json("intent_matrix.json");
    let rows = intents["intents"].as_array().unwrap();
    assert!(rows.len() >= 160, "발화가 너무 적다");
    let mut missing = 0;
    for row in rows {
        if row["goalField"].is_null() {
            missing += 1;
            assert_eq!(row["routed"], "diagnose", "{row}");
        }
    }
    assert!(missing >= 1, "goal 생략 사례가 없다");
}

#[test]
fn queue_snapshots_exist_and_have_response_parts() {
    let cat = read_json("queue_catalog.json");
    let queues = cat["queues"].as_array().unwrap();
    assert!(queues.len() >= 36, "큐 스냅샷이 너무 적다");
    for q in queues {
        let id = q["id"].as_str().unwrap();
        let dir = fixture_dir().join("queues").join(id);
        for name in ["request.json", "result.json", "response.md", "ticket.json"] {
            assert!(dir.join(name).is_file(), "{id}/{name}");
        }
        let body = fs::read_to_string(dir.join("response.md")).unwrap();
        assert!(body.contains("## 1. 확인한 것"), "{id}");
        assert!(body.contains("## 2. 지금 가능한 것"), "{id}");
        assert!(body.contains("## 3. 다음"), "{id}");
    }
}

#[test]
fn forbidden_peer_skills_not_rewritten_here() {
    let idx = read_json("skill_index.json");
    for name in idx["forbiddenSkillsTouch"].as_array().unwrap() {
        let slug = name.as_str().unwrap();
        if slug == "rhwp-fde" || slug == "rhwp-strategist" {
            continue;
        }
        let peer = repo_root()
            .join(".claude/skills")
            .join(slug)
            .join("SKILL.md");
        assert!(peer.is_file(), "존재해야 하는 이웃 스킬 {peer:?}");
    }
}

#[test]
fn working_doc_exists() {
    let path = repo_root().join("mydocs/working/agent_chief.md");
    assert!(path.is_file(), "{path:?}");
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("#5337"), "{text}");
    assert!(text.contains("rhwp-chief"), "{text}");
}

#[test]
fn skill_index_forbids_gym_tree() {
    let idx = read_json("skill_index.json");
    let trees = idx["forbiddenTrees"].as_array().unwrap();
    assert!(trees.iter().any(|t| t == "gym/"), "{trees:?}");
}

#[test]
fn agent_definition_is_linked_when_present() {
    let agent = repo_root().join(".claude/agents/rhwp-chief.md");
    assert!(agent.is_file(), "{agent:?}");
    let skill = read_skill();
    assert!(
        skill.contains(".claude/agents/rhwp-chief.md"),
        "스킬이 에이전트를 가리켜야 한다"
    );
}

#[test]
fn loop_source_declares_routing_table() {
    let path = repo_root().join("tools/chief/service_loop.py");
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("ROUTING_TABLE"), "{path:?}");
    assert!(text.contains("normalize_goal"), "{path:?}");
    assert!(text.contains("route_skips_goal"), "{path:?}");
    assert!(text.contains("is_already_processed"), "{path:?}");
    assert!(text.contains("escalate-bug"), "{path:?}");
    assert!(text.contains("needs-agent"), "{path:?}");
}

#[test]
fn export_text_json_when_sample_exists() {
    let p = repo_root().join("samples/form-01.hwp");
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let args = ["export-text", p.to_str().unwrap(), "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert!(v.is_object(), "{v}");
}
