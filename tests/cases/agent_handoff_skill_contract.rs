//! [#5339] rhwp-handoff 스킬 — 실사용 에이전트가 세션 사이에
//! orchestrator.py + replay --capsule/--parent 로 인수인계하는지 기계 가드.
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
    repo().join(".claude/skills/rhwp-handoff")
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
        "references/when-to-handoff.md",
        "references/orchestrator-protocol.md",
        "references/artifacts.md",
        "references/result-json.md",
        "references/journal-chain.md",
        "references/incoming-agent.md",
        "references/capsule-parent-chain.md",
        "references/work-receipt-boundary.md",
        "references/isolation-worktree.md",
        "references/staging-named-files.md",
        "references/no-documentcore.md",
        "references/exception-index.md",
        "references/exception-missing-capsule.md",
        "references/exception-parent-hash.md",
        "references/exception-dirty-worktree.md",
        "references/exception-disk-full.md",
        "references/exit-codes.md",
        "references/pitfalls.md",
        "references/decision-tree.md",
        "references/recipe-index.md",
        "references/envelope-field-catalog.md",
    ]
}

fn is_sha256(s: &str) -> bool {
    s.len() == 64
        && s.bytes().all(|b| b.is_ascii_hexdigit())
        && s.bytes().all(|b| !b.is_ascii_uppercase())
}

const HARD_GATE: &str = "cargo fmt --all -- --check";
const STALE_FMT: &str = "cargo fmt --check";

#[test]
fn skill_layout_has_required_files() {
    let skill = read_skill("SKILL.md");
    assert!(
        skill.contains("name: rhwp-handoff"),
        "SKILL.md frontmatter 에 name 이 없습니다"
    );
    for token in [
        "tools/handoff/orchestrator.py",
        "result.json",
        "--capsule",
        "--parent",
        "context budget",
        "session interrupt",
        "seat refill",
        "git add -A",
        "DocumentCore",
        "새 CLI",
        "gym",
        "exit 3",
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
        ".claude/skills/rhwp-onboarding/SKILL.md",
        ".claude/skills/rhwp-mcp-session/SKILL.md",
        ".claude/skills/rhwp-provenance/SKILL.md",
        ".claude/skills/rhwp-safe-edit/SKILL.md",
        ".claude/skills/rhwp-doc-triage/SKILL.md",
        ".claude/skills/rhwp-work-receipt/SKILL.md",
    ] {
        assert!(
            repo().join(forbidden).is_file(),
            "다른 스킬을 지우지 마세요: {forbidden}"
        );
    }
    assert!(
        repo().join("scripts/tests/test_agent_handoff.py").is_file(),
        "기존 오케스트레이터 시험을 지우지 마세요"
    );
    assert!(
        repo().join("tools/handoff/orchestrator.py").is_file(),
        "orchestrator.py 가 있어야 합니다"
    );
}

