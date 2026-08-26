//! [#5296] 반덤프 규칙 — 긴 문서에서 전문/전쪽 렌더를 스킬이 금지하는가.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn skill_text() -> String {
    let dir = repo_root().join(".claude/skills/rhwp-doc-triage");
    let mut all = fs::read_to_string(dir.join("SKILL.md")).unwrap();
    for name in [
        "00_tree.md",
        "07_when_to_stop.md",
        "10_context_budget.md",
        "15_anti_dump.md",
        "18_pagecount_routing.md",
    ] {
        all.push('\n');
        all.push_str(&fs::read_to_string(dir.join("references").join(name)).unwrap());
    }
    all
}

#[test]
fn long_doc_paths_forbid_unlimited_export_text() {
    let text = skill_text();
    assert!(text.contains("export-text 무제한") || text.contains("무제한"));
    assert!(text.contains("digest --pages 0..last") || text.contains("0..마지막"));
}

#[test]
fn huge_band_requires_limit_on_search() {
    let text = skill_text();
    assert!(text.contains("search --limit"));
}

#[test]
fn stop_s15_is_documented() {
    let text = skill_text();
    assert!(text.contains("S15"));
    assert!(
        text.contains("의례")
            || text.contains("강제 순회 아님")
            || text.contains("강제 순회가 아니다")
    );
}

#[test]
fn skill_declares_read_only_and_not_gym() {
    let skill =
        fs::read_to_string(repo_root().join(".claude/skills/rhwp-doc-triage/SKILL.md")).unwrap();
    assert!(skill.contains("읽기 전용"));
    assert!(skill.to_ascii_lowercase().contains("gym"));
    assert!(skill.contains("새 CLI") || skill.contains("새 서브"));
}
