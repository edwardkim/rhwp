//! [#3359] export 계열 위치 인자 파싱 회귀 테스트 (#3349 2차 조각).
//!
//! 계약: export-svg/render-tree/pdf/markdown/doclang 은 옵션이 파일 앞에 와도
//! 동작한다 (export-structure/export-text 와 동일 규약). 파일 선행을 강제하던
//! 시절에는 `-p 0 파일` 에서 `-p` 가 파일로 잡혀 "알 수 없는 옵션: 0" 으로 죽었다.
//! export-png 은 같은 레시피를 적용했으나 native-skia feature 빌드에서만 실행
//! 가능하므로 여기서는 다루지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SAMPLE: &str = "samples/hwp3-sample.hwp";

fn sample_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn unique_temp_path(label: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    std::env::temp_dir().join(format!("rhwp-3359-{label}-{}-{nonce}", std::process::id()))
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

/// 버그 재현 형태 그대로 — 옵션 전부가 파일 앞에 와도 산출물이 나온다.
#[test]
fn export_svg_options_before_file_succeeds() {
    let sample = sample_path();
    let out = unique_temp_path("svg");
    let out_s = out.to_str().expect("utf-8 경로");
    let args = [
        "export-svg",
        "-p",
        "0",
        "-o",
        out_s,
        sample.to_str().unwrap(),
    ];
    assert_code(&args, 0);
    let produced = std::fs::read_dir(&out)
        .expect("출력 폴더")
        .filter_map(Result::ok)
        .any(|e| e.path().extension().is_some_and(|x| x == "svg"));
    assert!(produced, "SVG 산출물이 있어야 합니다");
    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn export_markdown_options_before_file_succeeds() {
    let sample = sample_path();
    let out = unique_temp_path("md");
    let out_s = out.to_str().expect("utf-8 경로");
    let args = [
        "export-markdown",
        "-p",
        "0",
        "-o",
        out_s,
        sample.to_str().unwrap(),
    ];
    assert_code(&args, 0);
    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn export_render_tree_options_before_file_succeeds() {
    let sample = sample_path();
    let out = unique_temp_path("rt");
    let out_s = out.to_str().expect("utf-8 경로");
    let args = [
        "export-render-tree",
        "-p",
        "0",
        "-o",
        out_s,
        sample.to_str().unwrap(),
    ];
    assert_code(&args, 0);
    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn export_pdf_options_before_file_succeeds() {
    let sample = sample_path();
    let out = unique_temp_path("pdf");
    std::fs::create_dir_all(&out).expect("임시 폴더");
    let out_file = out.join("out.pdf");
    let out_file_s = out_file.to_str().expect("utf-8 경로");
    let args = [
        "export-pdf",
        "-o",
        out_file_s,
        "-p",
        "0",
        sample.to_str().unwrap(),
    ];
    assert_code(&args, 0);
    assert!(out_file.exists(), "PDF 산출물이 있어야 합니다");
    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn export_doclang_options_before_file_succeeds() {
    // doclang 은 HWP5/HWPX 입력 전용이라 HWP3 공용 샘플 대신 HWP5 샘플을 쓴다.
    let sample = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/field-01.hwp");
    let out = unique_temp_path("dclg");
    std::fs::create_dir_all(&out).expect("임시 폴더");
    let out_file = out.join("out.dclg.xml");
    let out_file_s = out_file.to_str().expect("utf-8 경로");
    let args = ["export-doclang", "-o", out_file_s, sample.to_str().unwrap()];
    assert_code(&args, 0);
    assert!(out_file.exists(), "DocLang 산출물이 있어야 합니다");
    let _ = std::fs::remove_dir_all(&out);
}

/// 파일을 두 번 주면 조용히 덮어쓰지 않고 즉시 사용법 오류다 (대표: export-svg).
#[test]
fn duplicate_file_is_usage_error() {
    let sample = sample_path();
    let s = sample.to_str().unwrap();
    let args = ["export-svg", s, s];
    let output = assert_code(&args, 2);
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("입력 파일은 하나만"),
        "{}",
        describe(&args, &output)
    );
}

/// 파일 없이 옵션만 주면 사용법 오류다 (no-args 케이스는 cli_exit_codes 가 커버).
#[test]
fn options_without_file_is_usage_error() {
    for args in [
        vec!["export-svg", "-p", "0"],
        vec!["export-markdown", "-p", "0"],
        vec!["export-render-tree", "-p", "0"],
        vec!["export-pdf", "-p", "0"],
        vec!["export-doclang", "--assets-dir", "x"],
    ] {
        let output = run(&args);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{}",
            describe(&args, &output)
        );
    }
}

/// 알 수 없는 플래그는 여전히 즉시 사용법 오류로 잡는다 (기존 계약 유지).
#[test]
fn unknown_flag_is_still_usage_error() {
    let sample = sample_path();
    let args = [
        "export-svg",
        "--fontpath",
        "./ttfs",
        sample.to_str().unwrap(),
    ];
    let output = assert_code(&args, 2);
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("알 수 없는 옵션: --fontpath"),
        "{}",
        describe(&args, &output)
    );
}
