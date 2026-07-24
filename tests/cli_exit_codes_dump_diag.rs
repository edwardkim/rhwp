//! [#3172] `dump`/`diag` CLI 종료 코드 계약 회귀 테스트.
//!
//! `#2707`(PR #2711)·`#3169`(PR #3171)이 각각 export-*/convert/export-hwpx 와
//! info/dump-note-shape/dump-endnote-lines/dump-pages/dump-records/build-from-ingest 에
//! 적용한 계약(0 성공 / 1 런타임 실패 / 2 사용법 오류)을, 두 PR이 명시적으로 범위 밖에
//! 남겨둔 `dump`(dump_controls)·`diag`(diag_document) 에도 동일하게 확장한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SAMPLE: &str = "samples/hwp3-sample.hwp";

fn sample_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn unique_temp_path(label: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "rhwp-exit-codes-dump-diag-{label}-{}-{nonce}",
        std::process::id()
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

fn assert_code(args: &[&str], expected: i32) -> Output {
    let output = run(args);
    assert_eq!(
        output.status.code(),
        Some(expected),
        "종료 코드 {expected} 를 기대했다\n{}",
        describe(args, &output)
    );
    output
}

#[test]
fn missing_arguments_report_usage_error() {
    for command in ["dump", "diag"] {
        assert_code(&[command], 2);
    }
}

#[test]
fn unreadable_input_reports_runtime_failure() {
    let missing = unique_temp_path("missing.hwp");
    let missing = missing.to_str().expect("utf-8 경로").to_string();

    assert_code(&["dump", &missing], 1);
    assert_code(&["diag", &missing], 1);
}

#[test]
fn unparseable_input_reports_runtime_failure() {
    let bogus = unique_temp_path("bogus-not-hwp");
    std::fs::write(&bogus, b"not a real hwp document").expect("파싱 실패 유도용 파일 생성");
    let bogus = bogus.to_str().expect("utf-8 경로").to_string();

    assert_code(&["dump", &bogus], 1);
    assert_code(&["diag", &bogus], 1);

    let _ = std::fs::remove_file(&bogus);
}

#[test]
fn successful_run_returns_zero() {
    let sample = sample_path();
    let sample = sample.to_str().expect("utf-8 경로");

    assert_code(&["dump", sample], 0);
    assert_code(&["diag", sample], 0);
}
