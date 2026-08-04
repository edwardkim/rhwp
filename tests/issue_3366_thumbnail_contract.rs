//! [#3366] `thumbnail` 종료 코드·파싱 계약 회귀 테스트.
//!
//! 계약: 알 수 없는 옵션·인자 없음·`-o` 값 누락·중복 positional 은 즉시 exit 2 (#2707),
//! 옵션은 파일 앞뒤 어디에 와도 동작한다 (#3349 규약). 종전에는 오타를 무시한 채
//! 산출물까지 만들고 exit 0 으로 끝났다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// PrvImage 를 실제로 가진 HWP5 샘플.
const SAMPLE: &str = "samples/2022년 국립국어원 업무계획.hwp";

fn sample_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let dir =
        std::env::temp_dir().join(format!("rhwp-3366-{label}-{}-{nonce}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("임시 폴더");
    dir
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

/// 종전 최악 사례 — 오타 옵션을 무시하고 산출물을 만들며 exit 0 이었다.
#[test]
fn unknown_option_is_usage_error_without_output() {
    let sample = sample_path();
    let dir = unique_temp_dir("unknown");
    let out = dir.join("t.png");
    let args = [
        "thumbnail",
        sample.to_str().unwrap(),
        "--no-such-option",
        "-o",
        out.to_str().unwrap(),
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("알 수 없는 옵션: --no-such-option"),
        "{}",
        describe(&args, &output)
    );
    assert!(
        !out.exists(),
        "사용법 오류 뒤에는 산출물을 만들면 안 됩니다"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// 인자 없음은 사용법 오류(2)다 — 종전 1.
#[test]
fn no_args_is_usage_error() {
    let args = ["thumbnail"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
}

/// `-o` 값 누락은 조용히 무시하지 않는다 — 종전 exit 0.
#[test]
fn output_without_value_is_usage_error() {
    let sample = sample_path();
    let args = ["thumbnail", sample.to_str().unwrap(), "-o"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
}

/// 옵션이 파일 앞에 와도 동작한다 (#3349 규약) — 종전에는 옵션이 파일 경로가 됐다.
#[test]
fn options_before_file_succeeds() {
    let sample = sample_path();
    let args = ["thumbnail", "--base64", sample.to_str().unwrap()];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    assert!(
        !output.stdout.is_empty(),
        "base64 출력이 있어야 합니다.\n{}",
        describe(&args, &output)
    );
}

/// 중복 positional 은 즉시 사용법 오류다.
#[test]
fn duplicate_file_is_usage_error() {
    let sample = sample_path();
    let s = sample.to_str().unwrap();
    let args = ["thumbnail", s, s];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("입력 파일은 하나만"),
        "{}",
        describe(&args, &output)
    );
}

/// 정상 추출(파일 출력)은 종전과 동일하게 동작한다.
#[test]
fn normal_extraction_still_works() {
    let sample = sample_path();
    let dir = unique_temp_dir("ok");
    let out = dir.join("thumb.png");
    let args = [
        "thumbnail",
        sample.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    assert!(out.exists(), "썸네일 파일이 있어야 합니다");
    let _ = std::fs::remove_dir_all(&dir);
}

/// 썸네일이 없는 입력은 종전대로 런타임 실패(1)다 — HWP3 는 PrvImage 가 없다.
#[test]
fn missing_preview_is_runtime_error() {
    let sample = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwp3-sample.hwp");
    let args = ["thumbnail", sample.to_str().unwrap()];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        describe(&args, &output)
    );
}
