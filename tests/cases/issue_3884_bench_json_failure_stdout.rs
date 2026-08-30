//! [#3884 G1] `bench` 실패 경로 stdout 0바이트 계약.
//!
//! `capabilities.jsonContract.failure` 는 단건 실패 시 stdout 0바이트를 선언한다.
//! `bench <문서> --json` 이 사람용 배너·표를 stdout 에 흘리며 exit 1 이 되던 구멍이
//! 그 대표 증상이다. `--json` 자체는 아직 JSON 봉투가 없어 사용법 오류(exit 2)로
//! 거부하되, 어떤 실패든 stdout 은 비고 사람용 전말은 stderr 로만 나간다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SAMPLE: &str = "samples/field-01.hwp";

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행")
}

fn describe(args: &[&str], out: &Output) -> String {
    format!(
        "args={args:?}\nexit={:?}\nstdout={}\nstderr={}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

#[test]
fn bench_json_on_missing_file_keeps_stdout_empty() {
    let args = ["bench", "no-such-file-3884.hwp", "--json"];
    let out = run(&args);
    assert_ne!(
        out.status.code(),
        Some(0),
        "실패인데 성공이면 안 된다: {}",
        describe(&args, &out)
    );
    assert!(
        out.stdout.is_empty(),
        "bench --json 실패 경로 stdout 은 0바이트여야 한다: {}",
        describe(&args, &out)
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        !err.is_empty(),
        "사람용 전말은 stderr 로 나가야 한다: {}",
        describe(&args, &out)
    );
}

#[test]
fn bench_missing_file_keeps_stdout_empty() {
    let args = ["bench", "no-such-file-3884.hwp"];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(1), "{}", describe(&args, &out));
    assert!(
        out.stdout.is_empty(),
        "전건 실패인데 stdout 이 비지 않았다: {}",
        describe(&args, &out)
    );
    assert!(
        !out.stderr.is_empty(),
        "실패 사유는 stderr 로 나가야 한다: {}",
        describe(&args, &out)
    );
}

#[test]
fn bench_partial_failure_keeps_stdout_empty() {
    let s = sample();
    let args = [
        "bench",
        s.to_str().unwrap(),
        "no-such-file-3884.hwp",
        "-n",
        "1",
    ];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(1), "{}", describe(&args, &out));
    assert!(
        out.stdout.is_empty(),
        "일부만 실패해도 stdout 은 비어야 한다: {}",
        describe(&args, &out)
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("no-such-file-3884.hwp"),
        "실패한 경로는 stderr 가 말해야 한다: {}",
        describe(&args, &out)
    );
}

#[test]
fn bench_json_flag_does_not_leak_human_table() {
    let s = sample();
    let args = ["bench", s.to_str().unwrap(), "--json"];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(2), "{}", describe(&args, &out));
    assert!(
        out.stdout.is_empty(),
        "--json 을 파일로 접어 반쪽 표를 stdout 에 흘리면 안 된다: {}",
        describe(&args, &out)
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("--json"),
        "무엇이 거부됐는지 stderr 가 말해야 한다: {}",
        describe(&args, &out)
    );
}

#[test]
fn bench_success_still_prints_the_table() {
    let s = sample();
    let args = ["bench", s.to_str().unwrap(), "-n", "1"];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains("=== bench:"),
        "성공하면 배너+표가 그대로 나와야 한다: {}",
        describe(&args, &out)
    );
}
