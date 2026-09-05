//! [#5322] rhwp-contributor 스킬 — 실사용 에이전트가 공식 8단
//! (이슈→분석→브랜치→구현→게이트→영수증→문서→한국어 PR) 을
//! 규약대로 닫는지 기계 가드.
//!
//! 새 CLI 를 만들지 않는다. 바이너리를 부르지 않는다. 픽스처와 문서만 읽는다.

#![cfg(not(target_arch = "wasm32"))]

use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn skill_dir() -> PathBuf {
    repo().join(".agents/skills/rhwp-contributor")
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
        "references/procedure-order.md",
        "references/issue-first.md",
        "references/analyze-canonical.md",
        "references/branch-isolation.md",
        "references/isolation-worktree.md",
        "references/implement-scope.md",
        "references/staging-named-files.md",
        "references/fmt-hard-gate.md",
        "references/rustfmt-unix.md",
        "references/clippy-and-tests.md",
        "references/visual-evidence.md",
        "references/work-receipt-pointers.md",
        "references/working-doc.md",
        "references/korean-pr.md",
        "references/pr-template-checkboxes.md",
        "references/exceptions.md",
        "references/pitfalls.md",
        "references/decision-tree.md",
        "references/recipe-index.md",
        "references/command-field-catalog.md",
    ]
}

const HARD_GATE: &str = "cargo fmt --all -- --check";
const STALE_FMT: &str = "cargo fmt --check";

#[test]
fn skill_layout_has_required_files() {
    let skill = read_skill("SKILL.md");
    assert!(
        skill.contains("name: rhwp-contributor"),
        "SKILL.md frontmatter 에 name 이 없습니다"
    );
    for token in [
        HARD_GATE,
        "cargo clippy -- -D warnings",
        "upstream/devel",
        "git add -A",
        "DocumentCore",
        "replay --capsule",
        "audit",
        "lineage",
        "mydocs/working",
        "closes #",
        "--body-file",
        "첫 체크박스",
        "newline_style",
        "Unix",
        "sparse",
        "autocrlf",
        "noci",
        "FAILURE",
        "gym",
        "새 CLI",
        "isolation",
    ] {
        assert!(skill.contains(token), "SKILL.md 에 {token} 가 없습니다");
    }
    for rel in required_references() {
        assert!(skill_dir().join(rel).is_file(), "필수 레퍼런스 누락: {rel}");
        let name = Path::new(rel)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        assert!(
            skill.contains(&name),
            "SKILL.md 가 {name} 를 가리켜야 합니다"
        );
    }
    assert!(
        skill_dir().join("fixtures/catalog.json").is_file(),
        "catalog.json 누락"
    );
}

#[test]
fn hard_gate_is_fmt_all_check_not_stale_wording() {
    let skill = read_skill("SKILL.md");
    assert!(skill.contains(HARD_GATE), "HARD GATE 정본 명령");
    assert!(
        skill.contains("낡은") || skill.contains("아님"),
        "낡은 cargo fmt --check 를 거절하는 문장이 있어야 합니다"
    );
    let gate = read_skill("references/fmt-hard-gate.md");
    assert!(gate.contains(HARD_GATE));
    assert!(gate.contains(STALE_FMT));
    assert!(gate.contains("낡은"));
    let first = skill.find(HARD_GATE).expect("hard gate");
    let stale = skill.find(STALE_FMT).expect("stale mentioned");
    let _ = (first, stale);
}

#[test]
fn skill_does_not_add_cli_and_stays_out_of_other_skills() {
    let skill = read_skill("SKILL.md");
    assert!(
        skill.contains("새 CLI") || skill.contains("새 명령을 만들지 않는다"),
        "SKILL.md 가 새 CLI 를 만들지 않는다고 밝혀야 합니다"
    );
    assert!(skill.contains("gym"), "gym 비범위를 밝혀야 합니다");
    assert!(
        skill.contains("DocumentCore"),
        "DocumentCore 발명 금지를 밝혀야 합니다"
    );
    let cargo = fs::read_to_string(repo().join("Cargo.toml")).expect("Cargo.toml");
    let bins = cargo.matches("[[bin]]").count();
    assert_eq!(
        bins, 2,
        "새 [[bin]] 을 추가하지 마세요 (rhwp, font-metric-gen 만): {bins}"
    );
    for forbidden in [
        ".agents/skills/rhwp-onboarding/SKILL.md",
        ".agents/skills/rhwp-mcp-session/SKILL.md",
        ".agents/skills/rhwp-provenance/SKILL.md",
        ".agents/skills/rhwp-safe-edit/SKILL.md",
        ".agents/skills/rhwp-doc-triage/SKILL.md",
        ".agents/skills/rhwp-work-receipt/SKILL.md",
    ] {
        assert!(
            repo().join(forbidden).is_file(),
            "다른 스킬을 지우지 마세요: {forbidden}"
        );
    }
}

