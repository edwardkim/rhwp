//! [#5308] rhwp-work-receipt 스킬 — 실사용 에이전트가 replay/audit/lineage
//! 로 노동을 증명하는지 기계 가드.
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
    repo().join(".claude/skills/rhwp-work-receipt")
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
        "references/replay-attest.md",
        "references/capsule-chain.md",
        "references/audit-accounting.md",
        "references/lineage-chronicle.md",
        "references/exit-codes.md",
        "references/pitfalls.md",
        "references/decision-tree.md",
        "references/envelope-field-catalog.md",
        "references/recipe-index.md",
    ]
}

fn is_sha256(s: &str) -> bool {
    s.len() == 64
        && s.bytes().all(|b| b.is_ascii_hexdigit())
        && s.bytes().all(|b| !b.is_ascii_uppercase())
}

#[test]
fn skill_layout_has_required_files() {
    let skill = read_skill("SKILL.md");
    assert!(
        skill.contains("name: rhwp-work-receipt"),
        "SKILL.md frontmatter 에 name 이 없습니다"
    );
    for token in [
        "inputSha256",
        "planSha256",
        "outputSha256",
        "--expect-output-sha256",
        "--capsule",
        "--parent",
        "reproducedRate",
        "parentOk",
        "lineageOk",
        "brokenAt",
        "exit 3",
        "toolVersion",
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
    assert_eq!(catalog["skill"], "rhwp-work-receipt");
    assert_eq!(catalog["issue"], 5308);
    assert_eq!(catalog["attributionClaim"], false);
    assert_eq!(catalog["signatureClaim"], false);
    assert_eq!(catalog["auditRecursive"], false);
    assert_eq!(catalog["auditGlob"], "*.capsule.json");
    let commands: Vec<&str> = catalog["commands"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert_eq!(commands, ["replay", "audit", "lineage"]);
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
fn capsules_are_self_consistent_and_children_keep_lineage() {
    let dir = skill_dir().join("fixtures/capsules");
    let mut seen = 0usize;
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if !name.ends_with(".capsule.json") {
            continue;
        }
        if name.starts_with("tamper_") || name.starts_with("toolversion_") {
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
    assert!(seen >= 24, "루트 캡슐이 너무 적습니다: {seen}");
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
    }
}

#[test]
fn audit_layouts_are_non_recursive() {
    let nested = skill_dir().join("fixtures/audit-layouts/nested-ignored");
    let top: Vec<_> = fs::read_dir(&nested)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().map(|t| t.is_file()).unwrap_or(false)
                && e.file_name().to_string_lossy().ends_with(".capsule.json")
        })
        .collect();
    assert_eq!(top.len(), 1, "직속 캡슐은 1개여야 합니다");
    assert!(nested.join("nested/hidden.capsule.json").is_file());

    let empty = skill_dir().join("fixtures/audit-layouts/empty");
    let empty_caps: Vec<_> = fs::read_dir(&empty)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".capsule.json"))
        .collect();
    assert!(empty_caps.is_empty());

    let mixed = read_json("fixtures/envelopes/audit_mixed.json");
    let total = mixed["total"].as_f64().unwrap();
    let reproduced = mixed["reproduced"].as_f64().unwrap();
    let rate = mixed["reproducedRate"].as_f64().unwrap();
    assert!((rate - reproduced / total).abs() < 1e-9);
    assert_eq!(mixed["_skillMeta"]["exit"], 3);
}

#[test]
fn relative_subdir_parent_is_resolved_from_capsule_file() {
    let child = read_json("fixtures/lineage-layouts/relative-subdir/child/b.capsule.json");
    assert_eq!(child["parent"]["capsule"], "../root/a.capsule.json");
    let parent_path =
        skill_dir().join("fixtures/lineage-layouts/relative-subdir/root/a.capsule.json");
    assert!(parent_path.is_file(), "상대 경로 부모가 있어야 합니다");
    assert!(is_sha256(child["parent"]["sha256"].as_str().unwrap_or("")));
}

#[test]
fn envelope_exits_are_only_known_codes() {
    let dir = skill_dir().join("fixtures/envelopes");
    let mut seen_three = false;
    let mut seen_one = false;
    let mut seen_two = false;
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
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
        assert!(
            matches!(cmd, "replay" | "audit" | "lineage" | "run"),
            "발명 명령: {cmd}"
        );
    }
    assert!(
        seen_three && seen_one && seen_two,
        "exit 1/2/3 표본이 모두 있어야 합니다"
    );
}

#[test]
fn scenario_catalog_does_not_invent_commands() {
    let cat = read_json("fixtures/scenario_catalog.json");
    assert!(cat["count"].as_u64().unwrap() >= 80);
    let allowed: BTreeSet<&str> = ["replay", "audit", "lineage", "run"].into_iter().collect();
    for sc in cat["scenarios"].as_array().unwrap() {
        if let Some(cmd) = sc["command"].as_str() {
            assert!(allowed.contains(cmd), "{} 발명 명령 {cmd}", sc["id"]);
        }
    }
}

#[test]
fn working_doc_records_the_issue_and_scope() {
    let path = repo().join("mydocs/working/agent_work_receipt.md");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    assert!(text.contains("#5308") || text.contains("5308"), "이슈 번호");
    assert!(text.contains("replay"), "replay");
    assert!(text.contains("audit"), "audit");
    assert!(text.contains("lineage"), "lineage");
    assert!(text.contains("gym"), "gym 비범위");
    assert!(
        text.contains("새 CLI") || text.contains("새 명령을 만들지 않"),
        "CLI 비범위"
    );
    assert!(text.contains("toolVersion"), "버전 함정");
}

#[test]
fn attest_and_verify_envelopes_teach_three_hashes() {
    let attest = read_json("fixtures/envelopes/replay_attest.json");
    assert_eq!(attest["mode"], "attest");
    assert!(attest["reproduced"].is_null());
    for key in ["inputSha256", "planSha256", "outputSha256"] {
        assert!(is_sha256(attest[key].as_str().unwrap()), "{key}");
    }
    let verify = read_json("fixtures/envelopes/replay_verify_mismatch.json");
    assert_eq!(verify["mode"], "verify");
    assert_eq!(verify["reproduced"], false);
    assert_eq!(verify["_skillMeta"]["exit"], 3);
}
