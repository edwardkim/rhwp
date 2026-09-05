//! [#5326] 에이전트 표면 스킬 계약.
//!
//! 바이너리를 부르지 않는다 — 이 PR 은 새 CLI 를 만들지 않고, 스킬·픽스처·
//! 작업 기록이 플레이북의 3층·3규칙·예외 4바늘을 가리키는지만 파일로 고정한다.
//! 도구 이름의 실재는 `scripts/tests/test_agent_surface.py` 가 소스에서 추출해
//! 대조한다.

#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn skill_dir() -> PathBuf {
    root()
        .join(".agents")
        .join("skills")
        .join("rhwp-agent-surface")
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("{} 읽기 실패: {e}", path.display()))
}

#[test]
fn skill_tree_exists_with_frontmatter() {
    let dir = skill_dir();
    let skill = read(&dir.join("SKILL.md"));
    assert!(skill.starts_with("---\n"), "frontmatter 시작 없음");
    assert!(
        skill.contains("name: rhwp-agent-surface"),
        "frontmatter name 불일치"
    );
    assert!(
        skill.contains("rhwp capabilities"),
        "실행 가능한 rhwp capabilities 참조가 없다"
    );
    for sub in ["references", "examples", "fixtures"] {
        assert!(dir.join(sub).is_dir(), "없는 폴더: {sub}");
    }
}

#[test]
fn skill_states_three_layers_and_three_rules() {
    let skill = read(&skill_dir().join("SKILL.md"));
    for needle in [
        "CLI `--json`",
        "MCP 무상태",
        "MCP 세션",
        "mcp_tool_definitions",
        "규칙 1",
        "규칙 2",
        "규칙 3",
        "identical:false",
        "isError",
        "capabilities --mcp",
        "capabilities --search",
    ] {
        assert!(skill.contains(needle), "SKILL.md 에 없음: {needle}");
    }
}

#[test]
fn skill_does_not_point_at_gym_path() {
    let skill = read(&skill_dir().join("SKILL.md"));
    assert!(
        !skill.to_ascii_lowercase().contains("gym/"),
        "SKILL.md 가 gym/ 경로를 실행 경로로 가리킨다"
    );
}

#[test]
fn skill_declares_boundary_against_sibling_skills() {
    let skill = read(&skill_dir().join("SKILL.md"));
    for sibling in ["rhwp-mcp-session", "rhwp-cli", "rhwp-codex"] {
        assert!(skill.contains(sibling), "경계 안내에 {sibling} 없음");
    }
    assert!(
        !skill.contains("\"mcpServers\""),
        "호스트 부착 JSON 을 이 스킬에 복제하지 말 것"
    );
}

#[test]
fn reference_and_example_files_exist() {
    let dir = skill_dir();
    let refs = [
        "three_layers.md",
        "rule1_single_source.md",
        "rule2_reuse_core.md",
        "rule3_judgment_is_data.md",
        "capabilities_how_to.md",
        "add_surface_piece.md",
        "acceptance_checklist.md",
        "exception_paths.md",
        "drift_guards.md",
        "forbidden_overlap.md",
    ];
    for name in refs {
        let path = dir.join("references").join(name);
        let body = read(&path);
        assert!(
            body.lines().count() > 10,
            "{name} 이 너무 짧다 ({})",
            body.lines().count()
        );
    }
    let examples = [
        "consume_capabilities.md",
        "add_json_command.md",
        "add_stateless_tool.md",
        "add_session_tool.md",
        "closed_handle_recovery.md",
        "profile_blocked.md",
        "drift_guard_fail.md",
        "missing_capabilities_key.md",
        "judgment_is_data.md",
    ];
    for name in examples {
        assert!(
            dir.join("examples").join(name).is_file(),
            "없는 레시피: {name}"
        );
    }
}

#[test]
fn fixtures_cover_layers_rules_exceptions() {
    let fix = skill_dir().join("fixtures");
    for name in [
        "layers.json",
        "rules.json",
        "allowlist.json",
        "exceptions/missing_capabilities_key.json",
        "exceptions/drift_guard_fail.json",
        "exceptions/closed_handle.json",
        "exceptions/profile_blocked.json",
        "add_surface/acceptance.json",
        "envelopes/ir_diff_not_identical.json",
        "envelopes/replace_zero.json",
        "envelopes/fill_not_found.json",
        "drift/capabilities_mcp_covers_every_json_command.json",
        "transcripts/capabilities_bare.json",
        "transcripts/capabilities_mcp.json",
        "session/hwp_open.json",
        "session/hwp_close.json",
        "session/hwp_doc_save.json",
    ] {
        assert!(fix.join(name).is_file(), "없는 픽스처: {name}");
    }
}

#[test]
fn closed_handle_fixture_points_at_hwp_open() {
    let text = read(
        &skill_dir()
            .join("fixtures")
            .join("exceptions")
            .join("closed_handle.json"),
    );
    assert!(
        text.contains("hwp_open"),
        "닫힌 핸들 복구가 hwp_open 이 아님"
    );
    assert!(text.contains("isError"), "닫힌 핸들은 isError 층");
}

#[test]
fn working_doc_closes_issue_5326() {
    let text = read(
        &root()
            .join("mydocs")
            .join("working")
            .join("archives")
            .join("agent_surface_skill.md"),
    );
    assert!(text.contains("#5326"), "작업 기록이 이슈를 가리키지 않음");
    assert!(text.contains("mcp_tool_definitions"), "SSOT 언급 없음");
    assert!(text.contains("feat/agent-surface"), "브랜치 기록 없음");
}

#[test]
fn playbook_remains_the_canonical_doc() {
    let text = read(
        &root()
            .join("mydocs")
            .join("manual")
            .join("agent_surface_playbook.md"),
    );
    assert!(text.contains("canonical: mydocs/manual/agent_surface_playbook.md"));
    assert!(text.contains("규칙 1"));
    assert!(text.contains("규칙 2"));
    assert!(text.contains("규칙 3"));
}
