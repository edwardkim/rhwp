//! #8160: sample16 전용 회귀. 이전 등록: tests/explain_contract.rs
//! [#3828] `explain` 출력 계약 회귀 테스트.
//!
//! 계약: `explain --json` 의 stdout 은 순수 JSON 한 덩어리이고 `schemaVersion` 을
//! 포함한다. 값 자체는 새 판정이 아니라 `info`·`export-structure`·`export-tables`·
//! `fields` 가 이미 계산한 값의 조합이므로, 이 시험은 그 조합이 실제 문서에서
//! 정확한지(표 크기·병합 여부·누름틀 이름·각주/미주 개수·암호 여부)와 종료 코드
//! 계약(#2707)을 검증한다. 암호 문서는 다른 명령과 같은 `load_document` 규약을
//! 타므로 explain 도 자동으로 같은 종료 코드를 낸다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 누름틀 11개, 표 없음.
/// 표 1개(19×9, 병합 셀 있음), 누름틀 없음.
/// 표·누름틀 모두 없는 일반 문서(HWP5).
/// 각주 9개.
/// 미주 6개.
/// 비밀번호 "123456" 로 걸린 HWP5 문서.
const SAMPLE_ENCRYPTED: &str = "samples/hwp3-sample16-hwp5-2024-password-123456.hwp";

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

/// [#3289] 아카이브 실행 시 컴파일타임 경로는 빌드 러너 전용이므로,
/// nextest가 런타임에 재매핑해 주입하는 CARGO_BIN_EXE_rhwp를 우선한다.
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

// ── 암호 문서 — 다른 명령과 같은 `load_document` 규약을 그대로 물려받는다 ──────

#[test]
fn issue_8160_explain_encrypted_document_without_password_is_usage_error() {
    // [#3828] 규약: 다른 조회 명령(info 등)과 같은 종료 코드(EXIT_USAGE=2) —
    // "암호로 보호돼 있어 상세 분석 불가"는 이 계약을 통해 자동으로 지켜진다.
    let p = sample(SAMPLE_ENCRYPTED);
    let args = ["explain", p.to_str().unwrap(), "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
    assert!(
        output.stdout.is_empty(),
        "암호 문서 거부 경로 stdout 은 비어야 합니다.\n{}",
        describe(&args, &output)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("암호"), "{stderr}");
}

#[test]
fn issue_8160_explain_encrypted_document_wrong_password_exit_runtime() {
    let p = sample(SAMPLE_ENCRYPTED);
    let args = [
        "explain",
        p.to_str().unwrap(),
        "--password",
        "wrong-password",
        "--json",
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
fn issue_8160_explain_encrypted_document_with_password_reports_encrypted_true() {
    let p = sample(SAMPLE_ENCRYPTED);
    let args = [
        "explain",
        p.to_str().unwrap(),
        "--password",
        "123456",
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(v["encrypted"], true, "{v}");
    let summary = v["summary"].as_str().unwrap();
    assert!(summary.contains("암호로 보호돼 있다"), "{summary}");
}
