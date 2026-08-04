//! [#3329] `edit fill-fields` 출력·안전 계약 회귀 테스트 (Stage 3 최소 조각).
//!
//! 편집 명령의 계약은 조회 명령보다 무겁다 — 파일을 바꾸기 때문이다.
//! ① `--dry-run` 은 절대 파일을 만들지 않는다 ② 실패하면 출력 파일을 쓰지 않는다
//! ③ 실제로 값이 반영됐는지는 **다시 읽어서** 확인한다.
//! 종료 코드는 #2707 계약(0/1/2)을 따른다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 누름틀 11개(회사명/작성자/부서명/전화번호/이메일/제목/목차1×5).
const SAMPLE: &str = "samples/field-01.hwp";

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn temp_out(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-edit-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ))
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
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

fn parse_json(args: &[&str], output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout 이 순수 JSON 이 아닙니다 ({e}).\n{}",
            describe(args, output)
        )
    })
}

#[test]
fn fill_fields_dry_run_reports_without_writing_file() {
    // 안전장치의 핵심: --dry-run 은 무엇이 바뀔지만 보고하고 파일을 만들지 않는다.
    let p = sample();
    let out = temp_out("dryrun");
    let args = [
        "edit",
        "fill-fields",
        p.to_str().unwrap(),
        "--data",
        r#"{"회사명":"주식회사 A"}"#,
        "-o",
        out.to_str().unwrap(),
        "--dry-run",
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    let v = parse_json(&args, &output);
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert_eq!(v["dryRun"], true, "{v}");
    assert!(v["source"].is_string(), "{v}");
    assert_eq!(v["filledCount"].as_u64().unwrap(), 1, "{v}");

    assert!(
        !out.exists(),
        "--dry-run 은 파일을 만들면 안 됩니다: {}",
        out.display()
    );
}

#[test]
fn fill_fields_writes_and_value_survives_reparse() {
    // 실제로 값이 들어갔는지는 "다시 읽어서" 확인해야 한다 — 보고만 믿지 않는다.
    let p = sample();
    let out = temp_out("write");
    let args = [
        "edit",
        "fill-fields",
        p.to_str().unwrap(),
        "--data",
        r#"{"회사명":"주식회사 검증"}"#,
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    assert!(out.exists(), "출력 파일이 생성되어야 합니다");

    let v = parse_json(&args, &output);
    assert_eq!(v["dryRun"], false, "{v}");
    assert_eq!(v["filledCount"].as_u64().unwrap(), 1, "{v}");
    assert!(v["output"].is_string(), "{v}");

    // 산출물을 fields --json 으로 다시 읽어 값이 실제로 반영됐는지 대조한다.
    let reread = run(&["fields", out.to_str().unwrap(), "--json"]);
    let rv: serde_json::Value =
        serde_json::from_slice(&reread.stdout).expect("산출물을 fields --json 으로 읽지 못함");
    let company = rv["fields"]
        .as_array()
        .expect("fields 배열")
        .iter()
        .find(|f| f["name"] == "회사명")
        .unwrap_or_else(|| panic!("회사명 필드를 찾지 못함: {rv}"));
    assert_eq!(
        company["value"], "주식회사 검증",
        "값이 산출물에 반영되어야 합니다: {company}"
    );

    let _ = std::fs::remove_file(&out);
}

#[test]
fn fill_fields_reports_unknown_field_names() {
    // 문서에 없는 이름은 조용히 무시하지 않고 보고한다 — 에이전트가 오타를 알아야 한다.
    let p = sample();
    let out = temp_out("unknown");
    let args = [
        "edit",
        "fill-fields",
        p.to_str().unwrap(),
        "--data",
        r#"{"회사명":"A","존재하지않는필드":"B"}"#,
        "-o",
        out.to_str().unwrap(),
        "--dry-run",
        "--json",
    ];
    let output = run(&args);
    let v = parse_json(&args, &output);
    let missing = v["notFound"].as_array().expect("notFound 배열");
    assert!(
        missing.iter().any(|m| m == "존재하지않는필드"),
        "없는 필드 이름이 보고되어야 합니다: {v}"
    );
    assert_eq!(v["filledCount"].as_u64().unwrap(), 1, "{v}");
}

#[test]
fn fill_fields_missing_file_exit_runtime_and_no_output() {
    let out = temp_out("missing");
    let args = [
        "edit",
        "fill-fields",
        "없는파일-edit.hwp",
        "--data",
        r#"{"a":"b"}"#,
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
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
    assert!(
        !out.exists(),
        "실패 시 출력 파일을 쓰면 안 됩니다: {}",
        out.display()
    );
}

#[test]
fn fill_fields_invalid_json_data_exit_usage() {
    let p = sample();
    let args = [
        "edit",
        "fill-fields",
        p.to_str().unwrap(),
        "--data",
        "{이건 JSON 이 아님",
        "--dry-run",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
}

#[test]
fn fill_fields_missing_data_exit_usage() {
    let p = sample();
    let args = ["edit", "fill-fields", p.to_str().unwrap()];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
}

#[test]
fn edit_unknown_subcommand_exit_usage() {
    let args = ["edit", "no-such-action"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
}
