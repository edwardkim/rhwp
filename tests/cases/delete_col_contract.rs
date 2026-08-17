//! [#5009] `edit delete-col` 계약.
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

fn first_top_table(path: &str) -> (usize, u16) {
    let bytes = std::fs::read(path).expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let g = extract_tables(doc.document())
        .into_iter()
        .find(|g| g.container_path.is_empty() && g.cols >= 2)
        .expect("열 2개 이상 최상위 표");
    (g.index, g.cols)
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-delcol-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn table_cols(path: &Path, index: usize) -> u16 {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    extract_tables(doc.document())
        .into_iter()
        .find(|g| g.index == index && g.container_path.is_empty())
        .expect("표")
        .cols
}

#[test]
fn delete_col_decreases_count() {
    let src = sample();
    let (idx, before) = first_top_table(&src);
    let out = temp("out");
    let idx_s = idx.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "delete-col",
            src.as_str(),
            "--table",
            &idx_s,
            "--col",
            "0",
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert_eq!(table_cols(&out, idx), before - 1);
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
            "delete-col",
            src.as_str(),
            "--table",
            &idx_s,
            "--col",
            "0",
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
        .any(|t| t["name"] == "hwp_delete_col"));
}
