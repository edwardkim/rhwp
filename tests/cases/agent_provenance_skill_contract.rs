//! [#5295] rhwp-provenance 스킬 — 실사용 에이전트가 문서 파생 값을
//! 미신뢰 데이터로 다루는지 기계 가드.
//!
//! 새 CLI 를 만들지 않는다. 권위는 `rhwp_contracts::provenance::MAP` 과
//! (있으면) `export-provenance-map --json` 이다.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use rhwp_contracts::provenance::MAP;
use serde_json::Value;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn skill_dir() -> PathBuf {
    repo().join(".claude/skills/rhwp-provenance")
}

fn read_skill(rel: &str) -> String {
    let path = skill_dir().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{} 읽기 실패: {e}", path.display()))
}

fn read_json(rel: &str) -> Value {
    let text = read_skill(rel);
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{rel} JSON 파싱 실패: {e}"))
}

fn unique_map() -> BTreeMap<&'static str, &'static [rhwp_contracts::provenance::UntrustedField]> {
    let mut out = BTreeMap::new();
    for e in MAP {
        out.entry(e.command).or_insert(e.untrusted);
    }
    out
}

fn required_references() -> &'static [&'static str] {
    &[
        "references/export-provenance-map.md",
        "references/untrusted-content-fields.md",
        "references/injection-boundaries.md",
        "references/forbidden-prompt-slots.md",
        "references/command-field-catalog.md",
        "references/consumption-playbook.md",
        "references/anti-patterns.md",
        "references/privilege-reduction.md",
    ]
}

fn required_fixtures() -> &'static [&'static str] {
    &[
        "fixtures/command-untrusted-fields.json",
        "fixtures/forbidden-prompt-slots.json",
        "fixtures/injection-boundaries.json",
        "fixtures/consumption-checklist.json",
        "fixtures/prompt-slot-cases.json",
        "fixtures/envelope-examples/search-untrusted.json",
        "fixtures/envelope-examples/info-untrusted.json",
        "fixtures/envelope-examples/export-text-untrusted.json",
        "fixtures/envelope-examples/capabilities-trusted.json",
        "fixtures/envelope-examples/export-provenance-map-trusted.json",
        "fixtures/envelope-examples/missing-keys-legacy.json",
    ]
}

fn required_forbidden_slots() -> &'static [&'static str] {
    &[
        "system_prompt",
        "tool_argument_path",
        "tool_name",
        "shell_command",
        "url_or_request_body",
        "run_plan",
        "privilege_decision",
        "log_or_issue",
        "log_title",
        "output_filename",
        "multimodal_instruction",
        "next_query",
    ]
}

fn required_boundaries() -> &'static [&'static str] {
    &["B1", "B2", "B3", "B4", "B5"]
}

#[test]
fn skill_layout_has_required_files() {
    let skill = read_skill("SKILL.md");
    assert!(
        skill.contains("name: rhwp-provenance"),
        "SKILL.md frontmatter 에 name 이 없습니다"
    );
    assert!(
        skill.contains("export-provenance-map"),
        "SKILL.md 가 지도 명령을 안내해야 합니다"
    );
    assert!(
        skill.contains("untrustedContent"),
        "SKILL.md 가 untrustedContent 를 설명해야 합니다"
    );
    assert!(
        skill.contains("untrustedFields"),
        "SKILL.md 가 untrustedFields 를 설명해야 합니다"
    );
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
    for rel in required_fixtures() {
        assert!(skill_dir().join(rel).is_file(), "필수 픽스처 누락: {rel}");
    }
}