#[test]
fn catalog_matches_disk_and_claims() {
    let catalog = read_json("fixtures/catalog.json");
    assert_eq!(catalog["skill"], "rhwp-contributor");
    assert_eq!(catalog["issue"], 5322);
    assert_eq!(catalog["hardGate"], HARD_GATE);
    assert_eq!(catalog["staleFmt"], STALE_FMT);
    assert_eq!(catalog["neverGitAddA"], true);
    assert_eq!(catalog["neverStealNamedWorktrees"], true);
    assert_eq!(catalog["neverInventDocumentCore"], true);
    assert_eq!(catalog["noNewCli"], true);
    assert_eq!(catalog["base"], "devel");
    assert_eq!(catalog["firstPrCheckbox"], HARD_GATE);
    assert_eq!(catalog["newlineStyle"], "Unix");
    for rel in catalog["envelopes"].as_array().unwrap() {
        let name = rel.as_str().unwrap();
        assert!(
            skill_dir().join("fixtures/envelopes").join(name).is_file(),
            "봉투 누락: {name}"
        );
    }
    for rel in catalog["examples"].as_array().unwrap() {
        let name = rel.as_str().unwrap();
        assert!(
            skill_dir().join("examples").join(name).is_file(),
            "예제 누락: {name}"
        );
    }
    for rel in catalog["references"].as_array().unwrap() {
        let name = rel.as_str().unwrap();
        assert!(
            skill_dir().join("references").join(name).is_file(),
            "레퍼런스 누락: {name}"
        );
    }
}

#[test]
fn rustfmt_toml_is_unix_and_gate_docs_agree() {
    let toml = fs::read_to_string(repo().join("rustfmt.toml")).expect("rustfmt.toml");
    assert!(
        toml.contains("newline_style = \"Unix\""),
        "rustfmt newline_style=Unix"
    );
    let unix = read_skill("references/rustfmt-unix.md");
    assert!(unix.contains("newline_style = \"Unix\"") || unix.contains("newline_style = Unix"));
    assert!(unix.contains("autocrlf"));
}

#[test]
fn forbidden_worktree_registry_lists_named_paths() {
    let reg = read_json("fixtures/layouts/forbidden-worktrees/registry.json");
    let forbidden: Vec<&str> = reg["forbidden"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    for needle in [
        r"C:\Users\swsz9\rhwp",
        r"C:\Users\swsz9\rhwp-handoff",
        r"C:\Users\swsz9\rhwp-scaffold-final",
        r"C:\Users\swsz9\rhwp-doc-repro",
    ] {
        assert!(
            forbidden.iter().any(|p| p.contains(needle) || *p == needle),
            "금지 목록에 {needle} 없음: {forbidden:?}"
        );
    }
    assert!(
        forbidden.iter().any(|p| p.contains("rhwp-desk")),
        "desk 계열 금지"
    );
    assert_eq!(reg["rule"], "never steal named worktrees");
}

#[test]
fn envelope_exits_are_only_known_codes() {
    let dir = skill_dir().join("fixtures/envelopes");
    let mut seen_three = false;
    let mut seen_one = false;
    let mut seen_two = false;
    let allowed_cmd: BTreeSet<&str> = [
        "git", "gh", "cargo", "python", "node", "rhwp", "replay", "audit", "lineage", "read",
    ]
    .into_iter()
    .collect();
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let env: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let code = env["_skillMeta"]["exit"].as_i64().expect("exit");
        assert!(
            matches!(code, 0..=3),
            "{} 알 수 없는 exit {code}",
            path.display()
        );
        match code {
            3 => seen_three = true,
            1 => seen_one = true,
            2 => seen_two = true,
            _ => {}
        }
        let cmd = env["_skillMeta"]["command"].as_str().unwrap();
        assert!(allowed_cmd.contains(cmd), "발명 명령: {cmd}");
        assert_eq!(env["_skillMeta"]["hardGate"], HARD_GATE);
        assert_eq!(env["_skillMeta"]["staleFmtRejected"], true);
    }
    assert!(
        seen_three && seen_one && seen_two,
        "exit 1/2/3 표본이 모두 있어야 합니다"
    );
}

