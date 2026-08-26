//! [#5335] rhwp-strategist 스킬 — 근거 대장·좌표·§5 게이트 계약.
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
    repo().join(".claude/skills/rhwp-strategist")
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
        "references/01_playbook_authority.md",
        "references/02_engagement_protocol.md",
        "references/03_corpus_map.md",
        "references/04_evidence_ledger.md",
        "references/05_claim_gate.md",
        "references/06_coordinate_rules.md",
        "references/07_search_extract_envelopes.md",
        "references/08_validate_exit.md",
        "references/09_out_of_scope.md",
        "references/10_fde_chief_boundary.md",
        "references/11_sws_audit.md",
        "references/12_pitfalls.md",
        "references/13_decision_tree.md",
        "references/14_recipe_index.md",
        "references/15_envelope_field_catalog.md",
        "references/16_journeys.md",
        "references/17_stop_rules.md",
        "references/18_handoff.md",
        "references/19_failed_document_ledger.md",
        "references/20_question_design.md",
    ]
}

#[test]
fn skill_layout_has_required_files() {
    let skill = read_skill("SKILL.md");
    assert!(
        skill.contains("name: rhwp-strategist"),
        "SKILL.md frontmatter 에 name 이 없습니다"
    );
    for token in [
        "engagement.json",
        "objective",
        "corpus",
        "questions",
        "tools/strategist/engagement.py",
        "evidence.json",
        "corpus_map.json",
        "--validate",
        "section",
        "paragraph",
        "page",
        "charOffset",
        "unlinked",
        "unknown-evidence",
        "placeholder",
        "ST-FORECAST",
        "ST-INVENT-PAGE",
        "ST-DROP-FAILED",
        "ST-GATE-FAIL",
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
        skill.contains("새 rhwp CLI") || skill.contains("새 CLI"),
        "SKILL.md 가 새 CLI 를 만들지 않는다고 밝혀야 합니다"
    );
    assert!(skill.contains("gym"), "gym 비범위를 밝혀야 합니다");
    assert!(
        skill.contains("엔진은 전략을 발명하지 않는다")
            || skill.contains("엔진은 전략을 만들지 않는다"),
        "엔진이 전략을 만들지 않는다고 밝혀야 합니다"
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
        ".claude/skills/rhwp-form-fill/SKILL.md",
        ".claude/agents/rhwp-fde.md",
        ".claude/agents/rhwp-chief.md",
    ] {
        assert!(
            repo().join(forbidden).is_file(),
            "다른 스킬/에이전트를 지우지 마세요: {forbidden}"
        );
    }
}

