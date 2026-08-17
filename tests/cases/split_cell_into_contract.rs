//! `edit split-cell-into` 계약.
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

/// 1×1 셀을 찾아 n×m 분할 대상으로 쓴다. 표 2 (0,1) 은 rowspan=2 라 쓰지 않는다.
fn first_1x1(path: &str) -> (usize, u16, u16, usize) {
    let bytes = std::fs::read(path).expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    for g in extract_tables(doc.document())
        .into_iter()
        .filter(|g| g.container_path.is_empty())
    {
        if let Some(c) = g.cells.iter().find(|c| c.row_span == 1 && c.col_span == 1) {
            return (g.index, c.row, c.col, g.cell_count);
        }
    }
    panic!("나눌 수 있는 1×1 셀이 없다");
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-splitinto-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn table_cells(path: &Path, index: usize) -> usize {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    extract_tables(doc.document())
        .into_iter()
        .find(|g| g.index == index && g.container_path.is_empty())
        .expect("표")
        .cell_count
}

#[test]
fn split_into_increases_cells() {
    let src = sample();
    let (idx, row, col, before) = first_1x1(&src);
    let out = temp("out");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "split-cell-into",
            src.as_str(),
            "--table",
            &idx.to_string(),
            "--row",
            &row.to_string(),
            "--col",
            &col.to_string(),
            "--rows",
            "1",
            "--cols",
            "2",
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert!(
        table_cells(&out, idx) > before,
        "1×2 분할 후 셀 수가 늘어야 한다"
    );
    HwpDocument::from_bytes(&std::fs::read(&out).unwrap()).expect("산출물 재파싱");
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = sample();
    let (idx, row, col, _) = first_1x1(&src);
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "split-cell-into",
            src.as_str(),
            "--table",
            &idx.to_string(),
            "--row",
            &row.to_string(),
            "--col",
            &col.to_string(),
            "--rows",
            "2",
            "--cols",
            "2",
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
    let (idx, row, col, _) = first_1x1(&src);
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "split-cell-into",
            src.as_str(),
            "--table",
            &idx.to_string(),
            "--row",
            &row.to_string(),
            "--col",
            &col.to_string(),
            "--rows",
            "1",
            "--cols",
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
        .any(|t| t["name"] == "hwp_split_cell_into"));
}
