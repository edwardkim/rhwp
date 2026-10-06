//! [#5311] rhwp-bulk-pipeline 스킬·픽스처 커버리지 가드.
//!
//! 새 CLI 를 넣지 않는다. 스킬 파일과 픽스처가 기존 batch 표면
//! (info / export-text / export-structure / export-tables / fields /
//! search / extract-data / convert / fill)을 가리키는지만 검사한다.
//! 바이너리 없이 커밋된 파일만 읽는다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn skill_dir() -> PathBuf {
    repo_root().join(".claude/skills/rhwp-bulk-pipeline")
}

fn fixture_dir() -> PathBuf {
    skill_dir().join("fixtures")
}

fn read_json(name: &str) -> serde_json::Value {
    let path = fixture_dir().join(name);
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{path:?} JSON: {e}"))
}

fn read_skill() -> String {
    fs::read_to_string(skill_dir().join("SKILL.md")).expect("SKILL.md")
}

fn read_ref(name: &str) -> String {
    fs::read_to_string(skill_dir().join("references").join(name))
        .unwrap_or_else(|e| panic!("{name}: {e}"))
}

#[test]
fn skill_frontmatter_names_bulk_pipeline() {
    let text = read_skill();
    assert!(text.starts_with("---\n"), "frontmatter 필요");
    assert!(text.contains("name: rhwp-bulk-pipeline"), "{text}");
    for needle in [
        "batch info",
        "batch export-text",
        "batch extract-data",
        "batch convert",
        "batch fill",
        "stdin",
        "NDJSON",
        "exitClass",
        "--password",
        "gym 이 아니고",
    ] {
        assert!(text.contains(needle), "SKILL.md 에 {needle} 없음");
    }
}

#[test]
fn references_listed_in_skill_index_exist() {
    let idx = read_json("skill_index.json");
    let refs = idx["references"].as_array().expect("references");
    assert!(refs.len() >= 30, "레퍼런스 30장 이상: {refs:?}");
    for r in refs {
        let name = r.as_str().expect("name");
        let path = skill_dir().join("references").join(name);
        assert!(path.is_file(), "누락 {path:?}");
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.len() > 200, "{name} 가 너무 짧다");
    }
}

#[test]
fn examples_listed_in_skill_index_exist() {
    let idx = read_json("skill_index.json");
    let examples = idx["examples"].as_array().expect("examples");
    assert!(examples.len() >= 12, "예제 12개: {examples:?}");
    for r in examples {
        let name = r.as_str().expect("name");
        let path = skill_dir().join("examples").join(name);
        assert!(path.is_file(), "누락 {path:?}");
        let body = fs::read_to_string(&path).unwrap();
        assert!(
            body.contains("rhwp batch") || body.contains("Get-ChildItem"),
            "{name}"
        );
    }
}

#[test]
fn fixtures_share_schema_and_issue() {
    for name in [
        "tree.json",
        "stop_rules.json",
        "skill_index.json",
        "axes.json",
        "envelopes.json",
        "exit_codes.json",
        "gate.json",
        "journeys.json",
        "intent_matrix.json",
        "password_reject.json",
        "convert_names.json",
        "fill_contract.json",
        "recipe9_gate.json",
        "traces_index.json",
    ] {
        let v = read_json(name);
        assert_eq!(v["schemaVersion"], "1.0", "{name}");
        assert_eq!(v["issue"], 5311, "{name}");
        assert_eq!(v["notGym"], true, "{name}");
        assert_eq!(v["noNewCli"], true, "{name}");
    }
}

#[test]
fn tree_fixture_declares_not_gym_and_no_new_cli() {
    let tree = read_json("tree.json");
    assert_eq!(tree["notGym"], true);
    assert_eq!(tree["noNewCli"], true);
    assert_eq!(tree["fillIsNotStdinList"], true);
    assert_eq!(tree["passwordRejected"], true);
    assert_eq!(tree["convertReservesNames"], true);
    let reuse = tree["coreReuse"].as_array().expect("coreReuse");
    let joined = reuse
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    for axis in [
        "batch info",
        "batch export-text",
        "batch extract-data",
        "batch convert",
        "batch fill",
    ] {
        assert!(joined.contains(axis), "{joined}");
    }
}

#[test]
fn stop_rule_ids_appear_in_docs() {
    let stop = read_ref("28_retry_classes.md") + &read_skill();
    let rules = read_json("stop_rules.json");
    for rule in rules["rules"].as_array().unwrap() {
        let id = rule["id"].as_str().unwrap();
        assert!(
            stop.contains(id) || read_skill().contains(id),
            "정지 장에 {id} 없음"
        );
    }
}

