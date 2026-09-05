//! [#5300] rhwp-form-fill 스킬·픽스처 커버리지 가드.
//!
//! 새 edit 로직을 넣지 않는다. 스킬 파일과 픽스처가 기존 CLI 표면
//! (fields / fill-fields / 이름[N] / batch fill / dry-run / verify /
//! sanitize)을 가리키는지만 검사하고, 표본이 있으면 `fields --json` 으로
//! 읽기 전용 대조만 한다.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_dir() -> PathBuf {
    repo_root().join(".agents/skills/rhwp-form-fill/references/fixtures")
}

fn skill_dir() -> PathBuf {
    repo_root().join(".agents/skills/rhwp-form-fill")
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
fn skill_frontmatter_names_form_fill() {
    let text = read_skill();
    assert!(text.starts_with("---\n"), "frontmatter 필요");
    assert!(text.contains("name: rhwp-form-fill"), "{text}");
    assert!(text.contains("fields"), "{text}");
    assert!(text.contains("fill-fields"), "{text}");
    assert!(text.contains("batch fill"), "{text}");
    assert!(text.contains("sanitize"), "{text}");
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
        "occurrence_catalog.json",
    ] {
        let v = read_json(name);
        assert_eq!(v["schemaVersion"], "1.0", "{name}");
        assert_eq!(v["issue"], 5300, "{name}");
    }
}

#[test]
fn tree_fixture_declares_not_gym_and_no_new_edit() {
    let tree = read_json("tree.json");
    assert_eq!(tree["notGym"], true);
    assert_eq!(tree["noNewCli"], true);
    assert_eq!(tree["noNewEditLogic"], true);
    let reuse = tree["coreReuse"].as_array().expect("coreReuse");
    let joined = reuse
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(joined.contains("set_field_value_by_name"), "{joined}");
    assert!(joined.contains("collect_all_fields"), "{joined}");
}

#[test]
fn stop_rule_ids_appear_in_docs() {
    let stop = read_ref("11_failure_signals.md");
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
fn occurrence_catalog_is_zero_based() {
    let occ = read_json("occurrence_catalog.json");
    assert_eq!(occ["zeroBased"], true);
    assert_eq!(occ["bareKeyMeansFirstMatch"], true);
    let cases = occ["cases"].as_array().unwrap();
    assert!(cases.len() >= 40, "순번 사례가 너무 적다");
    let has_zero = cases.iter().any(|c| c["key"] == "목차1[0]");
    let has_oob = cases.iter().any(|c| c["key"] == "목차1[5]");
    assert!(has_zero && has_oob, "{cases:?}");
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
            .join(".agents/skills")
            .join(slug)
            .join("SKILL.md");
        assert!(peer.is_file(), "존재해야 하는 이웃 스킬 {peer:?}");
    }
}

#[test]
fn fields_survey_on_form01_when_sample_exists() {
    let p = repo_root().join("samples/form-01.hwp");
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let args = ["fields", p.to_str().unwrap(), "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert_eq!(v["fieldCount"], 1, "{v}");
    assert_eq!(v["fields"][0]["name"], "myMsg01", "{v}");
}

#[test]
fn fields_survey_zero_is_success_when_sample_exists() {
    let p = repo_root().join("samples/hwp3-sample.hwp");
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let args = ["fields", p.to_str().unwrap(), "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(v["fieldCount"], 0, "{v}");
}

#[test]
fn fill_fields_dry_run_does_not_write_when_sample_exists() {
    let p = repo_root().join("samples/form-01.hwp");
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out = std::env::temp_dir().join(format!(
        "rhwp-aff-dry-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    let args = [
        "edit",
        "fill-fields",
        p.to_str().unwrap(),
        "--data",
        r#"{"myMsg01":"홍길동 귀하"}"#,
        "-o",
        out.to_str().unwrap(),
        "--dry-run",
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(v["dryRun"], true, "{v}");
    assert_eq!(v["filledCount"], 1, "{v}");
    assert!(
        !out.exists(),
        "--dry-run 은 파일을 만들면 안 됩니다: {}",
        out.display()
    );
}

#[test]
fn empty_csv_is_usage_error_when_sample_exists() {
    let form = repo_root().join("samples/form-01.hwp");
    let csv = fixture_dir().join("data/empty_header_only.csv");
    if !form.exists() || !csv.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out_dir = std::env::temp_dir().join(format!("rhwp-aff-empty-{}", std::process::id()));
    let _ = fs::create_dir_all(&out_dir);
    let args = [
        "batch",
        "fill",
        "--form",
        form.to_str().unwrap(),
        "--data",
        csv.to_str().unwrap(),
        "--out-dir",
        out_dir.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
}

#[test]
fn working_doc_exists() {
    let path = repo_root().join("mydocs/working/archives/agent_form_fill.md");
    assert!(path.is_file(), "{path:?}");
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("#5300"), "{text}");
    assert!(text.contains("rhwp-form-fill"), "{text}");
}

#[test]
fn skill_index_forbids_gym_tree() {
    let idx = read_json("skill_index.json");
    let trees = idx["forbiddenTrees"].as_array().unwrap();
    assert!(trees.iter().any(|t| t == "gym/"), "{trees:?}");
}
