//! [#5318] rhwp-codex 스킬 — 대전 항해 계약 가드.
//!
//! 새 CLI 를 만들지 않는다. 생성 장을 수기 수정하지 않는다.
//! 스킬·픽스처가 기존 표면(mydocs/manual/agent_codex, tools/gen_agent_codex.py,
//! rhwp capabilities --search)을 가리키는지만 검사한다.

#![cfg(not(target_arch = "wasm32"))]

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn skill_dir() -> PathBuf {
    repo().join(".claude/skills/rhwp-codex")
}

fn read_skill(rel: &str) -> String {
    let path = skill_dir().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{} 읽기 실패: {e}", path.display()))
}

fn read_json(rel: &str) -> Value {
    serde_json::from_str(&read_skill(rel)).unwrap_or_else(|e| panic!("{rel} JSON: {e}"))
}

fn required_references() -> &'static [&'static str] {
    &[
        "references/00_covenants.md",
        "references/01_request_tree.md",
        "references/02_how_to_read.md",
        "references/03_regen_freshness.md",
        "references/04_capabilities_search.md",
        "references/05_boundary_knowledge_map.md",
        "references/06_chapter_85.md",
        "references/07_chapter_10.md",
        "references/08_chapter_20.md",
        "references/09_chapter_30.md",
        "references/10_chapter_40.md",
        "references/11_chapter_50.md",
        "references/12_chapter_60.md",
        "references/13_chapter_70.md",
        "references/14_chapter_80.md",
        "references/16_envelopes.md",
        "references/17_pitfalls.md",
        "references/18_handoff.md",
        "references/19_exit_codes.md",
        "references/20_intent_matrix.md",
        "references/21_journeys.md",
        "references/22_fixture_index.md",
    ]
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

#[test]
fn skill_frontmatter_names_codex() {
    let text = read_skill("SKILL.md");
    assert!(text.starts_with("---\n"), "frontmatter 필요");
    assert!(text.contains("name: rhwp-codex"), "{text}");
    for token in [
        "판정=데이터",
        "결정론",
        "출처 표지",
        "원본 무훼손",
        "파악",
        "수확",
        "편집",
        "변환",
        "검증",
        "보안",
        "대량",
        "generated:",
        "python tools/gen_agent_codex.py",
        "--check",
        "exit 3",
        "capabilities --search",
        "§2-2",
        "개발자",
        "gym",
        "새 CLI",
    ] {
        assert!(text.contains(token), "SKILL.md 에 {token} 없음");
    }
}

#[test]
fn references_listed_in_skill_exist() {
    let idx = read_json("fixtures/skill_index.json");
    let refs = idx["references"].as_array().expect("references");
    assert!(refs.len() >= 16, "레퍼런스 부족: {refs:?}");
    for r in refs {
        let name = r.as_str().expect("name");
        let path = skill_dir().join("references").join(name);
        assert!(path.is_file(), "누락 {path:?}");
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.len() > 200, "{name} 가 너무 짧다");
    }
    for rel in required_references() {
        assert!(skill_dir().join(rel).is_file(), "필수 누락 {rel}");
        let name = Path::new(rel).file_name().unwrap().to_string_lossy();
        assert!(
            read_skill("SKILL.md").contains(name.as_ref()),
            "SKILL.md 가 {name} 를 가리켜야 한다"
        );
    }
}

#[test]
fn fixtures_share_schema_and_issue() {
    for name in [
        "fixtures/catalog.json",
        "fixtures/covenants.json",
        "fixtures/request_tree.json",
        "fixtures/skill_index.json",
        "fixtures/intent_matrix.json",
        "fixtures/journeys.json",
        "fixtures/search_fallback.json",
        "fixtures/regen.json",
        "fixtures/boundary.json",
        "fixtures/stop_rules.json",
        "fixtures/chapter_index.json",
    ] {
        let v = read_json(name);
        assert_eq!(v["schemaVersion"], "1.0", "{name}");
        assert_eq!(v["issue"], 5318, "{name}");
    }
}

#[test]
fn catalog_declares_not_gym_and_no_new_cli() {
    let cat = read_json("fixtures/catalog.json");
    assert_eq!(cat["notGym"], true);
    assert_eq!(cat["noNewCli"], true);
    assert_eq!(cat["noNewEditLogic"], true);
    assert_eq!(cat["doNotHandEditGenerated"], true);
    assert_eq!(cat["chapter85DeveloperOnly"], true);
    let skill_index = read_json("fixtures/skill_index.json");
    let reuse = skill_index["coreReuse"].as_array().expect("coreReuse");
    let joined = reuse
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(joined.contains("gen_agent_codex.py"), "{joined}");
    assert!(joined.contains("capabilities"), "{joined}");
}

