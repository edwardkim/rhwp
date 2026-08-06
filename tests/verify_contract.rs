//! [#4113 / #3918 승격 2호] `verify` — 독립 사후검증 게이트의 계약.
//!
//! 계약: 전부 만족 exit 0 / 불일치 **봉투를 먼저 내고** exit 3 / 실행 실패 stdout
//! 0 B + exit 1 / 조립 오류 exit 2. 봉투는 순수 JSON 하나이고 조건별 판정이
//! 데이터로 실리며, 문서 파생 값(expectations[].actual)은 출처 표지가 가린다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SAMPLE: &str = "samples/field-01.hwp";

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn stdout_json(out: &Output) -> serde_json::Value {
    serde_json::from_slice(&out.stdout).expect("stdout 이 순수 JSON 이 아니다")
}

#[test]
fn all_pass_is_exit_zero_with_envelope() {
    let s = sample();
    let out = run(&[
        "verify",
        s.to_str().unwrap(),
        "--expect-pages",
        "3",
        "--expect-format",
        "hwp5",
        "--json",
    ]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_eq!(v["verdict"], "pass");
    assert_eq!(v["failCount"], 0);
    assert_eq!(v["passCount"], 2);
    assert_eq!(v["schemaVersion"], "1.0");
    let kinds: Vec<&str> = v["expectations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["kind"].as_str().unwrap())
        .collect();
    assert_eq!(kinds, ["pages", "format"]);
}

#[test]
fn mismatch_emits_envelope_then_exit_three() {
    let s = sample();
    let out = run(&[
        "verify",
        s.to_str().unwrap(),
        "--expect-pages",
        "99",
        "--json",
    ]);
    assert_eq!(out.status.code(), Some(3), "판정 불일치는 exit 3");
    let v = stdout_json(&out);
    assert_eq!(v["verdict"], "fail");
    assert_eq!(v["failCount"], 1);
    let e = &v["expectations"][0];
    assert_eq!(e["pass"], false);
    assert_eq!(e["expected"], 99);
    assert_eq!(e["actual"], 3, "field-01 은 3쪽이다");
}

#[test]
fn contains_and_field_mark_untrusted_actuals() {
    let s = sample();
    let out = run(&[
        "verify",
        s.to_str().unwrap(),
        "--expect-contains",
        "회사명",
        "--expect-not-contains",
        "존재할리없는문자열zz",
        "--expect-field",
        "회사명=",
        "--json",
    ]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_eq!(v["untrustedContent"], true, "actual 은 문서 파생 값이다");
    let fields: Vec<&str> = v["untrustedFields"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|f| f.as_str())
        .collect();
    assert!(
        fields.contains(&"expectations[].actual"),
        "출처 지도의 선언이 봉투 표지에 나타나야 한다: {fields:?}"
    );
}

#[test]
fn runtime_failure_keeps_stdout_empty() {
    let out = run(&["verify", "없는파일.hwp", "--expect-pages", "1", "--json"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        out.stdout.is_empty(),
        "실행 실패는 stdout 0 B — 부분 봉투 금지"
    );
}

#[test]
fn usage_errors_are_exit_two() {
    let s = sample();
    // 기대 조건 0개
    let out = run(&["verify", s.to_str().unwrap(), "--json"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    // 미지 옵션 침묵 무시 금지
    let out = run(&[
        "verify",
        s.to_str().unwrap(),
        "--expect-pages",
        "3",
        "--bogus",
    ]);
    assert_eq!(out.status.code(), Some(2));
    // 잘못된 형식 토큰
    let out = run(&["verify", s.to_str().unwrap(), "--expect-format", "pdf"]);
    assert_eq!(out.status.code(), Some(2));
}
