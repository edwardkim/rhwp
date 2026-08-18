//! [#5312] rhwp-visual-regression 스킬·픽스처 커버리지 가드.
//!
//! 새 CLI 를 넣지 않는다. 스킬 파일과 픽스처가 기존 CLI 표면
//! (render-diff / ir-diff / thumbnail / export-png)을 가리키는지만
//! 검사하고, 표본이 있으면 읽기 전용 대조만 한다.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_dir() -> PathBuf {
    repo_root().join(".claude/skills/rhwp-visual-regression/fixtures")
}

fn skill_dir() -> PathBuf {
    repo_root().join(".claude/skills/rhwp-visual-regression")
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
fn skill_frontmatter_names_visual_regression() {
    let text = read_skill();
    assert!(text.starts_with("---\n"), "frontmatter 필요");
    assert!(text.contains("name: rhwp-visual-regression"), "{text}");
    assert!(text.contains("render-diff"), "{text}");
    assert!(text.contains("ir-diff"), "{text}");
    assert!(text.contains("thumbnail"), "{text}");
    assert!(text.contains("export-png"), "{text}");
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
        "status_catalog.json",
        "determinism.json",
    ] {
        let v = read_json(name);
        assert_eq!(v["schemaVersion"], "1.0", "{name}");
        assert_eq!(v["issue"], 5312, "{name}");
    }
}

#[test]
fn tree_fixture_declares_not_gym_and_no_new_cli() {
    let tree = read_json("tree.json");
    assert_eq!(tree["notGym"], true);
    assert_eq!(tree["noNewCli"], true);
    assert_eq!(tree["aaMustPass"], true);
    assert_eq!(tree["defaultMaxDispPx"], 1.0);
    assert_eq!(tree["structIgnoresThreshold"], true);
    assert_eq!(tree["thumbnailIsStoredPreview"], true);
    let reuse = tree["coreReuse"].as_array().expect("coreReuse");
    let joined = reuse
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(joined.contains("render_geom_diff"), "{joined}");
    assert!(joined.contains("ir-diff"), "{joined}");
}

#[test]
fn stop_rule_ids_appear_in_docs() {
    let stop = read_ref("14_failure_signals.md");
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
fn status_catalog_marks_struct_hard_but_docs_say_read_path() {
    let cat = read_json("status_catalog.json");
    let statuses = cat["statuses"].as_array().unwrap();
    let struct_s = statuses
        .iter()
        .find(|s| s["id"] == "STRUCT_MISMATCH")
        .unwrap();
    assert_eq!(struct_s["hard"], true);
    assert_eq!(struct_s["jsonExit"], 3);
    let chapter = read_ref("04_struct_mismatch.md");
    assert!(chapter.contains("경로"), "{chapter}");
    assert!(chapter.contains("반사"), "{chapter}");
}

#[test]
fn tsv_pass_fixture_has_contract_columns() {
    let path = fixture_dir().join("tsv/geom_inventory_pass.tsv");
    let text = fs::read_to_string(&path).unwrap();
    let header = text.lines().next().unwrap();
    assert_eq!(
        header,
        "sample\tstatus\tpages_a\tpages_b\tmax_disp\tworst_page\tstruct_pages\tover_pages\telapsed_ms\terror\tstruct_delta"
    );
    assert!(text.contains("form-01.hwp\tPASS"));
}

#[test]
fn render_diff_self_pass_when_sample_exists() {
    let p = repo_root().join("samples/form-01.hwp");
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let args = ["render-diff", p.to_str().unwrap(), "--via", "hwpx"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("status: PASS"), "{stdout}");
}

#[test]
fn render_diff_aa_pass_when_sample_exists() {
    let p = repo_root().join("samples/form-01.hwp");
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let path = p.to_str().unwrap();
    let args = ["render-diff", path, path];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("status: PASS"), "{stdout}");
}

#[test]
fn ir_diff_json_identical_exit_zero_when_sample_exists() {
    let p = repo_root().join("samples/hwp3-sample.hwp");
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let path = p.to_str().unwrap();
    let args = ["ir-diff", path, path, "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert_eq!(v["identical"], true, "{v}");
    assert_eq!(v["diffCount"], 0, "{v}");
}

#[test]
fn ir_diff_json_diff_is_exit_three_when_samples_exist() {
    let a = repo_root().join("samples/hwp3-sample.hwp");
    let b = repo_root().join("samples/SO-SUEOP.hwp");
    if !a.exists() || !b.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let args = [
        "ir-diff",
        a.to_str().unwrap(),
        b.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(3),
        "{}",
        describe(&args, &output)
    );
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(v["identical"], false, "{v}");
    assert!(v["diffCount"].as_u64().unwrap() > 0, "{v}");
}

#[test]
fn render_diff_missing_file_is_runtime_not_usage() {
    let args = ["render-diff", "samples/no-such-visual-regression.hwp"];
    let output = run(&args);
    let code = output.status.code();
    assert!(
        code == Some(1) || code == Some(2),
        "없는 파일은 1 또는 2: {}",
        describe(&args, &output)
    );
}

#[test]
fn working_doc_exists() {
    let path = repo_root().join("mydocs/working/agent_visual_regression.md");
    assert!(path.is_file(), "{path:?}");
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("#5312"), "{text}");
    assert!(text.contains("rhwp-visual-regression"), "{text}");
}

#[test]
fn skill_index_forbids_gym_tree() {
    let idx = read_json("skill_index.json");
    let trees = idx["forbiddenTrees"].as_array().unwrap();
    assert!(trees.iter().any(|t| t == "gym/"), "{trees:?}");
}
