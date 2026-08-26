//! [#3739] HWP → HWPX export가 동일 글자모양 ID의 run 경계도 보존하는지 검증한다.

#![cfg(not(target_arch = "wasm32"))]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static OUTPUT_DIR_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn sample_path(sample: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(sample)
}

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn unique_output_dir() -> PathBuf {
    let sequence = OUTPUT_DIR_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "rhwp-issue3739-{}-{}-{sequence}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ))
}

fn describe(args: &[String], output: &Output) -> String {
    format!(
        "명령: rhwp {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn assert_export_hwpx_verify_success(sample: &str) {
    let source = sample_path(sample);
    assert!(
        source.exists(),
        "회귀 입력 샘플이 없습니다: {}",
        source.display()
    );

    let output_dir = unique_output_dir();
    std::fs::create_dir(&output_dir).expect("임시 출력 디렉터리 생성");
    let output = output_dir.join("output.hwpx");
    let args = vec![
        "export-hwpx".to_string(),
        source.display().to_string(),
        output.display().to_string(),
        "--verify".to_string(),
        "--verify-pages".to_string(),
    ];

    let result = Command::new(rhwp_bin())
        .args(&args)
        .output()
        .expect("rhwp export-hwpx 실행");

    assert_eq!(
        result.status.code(),
        Some(0),
        "{}",
        describe(&args, &result)
    );
    assert!(
        output.is_file(),
        "HWPX 산출물이 없습니다: {}",
        output.display()
    );

    std::fs::remove_dir_all(&output_dir).expect("임시 출력 디렉터리 정리");
}

fn assert_password_protected_export_hwpx_success(sample: &str, verify_ir: bool) {
    let source = sample_path(sample);
    assert!(
        source.exists(),
        "회귀 입력 샘플이 없습니다: {}",
        source.display()
    );

    let output_dir = unique_output_dir();
    std::fs::create_dir(&output_dir).expect("임시 출력 디렉터리 생성");
    let output = output_dir.join("output.hwpx");
    let mut args = vec![
        "export-hwpx".to_string(),
        source.display().to_string(),
        output.display().to_string(),
        "--verify-pages".to_string(),
    ];
    if verify_ir {
        args.push("--verify".to_string());
    }
    args.push("--password-stdin".to_string());

    // Windows PowerShell/.NET의 pipe는 UTF-8 BOM을 앞에 붙일 수 있다. raw stdin
    // 경로도 그 실제 바이트열을 비밀번호로 오해하지 않아야 한다.
    let mut child = Command::new(rhwp_bin())
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("rhwp export-hwpx 실행");
    let mut stdin = child.stdin.take().expect("stdin pipe");
    stdin
        .write_all(b"\xEF\xBB\xBF123456\n")
        .expect("비밀번호 stdin 쓰기");
    drop(stdin);
    let result = child.wait_with_output().expect("rhwp 종료 대기");

    assert_eq!(
        result.status.code(),
        Some(0),
        "{}",
        describe(&args, &result)
    );
    assert!(
        output.is_file(),
        "HWPX 산출물이 없습니다: {}",
        output.display()
    );

    std::fs::remove_dir_all(&output_dir).expect("임시 출력 디렉터리 정리");
}

#[test]
fn export_hwpx_verify_preserves_same_char_shape_id_boundary() {
    assert_export_hwpx_verify_success("samples/lseg-04-indent.hwp");
}

#[test]
fn issue_3739_export_hwpx_verify_accepts_generated_field_command_parameters() {
    assert_export_hwpx_verify_success("samples/tac-img-02.hwp");
}

#[test]
fn issue_3739_export_hwpx_accepts_password_stdin_for_hwp3_hwp5_and_hwpx() {
    for sample in [
        "samples/HWP5-password-123456.hwpx",
        "samples/hwp3-sample16-hwp5-2024-password-123456.hwp",
    ] {
        assert_password_protected_export_hwpx_success(sample, false);
    }
}

#[test]
fn issue_3739_hwp3_password_export_hwpx_preserves_ir_and_pages() {
    // HWP3 object marker(U+FFFC)·하이퍼텍스트·빈 imgRect가 HWPX 표현으로 옮겨진 뒤에도
    // 암호 stdin(BOM 포함) 경로에서 --verify와 --verify-pages가 모두 통과해야 한다.
    assert_password_protected_export_hwpx_success("samples/HWP3-password-123456.hwp", true);
}
