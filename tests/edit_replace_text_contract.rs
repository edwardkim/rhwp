//! [#3373] `edit replace-text` 출력·안전 계약 회귀 테스트 (Stage 3 두 번째 조각).
//!
//! fill-fields(#3329)와 같은 편집 계약을 따른다 — ① `--dry-run` 은 절대 파일을 만들지
//! 않는다 ② 실패하면 출력 파일을 쓰지 않는다 ③ 반영 여부는 **다시 읽어서** 확인한다.
//! 추가 계약: ④ 치환 0건이면 출력 파일을 만들지 않는다 (무변경 산출물 금지).
//! 종료 코드는 #2707 계약(0/1/2)을 따른다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// "제안서" 등 반복 문자열을 가진 실제 문서 (누름틀 서식, 3쪽).
const SAMPLE: &str = "samples/field-01.hwp";

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn temp_out(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-replace-{tag}-{}-{}.hwp",
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

/// 검색으로 원본 매치 수를 얻는다 — 치환 기대값의 독립 출처.
fn count_matches(path: &Path, needle: &str) -> u64 {
    let args = ["search", path.to_str().unwrap(), needle, "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    parse_json(&args, &output)["matchCount"]
        .as_u64()
        .expect("matchCount")
}

/// 핵심 루프 — 치환 후 산출물을 다시 검색해 반영을 기계로 대조한다.
#[test]
fn replace_applies_and_verifies_by_reread() {
    let sample = sample();
    let needle = "회사";
    let expected = count_matches(&sample, needle);
    assert!(expected >= 1, "샘플에 매치가 있어야 합니다");

    let out = temp_out("apply");
    let args = [
        "edit",
        "replace-text",
        sample.to_str().unwrap(),
        "--find",
        needle,
        "--replace",
        "기관",
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
    let v = parse_json(&args, &output);
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert_eq!(v["replacedCount"].as_u64(), Some(expected), "{v}");
    assert_eq!(v["dryRun"], false, "{v}");
    assert!(out.exists(), "산출물이 있어야 합니다");

    // 재독 대조 — 산출물에서 원문은 0건, 새 문자열은 기대 건수 이상.
    assert_eq!(
        count_matches(&out, needle),
        0,
        "원문이 남아 있으면 안 됩니다"
    );
    assert!(count_matches(&out, "기관") >= expected);
    let _ = std::fs::remove_file(&out);
}

/// `--dry-run` 은 절대 파일을 만들지 않고, 건수는 실제 치환과 같다.
#[test]
fn dry_run_reports_count_without_output() {
    let sample = sample();
    let expected = count_matches(&sample, "회사");
    let out = temp_out("dry");
    let args = [
        "edit",
        "replace-text",
        sample.to_str().unwrap(),
        "--find",
        "회사",
        "--replace",
        "기관",
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
    assert_eq!(v["dryRun"], true, "{v}");
    assert_eq!(v["replacedCount"].as_u64(), Some(expected), "{v}");
    assert!(
        v.get("output").is_none(),
        "dry-run 은 output 을 보고하지 않습니다: {v}"
    );
    assert!(!out.exists(), "dry-run 은 파일을 만들면 안 됩니다");
}

/// 치환 0건이면 출력 파일을 만들지 않는다 — 무변경 산출물 금지.
#[test]
fn zero_matches_writes_no_output() {
    let sample = sample();
    let out = temp_out("zero");
    let args = [
        "edit",
        "replace-text",
        sample.to_str().unwrap(),
        "--find",
        "존재하지않는문자열XYZQ",
        "--replace",
        "무엇",
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
    let v = parse_json(&args, &output);
    assert_eq!(v["replacedCount"], 0, "{v}");
    assert!(v.get("output").is_none(), "0건이면 output 미보고: {v}");
    assert!(!out.exists(), "0건이면 파일을 만들면 안 됩니다");
}

/// 필수 인자 누락·빈 --find 는 사용법 오류(2)다.
#[test]
fn missing_or_empty_args_are_usage_errors() {
    let sample = sample();
    let s = sample.to_str().unwrap();
    for args in [
        vec!["edit", "replace-text", s, "--find", "가"], // --replace 누락
        vec!["edit", "replace-text", s, "--replace", "나"], // --find 누락
        vec!["edit", "replace-text", "--find", "가", "--replace", "나"], // 파일 누락
        vec!["edit", "replace-text", s, "--find", "", "--replace", "나"], // 빈 --find
    ] {
        let output = run(&args);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{}",
            describe(&args, &output)
        );
        assert!(
            output.stdout.is_empty(),
            "사용법 오류 시 stdout 은 0바이트여야 합니다.\n{}",
            describe(&args, &output)
        );
    }
}

/// 없는 파일은 런타임 실패(1) + 출력 파일 미생성.
#[test]
fn missing_input_is_runtime_error_without_output() {
    let out = temp_out("missing");
    let args = [
        "edit",
        "replace-text",
        "없는파일.hwp",
        "--find",
        "가",
        "--replace",
        "나",
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
    assert!(output.stdout.is_empty(), "{}", describe(&args, &output));
    assert!(!out.exists());
}
