//! `edit split-paragraph-in-cell` 계약.
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
                if p.text.chars().count() >= 2 {
                    return (g.index, c.row, c.col, pi, p.text.clone());
                }
            }
        }
    }
    panic!("글자 2개 이상인 본문 최상위 표 셀이 없다");
}

fn cell_paras(path: &Path, index: usize, row: u16, col: u16) -> Vec<String> {
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
        .paragraphs
        .iter()
        .map(|p| p.text.clone())
        .collect()
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-cellsplit-{tag}-{}-{}.hwpx",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn split_existing_cell_paragraph() {
    let src = sample();
    let (idx, row, col, cell_para, before) = first_cell_with_text(&src);
    let before_paras = cell_paras(Path::new(&src), idx, row, col);
    let out = temp("out");
    let idx_s = idx.to_string();
    let row_s = row.to_string();
    let col_s = col.to_string();
    let para_s = cell_para.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "split-paragraph-in-cell",
            src.as_str(),
            "--table",
            &idx_s,
            "--row",
            &row_s,
            "--col",
            &col_s,
            "--cell-para",
            &para_s,
            "--offset",
            "1",
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["table"], idx);
    assert_eq!(v["row"], row);
    assert_eq!(v["col"], col);
    assert_eq!(v["offset"], 1);
    let after = cell_paras(&out, idx, row, col);
    assert_eq!(after.len(), before_paras.len() + 1);
    let left: String = before.chars().take(1).collect();
    let right: String = before.chars().skip(1).collect();
    assert_eq!(after[cell_para], left);
    assert_eq!(after[cell_para + 1], right);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_json_has_fields_and_no_file() {
    let src = sample();
    let (idx, row, col, cell_para, _) = first_cell_with_text(&src);
    let out = temp("dry");
    let idx_s = idx.to_string();
    let row_s = row.to_string();
    let col_s = col.to_string();
    let para_s = cell_para.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "split-paragraph-in-cell",
            src.as_str(),
            "--table",
            &idx_s,
            "--row",
            &row_s,
            "--col",
            &col_s,
            "--cell-para",
            &para_s,
            "--offset",
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
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["dryRun"], true);
    assert_eq!(v["offset"], 1);
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "split-paragraph-in-cell",
            src.as_str(),
            "--table",
            "0",
            "--row",
            "0",
            "--col",
            "0",
            "--nope",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
}
