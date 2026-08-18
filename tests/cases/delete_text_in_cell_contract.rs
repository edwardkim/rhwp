//! `edit delete-text-in-cell` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

use rhwp::document_core::queries::table_extract::extract_tables;
use rhwp::model::control::Control;
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

fn first_cell_with_text(path: &str) -> (usize, u16, u16, usize, String) {
    let bytes = std::fs::read(path).expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    for g in extract_tables(doc.document()).into_iter() {
        if !g.container_path.is_empty() {
            continue;
        }
        let Some(Control::Table(table)) = doc.document().sections[g.section].paragraphs
            [g.paragraph]
            .controls
            .get(g.control)
        else {
            continue;
        };
        for c in &table.cells {
            for (pi, p) in c.paragraphs.iter().enumerate() {
                if p.text.chars().count() >= 1 {
                    return (g.index, c.row, c.col, pi, p.text.clone());
                }
            }
        }
    }
    panic!("글자가 있는 본문 최상위 표 셀이 없다");
}

fn cell_para_text(path: &Path, index: usize, row: u16, col: u16, cell_para: usize) -> String {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    let g = extract_tables(doc.document())
        .into_iter()
        .find(|g| g.index == index && g.container_path.is_empty())
        .expect("표");
    let Some(Control::Table(table)) = doc.document().sections[g.section].paragraphs[g.paragraph]
        .controls
        .get(g.control)
    else {
        panic!("표 컨트롤");
    };
    table
        .cells
        .iter()
        .find(|c| c.row == row && c.col == col)
        .expect("셀")
        .paragraphs[cell_para]
        .text
        .clone()
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-celldel-{tag}-{}-{}.hwpx",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn delete_text_in_cell_shortens() {
    let src = sample();
    let (idx, row, col, cell_para, before) = first_cell_with_text(&src);
    let out = temp("out");
    let idx_s = idx.to_string();
    let row_s = row.to_string();
    let col_s = col.to_string();
    let para_s = cell_para.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "delete-text-in-cell",
            src.as_str(),
            "--table",
            &idx_s,
            "--row",
            &row_s,
            "--col",
            &col_s,
            "--cell-para",
            &para_s,
            "--count",
            "1",
            "--offset",
            "0",
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    let after = cell_para_text(&out, idx, row, col, cell_para);
    assert_ne!(after, before, "before={before:?} after={after:?}");
    assert_eq!(after.chars().count(), before.chars().count() - 1);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = sample();
    let (idx, row, col, _, _) = first_cell_with_text(&src);
    let out = temp("dry");
    let idx_s = idx.to_string();
    let row_s = row.to_string();
    let col_s = col.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "delete-text-in-cell",
            src.as_str(),
            "--table",
            &idx_s,
            "--row",
            &row_s,
            "--col",
            &col_s,
            "--count",
            "1",
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
            "delete-text-in-cell",
            src.as_str(),
            "--table",
            "0",
            "--row",
            "0",
            "--col",
            "0",
            "--count",
            "1",
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
        .any(|t| t["name"] == "hwp_delete_text_in_cell"));
}