#[test]
fn skill_does_not_add_cli_and_stays_out_of_other_skills() {
    let skill = read_skill("SKILL.md");
    assert!(
        skill.contains("새 CLI") || skill.contains("새 명령을 만들지 않는다"),
        "SKILL.md 가 새 CLI 를 만들지 않는다고 밝혀야 합니다"
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
fn fixture_commands_match_provenance_map() {
    let fixture = read_json("fixtures/command-untrusted-fields.json");
    let commands = fixture["commands"].as_object().expect("commands 객체");
    let map = unique_map();

    let fixture_names: BTreeSet<&str> = commands.keys().map(String::as_str).collect();
    let map_names: BTreeSet<&str> = map.keys().copied().collect();
    let missing: Vec<_> = map_names.difference(&fixture_names).copied().collect();
    let extra: Vec<_> = fixture_names.difference(&map_names).copied().collect();
    assert!(missing.is_empty(), "픽스처에 없는 MAP 명령: {missing:?}");
    assert!(extra.is_empty(), "MAP 에 없는 픽스처 명령: {extra:?}");

    for (name, fields) in &map {
        let declared: BTreeSet<&str> = fields.iter().map(|f| f.path).collect();
        let got: BTreeSet<&str> = commands[*name]["untrusted"]
            .as_array()
            .expect("untrusted 배열")
            .iter()
            .filter_map(|v| v.as_str())
            .collect();
        assert_eq!(
            declared, got,
            "{name}: 픽스처 untrusted 가 MAP 과 다릅니다\nMAP={declared:?}\nFIX={got:?}"
        );
        let origins = commands[*name]["origins"]
            .as_object()
            .expect("origins 객체");
        for path in &declared {
            let origin = origins[*path]
                .as_str()
                .unwrap_or_else(|| panic!("{name}.{path} origin 없음"));
            assert!(
                !origin.trim().is_empty(),
                "{name}.{path} origin 이 비어 있습니다"
            );
        }
    }
}

#[test]
fn catalog_has_a_section_for_every_map_command() {
    let catalog = read_skill("references/command-field-catalog.md");
    let mut missing = Vec::new();
    for name in unique_map().keys() {
        let heading = format!("`{name}`");
        if !catalog.contains(&heading) {
            missing.push(*name);
        }
    }
    assert!(missing.is_empty(), "카탈로그에 절이 없는 명령: {missing:?}");
}

#[test]
fn forbidden_slots_are_complete_and_documented() {
    let fixture = read_json("fixtures/forbidden-prompt-slots.json");
    let slots: BTreeSet<&str> = fixture["forbiddenSlots"]
        .as_array()
        .expect("forbiddenSlots")
        .iter()
        .filter_map(|s| s["id"].as_str())
        .collect();
    for id in required_forbidden_slots() {
        assert!(slots.contains(id), "금지 자리 픽스처에 {id} 가 없습니다");
    }
    let allowed = fixture["allowedSlots"].as_array().expect("allowedSlots");
    let allowed_ids: BTreeSet<&str> = allowed.iter().filter_map(|s| s["id"].as_str()).collect();
    assert!(allowed_ids.contains("user_visible_surface"));
    assert!(allowed_ids.contains("fenced_model_block"));
    assert_eq!(allowed.len(), 2, "허용 자리는 둘뿐이어야 합니다");

    let skill = read_skill("SKILL.md");
    let slots_doc = read_skill("references/forbidden-prompt-slots.md");
    for id in required_forbidden_slots() {
        assert!(
            skill.contains(&format!("`{id}`")) || skill.contains(*id),
            "SKILL.md 가 {id} 를 안내해야 합니다"
        );
        assert!(
            slots_doc.contains(&format!("`{id}`")),
            "forbidden-prompt-slots.md 에 `{id}` 절이 있어야 합니다"
        );
    }
}

#[test]
fn injection_boundaries_cover_b1_to_b5() {
    let fixture = read_json("fixtures/injection-boundaries.json");
    let ids: BTreeSet<&str> = fixture["boundaries"]
        .as_array()
        .expect("boundaries")
        .iter()
        .filter_map(|b| b["id"].as_str())
        .collect();
    for id in required_boundaries() {
        assert!(ids.contains(id), "경계 픽스처에 {id} 가 없습니다");
    }
    let doc = read_skill("references/injection-boundaries.md");
    for id in required_boundaries() {
        assert!(
            doc.contains(id),
            "injection-boundaries.md 에 {id} 가 없습니다"
        );
    }
    let groups = fixture["commandGroups"].as_object().expect("commandGroups");
    assert!(groups.contains_key("본문-반출"));
    assert!(groups.contains_key("서식-메타"));
    assert!(groups.contains_key("보안-발췌"));
    assert!(groups.contains_key("편집-저널"));
    let map = unique_map();
    for (label, group) in groups {
        for cmd in group["commands"].as_array().expect("commands") {
            let name = cmd.as_str().expect("command name");
            assert!(
                map.contains_key(name),
                "교차표 {label} 의 {name} 이 MAP 에 없습니다"
            );
        }
    }
}

#[test]
fn consumption_checklist_ids_appear_in_playbook() {
    let fixture = read_json("fixtures/consumption-checklist.json");
    let playbook = read_skill("references/consumption-playbook.md");
    let steps = fixture["steps"].as_array().expect("steps");
    assert!(
        steps.len() >= 16,
        "체크리스트가 너무 짧습니다: {}",
        steps.len()
    );
    for step in steps {
        let id = step["id"].as_str().expect("id");
        assert!(
            playbook.contains(id),
            "consumption-playbook.md 에 {id} 가 없습니다"
        );
        let must = step["must"].as_str().expect("must");
        assert!(!must.trim().is_empty(), "{id} must 가 비어 있습니다");
    }
}

#[test]
fn prompt_slot_cases_use_known_slots_and_map_fields() {
    let cases = read_json("fixtures/prompt-slot-cases.json");
    let slots_fix = read_json("fixtures/forbidden-prompt-slots.json");
    let mut known_slots: BTreeSet<&str> = slots_fix["forbiddenSlots"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|s| s["id"].as_str())
        .collect();
    known_slots.insert("user_visible_surface");
    known_slots.insert("fenced_model_block");
    let map = unique_map();
    let list = cases["cases"].as_array().expect("cases");
    assert!(
        list.len() >= 30,
        "자리 사례가 너무 적습니다: {}",
        list.len()
    );
    let mut seen_deny = 0;
    let mut seen_allow = 0;
    for case in list {
        let id = case["id"].as_str().expect("id");
        let slot = case["slot"].as_str().expect("slot");
        assert!(known_slots.contains(slot), "{id}: 알 수 없는 자리 {slot}");
        let command = case["command"].as_str().expect("command");
        assert!(
            map.contains_key(command),
            "{id}: 명령 {command} 이 MAP 에 없습니다"
        );
        let field = case["field"].as_str().expect("field");
        let declared: BTreeSet<&str> = map[command].iter().map(|f| f.path).collect();
        assert!(
            declared.contains(field),
            "{id}: {command} 의 MAP 에 {field} 가 없습니다 ({declared:?})"
        );
        match case["verdict"].as_str().expect("verdict") {
            "deny" => seen_deny += 1,
            "allow" => seen_allow += 1,
            other => panic!("{id}: 알 수 없는 verdict {other}"),
        }
    }
    assert!(seen_deny >= 20, "거부 사례가 너무 적습니다: {seen_deny}");
    assert!(seen_allow >= 2, "허용 사례가 없습니다: {seen_allow}");
}

#[test]
fn envelope_examples_teach_true_false_and_missing() {
    let search = read_json("fixtures/envelope-examples/search-untrusted.json");
    assert_eq!(search["untrustedContent"], true);
    assert!(search["untrustedFields"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v == "matches[].text"));

    let info = read_json("fixtures/envelope-examples/info-untrusted.json");
    assert_eq!(info["untrustedContent"], true);
    assert!(info["untrustedFields"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v == "title"));

    let export_text = read_json("fixtures/envelope-examples/export-text-untrusted.json");
    assert_eq!(export_text["untrustedContent"], true);

    let caps = read_json("fixtures/envelope-examples/capabilities-trusted.json");
    assert_eq!(caps["untrustedContent"], false);
    assert!(caps["untrustedFields"].as_array().unwrap().is_empty());

    let map_ex = read_json("fixtures/envelope-examples/export-provenance-map-trusted.json");
    assert_eq!(map_ex["untrustedContent"], false);

    let legacy = read_json("fixtures/envelope-examples/missing-keys-legacy.json");
    assert!(legacy.get("untrustedContent").is_none());
    assert_eq!(legacy["consume"]["state"], "unmarked");
}

#[test]
fn working_doc_records_the_issue_and_scope() {
    let path = repo().join("mydocs/working/archives/agent_provenance.md");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    assert!(text.contains("#5295") || text.contains("5295"), "이슈 번호");
    assert!(text.contains("export-provenance-map"), "지도 명령");
    assert!(text.contains("untrustedContent"), "표지");
    assert!(text.contains("gym"), "gym 비범위");
    assert!(
        text.contains("새 CLI") || text.contains("새 명령을 만들지 않"),
        "CLI 비범위"
    );
}

#[test]
fn capability_registry_lists_rhwp_provenance() {
    let path = repo().join("mydocs/manual/agent_capability_registry.md");
    let text = fs::read_to_string(&path).expect("capability registry");
    assert!(
        text.contains("rhwp-provenance"),
        "capability 카탈로그에 rhwp-provenance 행이 있어야 합니다"
    );
    assert!(
        text.contains("CAP-5295"),
        "등록 식별번호는 CAP-5295 여야 합니다"
    );
}

#[test]
fn live_export_provenance_map_matches_fixture_when_binary_available() {
    let bin = match std::env::var("CARGO_BIN_EXE_rhwp") {
        Ok(p) => p,
        Err(_) => return,
    };
    let out = Command::new(&bin)
        .args(["export-provenance-map", "--json"])
        .output()
        .expect("export-provenance-map 실행");
    if !out.status.success() {
        panic!(
            "export-provenance-map 실패: {}\n{}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let live: Value = serde_json::from_slice(&out.stdout).expect("지도 JSON");
    assert_eq!(live["untrustedContent"], false);
    assert!(live["untrustedFields"].as_array().unwrap().is_empty());
    let fixture = read_json("fixtures/command-untrusted-fields.json");
    let live_cmds = live["commands"].as_object().expect("live commands");
    let fix_cmds = fixture["commands"].as_object().expect("fix commands");
    for name in fix_cmds.keys() {
        assert!(
            live_cmds.contains_key(name),
            "라이브 지도에 {name} 이 없습니다"
        );
        let live_paths: BTreeSet<&str> = live_cmds[name]["untrusted"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str())
            .collect();
        let fix_paths: BTreeSet<&str> = fix_cmds[name]["untrusted"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str())
            .collect();
        assert_eq!(live_paths, fix_paths, "{name} 경로 드리프트");
    }
}
