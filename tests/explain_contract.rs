//! [#3828] `explain` 출력 계약 회귀 테스트.
//!
//! 계약: `explain --json` 의 stdout 은 순수 JSON 한 덩어리이고 `schemaVersion` 을
//! 포함한다. 값 자체는 새 판정이 아니라 `info`·`export-structure`·`export-tables`·
//! `fields` 가 이미 계산한 값의 조합이므로, 이 시험은 그 조합이 실제 문서에서
//! 정확한지(표 크기·병합 여부·누름틀 이름·각주/미주 개수·암호 여부)와 종료 코드
//! 계약(#2707)을 검증한다. 암호 문서는 다른 명령과 같은 `load_document` 규약을
//! 타므로 explain 도 자동으로 같은 종료 코드를 낸다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 누름틀 11개, 표 없음.
const SAMPLE_FIELDS: &str = "samples/field-01.hwp";
/// 표 1개(19×9, 병합 셀 있음), 누름틀 없음.
const SAMPLE_TABLE: &str = "samples/table-001.hwp";
/// 표·누름틀 모두 없는 일반 문서(HWP5).
const SAMPLE_PLAIN: &str = "samples/para-001.hwp";
/// 각주 9개.
const SAMPLE_FOOTNOTE: &str = "samples/footnote-01.hwp";
/// 미주 6개.
const SAMPLE_ENDNOTE: &str = "samples/endnote-01.hwp";
/// 비밀번호 "123456" 로 걸린 HWP5 문서.
const SAMPLE_ENCRYPTED: &str = "samples/hwp3-sample16-hwp5-2024-password-123456.hwp";

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn describe(args: &[&str], output: &Output) -> String {
    format!(
        "명령: rhwp {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn run_json(rel: &str) -> serde_json::Value {
    let p = sample(rel);
    let args = ["explain", p.to_str().unwrap(), "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout 이 순수 JSON 이 아닙니다 ({e}).\n{}",
            describe(&args, &output)
        )
    })
}

/// [#3289] 아카이브 실행 시 컴파일타임 경로는 빌드 러너 전용이므로,
/// nextest가 런타임에 재매핑해 주입하는 CARGO_BIN_EXE_rhwp를 우선한다.
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

#[test]
fn explain_json_envelope_contract() {
    let v = run_json(SAMPLE_FIELDS);
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert!(v["source"].is_string(), "{v}");
    assert!(v["format"].is_string(), "{v}");
    assert!(v["pageCount"].as_u64().is_some(), "{v}");
    assert!(v["paragraphCount"].as_u64().is_some(), "{v}");
    assert!(v["tables"].is_array(), "{v}");
    assert!(v["fields"].is_array(), "{v}");
    assert!(v["footnoteCount"].as_u64().is_some(), "{v}");
    assert!(v["endnoteCount"].as_u64().is_some(), "{v}");
    assert!(v["encrypted"].is_boolean(), "{v}");
    assert!(v["summary"].is_string(), "{v}");
}

#[test]
fn explain_reports_field_names() {
    let v = run_json(SAMPLE_FIELDS);
    let names: Vec<&str> = v["fields"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|f| f.as_str())
        .collect();
    assert!(
        names.len() >= 3,
        "이름 있는 필드가 여럿이어야 합니다: {names:?}"
    );
    assert!(names.contains(&"회사명"), "{names:?}");
    assert_eq!(v["tables"].as_array().unwrap().len(), 0, "{v}");
    let summary = v["summary"].as_str().unwrap();
    assert!(
        summary.contains("누름틀이"),
        "요약에 누름틀 문장이 있어야 합니다: {summary}"
    );
    assert!(summary.contains("회사명"), "{summary}");
    assert!(
        summary.contains("표는 없다"),
        "표 없는 문서는 '표는 없다' 문장이어야 합니다: {summary}"
    );
}

#[test]
fn explain_reports_table_shape_and_merge() {
    let v = run_json(SAMPLE_TABLE);
    let tables = v["tables"].as_array().unwrap();
    assert_eq!(tables.len(), 1, "{v}");
    let t = &tables[0];
    assert_eq!(t["index"], 0, "{t}");
    assert_eq!(t["rows"], 19, "{t}");
    assert_eq!(t["cols"], 9, "{t}");
    assert_eq!(t["hasMergedCells"], true, "{t}");
    // 표 항목에는 셀 텍스트·캡션이 없어야 한다 — 요약은 크기/병합 여부만 담는다.
    assert!(t.get("cells").is_none(), "{t}");
    assert!(t.get("text").is_none(), "{t}");
    let summary = v["summary"].as_str().unwrap();
    assert!(summary.contains("표 1(19×9, 병합 셀 있음)"), "{summary}");
    assert_eq!(v["fields"].as_array().unwrap().len(), 0, "{v}");
    assert!(summary.contains("누름틀은 없다"), "{summary}");
}

