//! [#3884 G2] dump·diag·bench 미지 플래그 거부 계약.
//!
//! 이 셋은 capabilities 에 `json`·`flags` 를 선언하지 않는 진단 명령이다.
//! `--json` / `--bogus-flag` 를 침묵 무시하고 exit 0 으로 사람용 텍스트를 내면
//! 에이전트는 JSON 을 기대하고 파싱하다 깨진다. 모르는 옵션은 사용법 오류:
//! exit 2, stdout 0바이트, stderr 에 그 플래그 이름.
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

fn assert_rejects(args: &[&str], flag: &str) {
    let out = run(args);
    assert_eq!(
        out.status.code(),
        Some(2),
        "미지 플래그는 사용법 오류다: {}",
        describe(args, &out)
    );
    assert!(
        out.stdout.is_empty(),
        "거부하면서 stdout 을 오염시키면 안 된다: {}",
        describe(args, &out)
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains(flag),
        "무엇이 거부됐는지 stderr 가 말해야 한다({flag}): {}",
        describe(args, &out)
    );
}

#[test]
fn dump_rejects_bogus_flag() {
    let s = sample();
    assert_rejects(
        &["dump", s.to_str().unwrap(), "--bogus-flag"],
        "--bogus-flag",
    );
}

#[test]
fn dump_rejects_json_until_it_has_a_json_contract() {
    let s = sample();
    assert_rejects(&["dump", s.to_str().unwrap(), "--json"], "--json");
}

#[test]
fn dump_rejects_flag_in_file_position() {
    assert_rejects(&["dump", "--json"], "--json");
}

#[test]
fn dump_rejects_a_flag_consumed_as_a_filter_value() {
    let s = sample();
    assert_rejects(
        &["dump", s.to_str().unwrap(), "--section", "--bogus-flag"],
        "--section",
    );
}

#[test]
fn dump_still_accepts_declared_filters() {
    let s = sample();
    let args = ["dump", s.to_str().unwrap(), "--section", "0"];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    assert!(!out.stdout.is_empty(), "{}", describe(&args, &out));
}

#[test]
fn diag_rejects_bogus_flag() {
    let s = sample();
    assert_rejects(
        &["diag", s.to_str().unwrap(), "--bogus-flag"],
        "--bogus-flag",
    );
}

#[test]
fn diag_rejects_json_flag() {
    let s = sample();
    assert_rejects(&["diag", s.to_str().unwrap(), "--json"], "--json");
}

#[test]
fn diag_rejects_flag_in_file_position() {
    assert_rejects(&["diag", "--verbose"], "--verbose");
}

#[test]
fn bench_rejects_bogus_flag() {
    let s = sample();
    assert_rejects(
        &["bench", s.to_str().unwrap(), "--bogus-flag"],
        "--bogus-flag",
    );
}

#[test]
fn bench_rejects_json_flag() {
    let s = sample();
    assert_rejects(&["bench", s.to_str().unwrap(), "--json"], "--json");
}

#[test]
fn bench_rejects_a_flag_consumed_as_an_option_value() {
    let s = sample();
    assert_rejects(
        &["bench", s.to_str().unwrap(), "--iters", "--bogus-flag"],
        "--iters",
    );
}
