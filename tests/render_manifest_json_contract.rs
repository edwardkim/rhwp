//! [#3286] `export-svg --json` 산출물 매니페스트 계약 회귀 테스트.
//!
//! 렌더는 **파일을 만드는** 명령이라, 에이전트가 다음 단계(VLM 확인 등)로 넘어가려면
//! "어떤 파일이 어느 페이지로 생겼는가"를 알아야 한다. 사람용 출력을 파싱하게 두면
//! 계약이 없다. 종료 코드는 #2707 계약을 따른다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SAMPLE: &str = "samples/hwp3-sample.hwp";

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn temp_dir(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-render-manifest-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
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
        "명령: rhwp {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn export_svg_json_manifest_contract() {
    let p = sample();
    let out = temp_dir("contract");
    let args = [
        "export-svg",
        p.to_str().unwrap(),
        "-p",
        "3",
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

    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout 이 순수 JSON 이 아닙니다 ({e}).\n{}",
            describe(&args, &output)
        )
    });
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert_eq!(v["format"], "svg", "{v}");
    assert!(v["source"].is_string(), "{v}");
    assert!(v["outputDir"].is_string(), "{v}");
    assert!(v["pageCount"].as_u64().unwrap() >= 1, "{v}");

    let pages = v["pages"].as_array().expect("pages 배열");
    assert_eq!(
        pages.len() as u64,
        v["renderedCount"].as_u64().unwrap(),
        "renderedCount 는 pages 길이와 같다: {v}"
    );
    assert_eq!(pages.len(), 1, "-p 3 이므로 한 장이어야 합니다: {v}");

    let entry = &pages[0];
    assert_eq!(entry["page"], 3, "요청한 페이지 번호가 실려야 합니다: {v}");
    assert!(entry["bytes"].as_u64().unwrap() > 0, "{entry}");

    // 매니페스트의 경로는 실제로 존재해야 한다 — 에이전트가 바로 읽을 수 있어야 하므로.
    let path = entry["path"].as_str().expect("path 문자열");
    assert!(
        Path::new(path).exists(),
        "매니페스트 경로가 실재하지 않습니다: {path}\n{v}"
    );

    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn export_svg_default_output_unchanged() {
    // 기존 소비자 보호: --json 없이는 종전 사람용 출력 그대로.
    let p = sample();
    let out = temp_dir("guard");
    let args = [
        "export-svg",
        p.to_str().unwrap(),
        "-p",
        "3",
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("문서 로드 완료") && stdout.contains("내보내기 완료"),
        "기본 출력이 바뀌면 안 됩니다.\n{}",
        describe(&args, &output)
    );
    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn export_svg_json_missing_file_exit_runtime_silent_stdout() {
    let out = temp_dir("missing");
    let args = [
        "export-svg",
        "없는파일-render.hwp",
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
    assert!(
        output.stdout.is_empty(),
        "실패 경로 stdout 은 비어야 합니다.\n{}",
        describe(&args, &output)
    );
    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn export_svg_json_write_failure_exit_runtime_silent_stdout() {
    // 출력 폴더 자리에 일반 파일을 두면 어느 플랫폼에서도 쓰기 실패를 재현할 수 있다.
    let out = temp_dir("write-failure");
    std::fs::write(&out, b"not a directory").expect("출력 경로 파일 생성");
    let p = sample();
    let args = [
        "export-svg",
        p.to_str().unwrap(),
        "-p",
        "0",
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
    assert!(
        output.stdout.is_empty(),
        "부분 매니페스트를 stdout에 출력하면 안 됩니다.\n{}",
        describe(&args, &output)
    );
    let _ = std::fs::remove_file(&out);
}

/// [#3289] 아카이브 실행 시 컴파일타임 경로는 빌드 러너 전용이므로,
/// nextest가 런타임에 재매핑해 주입하는 CARGO_BIN_EXE_rhwp를 우선한다.
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}