#[test]
fn noci_is_not_failure_and_failure_is_not_noci() {
    let noci = read_json("fixtures/envelopes/ci_noci.json");
    assert_eq!(noci["classification"], "noci");
    assert_eq!(noci["isFailure"], false);
    assert_eq!(noci["_skillMeta"]["exit"], 0);
    let fail = read_json("fixtures/envelopes/ci_failure.json");
    assert_eq!(fail["classification"], "FAILURE");
    assert_eq!(fail["isNoci"], false);
    assert_eq!(fail["_skillMeta"]["exit"], 3);
}

#[test]
fn stale_fmt_envelope_is_rejected() {
    let env = read_json("fixtures/envelopes/fmt_stale_check_only.json");
    assert_eq!(env["command"], STALE_FMT);
    assert_eq!(env["acceptedAsGate"], false);
    assert_eq!(env["mustUse"], HARD_GATE);
    assert_eq!(env["_skillMeta"]["exit"], 2);
}

#[test]
fn git_add_a_envelope_is_rejected() {
    let env = read_json("fixtures/envelopes/git_add_a_rejected.json");
    assert_eq!(env["command"], "git add -A");
    assert_eq!(env["rejected"], true);
}

#[test]
fn scenario_catalog_does_not_invent_commands() {
    let cat = read_json("fixtures/scenario_catalog.json");
    assert!(cat["count"].as_u64().unwrap() >= 80);
    let allowed: BTreeSet<&str> = [
        "git", "gh", "cargo", "python", "node", "rhwp", "replay", "audit", "lineage", "read",
    ]
    .into_iter()
    .collect();
    for sc in cat["scenarios"].as_array().unwrap() {
        if let Some(cmd) = sc["command"].as_str() {
            assert!(allowed.contains(cmd), "{} 발명 명령 {cmd}", sc["id"]);
        }
        assert_eq!(sc["hardGate"], HARD_GATE);
    }
}

#[test]
fn working_doc_records_the_issue_and_scope() {
    let path = repo().join("mydocs/working/archives/agent_contributor.md");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    assert!(text.contains("#5322") || text.contains("5322"), "이슈 번호");
    assert!(text.contains(HARD_GATE), "HARD GATE");
    assert!(text.contains("gym"), "gym 비범위");
    assert!(
        text.contains("새 CLI") || text.contains("새 명령을 만들지 않"),
        "CLI 비범위"
    );
    assert!(text.contains("DocumentCore"), "DocumentCore 금지");
    assert!(text.contains("git add -A"), "add -A 금지");
    assert!(text.contains("replay"), "영수증 포인터");
}

#[test]
fn pr_body_fixture_closes_issue_and_starts_with_fmt_gate() {
    let body = read_skill("fixtures/pr-bodies/closes_5322.md");
    assert!(body.contains("closes #5322"));
    let checkbox = format!("- [x] `{HARD_GATE}`");
    assert!(
        body.contains(&checkbox) || body.contains(HARD_GATE),
        "본문에 fmt 게이트 칸"
    );
    let first_check = body.find("- [").expect("checkbox");
    let window = &body[first_check..first_check + 80];
    assert!(
        window.contains(HARD_GATE),
        "첫 체크박스가 fmt 게이트여야 합니다: {window}"
    );
}

#[test]
fn work_receipt_is_pointer_only() {
    let pointers = read_skill("references/work-receipt-pointers.md");
    assert!(pointers.contains("rhwp replay"));
    assert!(pointers.contains("--capsule"));
    assert!(pointers.contains("rhwp audit"));
    assert!(pointers.contains("rhwp lineage"));
    assert!(
        pointers.contains("다시 쓰지") || pointers.contains("복제하지"),
        "영수증 스킬 재작성 금지"
    );
}
