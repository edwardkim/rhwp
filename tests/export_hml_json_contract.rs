//! [#3616] `export-hml --json` — HML 재직렬화의 기계 계약 (M5 커버리지 마감 조각).
//! 계약 모양은 산출물 축(#3596)과 동일: 동작 무변경, `--json` 에서만 stdout 순수 JSON,
//! 실패 경로 stdout 비움.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SAMPLE: &str = "samples/hml/formatting_table.hml";

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn temp_path(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-hmljson-{tag}-{}-{}.hml",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
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

#[test]
fn export_hml_json_envelope() {
    let p = sample();
    if !p.exists() {
        eprintln!("표본 없음 — 건너뜀");
        return;
    }
    let out = temp_path("env");
    let args = [
        "export-hml",
        p.to_str().unwrap(),
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
    let v: serde_json::Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|e| panic!("stdout 순수 JSON 아님 ({e})\n{}", describe(&args, &output)));
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert_eq!(v["format"], "hml", "{v}");
    let bytes = v["bytes"].as_u64().expect("bytes");
    let meta = std::fs::metadata(&out).expect("산출물 실존");
    assert_eq!(meta.len(), bytes, "보고 bytes ≠ 실제 크기");

    // 산출물이 HML 파서로 재파싱 가능해야 한다 (재직렬화 유효성 실측).
    let info = run(&["info", out.to_str().unwrap(), "--json"]);
    let iv: serde_json::Value = serde_json::from_slice(&info.stdout).expect("info --json");
    assert_eq!(iv["format"], "hml", "재파싱 실측: {iv}");

    let _ = std::fs::remove_file(&out);
}

#[test]
fn export_hml_json_runtime_failure_keeps_stdout_empty() {
    let out = temp_path("fail");
    let args = [
        "export-hml",
        "없는파일-hmljson.hml",
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
        "실패 경로 stdout 은 비어야 합니다\n{}",
        describe(&args, &output)
    );
}

#[test]
fn capabilities_reports_export_hml_json() {
    let output = run(&["capabilities"]);
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("caps");
    let entry = v["commands"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "export-hml")
        .expect("export-hml 등재");
    assert_eq!(entry["json"], true, "{entry}");

    let mcp = run(&["capabilities", "--mcp"]);
    let m: serde_json::Value = serde_json::from_slice(&mcp.stdout).expect("mcp");
    assert!(
        m["tools"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["name"] == "hwp_export_hml"),
        "hwp_export_hml 누락"
    );
}
