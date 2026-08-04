//! [#3274] `ir-diff --json` 출력 계약 + 오류 종료 코드 정정 회귀 테스트.
//!
//! 계약: `--json` 의 stdout 은 순수 JSON 한 줄이고 `schemaVersion` 을 포함한다.
//! 종료 코드 — 0: 동일, 3: 차이 발견(--json 전용), 1: 읽기/파싱 실패, 2: 사용법 오류.
//! 기본(텍스트) 모드의 정상 비교는 차이가 있어도 exit 0 을 유지한다(기존 소비자 보호).
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SAMPLE_A: &str = "samples/hwp3-sample.hwp";
const SAMPLE_B: &str = "samples/SO-SUEOP.hwp";
/// 구역 수 차이 경로용: SAMPLE_A(1구역) 과 구역 수가 다른 다구역 문서.
const SAMPLE_MULTI: &str = "samples/aift.hwp";

/// [#3274] 봉투 계약 불변식 — 이 세 관계가 늘 성립해야 한다.
/// `identical` ⇔ `diffCount == 0` ⇔ `categories` 가 비어 있음.
/// 구역 수 차이가 diffCount 에 미집계되던 버그는 바로 이 불변식을 깨뜨렸다
/// (categories 는 비어있지 않은데 identical:true·diffCount:0).
fn assert_envelope_invariants(v: &serde_json::Value) {
    let identical = v["identical"].as_bool().expect("identical bool");
    let diff_count = v["diffCount"].as_u64().expect("diffCount u64");
    let cats = v["categories"].as_object().expect("categories 객체");
    assert_eq!(identical, diff_count == 0, "identical ⇔ diffCount==0: {v}");
    assert_eq!(
        identical,
        cats.is_empty(),
        "identical ⇔ categories 비어있음: {v}"
    );
}

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

fn parse_stdout_json(args: &[&str], output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout 이 순수 JSON 이 아닙니다 ({e}).\n{}",
            describe(args, output)
        )
    })
}

#[test]
fn ir_diff_json_identical_exit_zero() {
    let a = sample(SAMPLE_A);
    let a_str = a.to_str().unwrap();
    let args = ["ir-diff", a_str, a_str, "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout.lines().filter(|l| !l.trim().is_empty()).count(),
        1,
        "봉투는 한 줄이어야 합니다.\n{}",
        describe(&args, &output)
    );
    let v = parse_stdout_json(&args, &output);
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert!(v["a"].is_string(), "{v}");
    assert!(v["b"].is_string(), "{v}");
    assert_eq!(v["identical"], true, "{v}");
    assert_eq!(v["diffCount"], 0, "{v}");
    assert!(v["categories"].is_object(), "{v}");
    assert_envelope_invariants(&v);
}

#[test]
fn ir_diff_json_differs_exit_three() {
    // [#2707] 계약의 "IR 차이" 코드(3)와 같은 의미 — 파이프라인 게이트가 성립한다.
    let a = sample(SAMPLE_A);
    let b = sample(SAMPLE_B);
    let args = [
        "ir-diff",
        a.to_str().unwrap(),
        b.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(3),
        "{}",
        describe(&args, &output)
    );

    let v = parse_stdout_json(&args, &output);
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert_eq!(v["identical"], false, "{v}");
    assert!(v["diffCount"].as_u64().unwrap() >= 1, "{v}");
    assert!(!v["categories"].as_object().unwrap().is_empty(), "{v}");
    assert_envelope_invariants(&v);
}

#[test]
fn ir_diff_json_section_count_diff_is_counted() {
    // [#3274 회귀] 구역 수 차이가 diffCount 에 집계되는지 — 종전엔 total_diffs
    // 선언이 구역 수 비교 뒤에 있어 이 차이가 누락됐고, 구역 하나 덧붙은 변환본이
    // identical:true·exit 0 으로 게이트를 통과했다.
    let a = sample(SAMPLE_A); // 1구역
    let m = sample(SAMPLE_MULTI); // 다구역
    let args = [
        "ir-diff",
        a.to_str().unwrap(),
        m.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(3),
        "{}",
        describe(&args, &output)
    );

    let v = parse_stdout_json(&args, &output);
    // 구역 수 카테고리가 실제로 봉투에 잡혀야 한다.
    let cats = v["categories"].as_object().expect("categories 객체");
    assert!(
        cats.keys().any(|k| k.contains("구역")),
        "구역 수 차이가 categories 에 나타나야 합니다: {v}"
    );
    // 그리고 그 차이가 diffCount 에도 반영되어 게이트가 성립해야 한다(불변식).
    assert_eq!(v["identical"], false, "{v}");
    assert_envelope_invariants(&v);
}

#[test]
fn ir_diff_json_flag_not_swallowed_as_value() {
    // [#3274] 값 누락 옵션이 뒤따르는 플래그를 값으로 삼키지 않는다 — `--max-lines --json`
    // 에서 --json 이 살아남아 게이트(exit 3)가 성립해야 한다. 종전엔 텍스트 모드로
    // 떨어져 차이가 있어도 exit 0 으로 조용히 통과했다.
    let a = sample(SAMPLE_A);
    let b = sample(SAMPLE_B);
    let args = [
        "ir-diff",
        a.to_str().unwrap(),
        b.to_str().unwrap(),
        "--max-lines",
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(3),
        "{}",
        describe(&args, &output)
    );
    let v = parse_stdout_json(&args, &output);
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert_eq!(v["identical"], false, "{v}");
}

#[test]
fn ir_diff_json_missing_file_exit_runtime_silent_stdout() {
    let a = sample(SAMPLE_A);
    let args = [
        "ir-diff",
        a.to_str().unwrap(),
        "없는파일-irdiff.hwp",
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
}

#[test]
fn ir_diff_usage_error_exit_two() {
    // [#3274] 인자 부족은 사용법 오류다 — 종전엔 exit 0 으로 끝나던 결함.
    let args = ["ir-diff"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
}

#[test]
fn ir_diff_default_mode_missing_file_exit_runtime() {
    // [#3274] 기본 모드도 읽기 실패는 exit 1 (#2707 정렬) — 종전 exit 0 결함 정정.
    let a = sample(SAMPLE_A);
    let args = [
        "ir-diff",
        a.to_str().unwrap(),
        "없는파일-irdiff-default.hwp",
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
fn ir_diff_default_mode_diff_found_still_exit_zero() {
    // 무회귀 가드: 기본(텍스트) 모드는 차이가 있어도 exit 0 — 기존 소비자 계약.
    let a = sample(SAMPLE_A);
    let b = sample(SAMPLE_B);
    let args = ["ir-diff", a.to_str().unwrap(), b.to_str().unwrap()];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("비교 완료"),
        "기본 출력 형식이 바뀌면 안 됩니다.\n{}",
        describe(&args, &output)
    );
}

/// [#3289] 아카이브 실행 시 컴파일타임 경로는 빌드 러너 전용이므로,
/// nextest가 런타임에 재매핑해 주입하는 CARGO_BIN_EXE_rhwp를 우선한다.
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}
