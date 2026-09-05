//! `edit apply-char-format-in-cell` 계약.
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

/// extract_tables 에서 본문 최상위 표의 1×1 텍스트 셀을 찾는다. (0,0) 고정 금지.
/// 반환: (table, row, col, cell_para, char_len)
fn first_plain_text_cell(path: &str) -> (usize, u16, u16, usize, usize) {
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
                let n = p.text.chars().count();
                if n >= 1 {
                    return (g.index, c.row, c.col, pi, n);
                }
            }
        }
    }
    panic!("1×1 텍스트 셀이 없다");
}

fn cell_has_bold(path: &Path, table: usize, row: u16, col: u16, cell_para: usize) -> bool {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    let Some(grid) = extract_tables(doc.document())
        .into_iter()
        .find(|g| g.index == table && g.container_path.is_empty())
    else {
        return false;
    };
    let Some(Control::Table(tbl)) = doc.document().sections[grid.section].paragraphs
        [grid.paragraph]
        .controls
        .get(grid.control)
    else {
        return false;
    };
    let Some(cell) = tbl.cells.iter().find(|c| c.row == row && c.col == col) else {
        return false;
    };
    let Some(para) = cell.paragraphs.get(cell_para) else {
        return false;
    };
    let shapes = &doc.document().doc_info.char_shapes;
    para.char_shapes.iter().any(|cs| {
        shapes
            .get(cs.char_shape_id as usize)
            .is_some_and(|s| s.bold)
    })
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-chfmtcell-{tag}-{}-{}.hwpx",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn apply_bold_is_visible_and_reparses() {
    let src = sample();
    let (table, row, col, cell_para, char_len) = first_plain_text_cell(&src);
    let end = char_len.min(1);
    let out = temp("out");
    let table_s = table.to_string();
    let row_s = row.to_string();
    let col_s = col.to_string();
    let para_s = cell_para.to_string();
    let end_s = end.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-char-format-in-cell",
            src.as_str(),
            "--table",
            &table_s,
            "--row",
            &row_s,
            "--col",
            &col_s,
            "--cell-para",
            &para_s,
            "--start",
            "0",
            "--end",
            &end_s,
            "--bold",
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
    assert_eq!(v["bold"], true);
    assert!(out.exists(), "산출 파일이 없다");
    let reparsed = HwpDocument::from_bytes(&std::fs::read(&out).unwrap());
    assert!(reparsed.is_ok(), "산출 재파싱 실패: {:?}", reparsed.err());
    assert!(
        cell_has_bold(&out, table, row, col, cell_para),
        "굵게가 저장본에 없다 table={table} ({row},{col}) para={cell_para}"
    );
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = sample();
    let (table, row, col, _cell_para, _len) = first_plain_text_cell(&src);
    let out = temp("dry");
    let table_s = table.to_string();
    let row_s = row.to_string();
    let col_s = col.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-char-format-in-cell",
            src.as_str(),
            "--table",
            &table_s,
            "--row",
            &row_s,
            "--col",
            &col_s,
            "--props",
            r#"{"bold":true}"#,
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
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-char-format-in-cell",
            src.as_str(),
            "--table",
            "0",
            "--row",
            "0",
            "--col",
            "0",
            "--bold",
            "--nope",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(out.stdout.len(), 0);
}
