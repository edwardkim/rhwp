//! [#5307] rhwp-security-sweep 스킬 — 실사용 에이전트 보안 스윕 계약.
//!
//! 새 CLI 를 만들지 않는다. 권위는 cli_commands.md 와 스킬 픽스처다.
#![cfg(not(target_arch = "wasm32"))]

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

fn required_references() -> &'static [&'static str] {
    &[
        "references/00_tree.md",
        "references/01_hidden_text.md",
        "references/02_injection.md",
        "references/03_unicode.md",
        "references/04_redact_dry_run.md",
        "references/05_pii_rules.md",
        "references/06_no_raw.md",
        "references/07_redact_sanitize_pair.md",
        "references/08_resweep_gate.md",
        "references/09_receive_path.md",
        "references/10_exit_codes.md",
        "references/11_untrusted_content.md",
        "references/12_envelopes.md",
        "references/13_pitfalls.md",
        "references/14_journeys.md",
        "references/15_anti_patterns.md",
        "references/16_scan_scopes.md",
        "references/17_watermark_out_of_scope.md",
        "references/18_automation.md",
        "references/19_field_catalog.md",
        "references/20_worked_traces.md",
        "references/21_cli_surface.md",
    ]
}

#[test]
fn skill_frontmatter_and_index() {
    let skill = read_skill("SKILL.md");
    assert!(skill.starts_with("---\n"), "frontmatter");
    assert!(skill.contains("name: rhwp-security-sweep"));
    for token in [
        "inspect hidden-text",
        "inspect injection",
        "inspect unicode",
        "edit redact",
        "edit sanitize",
        "--no-raw",
        "findingCount == 0",
        "clean == true",
        "info → digest → fields → inspect",
        "gym",
        "새 CLI",
    ] {
        assert!(skill.contains(token), "SKILL.md 에 {token} 없음");
    }
    let idx = read_json("fixtures/skill_index.json");
    assert_eq!(idx["skill"], "rhwp-security-sweep");
    assert_eq!(idx["issue"], 5307);
    assert_eq!(idx["gym"], false);
    assert_eq!(idx["newCli"], false);
    let refs = idx["references"].as_array().unwrap();
    assert!(refs.len() >= 22, "레퍼런스 22장: {}", refs.len());
    for r in refs {
        let name = r.as_str().unwrap();
        let path = skill_dir().join("references").join(name);
        assert!(path.is_file(), "누락 {path:?}");
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.len() > 400, "{name} 가 너무 짧다");
        assert!(skill.contains(name), "SKILL.md 가 {name} 를 가리켜야 한다");
    }
}

#[test]
fn required_reference_files_exist() {
    for rel in required_references() {
        assert!(skill_dir().join(rel).is_file(), "누락 {rel}");
    }
    assert!(skill_dir().join("examples/README.md").is_file());
}

#[test]
fn skill_does_not_add_cli_and_stays_out_of_peers() {
    let skill = read_skill("SKILL.md");
    assert!(
        skill.contains("새 CLI") || skill.contains("새 명령을 만들지 않는다"),
        "새 CLI 금지 문구"
    );
    let cargo = fs::read_to_string(repo().join("Cargo.toml")).expect("Cargo.toml");
    let bins = cargo.matches("[[bin]]").count();
    assert_eq!(bins, 2, "새 [[bin]] 금지: {bins}");
    let idx = read_json("fixtures/skill_index.json");
    for name in idx["forbiddenSkillsTouch"].as_array().unwrap() {
        let slug = name.as_str().unwrap();
        let peer = repo().join(".agents/skills").join(slug).join("SKILL.md");
        assert!(peer.is_file(), "이웃 스킬을 지우지 말 것: {peer:?}");
    }
}

#[test]
fn working_doc_records_issue_and_scope() {
    let path = repo().join("mydocs/working/archives/agent_security_sweep.md");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    assert!(text.contains("5307"), "이슈 번호");
    assert!(text.contains("inspect hidden-text"));
    assert!(text.contains("--no-raw"));
    assert!(text.contains("gym"));
    assert!(text.contains("새 CLI") || text.contains("새 서브커맨드"));
}

#[test]
fn capability_registry_lists_security_sweep() {
    let path = repo().join("mydocs/manual/agent_capability_registry.md");
    if !path.is_file() {
        return;
    }
    let text = fs::read_to_string(&path).expect("capability registry");
    assert!(text.contains("rhwp-security-sweep"), "카탈로그 행");
    assert!(text.contains("CAP-5307"), "CAP-5307");
}

#[test]
fn examples_listed_in_index_exist() {
    let idx = read_json("fixtures/skill_index.json");
    for name in idx["examples"].as_array().unwrap() {
        let n = name.as_str().unwrap();
        let path = skill_dir().join("examples").join(n);
        assert!(path.is_file(), "예제 누락 {path:?}");
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.contains("rhwp"), "{n} 에 명령 없음");
    }
}
