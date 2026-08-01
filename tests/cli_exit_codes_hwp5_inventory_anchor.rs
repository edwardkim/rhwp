//! `hwp5-inventory` / `hwp5-inventory-diff` / `hwp5-anchor-trace` 종료 코드 계약.
//!
//! 세 명령의 진입점은 `pub fn run(args: &[String])` — 반환형이 유닛이라 `main` 의
//! `exit_with`를 통과하지 못하고 항상 프로세스 기본 종료 코드(0)로 끝났다. 인자를
//! 아예 빠뜨리든, 존재하지 않는 파일을 넘기든 무조건 exit 0이었다 — `&&`나 `set -e`로
//! 엮은 스크립트, CI 게이트가 실패를 성공으로 읽는다 (#2707/#3382 계열과 동일한 결함
//! 유형이 이 세 명령에만 남아 있었다).
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SAMPLE: &str = "samples/2010-01-06.hwp";

fn sample_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
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

fn assert_code(args: &[&str], expected: i32) -> Output {
    let output = run(args);
    assert_eq!(
        output.status.code(),
        Some(expected),
        "{}",
        describe(args, &output)
    );
    output
}

/// 인자 없이 호출 → 사용법 오류(2). 종전에는 exit 0이었다.
#[test]
fn missing_arguments_report_usage_error() {
    for cmd in ["hwp5-inventory", "hwp5-inventory-diff", "hwp5-anchor-trace"] {
        let output = assert_code(&[cmd], 2);
        assert!(
            String::from_utf8_lossy(&output.stdout).trim().is_empty(),
            "사용법 안내는 stderr 로 나가야 합니다({cmd}): {}",
            describe(&[cmd], &output)
        );
    }
}

/// 명시적 `--help`는 성공(0)이어야 한다 — 도움말을 물어본 호출까지 실패로 만들면 안 된다.
#[test]
fn explicit_help_reports_success() {
    for cmd in ["hwp5-inventory", "hwp5-inventory-diff", "hwp5-anchor-trace"] {
        assert_code(&[cmd, "--help"], 0);
    }
}

/// 존재하지 않는 입력 파일 → 런타임 실패(1). 종전에는 이 경로도 exit 0이었다.
#[test]
fn unreadable_input_reports_runtime_failure() {
    assert_code(&["hwp5-inventory", "does-not-exist.hwp"], 1);
    assert_code(
        &[
            "hwp5-inventory-diff",
            "does-not-exist-a.hwp",
            "does-not-exist-b.hwp",
        ],
        1,
    );
    assert_code(
        &["hwp5-anchor-trace", "does-not-exist.hwp", "--needle", "x"],
        1,
    );
}

/// 성공 경로는 여전히 0이어야 한다 (회귀 방지).
#[test]
fn successful_runs_return_zero() {
    let sample = sample_path();
    let sample = sample.to_str().expect("valid utf8 path");

    assert_code(&["hwp5-inventory", sample], 0);
    assert_code(&["hwp5-inventory-diff", sample, sample], 0);
    assert_code(&["hwp5-anchor-trace", sample, "--needle", "x"], 0);
}

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}
