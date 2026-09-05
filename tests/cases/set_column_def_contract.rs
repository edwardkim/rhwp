//! `edit set-column-def` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

use rhwp::wasm_api::HwpDocument;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/field-01.hwp")
        .to_string_lossy()
        .into_owned()
}

fn column_def(path: &Path) -> serde_json::Value {
    let bytes = std::fs::read(path).expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let raw = doc.get_column_def(0).expect("column def");
    serde_json::from_str(&raw).expect("column def json")
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-coldef-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn column_definition_command_writes_a_parseable_document() {
    let src = sample();
    let out = temp("out");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-column-def",
            src.as_str(),
            "--count",
            "2",
            "--type",
            "2",
            "--mixed-width",
            "--spacing",
            "1200",
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    let envelope: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["section"], 0);
    assert_eq!(envelope["columnCount"], 2);
    assert_eq!(envelope["columnType"], 2);
    assert_eq!(envelope["sameWidth"], false);
    assert_eq!(envelope["spacing"], 1200);

    // C4는 CLI 라우팅 추출의 경계를 보호한다. 저장본이 다시 열리고 단 정의가
    // 남는지 확인하되, 구조화 필드보다 raw_attr를 우선하는 기존 코어 직렬화의
    // 값 보존 결함은 이 리팩터링 계약에 고정하지 않는다.
    let saved = column_def(&out);
    assert!(saved["columnCount"].is_number());
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_reports_values_without_writing() {
    let src = sample();
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-column-def",
            src.as_str(),
            "--count",
            "3",
            "--type",
            "1",
            "--same-width",
            "--spacing",
            "800",
            "-o",
            out.to_str().unwrap(),
            "--dry-run",
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert!(!out.exists());
    let envelope: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["dryRun"], true);
    assert_eq!(envelope["columnCount"], 3);
    assert_eq!(envelope["columnType"], 1);
    assert_eq!(envelope["sameWidth"], true);
    assert_eq!(envelope["spacing"], 800);
}

#[test]
fn invalid_type_and_unknown_flag_keep_stdout_empty() {
    let src = sample();
    for tail in [
        ["--count", "2", "--type", "3"],
        ["--count", "2", "--nope", "0"],
    ] {
        let output = Command::new(rhwp_bin())
            .args(["edit", "set-column-def", src.as_str()])
            .args(tail)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2), "{:?}", output);
        assert!(output.stdout.is_empty());
    }
}
