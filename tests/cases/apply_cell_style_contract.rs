//! `edit apply-cell-style` 계약.
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

struct CellAddr {
    table: usize,
    row: u16,
    col: u16,
    cell_para: usize,
    current_style: usize,
    other_style: usize,
}

fn first_cell_and_other_style(path: &str) -> CellAddr {
    let bytes = std::fs::read(path).expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let styles = &doc.document().doc_info.styles;
    assert!(styles.len() >= 2, "스타일이 2개 미만이라 판별이 안 된다");
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
                if p.text.chars().count() < 2 {
                    continue;
                }
                let current = p.style_id as usize;
                let other = styles.iter().enumerate().find_map(|(id, st)| {
                    if id != current && st.style_type == 0 {
                        Some(id)
                    } else {
                        None
                    }
                });
                if let Some(other) = other {
                    return CellAddr {
                        table: g.index,
                        row: c.row,
                        col: c.col,
                        cell_para: pi,
                        current_style: current,
                        other_style: other,
                    };
                }
            }
        }
    }
    panic!("다른 문단 스타일을 고를 본문 최상위 표 셀이 없다");
}

fn cell_style_id(path: &Path, index: usize, row: u16, col: u16, cell_para: usize) -> usize {
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
        .style_id as usize
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-cellstyle-{tag}-{}-{}.hwpx",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn apply_cell_style_is_visible() {
    let src = sample();
    let addr = first_cell_and_other_style(&src);
    assert_ne!(addr.current_style, addr.other_style);
    let out = temp("out");
    let table_s = addr.table.to_string();
    let row_s = addr.row.to_string();
    let col_s = addr.col.to_string();
    let para_s = addr.cell_para.to_string();
    let style_s = addr.other_style.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-cell-style",
            src.as_str(),
            "--table",
            &table_s,
            "--row",
            &row_s,
            "--col",
            &col_s,
            "--cell-para",
            &para_s,
            "--style",
            &style_s,
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
    assert_eq!(v["paragraph"], addr.cell_para);
    assert_eq!(v["ctrl"], addr.other_style);
    assert_eq!(
        cell_style_id(&out, addr.table, addr.row, addr.col, addr.cell_para),
        addr.other_style,
        "셀 문단 스타일이 저장본에 없다"
    );
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_json_has_fields_and_no_file() {
    let src = sample();
    let addr = first_cell_and_other_style(&src);
    let out = temp("dry");
    let table_s = addr.table.to_string();
    let row_s = addr.row.to_string();
    let col_s = addr.col.to_string();
    let style_s = addr.other_style.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-cell-style",
            src.as_str(),
            "--table",
            &table_s,
            "--row",
            &row_s,
            "--col",
            &col_s,
            "--style",
            &style_s,
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
    assert_eq!(v["ctrl"], addr.other_style);
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-cell-style",
            src.as_str(),
            "--table",
            "0",
            "--row",
            "0",
            "--col",
            "0",
            "--style",
            "0",
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
        .any(|t| t["name"] == "hwp_apply_cell_style"));
}