#[test]
fn explain_document_without_tables_or_fields_is_empty_not_error() {
    let v = run_json(SAMPLE_PLAIN);
    assert_eq!(v["tables"].as_array().unwrap().len(), 0, "{v}");
    assert_eq!(v["fields"].as_array().unwrap().len(), 0, "{v}");
    assert_eq!(v["footnoteCount"], 0, "{v}");
    assert_eq!(v["endnoteCount"], 0, "{v}");
    let summary = v["summary"].as_str().unwrap();
    assert!(summary.contains("표는 없다"), "{summary}");
    assert!(summary.contains("누름틀은 없다"), "{summary}");
    assert!(summary.contains("각주와 미주는 모두 없다"), "{summary}");
}

#[test]
fn explain_reports_footnote_count() {
    let v = run_json(SAMPLE_FOOTNOTE);
    let footnotes = v["footnoteCount"].as_u64().expect("footnoteCount");
    assert!(footnotes >= 1, "{v}");
    assert_eq!(v["endnoteCount"], 0, "{v}");
    let summary = v["summary"].as_str().unwrap();
    assert!(
        summary.contains(&format!("각주가 {footnotes}개")),
        "{summary}"
    );
}

#[test]
fn explain_reports_endnote_count() {
    let v = run_json(SAMPLE_ENDNOTE);
    let endnotes = v["endnoteCount"].as_u64().expect("endnoteCount");
    assert!(endnotes >= 1, "{v}");
    assert_eq!(v["footnoteCount"], 0, "{v}");
    let summary = v["summary"].as_str().unwrap();
    assert!(
        summary.contains(&format!("미주가 {endnotes}개")),
        "{summary}"
    );
}

#[test]
fn explain_default_output_is_human_summary() {
    let p = sample(SAMPLE_FIELDS);
    let args = ["explain", p.to_str().unwrap()];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        serde_json::from_str::<serde_json::Value>(&stdout).is_err(),
        "기본 출력은 JSON 이 아니어야 합니다(--json 전용).\n{}",
        describe(&args, &output)
    );
    assert!(stdout.contains("이 문서는"), "{stdout}");
}

#[test]
fn explain_missing_file_exit_runtime_silent_stdout() {
    let args = ["explain", "없는파일-explain.hwp", "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        describe(&args, &output)
    );
    assert!(
        output.stdout.is_empty(),
        "실패 경로 stdout 은 비어야 합니다.\n{}",
        describe(&args, &output)
    );
}

#[test]
fn explain_usage_error_exit_two() {
    let args = ["explain"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
}

#[test]
fn explain_multiple_files_exit_usage() {
    let p = sample(SAMPLE_FIELDS);
    let args = [
        "explain",
        p.to_str().unwrap(),
        p.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
}

// ── 암호 문서 — 다른 명령과 같은 `load_document` 규약을 그대로 물려받는다 ──────

#[test]
fn explain_encrypted_document_without_password_is_usage_error() {
    // [#3828] 규약: 다른 조회 명령(info 등)과 같은 종료 코드(EXIT_USAGE=2) —
    // "암호로 보호돼 있어 상세 분석 불가"는 이 계약을 통해 자동으로 지켜진다.
    let p = sample(SAMPLE_ENCRYPTED);
    let args = ["explain", p.to_str().unwrap(), "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
    assert!(
        output.stdout.is_empty(),
        "암호 문서 거부 경로 stdout 은 비어야 합니다.\n{}",
        describe(&args, &output)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("암호"), "{stderr}");
}

#[test]
fn explain_encrypted_document_wrong_password_exit_runtime() {
    let p = sample(SAMPLE_ENCRYPTED);
    let args = [
        "explain",
        p.to_str().unwrap(),
        "--password",
        "wrong-password",
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        describe(&args, &output)
    );
}

#[test]
fn explain_encrypted_document_with_password_reports_encrypted_true() {
    let p = sample(SAMPLE_ENCRYPTED);
    let args = [
        "explain",
        p.to_str().unwrap(),
        "--password",
        "123456",
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(v["encrypted"], true, "{v}");
    let summary = v["summary"].as_str().unwrap();
    assert!(summary.contains("암호로 보호돼 있다"), "{summary}");
}
