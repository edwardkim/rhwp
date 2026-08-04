//! [#3358] build-from-ingest 미지 필드 거부 회귀 테스트.
//!
//! 계약: ingest JSON 의 필드명 오타·구조 착오는 조용히 무시되지 않고(내용 침묵 유실 금지)
//! 종료 코드 1 + 무엇이 왜 틀렸는지 알리는 오류로 즉시 실패한다. 출력 파일은 만들지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let dir =
        std::env::temp_dir().join(format!("rhwp-3358-{label}-{}-{nonce}", std::process::id()));
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

/// 관찰된 실제 사고 형태(boxed 에 text) — 종전에는 exit 0 + 빈 박스로 내용이 유실됐다.
#[test]
fn boxed_with_text_fails_fast_without_output() {
    let dir = unique_temp_dir("boxed-text");
    let ingest = dir.join("wrong.json");
    std::fs::write(
        &ingest,
        r#"{
            "version": "1",
            "questions": [{
                "number": 1,
                "stem": "보고 개요",
                "stem_blocks": [
                    {"type": "text", "text": "1. 보고 개요"},
                    {"type": "boxed", "text": "소속:      성명:      보고일:"}
                ],
                "choices": []
            }]
        }"#,
    )
    .expect("입력 작성");
    let out = dir.join("out.hwpx");
    let args = [
        "build-from-ingest",
        ingest.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        describe(&args, &output)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("boxed 블록에 허용되지 않는 필드 'text'"),
        "무엇이 틀렸는지 알려야 합니다.\n{}",
        describe(&args, &output)
    );
    assert!(
        stderr.contains("blocks"),
        "올바른 필드 힌트가 있어야 합니다.\n{}",
        describe(&args, &output)
    );
    assert!(!out.exists(), "실패 시 출력 파일을 만들면 안 됩니다");
    let _ = std::fs::remove_dir_all(&dir);
}

/// 최상위 필드명 오타도 즉시 잡는다.
#[test]
fn top_level_typo_fails_fast() {
    let dir = unique_temp_dir("typo");
    let ingest = dir.join("typo.json");
    std::fs::write(
        &ingest,
        r#"{"version":"1","defaul_font":"바탕","questions":[]}"#,
    )
    .expect("입력 작성");
    let out = dir.join("out.hwpx");
    let args = [
        "build-from-ingest",
        ingest.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        describe(&args, &output)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("defaul_font"),
        "{}",
        describe(&args, &output)
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// 공식 예제 2종은 종전과 동일하게 성공한다 (무회귀).
#[test]
fn official_samples_still_build() {
    let dir = unique_temp_dir("samples");
    for sample in [
        "tools/rhwp-ingest/schema/sample_minimal.json",
        "tools/rhwp-ingest/schema/sample_structured.json",
    ] {
        let input = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
        let out = dir.join(format!(
            "{}.hwpx",
            input.file_stem().unwrap().to_str().unwrap()
        ));
        let args = [
            "build-from-ingest",
            input.to_str().unwrap(),
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
        assert!(out.exists(), "산출물이 있어야 합니다: {sample}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}
