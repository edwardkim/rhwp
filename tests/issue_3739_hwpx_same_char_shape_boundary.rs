//! [#3739] HWP → HWPX export가 동일 글자모양 ID의 run 경계도 보존하는지 검증한다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SAMPLE: &str = "samples/lseg-04-indent.hwp";

fn sample_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn unique_output_dir() -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-issue3739-{}-{}",
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

#[test]
fn export_hwpx_verify_preserves_same_char_shape_id_boundary() {
    let source = sample_path();
    assert!(
        source.exists(),
        "회귀 입력 샘플이 없습니다: {}",
        source.display()
    );

    let output_dir = unique_output_dir();
    std::fs::create_dir(&output_dir).expect("임시 출력 디렉터리 생성");
    let output = output_dir.join("lseg-04-indent.hwpx");
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
