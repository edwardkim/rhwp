//! [#3349] `export-text` 위치 인자 파싱 회귀 테스트.
//!
//! 계약: 옵션은 파일 앞뒤 어디에 와도 같은 결과를 낸다 (export-structure/export-tables 와
//! 동일 규약). 파일 선행을 강제하던 시절에는 `-p 0 --json 파일` 에서 `-p` 가 파일로 잡혀
//! "알 수 없는 옵션: 0" 으로 죽었다.
#![cfg(not(target_arch = "wasm32"))]

use std::process::{Command, Output};

/// 파싱까지 성공하는 실제 샘플 (cli_json_contract.rs 와 동일).
const SAMPLE: &str = "samples/hwp3-sample.hwp";

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
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

fn stdout_json(args: &[&str]) -> serde_json::Value {
    let output = run(args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "exit 0 이어야 합니다.\n{}",
        describe(args, &output)
    );
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout 이 순수 JSON 이 아닙니다 ({e}).\n{}",
            describe(args, &output)
        )
    })
}

/// 버그 재현 형태 그대로 — 옵션 전부가 파일 앞에 와도 동작한다.
#[test]
fn options_before_file_succeeds() {
    let value = stdout_json(&["export-text", "--json", "-p", "0", SAMPLE]);
    assert_eq!(value["schemaVersion"], "1.0");
    assert_eq!(value["source"], SAMPLE);
    assert_eq!(value["pageCount"], 1);
    assert_eq!(value["pages"][0]["page"], 0);
}

/// `--json` 특례가 아니라 모든 옵션이 위치 무관임을 고정한다.
#[test]
fn page_before_json_before_file_succeeds() {
    let value = stdout_json(&["export-text", "-p", "0", "--json", SAMPLE]);
    assert_eq!(value["pages"][0]["page"], 0);
}

/// 옵션 순서가 달라도 결과 JSON 은 완전히 같다.
#[test]
fn all_orders_produce_identical_json() {
    let file_first = stdout_json(&["export-text", SAMPLE, "--json", "-p", "0"]);
    let flag_first = stdout_json(&["export-text", "--page", "0", "--json", SAMPLE]);
    let interleaved = stdout_json(&["export-text", "--json", SAMPLE, "-p", "0"]);
    assert_eq!(file_first, flag_first);
    assert_eq!(file_first, interleaved);
}

/// 파일을 두 번 주면 조용히 덮어쓰지 않고 즉시 사용법 오류다.
#[test]
fn duplicate_file_is_usage_error() {
    let args = ["export-text", SAMPLE, SAMPLE, "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "exit 2 여야 합니다.\n{}",
        describe(&args, &output)
    );
    assert!(
        output.stdout.is_empty(),
        "실패 시 stdout 은 0바이트여야 합니다.\n{}",
        describe(&args, &output)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("입력 파일은 하나만"),
        "중복 positional 오류 메시지가 나와야 합니다.\n{}",
        describe(&args, &output)
    );
}

/// 알 수 없는 플래그는 여전히 사용법 오류로 잡는다 (기존 계약 유지).
#[test]
fn unknown_flag_is_usage_error() {
    let args = ["export-text", "--nope", SAMPLE];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "exit 2 여야 합니다.\n{}",
        describe(&args, &output)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("알 수 없는 옵션"),
        "알 수 없는 옵션 메시지가 나와야 합니다.\n{}",
        describe(&args, &output)
    );
}
