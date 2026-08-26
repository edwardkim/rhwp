//! `edit insert-text-in-cell` 계약.
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

fn first_top_table(path: &str) -> usize {
    let bytes = std::fs::read(path).expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    extract_tables(doc.document())
        .into_iter()
        .find(|g| g.container_path.is_empty())
        .expect("본문 최상위 표")
        .index
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-cellins-{tag}-{}-{}.hwpx",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn cell0(path: &Path, index: usize) -> String {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    extract_tables(doc.document())
        .into_iter()
        .find(|g| g.index == index && g.container_path.is_empty())
        .expect("표")
        .cells
        .into_iter()
        .find(|c| c.row == 0 && c.col == 0)
        .expect("셀 0,0")
        .text
}

#[test]
fn insert_text_in_cell_prefixes() {
    let src = sample();
    let idx = first_top_table(&src);
    let before = cell0(Path::new(&src), idx);
    let out = temp("out");
    let idx_s = idx.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "insert-text-in-cell",
            src.as_str(),
            "--table",
            &idx_s,
            "--row",
            "0",
            "--col",
            "0",
            "--text",
            "가",
            "--offset",
            "0",
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    let after = cell0(&out, idx);
    assert!(after.starts_with('가'), "before={before:?} after={after:?}");
    assert_ne!(after, before);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = sample();
    let idx = first_top_table(&src);
    let out = temp("dry");
    let idx_s = idx.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "insert-text-in-cell",
            src.as_str(),
            "--table",
            &idx_s,
            "--row",
            "0",
            "--col",
            "0",
            "--text",
            "가",
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
fn unknown_flag_empty_stdout() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "insert-text-in-cell",
            src.as_str(),
            "--table",
            "0",
            "--row",
            "0",
            "--col",
            "0",
            "--text",
            "가",
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
        .any(|t| t["name"] == "hwp_insert_text_in_cell"));
}
