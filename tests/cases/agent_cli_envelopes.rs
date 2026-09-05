//! [#5316] 예외 봉투 — missing file / bad page / native-skia / load fail.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use serde_json::Value;

fn skill_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".agents/skills/rhwp-cli")
}

fn read_text(rel: &str) -> String {
    let path = skill_dir().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn read_json(rel: &str) -> Value {
    serde_json::from_str(&read_text(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

#[test]
fn four_primary_kinds_are_documented() {
    let idx = read_json("fixtures/skill_index.json");
    let kinds: BTreeSet<&str> = idx["exceptionKinds"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    for k in [
        "missing-file",
        "bad-page-index",
        "native-skia-missing",
        "load-fail",
    ] {
        assert!(kinds.contains(k), "{k}");
    }
    let skill = read_text("SKILL.md") + &read_text("references/21_exception_envelopes.md");
    assert!(skill.contains("오류: 파일을 읽을 수 없습니다"));
    assert!(skill.contains("오류: 페이지 번호가 범위를 벗어났습니다"));
    assert!(skill.contains("오류: export-png 명령은 native-skia feature 가 활성화되어야 합니다."));
    assert!(skill.contains("오류: 문서 파싱 실패"));
}

#[test]
fn missing_file_is_runtime_one() {
    let env = read_json("fixtures/envelopes/missing_file.json");
    assert_eq!(env["kind"], "missing-file");
    assert_eq!(env["exitCode"], 1);
    assert_eq!(env["exitClass"], "runtime");
    assert_eq!(env["stdoutEmpty"], true);
    assert!(env["stderrContains"]
        .as_str()
        .unwrap()
        .contains("파일을 읽을 수 없습니다"));
}

#[test]
fn bad_page_index_is_usage_two() {
    let env = read_json("fixtures/envelopes/bad_page_index.json");
    assert_eq!(env["kind"], "bad-page-index");
    assert_eq!(env["exitCode"], 2);
    assert_eq!(env["exitClass"], "usage");
    assert!(env["stderrContains"]
        .as_str()
        .unwrap()
        .contains("페이지 번호가 범위를 벗어났습니다"));
}

#[test]
fn native_skia_missing_is_usage_two() {
    let env = read_json("fixtures/envelopes/native_skia_missing.json");
    assert_eq!(env["kind"], "native-skia-missing");
    assert_eq!(env["command"], "export-png");
    assert_eq!(env["exitCode"], 2);
    assert!(env["stderrContains"]
        .as_str()
        .unwrap()
        .contains("native-skia feature"));
}

#[test]
fn load_fail_is_runtime_one() {
    let env = read_json("fixtures/envelopes/load_fail.json");
    assert_eq!(env["kind"], "load-fail");
    assert_eq!(env["exitCode"], 1);
    assert!(env["stderrContains"]
        .as_str()
        .unwrap()
        .contains("문서 파싱 실패"));
}

#[test]
fn png_and_pdf_direct_feature_exits_differ() {
    let png = read_json("fixtures/exit_codes.json");
    assert_eq!(png["pngMissingFeature"], 2);
    assert_eq!(png["pdfDirectMissingFeature"], 1);
    let pdf = read_json("fixtures/envelopes/native_skia_direct_pdf.json");
    assert_eq!(pdf["exitCode"], 1);
    assert_eq!(pdf["command"], "export-pdf");
}

#[test]
fn ir_diff_json_mismatch_is_data() {
    let env = read_json("fixtures/envelopes/ir_diff_mismatch.json");
    assert_eq!(env["exitCode"], 3);
    assert_eq!(env["identical"], false);
    assert!(env["diffCount"].as_u64().unwrap() > 0);
    let codes = read_json("fixtures/exit_codes.json");
    assert_eq!(codes["irDiffJsonMismatch"], 3);
    assert_eq!(codes["irDiffTextMismatch"], 0);
}

#[test]
fn envelope_exit_meta_is_allowed_set() {
    let dir = skill_dir().join("fixtures/envelopes");
    let mut seen = 0usize;
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let env: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let exit = env["_skillMeta"]["exit"].as_i64().unwrap_or(-1);
        assert!(matches!(exit, 0..=4), "{} exit {exit}", path.display());
        seen += 1;
    }
    assert!(seen >= 16, "봉투가 너무 적다: {seen}");
}

#[test]
fn envelope_keys_catalog_is_nonempty() {
    let keys = read_json("fixtures/envelope_keys.json");
    for (cmd, required) in keys.as_object().unwrap() {
        let req = required.as_array().unwrap();
        assert!(!req.is_empty(), "{cmd} 키 목록 비어 있음");
    }
}
