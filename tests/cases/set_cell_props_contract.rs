//! `edit set-cell-props` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

use rhwp::document_core::queries::table_extract::extract_tables;
use rhwp::model::control::Control;
use rhwp::model::table::VerticalAlign;
use rhwp::wasm_api::HwpDocument;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-cellprop-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn fixture_plain_table() -> PathBuf {
    let mut doc = HwpDocument::create_empty();
    doc.create_table_native(0, 0, 0, 3, 3)
        .expect("병합 없는 표");
    let out = temp("fx");
    std::fs::write(&out, doc.export_hwp().expect("export")).unwrap();
    out
}

fn first_top_index(path: &Path) -> usize {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    extract_tables(doc.document())
        .into_iter()
        .find(|g| g.container_path.is_empty())
        .expect("본문 최상위 표")
        .index
}

fn first_cell_addr(path: &Path) -> (u16, u16) {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    for s in &doc.document().sections {
        for p in &s.paragraphs {
            for c in &p.controls {
                if let Control::Table(t) = c {
                    let cell = t.cells.first().expect("표 셀");
                    return (cell.row, cell.col);
                }
            }
        }
    }
    panic!("표 셀이 없다");
}

fn cell_valign(path: &Path, row: u16, col: u16) -> VerticalAlign {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    for s in &doc.document().sections {
        for p in &s.paragraphs {
            for c in &p.controls {
                if let Control::Table(t) = c {
                    return t
                        .cells
                        .iter()
                        .find(|cell| cell.row == row && cell.col == col)
                        .expect("대상 셀")
                        .vertical_align;
                }
            }
        }
    }
    panic!("표 셀이 없다");
}

#[test]
fn set_cell_props_centers_scanned_cell() {
    let src = fixture_plain_table();
    let idx = first_top_index(&src);
    let (row, col) = first_cell_addr(&src);
    let before = cell_valign(&src, row, col);
    let (want, props) = match before {
        VerticalAlign::Center => (VerticalAlign::Bottom, r#"{"verticalAlign":2}"#),
        _ => (VerticalAlign::Center, r#"{"verticalAlign":1}"#),
    };
    let out = temp("out");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-cell-props",
            src.to_str().unwrap(),
            "--table",
            &idx.to_string(),
            "--row",
            &row.to_string(),
            "--col",
            &col.to_string(),
            "--props",
            props,
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert_eq!(cell_valign(&out, row, col), want);
    HwpDocument::from_bytes(&std::fs::read(&out).unwrap()).expect("산출물 재파싱");
    let _ = std::fs::remove_file(&src);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = fixture_plain_table();
    let idx = first_top_index(&src);
    let (row, col) = first_cell_addr(&src);
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-cell-props",
            src.to_str().unwrap(),
            "--table",
            &idx.to_string(),
            "--row",
            &row.to_string(),
            "--col",
            &col.to_string(),
            "--props",
            r#"{"verticalAlign":1}"#,
            "-o",
            out.to_str().unwrap(),
            "--dry-run",
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert!(!out.exists());
    let _ = std::fs::remove_file(&src);
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = fixture_plain_table();
    let (row, col) = first_cell_addr(&src);
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-cell-props",
            src.to_str().unwrap(),
            "--table",
            "0",
            "--row",
            &row.to_string(),
            "--col",
            &col.to_string(),
            "--props",
            r#"{"verticalAlign":1}"#,
            "--nope",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    let _ = std::fs::remove_file(&src);
}
