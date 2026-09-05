//! [#5316] rhwp-cli 스킬 — 실사용 에이전트 CLI 분석·디버깅 계약.
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
    repo().join(".agents/skills/rhwp-cli")
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
        "references/01_request_command_map.md",
        "references/02_export_svg.md",
        "references/03_export_png.md",
        "references/04_export_pdf.md",
        "references/05_export_text.md",
        "references/06_export_markdown.md",
        "references/07_dump_pages.md",
        "references/08_dump.md",
        "references/09_dump_records.md",
        "references/10_diag.md",
        "references/11_info.md",
        "references/12_export_render_tree.md",
        "references/13_ir_diff.md",
        "references/14_thumbnail.md",
        "references/15_convert.md",
        "references/16_hwp5_family.md",
        "references/17_layout_debug_order.md",
        "references/18_page_units.md",
        "references/19_roundtrip_vs_hangul.md",
        "references/20_hwpx_hwp_save_contract.md",
        "references/21_exception_envelopes.md",
        "references/22_exit_codes.md",
        "references/23_pitfalls.md",
        "references/24_anti_patterns.md",
        "references/25_journeys.md",
        "references/26_cli_surface.md",
        "references/27_field_catalog.md",
        "references/28_worked_traces.md",
    ]
}

#[test]
fn skill_frontmatter_and_index() {
    let skill = read_skill("SKILL.md");
    assert!(skill.starts_with("---\n"), "frontmatter");
    assert!(skill.contains("name: rhwp-cli"));
    for token in [
        "export-svg",
        "export-png",
        "export-pdf",
        "export-text",
        "export-markdown",
        "dump-pages",
        "dump-records",
        "diag",
        "info",
        "export-render-tree",
        "ir-diff",
        "thumbnail",
        "convert",
        "hwp5-inventory-diff",
        "--debug-overlay",
        "HWPUNIT",
        "oracle",
        "generated",
        "gym",
        "새 CLI",
    ] {
        assert!(skill.contains(token), "SKILL.md 에 {token} 없음");
    }
    let idx = read_json("fixtures/skill_index.json");
    assert_eq!(idx["skill"], "rhwp-cli");
    assert_eq!(idx["issue"], 5316);
    assert_eq!(idx["gym"], false);
    assert_eq!(idx["newCli"], false);
    assert_eq!(idx["pageZeroBased"], true);
    assert_eq!(idx["selfRoundTripIsNotHangul"], true);
    let refs = idx["references"].as_array().unwrap();
    assert!(refs.len() >= 29, "레퍼런스 29장: {}", refs.len());
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
    let path = repo().join("mydocs/working/archives/agent_cli.md");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    assert!(text.contains("5316"), "이슈 번호");
    assert!(text.contains("export-svg"));
    assert!(text.contains("--debug-overlay"));
    assert!(text.contains("oracle"));
    assert!(text.contains("gym"));
    assert!(text.contains("새 CLI") || text.contains("새 서브커맨드"));
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

#[test]
fn gym_dir_untouched_by_skill_text() {
    let skill = read_skill("SKILL.md");
    assert!(
        skill.contains("gym 이 아니다") || skill.contains("gym/"),
        "gym 비범위"
    );
    assert!(
        !skill.contains("gym run") && !skill.contains("gym pack"),
        "gym 실행을 시키지 말 것"
    );
}
