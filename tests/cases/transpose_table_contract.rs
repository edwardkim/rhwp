//! [#5108] `edit transpose-table` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;
use std::process::Command;

use rhwp::document_core::queries::table_extract::extract_tables;
use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-transpose-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn fixture_2x3() -> PathBuf {
    let mut doc = HwpDocument::create_empty();
    doc.create_table_native(0, 0, 0, 2, 3).expect("2x3 표 생성");
    for row in 0u16..2 {
        for col in 0u16..3 {
            let cell_idx = (row * 3 + col) as usize;
            let text = format!("r{row}c{col}");
            doc.insert_text_in_cell_native(0, 0, 0, cell_idx, 0, 0, &text)
                .expect("셀 텍스트");
        }
    }
    let out = temp("fx");
    std::fs::write(&out, doc.export_hwp().expect("export")).unwrap();
    out
}

fn table_shape_and_cell(path: &std::path::Path) -> (u16, u16, String) {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    let g = extract_tables(doc.document())
        .into_iter()
        .find(|g| g.container_path.is_empty())
        .expect("표");
    let Some(Control::Table(table)) = doc.document().sections[g.section].paragraphs[g.paragraph]
        .controls
        .get(g.control)
    else {
        panic!("표 컨트롤");
    };
    let text = table
        .cells
        .iter()
        .find(|c| c.row == 0 && c.col == 1)
        .expect("0,1")
        .paragraphs[0]
        .text
        .clone();
    (table.row_count, table.col_count, text)
}

#[test]
fn transpose_swaps_rows_and_cols() {
    let src = fixture_2x3();
    let out = temp("out");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "transpose-table",
            src.to_str().unwrap(),
            "--table",
            "0",
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["table"], 0);
    assert_eq!(v["sourceRows"], 2);
    assert_eq!(v["sourceCols"], 3);
    assert_eq!(v["targetRows"], 3);
    assert_eq!(v["targetCols"], 2);
    let (rows, cols, at_01) = table_shape_and_cell(&out);
    assert_eq!((rows, cols), (3, 2));
    assert_eq!(at_01, "r1c0", "전치 후 (0,1) 은 원래 (1,0)");
    let _ = std::fs::remove_file(&src);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = fixture_2x3();
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "transpose-table",
            src.to_str().unwrap(),
            "--table",
            "0",
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
    assert_eq!(v["table"], 0);
    let _ = std::fs::remove_file(&src);
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = fixture_2x3();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "transpose-table",
            src.to_str().unwrap(),
            "--table",
            "0",
            "--nope",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    let _ = std::fs::remove_file(&src);
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
        .any(|t| t["name"] == "hwp_transpose_table"));
}