#[test]
fn journeys_use_known_stop_ids() {
    let journeys = read_json("journeys.json");
    let stops = read_json("stop_rules.json");
    let mut ids = HashSet::new();
    for r in stops["rules"].as_array().unwrap() {
        ids.insert(r["id"].as_str().unwrap().to_string());
    }
    let items = journeys["journeys"].as_array().unwrap();
    assert!(items.len() >= 80, "여정이 너무 적다");
    for j in items {
        let stop = j["stop"].as_str().unwrap();
        assert!(ids.contains(stop), "여정 정지 {stop} 미정의");
        assert!(!j["steps"].as_array().unwrap().is_empty());
        assert_eq!(j["notGym"], true);
    }
}

#[test]
fn nine_axes_are_declared() {
    let axes = read_json("axes.json");
    let list = axes["axes"].as_array().unwrap();
    assert_eq!(list.len(), 9, "{list:?}");
    let mut ids = HashSet::new();
    for a in list {
        ids.insert(a["id"].as_str().unwrap().to_string());
        assert!(!a["successKeys"].as_array().unwrap().is_empty());
    }
    for need in [
        "info",
        "export-text",
        "export-structure",
        "export-tables",
        "fields",
        "search",
        "extract-data",
        "convert",
        "fill",
    ] {
        assert!(ids.contains(need), "축 {need} 없음");
    }
    let fill = list.iter().find(|a| a["id"] == "fill").unwrap();
    assert_eq!(fill["stdin"], false, "fill 은 stdin 목록이 아니다");
}

#[test]
fn recipe9_gate_is_five_equals_four_plus_one() {
    let gate = read_json("recipe9_gate.json");
    assert_eq!(gate["input"], 5);
    assert_eq!(gate["success"], 4);
    assert_eq!(gate["failure"], 1);
    assert_eq!(gate["exit"], 1);
    assert_eq!(gate["measured"], true);
    let rows = read_json("export_text_rows.json");
    let recs = rows["rows"].as_array().unwrap();
    assert_eq!(recs.len(), 5);
    let errors = recs.iter().filter(|r| r.get("error").is_some()).count();
    assert_eq!(errors, 1);
    assert_eq!(recs[4]["exitClass"], "runtime");
    assert_eq!(recs[4]["source"], "samples/없는파일.hwp");
}

#[test]
fn failure_envelope_keys() {
    let env = read_json("envelopes.json");
    let required = env["failure"]["required"].as_array().unwrap();
    for key in ["schemaVersion", "source", "error", "exitClass"] {
        assert!(required.iter().any(|k| k == key), "{required:?}");
    }
    assert_eq!(env["failure"]["exitClass"], "runtime");
    let example = &env["failure"]["example"];
    assert_eq!(example["exitClass"], "runtime");
    assert!(example["error"].as_str().unwrap().contains("os error 2"));
}

#[test]
fn password_flags_are_usage_error() {
    let pw = read_json("password_reject.json");
    assert_eq!(pw["exit"], 2);
    assert_eq!(pw["consumesStdin"], false);
    let flags = pw["rejectedFlags"].as_array().unwrap();
    for f in [
        "--password",
        "--password-stdin",
        "--output-password",
        "--output-password-stdin",
    ] {
        assert!(flags.iter().any(|x| x == f), "{flags:?}");
    }
    let skill = read_skill();
    assert!(skill.contains("exit 2"), "{skill}");
    assert!(read_ref("15_no_global_password.md").contains("--password"));
}

#[test]
fn convert_reserves_names_and_writes_nothing_on_collision() {
    let conv = read_json("convert_names.json");
    assert_eq!(conv["reserveBeforeWrite"], true);
    assert_eq!(conv["caseCollisionIsError"], true);
    assert_eq!(conv["partialWrite"], false);
    assert_eq!(conv["exitOnCollision"], 2);
    assert_eq!(conv["mcpExcluded"], true);
    let cases = conv["cases"].as_array().unwrap();
    assert!(cases
        .iter()
        .any(|c| c["ok"] == false && c["reason"] == "case"));
    assert!(read_ref("16_convert_name_reservation.md").contains("한 파일도"));
}

#[test]
fn fill_is_form_plus_data_not_stdin_list() {
    let fill = read_json("fill_contract.json");
    assert_eq!(fill["stdinIsNotFileList"], true);
    assert_eq!(fill["dryRunStillNeedsOutDir"], true);
    assert_eq!(fill["emptyCsvExit"], 2);
    assert_eq!(fill["rowZeroBased"], true);
    let skill = read_skill();
    assert!(skill.contains("--form"), "{skill}");
    assert!(skill.contains("stdin 파일 목록을 파이프"), "{skill}");
}

