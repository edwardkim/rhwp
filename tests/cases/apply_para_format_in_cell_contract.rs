//! `edit apply-para-format-in-cell` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

use rhwp::document_core::queries::table_extract::extract_tables;
use rhwp::model::control::Control;
use rhwp::model::style::Alignment;
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

fn first_plain_text_cell(path: &str) -> (usize, u16, u16, usize) {
    let bytes = std::fs::read(path).expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    for g in extract_tables(doc.document()) {
        if !g.container_path.is_empty() {
            continue;
        }
        let Some(Control::Table(tbl)) = doc.document().sections[g.section].paragraphs[g.paragraph]
            .controls
            .get(g.control)
        else {
            continue;
        };
        for c in &tbl.cells {
            if c.row_span != 1 || c.col_span != 1 {
                continue;
            }
            for (pi, p) in c.paragraphs.iter().enumerate() {
                if !p.text.trim().is_empty() {
                    return (g.index, c.row, c.col, pi);
                }
            }
        }
    }
    panic!("1×1 텍스트 셀이 없다");
}

fn cell_alignment(path: &Path, table: usize, row: u16, col: u16, cell_para: usize) -> Alignment {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    let grid = extract_tables(doc.document())
        .into_iter()
        .find(|g| g.index == table && g.container_path.is_empty())
        .expect("표");
    let Control::Table(tbl) =
        &doc.document().sections[grid.section].paragraphs[grid.paragraph].controls[grid.control]
    else {
        panic!("표 컨트롤 아님");
    };
    let cell = tbl
        .cells
        .iter()
        .find(|c| c.row == row && c.col == col)
        .expect("셀");
    let id = cell.paragraphs[cell_para].para_shape_id;
    doc.document().doc_info.para_shapes[id as usize].alignment
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-pafmtcell-{tag}-{}-{}.hwpx",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn apply_center_and_reparse() {
    let src = sample();
    let (table, row, col, cell_para) = first_plain_text_cell(&src);
    let before = cell_alignment(Path::new(&src), table, row, col, cell_para);
    assert_ne!(
        before,
        Alignment::Center,
        "샘플이 이미 가운데라 판별이 안 된다"
    );
    let out = temp("out");
    let table_s = table.to_string();
    let row_s = row.to_string();
    let col_s = col.to_string();
    let para_s = cell_para.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-para-format-in-cell",
            src.as_str(),
            "--table",
            &table_s,
            "--row",
            &row_s,
            "--col",
            &col_s,
            "--cell-para",
            &para_s,
            "--props",
            r#"{"alignment":"center"}"#,
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["table"], table);
    assert_eq!(v["row"], row);
    assert_eq!(v["col"], col);
    assert!(out.exists());
    assert!(HwpDocument::from_bytes(&std::fs::read(&out).unwrap()).is_ok());
    assert_eq!(
        cell_alignment(&out, table, row, col, cell_para),
        Alignment::Center
    );
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = sample();
    let (table, row, col, _cell_para) = first_plain_text_cell(&src);
    let out = temp("dry");
    let table_s = table.to_string();
    let row_s = row.to_string();
    let col_s = col.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-para-format-in-cell",
            src.as_str(),
            "--table",
            &table_s,
            "--row",
            &row_s,
            "--col",
            &col_s,
            "--props",
            r#"{"alignment":"center"}"#,
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
            "apply-para-format-in-cell",
            src.as_str(),
            "--table",
            "0",
            "--row",
            "0",
            "--col",
            "0",
            "--props",
            "{}",
            "--nope",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(out.stdout.len(), 0);
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
        .any(|t| t["name"] == "hwp_apply_para_format_in_cell"));
}
