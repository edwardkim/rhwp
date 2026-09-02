//! [#5028] `edit delete-table` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

use rhwp::document_core::queries::table_extract::extract_tables;
use rhwp::wasm_api::HwpDocument;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx")
        .to_string_lossy()
        .into_owned()
}

fn first_top_table(path: &str) -> (usize, usize) {
    let bytes = std::fs::read(path).expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let grids = extract_tables(doc.document());
    let n = grids.iter().filter(|g| g.container_path.is_empty()).count();
    let g = grids
        .into_iter()
        .find(|g| g.container_path.is_empty())
        .expect("본문 최상위 표");
    (g.index, n)
}

fn top_table_count(path: &Path) -> usize {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    extract_tables(doc.document())
        .into_iter()
        .filter(|g| g.container_path.is_empty())
        .count()
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-deltbl-{tag}-{}-{}.hwpx",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn delete_table_decreases_count() {
    let src = sample();
    let (idx, before) = first_top_table(&src);
    assert!(before >= 1, "최상위 표가 있어야 한다");
    let out = temp("out");
    let idx_s = idx.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "delete-table",
            src.as_str(),
            "--table",
            &idx_s,
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert_eq!(top_table_count(&out), before - 1);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = sample();
    let (idx, _) = first_top_table(&src);
    let out = temp("dry");
    let idx_s = idx.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "delete-table",
            src.as_str(),
            "--table",
            &idx_s,
            "-o",
            out.to_str().unwrap(),
            "--dry-run",
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert!(!out.exists());
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
        .any(|t| t["name"] == "hwp_delete_table"));
}
