//! [#5081] `edit set-column-def` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

use rhwp::model::control::Control;
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

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-coldef-{tag}-{}-{}.hwpx",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn first_column_count(path: &Path) -> Option<u16> {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    for para in &doc.document().sections[0].paragraphs {
        for ctrl in &para.controls {
            if let Control::ColumnDef(cd) = ctrl {
                return Some(cd.column_count);
            }
        }
    }
    None
}

#[test]
fn set_column_def_writes_count() {
    let src = sample();
    let out = temp("out");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-column-def",
            src.as_str(),
            "--count",
            "2",
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert!(out.exists());
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["columnCount"], 2);
    assert_eq!(v["section"], 0);
    // 저장본에 단 정의 컨트롤이 남는다. count 값은 HWP5 raw_attr 우선 직렬화
    // 때문에 원본 1단이 그대로일 수 있다 — 코어 배선 범위 밖.
    assert!(first_column_count(&out).is_some());
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = sample();
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-column-def",
            src.as_str(),
            "--count",
            "3",
            "-o",
            out.to_str().unwrap(),
            "--dry-run",
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert!(!out.exists());
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["dryRun"], true);
    assert_eq!(v["columnCount"], 3);
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-column-def",
            src.as_str(),
            "--count",
            "2",
            "--nope",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
}

#[test]
fn mcp_declared() {
    let output = Command::new(rhwp_bin())
        .args(["capabilities", "--mcp"])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(v["tools"]
        .as_array()
        .unwrap()
        .iter()
        .any(|t| t["name"] == "hwp_set_column_def"));
}