#[test]
fn catalog_matches_disk_and_claims() {
    let catalog = read_json("fixtures/catalog.json");
    assert_eq!(catalog["skill"], "rhwp-strategist");
    assert_eq!(catalog["issue"], 5335);
    assert_eq!(catalog["capability"], "CAP-4903");
    assert_eq!(catalog["engineDoesNotInventStrategy"], true);
    assert_eq!(catalog["attributionClaim"], false);
    let commands: Vec<&str> = catalog["commands"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    let allowed: BTreeSet<&str> = [
        "info",
        "search",
        "extract-data",
        "explain",
        "scaffold",
        "capabilities",
    ]
    .into_iter()
    .collect();
    for c in &commands {
        assert!(allowed.contains(c), "발명 명령: {c}");
    }
    assert!(commands.contains(&"search"));
    assert!(commands.contains(&"extract-data"));
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
fn tree_fixture_declares_layers_and_gates() {
    let tree = read_json("fixtures/tree.json");
    assert_eq!(tree["notGym"], true);
    assert_eq!(tree["noNewCli"], true);
    assert_eq!(tree["neverInventMissingPage"], true);
    assert_eq!(tree["failedDocsStayFailed"], true);
    assert_eq!(tree["section5Gate"], true);
    assert_eq!(tree["engineDoesNotInventStrategy"], true);
    assert_eq!(tree["layers"]["fde"], "live symptoms");
    assert_eq!(tree["layers"]["chief"], "request queue");
    assert_eq!(tree["layers"]["strategist"], "objective + corpus");
    let out = tree["outOfScope"].as_array().unwrap();
    let joined = out
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        joined.contains("forecast") || joined.contains("전망"),
        "{joined}"
    );
}

#[test]
fn engagement_fixtures_require_objective_corpus_questions() {
    for name in ["gov_rfp.json", "quarterly.json", "mixed_failed.json"] {
        let eng = read_json(&format!("fixtures/engagements/{name}"));
        assert!(
            eng["objective"].as_str().unwrap().chars().count() > 4,
            "{name}"
        );
        assert!(eng["corpus"].as_str().unwrap().contains("corpus"), "{name}");
        let qs = eng["questions"].as_array().expect(name);
        assert!(!qs.is_empty(), "{name} questions");
    }
    let missing_q = read_json("fixtures/engagements/invalid_missing_questions.json");
    assert!(missing_q.get("questions").is_none());
    let empty = read_json("fixtures/engagements/invalid_empty_questions.json");
    assert!(empty["questions"].as_array().unwrap().is_empty());
}

#[test]
fn search_envelope_omits_missing_page() {
    let env = read_json("fixtures/envelopes/search_missing_page.json");
    let m = &env["matches"][0];
    assert!(
        m.get("page").is_none(),
        "없는 page 를 발명하면 안 됩니다: {m}"
    );
    assert!(m.get("section").is_some());
    assert!(m.get("paragraph").is_some());
    assert!(m.get("charOffset").is_some());
    let with_page = read_json("fixtures/envelopes/search_with_page.json");
    assert!(with_page["matches"][0].get("page").is_some());
}

#[test]
fn ledger_copy_coords_never_invents_page() {
    let missing = read_json("fixtures/ledgers/gov_rfp_missing_page.json");
    let mut saw_omitted = false;
    for e in missing["entries"].as_array().unwrap() {
        if e["id"] == "EV-2" {
            assert!(
                e.get("page").is_none(),
                "EV-2 는 page 를 생략해야 합니다: {e}"
            );
            saw_omitted = true;
        }
        if let Some(page) = e.get("page") {
            assert!(page.is_u64() || page.is_i64(), "page 는 정수: {page}");
        }
    }
    assert!(saw_omitted, "page 생략 표본이 없습니다");
    assert_eq!(missing["generatedBy"], "tools/strategist/engagement.py");
    assert_eq!(
        missing["entryCount"].as_u64().unwrap(),
        missing["entries"].as_array().unwrap().len() as u64
    );
}

#[test]
fn failed_documents_stay_in_corpus_map() {
    let map = read_json("fixtures/corpus_maps/mixed_failed.json");
    let docs = map["documents"].as_array().unwrap();
    assert_eq!(map["documentCount"].as_u64().unwrap(), docs.len() as u64);
    let failed: Vec<_> = docs.iter().filter(|d| d["status"] == "failed").collect();
    assert!(failed.len() >= 2, "실패 문서가 사라졌습니다: {docs:?}");
    let ok = docs.iter().filter(|d| d["status"] == "ok").count() as u64;
    assert_eq!(map["mappedCount"].as_u64().unwrap(), ok);
    assert!(ok < map["documentCount"].as_u64().unwrap());
    for d in &failed {
        assert!(d.get("info").is_none(), "failed 행에 info 를 채우지 마세요");
        assert!(d.get("infoExit").is_some(), "infoExit 가 없습니다");
    }
}

#[test]
fn section5_gate_kinds_are_complete() {
    let mut kinds = BTreeSet::new();
    for name in [
        "pass.json",
        "placeholder.json",
        "unknown_evidence.json",
        "unlinked.json",
        "mixed_violations.json",
    ] {
        let v = read_json(&format!("fixtures/validate/{name}"));
        assert_eq!(v["mode"], "validate");
        if v["verdict"] == "pass" {
            assert_eq!(v["violationCount"], 0);
            assert_eq!(v["_skillMeta"]["exit"], 0);
        } else {
            assert_eq!(v["verdict"], "fail");
            assert_eq!(v["_skillMeta"]["exit"], 3);
            for viol in v["violations"].as_array().unwrap() {
                kinds.insert(viol["kind"].as_str().unwrap().to_string());
            }
        }
    }
    for need in ["placeholder", "unknown-evidence", "unlinked"] {
        assert!(
            kinds.contains(need),
            "게이트 kind 누락: {need} in {kinds:?}"
        );
    }
}

#[test]
fn extract_data_amount_keeps_normalized() {
    let env = read_json("fixtures/envelopes/extract_amount.json");
    let item = &env["items"][0];
    assert_eq!(item["kind"], "amount");
    assert_eq!(item["raw"], "3,180백만원");
    assert_eq!(item["normalized"], 3180000000_i64);
    assert_eq!(item["currency"], "KRW");
    for key in ["section", "paragraph", "page", "charOffset"] {
        assert!(item.get(key).is_some(), "{key}");
    }
}

#[test]
fn stop_rule_ids_appear_in_docs() {
    let stop_doc = read_skill("references/17_stop_rules.md");
    let skill = read_skill("SKILL.md");
    let rules = read_json("fixtures/stop_rules.json");
    for rule in rules["rules"].as_array().unwrap() {
        let id = rule["id"].as_str().unwrap();
        assert!(
            stop_doc.contains(id) || skill.contains(id),
            "정지 장에 {id} 없음"
        );
    }
}

#[test]
fn scenario_catalog_does_not_invent_commands() {
    let cat = read_json("fixtures/scenario_catalog.json");
    assert!(cat["count"].as_u64().unwrap() >= 24);
    let allowed: BTreeSet<&str> = [
        "info",
        "search",
        "extract-data",
        "explain",
        "scaffold",
        "capabilities",
    ]
    .into_iter()
    .collect();
    for sc in cat["scenarios"].as_array().unwrap() {
        if let Some(cmd) = sc["command"].as_str() {
            assert!(allowed.contains(cmd), "{} 발명 명령 {cmd}", sc["id"]);
        }
    }
}

#[test]
fn working_doc_records_the_issue_and_scope() {
    let path = repo().join("mydocs/working/agent_strategist.md");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    assert!(text.contains("#5335") || text.contains("5335"), "이슈 번호");
    assert!(text.contains("engagement.py"), "엔진");
    assert!(
        text.contains("근거 대장") || text.contains("evidence"),
        "대장"
    );
    assert!(text.contains("gym"), "gym 비범위");
    assert!(
        text.contains("새 CLI") || text.contains("새 명령을 만들지 않"),
        "CLI 비범위"
    );
    assert!(
        text.contains("전망") || text.contains("forecast"),
        "전망 비범위"
    );
}

#[test]
fn agent_file_links_the_skill() {
    let agent = repo().join(".claude/agents/rhwp-strategist.md");
    let text = fs::read_to_string(&agent).expect("rhwp-strategist.md");
    assert!(
        text.contains("skills/rhwp-strategist/SKILL.md"),
        "에이전트가 스킬을 가리켜야 합니다"
    );
}

#[test]
fn capability_registry_points_at_skill() {
    let path = repo().join("mydocs/manual/agent_capability_registry.md");
    let text = fs::read_to_string(&path).expect("registry");
    assert!(text.contains("CAP-4903"));
    assert!(text.contains("rhwp-strategist"));
    assert!(
        text.contains("skills/rhwp-strategist/SKILL.md"),
        "등록부가 스킬 진입점을 가리켜야 합니다"
    );
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
        "catalog.json",
    ] {
        let v = read_json(&format!("fixtures/{name}"));
        assert_eq!(v["schemaVersion"], "1.0", "{name}");
        assert_eq!(v["issue"], 5335, "{name}");
    }
}

#[test]
fn truncated_search_is_visible() {
    let env = read_json("fixtures/envelopes/search_truncated.json");
    assert_eq!(env["truncated"], true);
    assert_eq!(env["totalMatchCount"], 41);
    assert_eq!(env["omittedCount"], 36);
    let ledger = read_json("fixtures/ledgers/gov_rfp_truncated.json");
    let rows = ledger["truncatedSearches"].as_array().unwrap();
    assert!(!rows.is_empty(), "절단 배열이 비었습니다");
    assert!(rows[0]["omittedCount"].as_u64().unwrap() > 0);
}
