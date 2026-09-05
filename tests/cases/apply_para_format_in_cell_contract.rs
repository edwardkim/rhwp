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

struct CellAddr {
    table: usize,
    row: u16,
    col: u16,
    cell_para: usize,
}

fn first_cell(path: &str) -> CellAddr {
    let bytes = std::fs::read(path).expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    for g in extract_tables(doc.document()) {
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
                    return CellAddr {
                        table: g.index,
                        row: c.row,
                        col: c.col,
                        cell_para: pi,
                    };
                }
            }
        }
    }
    panic!("본문 최상위 표 셀이 없다");
}

fn cell_alignment(path: &Path, index: usize, row: u16, col: u16, cell_para: usize) -> Alignment {
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
    let psid = table
        .cells
        .iter()
        .find(|c| c.row == row && c.col == col)
        .expect("셀")
        .paragraphs[cell_para]
        .para_shape_id;
    doc.document().doc_info.para_shapes[psid as usize].alignment
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-cellpfmt-{tag}-{}-{}.hwpx",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn apply_center_is_visible() {
    let src = sample();
    let addr = first_cell(&src);
    let current = cell_alignment(
        Path::new(&src),
        addr.table,
        addr.row,
        addr.col,
        addr.cell_para,
    );
    let (target, props) = if current != Alignment::Center {
        (Alignment::Center, r#"{"alignment":"center"}"#)
    } else {
        (Alignment::Right, r#"{"alignment":"right"}"#)
    };
    let out = temp("out");
    let table_s = addr.table.to_string();
    let row_s = addr.row.to_string();
    let col_s = addr.col.to_string();
    let para_s = addr.cell_para.to_string();
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
            props,
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["table"], addr.table);
    assert_eq!(v["row"], addr.row);
    assert_eq!(v["col"], addr.col);
    assert_eq!(
        cell_alignment(&out, addr.table, addr.row, addr.col, addr.cell_para),
        target,
        "셀 문단 정렬이 저장본에 없다"
    );
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_json_has_fields_and_no_file() {
    let src = sample();
    let addr = first_cell(&src);
    let out = temp("dry");
    let table_s = addr.table.to_string();
    let row_s = addr.row.to_string();
    let col_s = addr.col.to_string();
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
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["dryRun"], true);
    assert_eq!(v["table"], addr.table);
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
            r#"{"alignment":"center"}"#,
            "--nope",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
}
