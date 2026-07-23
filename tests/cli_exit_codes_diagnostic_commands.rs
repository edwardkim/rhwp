//! [#2707 후속] `info`/`dump-note-shape`/`dump-endnote-lines`/`dump-pages`/
//! `dump-records`/`build-from-ingest` 도 export 계열과 동일한 종료 코드 계약을 따라야 한다.
//!
//! #2707 은 export-* / convert / export-hwpx 만 고쳤고, 같은 클래스(치명 실패에도
//! 종료 코드 0)가 이 진단·조립 명령들에 남아 있다고 명시적으로 §6.1 에 기록했다
//! (`mydocs/report/task_m100_2707_report.md`). 이 테스트는 그 잔여를 봉인한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SAMPLE: &str = "samples/hwp3-sample.hwp";
/// HWP5(CFB) 샘플 — `dump-records` 는 HWP3 CFB 아닌 입력을 지원하지 않는다.
const HWP5_SAMPLE: &str = "samples/2010-01-06.hwp";

fn sample_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn hwp5_sample_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(HWP5_SAMPLE)
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

/// 인자 없이 호출 → 사용법 오류(2).
#[test]
fn missing_arguments_report_usage_error() {
    for cmd in ["info", "dump-note-shape", "dump-pages", "dump-records"] {
        assert_code(&[cmd], 2);
    }
    // dump-endnote-lines 는 인자 4개 미만이면 사용법 오류.
    assert_code(&["dump-endnote-lines", "x.hwp"], 2);
    assert_code(&["build-from-ingest"], 2);
}

/// 존재하지 않는 입력 파일 → 런타임 실패(1). #2707 이전에는 전부 0이었다.
#[test]
fn unreadable_input_reports_runtime_failure() {
    assert_code(&["info", "does-not-exist.hwp"], 1);
    assert_code(&["dump-note-shape", "does-not-exist.hwp"], 1);
    assert_code(&["dump-pages", "does-not-exist.hwp"], 1);
    assert_code(&["dump-records", "does-not-exist.hwp"], 1);
    assert_code(
        &["dump-endnote-lines", "does-not-exist.hwp", "0", "0", "0"],
        1,
    );
    assert_code(
        &["build-from-ingest", "does-not-exist.json", "-o", "out.hwpx"],
        1,
    );
}

/// dump-pages 페이지 범위 초과 → 사용법 오류(2) (형제 명령과 정합, #2551 후속 확인).
#[test]
fn dump_pages_out_of_range_reports_usage_error() {
    let sample = sample_path();
    let sample = sample.to_str().expect("valid utf8 path");
    assert_code(&["dump-pages", sample, "-p", "999999"], 2);
}

/// build-from-ingest 출력 경로 누락 → 사용법 오류(2).
#[test]
fn build_from_ingest_missing_output_reports_usage_error() {
    assert_code(
        &[
            "build-from-ingest",
            "tools/rhwp-ingest/schema/sample_minimal.json",
        ],
        2,
    );
}

/// 성공 경로는 여전히 0이어야 한다 (회귀 방지).
#[test]
fn successful_diagnostic_commands_return_zero() {
    let sample = sample_path();
    let sample = sample.to_str().expect("valid utf8 path");
    for cmd in ["info", "dump-note-shape", "dump-pages"] {
        let output = run(&[cmd, sample]);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}",
            describe(&[cmd, sample], &output)
        );
    }

    let hwp5_sample = hwp5_sample_path();
    let hwp5_sample = hwp5_sample.to_str().expect("valid utf8 path");
    let output = run(&["dump-records", hwp5_sample]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&["dump-records", hwp5_sample], &output)
    );
}