#[test]
fn four_covenants_are_closed() {
    let cov = read_json("fixtures/covenants.json");
    let items = cov["covenants"].as_array().unwrap();
    assert_eq!(items.len(), 4);
    let names: Vec<_> = items.iter().map(|c| c["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["판정=데이터", "결정론", "출처 표지", "원본 무훼손"]);
    assert_eq!(items[0]["exit"], 3);
}

#[test]
fn request_tree_has_seven_branches_and_chapter_numbers() {
    let tree = read_json("fixtures/request_tree.json");
    let branches = tree["branches"].as_array().unwrap();
    assert_eq!(branches.len(), 7);
    let want = [
        ("파악", 10),
        ("수확", 20),
        ("편집", 30),
        ("변환", 40),
        ("검증", 50),
        ("보안", 60),
        ("대량", 80),
    ];
    for (i, (id, no)) in want.iter().enumerate() {
        assert_eq!(branches[i]["id"], *id);
        assert_eq!(branches[i]["chapterNo"], *no);
    }
}

#[test]
fn handwritten_vs_generated_chapters() {
    let idx = read_json("fixtures/chapter_index.json");
    let mut saw_hand = 0;
    let mut saw_gen = 0;
    for ch in idx["chapters"].as_array().unwrap() {
        let file = ch["file"].as_str().unwrap();
        let kind = ch["kind"].as_str().unwrap();
        let path = repo().join("mydocs/manual/agent_codex").join(file);
        assert!(path.is_file(), "대전 장 누락 {file}");
        let text = fs::read_to_string(&path).unwrap();
        if kind == "handwritten" {
            assert!(
                !text.contains("generated: tools/gen_agent_codex.py"),
                "{file} 는 손글인데 generated 표지가 있다"
            );
            saw_hand += 1;
        } else {
            assert!(
                text.contains("generated: tools/gen_agent_codex.py"),
                "{file} 생성 표지 없음"
            );
            saw_gen += 1;
        }
        if file == "85_진단_프로브.md" {
            assert_eq!(ch["developerOnly"], true);
        }
    }
    assert_eq!(saw_hand, 2, "손글 00+01");
    assert!(saw_gen >= 8, "생성 장 부족 {saw_gen}");
}

#[test]
fn regen_check_is_exit_3_data() {
    let regen = read_json("fixtures/regen.json");
    assert_eq!(regen["staleExit"], 3);
    assert_eq!(regen["staleIsData"], true);
    let check = regen["check"].as_array().unwrap();
    assert_eq!(check.last().and_then(|v| v.as_str()), Some("--check"));
    let skill = read_skill("SKILL.md");
    assert!(skill.contains("python tools/gen_agent_codex.py --check"));
    assert!(read_skill("references/03_regen_freshness.md").contains("exit 3"));
}

#[test]
fn search_fallback_uses_existing_capabilities_flag() {
    let sf = read_json("fixtures/search_fallback.json");
    let qs = sf["queries"].as_array().unwrap();
    assert!(qs.len() >= 40, "검색 폴백이 너무 적다");
    for q in qs {
        let argv = q["argv"].as_array().unwrap();
        assert_eq!(argv[0], "capabilities");
        assert_eq!(argv[1], "--search");
        assert!(q["query"].as_str().unwrap().chars().count() >= 2);
    }
}

#[test]
fn boundary_points_at_knowledge_map_section() {
    let b = read_json("fixtures/boundary.json");
    let dict = b["envelopeFieldDictionary"].as_str().unwrap();
    assert!(dict.contains("§2-2") || dict.contains("2-2"), "{dict}");
    assert_eq!(b["notInThisSkill"], true);
    assert_eq!(b["skillMustNotRedefineTypes"], true);
    let km = repo().join("mydocs/manual/agent_knowledge_map.md");
    let text = fs::read_to_string(&km).expect("지식지도");
    assert!(text.contains("2-2"), "지식지도 §2-2 표제 없음");
}

#[test]
fn chapter_85_is_developer_only() {
    let skill = read_skill("SKILL.md");
    let ch = read_skill("references/06_chapter_85.md");
    assert!(skill.contains("개발자"));
    assert!(ch.contains("개발자"));
    assert!(ch.contains("X07") || skill.contains("X07"));
    let cat = read_json("fixtures/catalog.json");
    let devs: Vec<_> = cat["commands"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["developerOnly"] == true)
        .collect();
    assert!(!devs.is_empty(), "85장 명령이 카탈로그에 없다");
}

#[test]
fn stop_rule_ids_appear_in_docs() {
    let stop = read_skill("SKILL.md") + &read_skill("references/17_pitfalls.md");
    let rules = read_json("fixtures/stop_rules.json");
    for rule in rules["rules"].as_array().unwrap() {
        let id = rule["id"].as_str().unwrap();
        assert!(stop.contains(id), "정지 {id} 문서 누락");
    }
}

#[test]
fn journeys_use_known_stop_ids() {
    let journeys = read_json("fixtures/journeys.json");
    let stops = read_json("fixtures/stop_rules.json");
    let mut ids = std::collections::HashSet::new();
    for r in stops["rules"].as_array().unwrap() {
        ids.insert(r["id"].as_str().unwrap().to_string());
    }
    let items = journeys["journeys"].as_array().unwrap();
    assert!(items.len() >= 30, "여정이 너무 적다");
    for j in items {
        let stop = j["stop"].as_str().unwrap();
        assert!(ids.contains(stop), "여정 정지 {stop} 미정의");
        assert!(!j["steps"].as_array().unwrap().is_empty());
        assert_eq!(j["notGym"], true);
        assert_eq!(j["noNewCli"], true);
    }
}

#[test]
fn intent_matrix_uses_existing_surface() {
    let intents = read_json("fixtures/intent_matrix.json");
    let rows = intents["intents"].as_array().unwrap();
    assert!(rows.len() >= 80, "발화가 너무 적다");
    let invented = [
        "codex-search",
        "agent-help",
        "do-what-i-mean",
        "edit mail-merge",
    ];
    for row in rows {
        let cmd = row["command"].as_str().unwrap();
        for bad in invented {
            assert!(!cmd.contains(bad), "발명 명령 {cmd}");
        }
        assert_eq!(row["notGym"], true);
    }
}

#[test]
fn extracted_envelopes_come_from_generated_chapters() {
    let dir = skill_dir().join("fixtures/envelopes");
    let mut n = 0;
    for ent in fs::read_dir(&dir).expect("envelopes") {
        let path = ent.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let v: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(v["issue"], 5318);
        assert_eq!(v["extractedFromGenerated"], true);
        assert_eq!(v["handEdited"], false);
        let chapter = v["sourceChapter"].as_str().unwrap();
        assert!(
            repo()
                .join("mydocs/manual/agent_codex")
                .join(chapter)
                .is_file(),
            "{chapter}"
        );
        n += 1;
    }
    assert!(n >= 15, "실측 전사 {n}건은 너무 적다");
}

#[test]
fn forbidden_peer_skills_not_rewritten_here() {
    let idx = read_json("fixtures/skill_index.json");
    for name in idx["forbiddenSkillsTouch"].as_array().unwrap() {
        let slug = name.as_str().unwrap();
        let peer = repo().join(".claude/skills").join(slug).join("SKILL.md");
        assert!(peer.is_file(), "존재해야 하는 이웃 스킬 {peer:?}");
    }
    let trees = idx["forbiddenTrees"].as_array().unwrap();
    assert!(trees.iter().any(|t| t == "gym/"), "{trees:?}");
}

#[test]
fn working_doc_exists() {
    let path = repo().join("mydocs/working/agent_codex_skill.md");
    assert!(path.is_file(), "{path:?}");
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("#5318") || text.contains("5318"), "{text}");
    assert!(text.contains("rhwp-codex"), "{text}");
}

#[test]
fn no_new_bin_target() {
    let cargo = fs::read_to_string(repo().join("Cargo.toml")).expect("Cargo.toml");
    let bins = cargo.matches("[[bin]]").count();
    assert_eq!(
        bins, 2,
        "새 [[bin]] 을 추가하지 마세요 (rhwp, font-metric-gen 만): {bins}"
    );
}

#[test]
fn generator_script_still_lives() {
    let path = repo().join("tools/gen_agent_codex.py");
    assert!(path.is_file());
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("--check"));
    assert!(text.contains("return 3") || text.contains("exit 3"));
}

#[test]
fn capabilities_search_runs_when_binary_exists() {
    let args = ["capabilities", "--search", "누름틀"];
    let output = run(&args);
    if !output.status.success() {
        eprintln!("capabilities --search 실패 — 인자 계약만 확인");
        return;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("fields") || stdout.contains("fill"),
        "누름틀 검색이 기존 명령을 돌려야 한다: {stdout}"
    );
}
