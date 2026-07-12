//! #1638 변환 검증 게이트 CLI 회귀 테스트.
//!
//! `hwp2hwpx` 신규 명령을 만들지 않고 현재 CLI 표면(`export-hwpx`, `convert`)에
//! `--verify` / `--verify-pages`를 붙이는지 확인한다.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_output(name: &str, ext: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "rhwp-issue-1638-{}-{}-{}.{}",
        std::process::id(),
        nanos,
        name,
        ext
    ))
}

#[test]
fn export_hwpx_verify_and_verify_pages_pass() {
    let exe = env!("CARGO_BIN_EXE_rhwp");
    let out = temp_output("export", "hwpx");

    let output = Command::new(exe)
        .args([
            "export-hwpx",
            "samples/hwpx/blank_hwpx.hwpx",
            out.to_str().expect("utf-8 path"),
            "--verify",
            "--verify-pages",
        ])
        .output()
        .expect("rhwp export-hwpx 실행 실패");

    assert!(
        output.status.success(),
        "export-hwpx 검증 게이트 실패\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(out.exists(), "검증 통과 후 HWPX 산출물이 존재해야 한다");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("검증 통과(--verify-pages)"));
    assert!(stdout.contains("검증 통과(--verify): IR 차이 없음"));

    let _ = fs::remove_file(out);
}

#[test]
fn convert_verify_and_verify_pages_pass_for_hwp_source() {
    let exe = env!("CARGO_BIN_EXE_rhwp");
    let out = temp_output("convert", "hwp");

    let output = Command::new(exe)
        .args([
            "convert",
            "samples/hwp3-sample.hwp",
            out.to_str().expect("utf-8 path"),
            "--verify",
            "--verify-pages",
        ])
        .output()
        .expect("rhwp convert 실행 실패");

    assert!(
        output.status.success(),
        "convert 검증 게이트 실패\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(out.exists(), "검증 통과 후 HWP 산출물이 존재해야 한다");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("검증 통과(--verify-pages)"));
    assert!(stdout.contains("검증 통과(--verify): IR 차이 없음"));

    let _ = fs::remove_file(out);
}
