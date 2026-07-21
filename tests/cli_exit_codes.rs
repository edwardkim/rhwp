//! [#2707] CLI 종료 코드 계약 회귀 테스트.
//!
//! 계약: 0 성공 / 1 런타임 실패(읽기·파싱·렌더·쓰기) / 2 사용법 오류
//! (인자 없음, 알 수 없는 옵션, 알 수 없는 명령, 페이지 범위 초과).
//! 3(`--verify` IR 차이)·4(`--verify-pages` 페이지 수 불일치)는 기존 문서화 계약이라
//! 본 테스트가 다루지 않는다 — `tests/issue_1638_convert_verify_gate.rs` 참조.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 파싱까지 성공하는 실제 샘플 (페이지 범위 초과·쓰기 실패 경로 검증용).
const SAMPLE: &str = "samples/hwp3-sample.hwp";

/// 인자 없이 호출했을 때 사용법 오류(2)가 나와야 하는 명령들.
const COMMANDS_WITHOUT_ARGS: &[&str] = &[
    "export-svg",
    "export-render-tree",
    "export-structure",
    "export-text",
    "export-markdown",
    "convert",
    "export-hwpx",
];

fn sample_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn unique_temp_path(label: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "rhwp-exit-codes-{label}-{}-{nonce}",
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

// --- 2: 사용법 오류 -------------------------------------------------------

#[test]
fn missing_arguments_report_usage_error() {
    for command in COMMANDS_WITHOUT_ARGS {
        assert_code(&[command], 2);
    }
}

#[test]
fn unknown_command_writes_usage_to_stderr_and_fails() {
    let args = ["expport-svg", "foo.hwp"];
    let output = assert_code(&args, 2);

    assert!(
        String::from_utf8_lossy(&output.stderr).contains("알 수 없는 명령"),
        "알 수 없는 명령을 stderr 로 알려야 한다\n{}",
        describe(&args, &output)
    );
    assert!(
        output.stdout.is_empty(),
        "사용법 안내가 stdout 을 오염시키면 안 된다\n{}",
        describe(&args, &output)
    );
}

#[test]
fn missing_command_reports_usage_error() {
    let args: [&str; 0] = [];
    let output = assert_code(&args, 2);

    assert!(
        String::from_utf8_lossy(&output.stderr).contains("명령을 지정해주세요"),
        "명령 누락을 stderr 로 알려야 한다\n{}",
        describe(&args, &output)
    );
    assert!(output.stdout.is_empty());
}

#[test]
fn unknown_option_is_fatal_instead_of_silently_ignored() {
    // `--font-path` 오타. 경고만 찍고 렌더를 계속하면 잘못된 산출물이 성공으로 보고된다.
    let sample = sample_path();
    let sample = sample.to_str().expect("utf-8 경로");
    let output_dir = unique_temp_path("unknown-option");
    let output_dir = output_dir.to_str().expect("utf-8 경로").to_string();

    let args = [
        "export-svg",
        sample,
        "--fontpath",
        "./ttfs",
        "-o",
        &output_dir,
    ];
    let output = assert_code(&args, 2);

    assert!(
        String::from_utf8_lossy(&output.stderr).contains("알 수 없는 옵션: --fontpath"),
        "어떤 옵션이 문제인지 알려야 한다\n{}",
        describe(&args, &output)
    );
    assert!(
        !Path::new(&output_dir).exists(),
        "옵션 파싱 실패 뒤에는 산출물을 만들면 안 된다"
    );
}

#[test]
fn page_out_of_range_reports_usage_error() {
    let sample = sample_path();
    let sample = sample.to_str().expect("utf-8 경로");
    let output_dir = unique_temp_path("page-range");
    let output_dir = output_dir.to_str().expect("utf-8 경로").to_string();

    assert_code(&["export-text", sample, "-p", "9999", "-o", &output_dir], 2);

    let _ = std::fs::remove_dir_all(&output_dir);
}

// --- 1: 런타임 실패 -------------------------------------------------------

#[test]
fn unreadable_input_reports_runtime_failure() {
    let missing = unique_temp_path("missing.hwp");
    let missing = missing.to_str().expect("utf-8 경로").to_string();
    let out_dir = unique_temp_path("runtime-out");
    let out_dir = out_dir.to_str().expect("utf-8 경로").to_string();
    let out_file = unique_temp_path("runtime-out.hwpx");
    let out_file = out_file.to_str().expect("utf-8 경로").to_string();

    for args in [
        vec!["export-svg", &missing, "-o", &out_dir],
        vec!["export-render-tree", &missing, "-o", &out_dir],
        vec!["export-text", &missing, "-o", &out_dir],
        vec!["export-markdown", &missing, "-o", &out_dir],
        vec!["export-structure", &missing],
        vec!["convert", &missing, &out_file],
        vec!["export-hwpx", &missing, &out_file],
    ] {
        assert_code(&args, 1);
    }
}

#[test]
fn page_write_failure_is_counted_and_reported() {
    // 출력 폴더 자리에 일반 파일을 두면 모든 페이지 저장이 실패한다.
    // 이때 성공 메시지는 0개를 보고하고 종료 코드는 1이어야 한다.
    let blocker = unique_temp_path("blocker-not-a-dir");
    std::fs::write(&blocker, b"not a directory").expect("차단용 파일 생성");
    let blocker_arg = blocker.to_str().expect("utf-8 경로").to_string();
    let sample = sample_path();
    let sample = sample.to_str().expect("utf-8 경로");

    let args = ["export-text", sample, "-o", &blocker_arg];
    let output = assert_code(&args, 1);

    assert!(
        String::from_utf8_lossy(&output.stdout).contains("텍스트 내보내기 완료: 0개 TXT 파일"),
        "실제로 쓴 페이지 수(0)를 보고해야 한다\n{}",
        describe(&args, &output)
    );

    let _ = std::fs::remove_file(&blocker);
}

// --- 0: 성공 경로 회귀 방지 ----------------------------------------------

#[test]
fn help_and_version_still_succeed() {
    for args in [["--help"], ["--version"], ["-h"], ["-V"]] {
        assert_code(&args, 0);
    }
}

#[test]
fn successful_export_returns_zero() {
    let sample = sample_path();
    let sample = sample.to_str().expect("utf-8 경로");
    let output_dir = unique_temp_path("success");
    let output_dir = output_dir.to_str().expect("utf-8 경로").to_string();

    assert_code(&["export-text", sample, "-p", "0", "-o", &output_dir], 0);

    let _ = std::fs::remove_dir_all(&output_dir);
}

// --- export-png (feature 게이트) -----------------------------------------

#[cfg(feature = "native-skia")]
#[test]
fn export_png_follows_the_same_contract() {
    let missing = unique_temp_path("missing-png.hwp");
    let missing = missing.to_str().expect("utf-8 경로").to_string();
    let out_dir = unique_temp_path("png-out");
    let out_dir = out_dir.to_str().expect("utf-8 경로").to_string();

    assert_code(&["export-png"], 2);
    assert_code(&["export-png", &missing, "-o", &out_dir], 1);
}

#[cfg(not(feature = "native-skia"))]
#[test]
fn export_png_without_native_skia_reports_usage_error() {
    // feature 가 빠진 바이너리에서 기능이 아예 없는데 0으로 끝나면 스크립트가 성공으로 읽는다.
    let args = ["export-png", "foo.hwp"];
    let output = assert_code(&args, 2);

    assert!(
        String::from_utf8_lossy(&output.stderr).contains("native-skia"),
        "왜 못 쓰는지 알려야 한다\n{}",
        describe(&args, &output)
    );
}