#[test]
fn exit_aggregation_codes() {
    let exits = read_json("exit_codes.json");
    let rows = exits["aggregation"].as_array().unwrap();
    let mut codes = HashSet::new();
    for r in rows {
        codes.insert(r["code"].as_u64().unwrap());
    }
    assert_eq!(codes, HashSet::from([0, 1, 2, 3, 4]));
    let skill = read_skill();
    assert!(skill.contains("verify-pages"), "{skill}");
    assert!(read_ref("18_exit_aggregation.md").contains("4"));
}

#[test]
fn extract_data_limit_is_per_document() {
    let rows = read_json("extract_rows.json");
    let recs = rows["rows"].as_array().unwrap();
    let first = recs
        .iter()
        .find(|r| r["source"].as_str().unwrap().contains("국립국어원"))
        .unwrap();
    assert_eq!(first["itemCount"], 3);
    assert_eq!(first["totalItemCount"], 297);
    assert_eq!(first["truncated"], true);
    assert!(read_ref("10_axis_extract_data.md").contains("문서마다"));
}

#[test]
fn transcripts_are_pure_ndjson() {
    let idx = read_json("traces_index.json");
    let traces = idx["traces"].as_array().unwrap();
    assert!(traces.len() >= 20, "{}", traces.len());
    for t in traces {
        let rel = t["transcript"].as_str().unwrap();
        let path = skill_dir().join(rel);
        assert!(path.is_file(), "{path:?}");
        let text = fs::read_to_string(&path).unwrap();
        if t["exit"] == 2 {
            assert!(
                text.trim().is_empty(),
                "사용법 오류 전사는 비어야 한다: {rel}"
            );
            continue;
        }
        for (i, line) in text.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let v: serde_json::Value =
                serde_json::from_str(line).unwrap_or_else(|e| panic!("{rel}:{i}: {e}"));
            assert!(v.is_object(), "{rel}:{i} 객체 아님");
            assert_eq!(v["schemaVersion"], "1.0", "{rel}:{i}");
        }
    }
}

#[test]
fn recipe9_transcript_order_matches_list() {
    let list = fs::read_to_string(skill_dir().join("examples/lists/recipe9.txt")).unwrap();
    let paths: Vec<&str> = list.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(paths.len(), 5);
    let nd = fs::read_to_string(skill_dir().join("examples/transcripts/T02.ndjson")).unwrap();
    let recs: Vec<serde_json::Value> = nd
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(recs.len(), paths.len());
    for (p, r) in paths.iter().zip(recs.iter()) {
        assert_eq!(r["source"], *p, "입력 순서 보존 실패");
    }
    assert!(recs[4].get("error").is_some());
}

#[test]
fn forbidden_peer_skills_not_rewritten_here() {
    let idx = read_json("skill_index.json");
    for name in idx["forbiddenTrees"].as_array().unwrap() {
        assert_eq!(name, "gym/");
    }
    let skill = read_skill();
    assert!(!skill.contains("gym/packs"), "{skill}");
    for invented in ["batch merge", "batch export-markdown", "batch thumbnail"] {
        assert!(!skill.contains(invented), "발명 명령 {invented}");
    }
}

#[test]
fn working_doc_exists() {
    let path = repo_root().join("mydocs/working/archives/agent_bulk_pipeline.md");
    assert!(path.is_file(), "{path:?}");
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("#5311"), "{text}");
    assert!(text.contains("rhwp-bulk-pipeline"), "{text}");
    assert!(text.contains("gym"), "{text}");
}

#[test]
fn intent_matrix_covers_password_and_fill() {
    let intents = read_json("intent_matrix.json");
    let items = intents["intents"].as_array().unwrap();
    assert!(items.len() >= 100, "{}", items.len());
    let blob: String = items
        .iter()
        .filter_map(|i| i["utterance"].as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(blob.contains("비밀번호") || blob.contains("password"));
    assert!(blob.contains("메일머지"));
    assert!(items.iter().all(|i| i["notGym"] == true));
}

#[test]
fn jq_recipes_split_error_from_success() {
    let jq = read_json("jq_recipes.json");
    let recs = jq["recipes"].as_array().unwrap();
    let joined: String = recs
        .iter()
        .filter_map(|r| r["jq"].as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(joined.contains("select(.error)"));
    assert!(joined.contains("select(.error|not)"));
}
