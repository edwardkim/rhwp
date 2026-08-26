//! #4586 `convert` 출력 형식과 gym T12의 HWPX 계약 회귀 테스트.
//!
//! `convert`는 편집 가능한 HWP5를 만드는 명령이다. 출력 이름만 `.hwpx`로 주어
//! HWP5를 HWPX처럼 위장하면 gym의 형식·동등성 판정까지 거짓 양성이 되므로,
//! 디스크를 쓰기 전에 `.hwp` 출력 계약을 강제한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

const SAMPLE: &str = "samples/field-01.hwp";

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn unique_output(tag: &str, extension: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "rhwp-issue-4586-{}-{nanos}-{tag}.{extension}",
        std::process::id()
    ))
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn describe(args: &[&str], output: &Output) -> String {
    format!(
        "명령: rhwp {}\nstatus: {:?}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn convert_rejects_hwpx_output_without_creating_hwp5_bytes() {
    let out = unique_output("misnamed", "hwpx");
    let out_text = out.to_str().expect("utf-8 path").to_string();
    let args = ["convert", SAMPLE, &out_text, "--verify", "--json"];
    let output = run(&args);
    let created = out.exists();
    let _ = std::fs::remove_file(&out);

    assert_eq!(
        output.status.code(),
        Some(2),
        "HWP5를 .hwpx 이름으로 저장하면 안 된다\n{}",
        describe(&args, &output)
    );
    assert!(
        output.stdout.is_empty(),
        "사용법 오류의 stdout은 비어야 한다\n{}",
        describe(&args, &output)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(".hwp") && stderr.contains("export-hwpx"),
        "허용 형식과 대체 명령을 함께 안내해야 한다\n{}",
        describe(&args, &output)
    );
    assert!(
        !created,
        "거부된 출력 파일이 남으면 안 된다: {}",
        out.display()
    );
}

#[test]
fn invalid_convert_output_is_usage_error_before_input_io() {
    let out = unique_output("preflight", "hwpx");
    let missing = unique_output("missing-input", "hwp");
    let missing_text = missing.to_str().expect("utf-8 path").to_string();
    let out_text = out.to_str().expect("utf-8 path").to_string();
    let args = ["convert", &missing_text, &out_text, "--json"];
    let output = run(&args);
    let _ = std::fs::remove_file(&out);

    assert_eq!(
        output.status.code(),
        Some(2),
        "출력 형식 오류는 입력 IO보다 먼저 판정해야 한다\n{}",
        describe(&args, &output)
    );
    assert!(output.stdout.is_empty(), "{}", describe(&args, &output));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("export-hwpx"),
        "{}",
        describe(&args, &output)
    );
    assert!(!out.exists(), "사용법 오류 뒤 산출물이 없어야 한다");
}

#[test]
fn convert_accepts_case_insensitive_hwp_extension() {
    let sample = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let out = unique_output("uppercase", "HWP");
    let sample_text = sample.to_str().expect("utf-8 path").to_string();
    let out_text = out.to_str().expect("utf-8 path").to_string();
    let args = ["convert", &sample_text, &out_text, "--json"];
    let output = run(&args);
    let created = out.exists();
    let _ = std::fs::remove_file(&out);

    assert_eq!(
        output.status.code(),
        Some(0),
        "대문자 .HWP도 유효한 HWP5 출력이다\n{}",
        describe(&args, &output)
    );
    assert!(created, "성공한 convert 산출물이 있어야 한다");
}