#[test]
fn catalog_matches_disk_and_claims() {
    let catalog = read_json("fixtures/catalog.json");
    assert_eq!(catalog["skill"], "rhwp-handoff");
    assert_eq!(catalog["issue"], 5339);
    assert_eq!(catalog["hardGate"], HARD_GATE);
    assert_eq!(catalog["staleFmt"], STALE_FMT);
    assert_eq!(catalog["neverGitAddA"], true);
    assert_eq!(catalog["neverStealNamedWorktrees"], true);
    assert_eq!(catalog["neverInventDocumentCore"], true);
    assert_eq!(catalog["noNewCli"], true);
    assert_eq!(catalog["receiptIsSingleJobProof"], true);
    assert_eq!(catalog["sessionHandoffIsNotReceipt"], true);
    assert_eq!(catalog["receiptSkill"], "rhwp-work-receipt");
    assert_eq!(catalog["orchestrator"], "tools/handoff/orchestrator.py");
    assert_eq!(catalog["base"], "devel");
    assert_eq!(catalog["newlineStyle"], "Unix");
    assert_eq!(catalog["gym"], false);
    let triggers: Vec<&str> = catalog["triggers"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert_eq!(
        triggers,
        ["context_budget", "session_interrupt", "seat_refill"]
    );
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
}

#[test]
fn rustfmt_toml_is_unix_and_gate_docs_agree() {
    let toml = fs::read_to_string(repo().join("rustfmt.toml")).expect("rustfmt.toml");
    assert!(
        toml.contains("newline_style = \"Unix\""),
        "rustfmt newline_style=Unix"
    );
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
fn capsules_are_self_consistent_and_children_keep_lineage() {
    let dir = skill_dir().join("fixtures/capsules");
    let mut seen = 0usize;
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if !name.ends_with(".capsule.json") || !name.starts_with('s') {
            continue;
        }
        let cap: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(cap["kind"], "workCapsule", "{name}");
        let plan_text = cap["planText"].as_str().expect("planText");
        assert!(
            is_sha256(cap["receipt"]["planSha256"].as_str().unwrap_or("")),
            "{name} planSha256"
        );
        let parsed: Value = serde_json::from_str(plan_text).unwrap();
        assert_eq!(cap["plan"], parsed, "{name} plan vs planText");
        let steps = cap["plan"]["steps"].as_array().unwrap().len() as u64;
        assert_eq!(
            cap["receipt"]["steps"].as_u64(),
            Some(steps),
            "{name} steps"
        );
        for key in ["inputSha256", "planSha256", "outputSha256"] {
            let hex = cap["receipt"][key].as_str().unwrap();
            assert!(is_sha256(hex), "{name} {key} 가 소문자 64hex 가 아님");
        }
        if let Some(parent) = cap.get("parent") {
            if !parent.is_null() {
                let rel = parent["capsule"].as_str().expect("parent.capsule");
                assert!(
                    !Path::new(rel).is_absolute(),
                    "{name} parent 경로는 상대여야 합니다: {rel}"
                );
                assert!(
                    is_sha256(parent["sha256"].as_str().unwrap_or("")),
                    "{name} parent.sha256"
                );
            }
        }
        seen += 1;
    }
    assert!(seen >= 24, "세션 캡슐이 너무 적습니다: {seen}");
}

#[test]
fn child_input_hash_equals_parent_output_hash() {
    let index = read_json("fixtures/capsule_index.json");
    let mut parent_out = std::collections::BTreeMap::new();
    for root in index["roots"].as_array().unwrap() {
        parent_out.insert(
            root["file"].as_str().unwrap().to_string(),
            root["outputSha256"].as_str().unwrap().to_string(),
        );
    }
    for child in index["children"].as_array().unwrap() {
        let parent = child["parent"].as_str().unwrap();
        let expect = parent_out
            .get(parent)
            .unwrap_or_else(|| panic!("부모 없음: {parent}"));
        assert_eq!(
            child["inputSha256"].as_str(),
            Some(expect.as_str()),
            "{} lineageOk",
            child["file"]
        );
        assert_eq!(child["lineageOk"], true);
        assert_eq!(child["parentPathRelativeToCapsuleFile"], true);
        parent_out.insert(
            child["file"].as_str().unwrap().to_string(),
            child["outputSha256"].as_str().unwrap().to_string(),
        );
    }
}

#[test]
fn envelope_exits_are_only_known_codes() {
    let dir = skill_dir().join("fixtures/envelopes");
    let mut seen_three = false;
    let mut seen_one = false;
    let mut seen_two = false;
    let allowed_cmd: BTreeSet<&str> = ["git", "python", "rhwp", "read", "cargo"]
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
            matches!(code, 0..=4),
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
        assert_eq!(env["_skillMeta"]["neverGitAddA"], true);
    }
    assert!(
        seen_three && seen_one && seen_two,
        "exit 1/2/3 표본이 모두 있어야 합니다"
    );
}

