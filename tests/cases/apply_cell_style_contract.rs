//! `edit apply-cell-style` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use rhwp::document_core::queries::table_extract::extract_tables;
use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;

static SEQ: AtomicU64 = AtomicU64::new(0);

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx")
}

fn temp(tag: &str) -> PathBuf {
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "rhwp-cellstyle-{tag}-{}-{n}-{}.hwpx",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[derive(Clone, Copy)]
struct CellAddr {
    table: usize,
    row: u16,
    col: u16,
    cell_para: usize,
}

fn first_cell(path: &Path) -> CellAddr {
    let bytes = std::fs::read(path).expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    for grid in extract_tables(doc.document()) {
        if !grid.container_path.is_empty() {
            continue;
        }
        let Some(Control::Table(table)) = doc.document().sections[grid.section].paragraphs
            [grid.paragraph]
            .controls
            .get(grid.control)
        else {
            continue;
        };
        for cell in &table.cells {
            if !cell.paragraphs.is_empty() {
                return CellAddr {
                    table: grid.index,
                    row: cell.row,
                    col: cell.col,
                    cell_para: 0,
                };
            }
        }
    }
    panic!("본문 최상위 표 셀이 없다");
}

fn fixture_with_extra_style() -> (PathBuf, CellAddr, usize) {
    let source = sample();
    let bytes = std::fs::read(&source).expect("sample");
    let mut doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let style_id = doc.create_style(
        r#"{"name":"셀 계약","englishName":"Cell Contract","type":0,"nextStyleId":0}"#,
    );
    assert!(style_id >= 0, "스타일 생성");
    let fixture = temp("fixture");
    std::fs::write(&fixture, doc.export_hwpx_native().expect("fixture export"))
        .expect("fixture write");
    let addr = first_cell(&fixture);
    (fixture, addr, style_id as usize)
}

fn cell_style_id(path: &Path, addr: CellAddr) -> u8 {
    let bytes = std::fs::read(path).expect("saved document");
    let doc = HwpDocument::from_bytes(&bytes).expect("saved parse");
    let grid = extract_tables(doc.document())
        .into_iter()
        .find(|grid| grid.index == addr.table && grid.container_path.is_empty())
        .expect("table");
    let Some(Control::Table(table)) = doc.document().sections[grid.section].paragraphs
        [grid.paragraph]
        .controls
        .get(grid.control)
    else {
        panic!("table control");
    };
    table
        .cells
        .iter()
        .find(|cell| cell.row == addr.row && cell.col == addr.col)
        .expect("cell")
        .paragraphs[addr.cell_para]
        .style_id
}

#[test]
fn apply_cell_style_sets_saved_style_id() {
    let (fixture, addr, style_id) = fixture_with_extra_style();
    let out = temp("out");
    let table = addr.table.to_string();
    let row = addr.row.to_string();
    let col = addr.col.to_string();
    let cell_para = addr.cell_para.to_string();
    let style = style_id.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-cell-style",
            fixture.to_str().unwrap(),
            "--table",
            &table,
            "--row",
            &row,
            "--col",
            &col,
            "--cell-para",
            &cell_para,
            "--style",
            &style,
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    let envelope: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["table"], addr.table);
    assert_eq!(envelope["row"], addr.row);
    assert_eq!(envelope["col"], addr.col);
    assert_eq!(envelope["paragraph"], addr.cell_para);
    assert_eq!(envelope["ctrl"], style_id);
    assert_eq!(cell_style_id(&out, addr) as usize, style_id);
    let _ = std::fs::remove_file(&fixture);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_reports_address_without_writing() {
    let (fixture, addr, style_id) = fixture_with_extra_style();
    let out = temp("dry");
    let table = addr.table.to_string();
    let row = addr.row.to_string();
    let col = addr.col.to_string();
    let style = style_id.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-cell-style",
            fixture.to_str().unwrap(),
            "--table",
            &table,
            "--row",
            &row,
            "--col",
            &col,
            "--style",
            &style,
            "-o",
            out.to_str().unwrap(),
            "--dry-run",
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert!(!out.exists());
    let envelope: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["dryRun"], true);
    assert_eq!(envelope["table"], addr.table);
    assert_eq!(envelope["ctrl"], style_id);
    let _ = std::fs::remove_file(&fixture);
}

#[test]
fn invalid_style_and_unknown_flag_keep_stdout_empty() {
    let source = sample();
    for tail in [
        ["--style", "999999", "--table", "0"],
        ["--style", "0", "--nope", "0"],
    ] {
        let output = Command::new(rhwp_bin())
            .args(["edit", "apply-cell-style", source.to_str().unwrap()])
            .args(tail)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2), "{:?}", output);
        assert!(output.stdout.is_empty());
    }
}