#[test]
fn git_add_a_and_named_checkout_are_rejected() {
    let env = read_json("fixtures/envelopes/git_add_a_rejected.json");
    assert_eq!(env["command"], "git add -A");
    assert_eq!(env["rejected"], true);
    let named = read_json("fixtures/envelopes/named_worktree_checkout_rejected.json");
    assert_eq!(named["rejected"], true);
    let core = read_json("fixtures/envelopes/documentcore_invented.json");
    assert_eq!(core["rejected"], true);
}

#[test]
fn orchestrator_accepted_and_boundary_shapes() {
    let acc = read_json("fixtures/envelopes/orch_accepted.json");
    assert_eq!(acc["protocol"], "DAP/1.0");
    assert_eq!(acc["operation"], "agent.handoff");
    assert_eq!(acc["tool"], "rhwp-handoff-orchestrator");
    assert_eq!(acc["outcome"], "accepted");
    assert_eq!(acc["nextAction"]["action"], "consume");
    assert_eq!(acc["untrustedContent"], true);
    let bd = read_json("fixtures/envelopes/orch_boundary.json");
    assert_eq!(bd["code"], 4000);
    assert_eq!(bd["outcome"], "rejected");
    assert_eq!(bd["attempts"].as_array().unwrap().len(), 1);
}

#[test]
fn exception_exits_match_the_skill_table() {
    assert_eq!(
        read_json("fixtures/exceptions/missing_capsule.json")["_skillMeta"]["exit"],
        1
    );
    assert_eq!(
        read_json("fixtures/exceptions/parent_hash_mismatch.json")["_skillMeta"]["exit"],
        3
    );
    assert_eq!(
        read_json("fixtures/exceptions/dirty_named_worktree.json")["_skillMeta"]["exit"],
        2
    );
    assert_eq!(
        read_json("fixtures/exceptions/disk_full.json")["_skillMeta"]["exit"],
        1
    );
}

#[test]
fn incoming_reads_three_files_in_order() {
    let order = read_json("fixtures/incoming/read-order.json");
    let files: Vec<&str> = order["order"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert_eq!(
        files,
        [
            "last result.json",
            "last *.capsule.json",
            "last working doc"
        ]
    );
}

#[test]
fn scenario_catalog_does_not_invent_commands() {
    let cat = read_json("fixtures/scenario_catalog.json");
    assert!(cat["count"].as_u64().unwrap() >= 80);
    let allowed: BTreeSet<&str> = [
        "git", "python", "rhwp", "read", "cargo", "replay", "audit", "lineage", "run",
    ]
    .into_iter()
    .collect();
    for sc in cat["scenarios"].as_array().unwrap() {
        if let Some(cmd) = sc["command"].as_str() {
            assert!(allowed.contains(cmd), "{} 발명 명령 {cmd}", sc["id"]);
        }
        assert_eq!(sc["hardGate"], HARD_GATE);
        assert_eq!(sc["noNewCli"], true);
    }
}

#[test]
fn working_doc_records_the_issue_and_scope() {
    let path = repo().join("mydocs/working/agent_handoff.md");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    assert!(text.contains("#5339") || text.contains("5339"), "이슈 번호");
    assert!(text.contains(HARD_GATE), "HARD GATE");
    assert!(text.contains("gym"), "gym 비범위");
    assert!(
        text.contains("새 CLI") || text.contains("새 명령을 만들지 않"),
        "CLI 비범위"
    );
    assert!(text.contains("DocumentCore"), "DocumentCore 금지");
    assert!(text.contains("git add -A"), "add -A 금지");
    assert!(text.contains("orchestrator.py"), "오케스트레이터");
    assert!(text.contains("replay"), "캡슐 포인터");
    assert!(text.contains("5000"), "줄 수 목표");
}

#[test]
fn work_receipt_is_pointer_only() {
    let pointers = read_skill("references/work-receipt-boundary.md");
    assert!(pointers.contains("rhwp-work-receipt"));
    assert!(pointers.contains("단건"));
    assert!(
        pointers.contains("다시 쓰") || pointers.contains("복제하지"),
        "영수증 스킬 재작성 금지"
    );
}
