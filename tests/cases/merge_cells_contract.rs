//! [#4997] `edit merge-cells` 계약.
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

/// 샘플 첫 표의 (0,0)-(0,1) 은 (0,1) rowspan=2 라 네이티브가 거절한다.
/// 앵커 셀을 훑어 span 1×1 인 인접 쌍을 고른다.
fn first_merge_range(path: &str) -> (usize, u16, u16, u16, u16, usize) {
    let bytes = std::fs::read(path).expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    for g in extract_tables(doc.document())
        .into_iter()
        .filter(|g| g.container_path.is_empty())
    {
        for c in &g.cells {
            if c.row_span != 1 || c.col_span != 1 {
                continue;
            }
            if g.cells
                .iter()
                .any(|n| n.row == c.row && n.col == c.col + 1 && n.row_span == 1 && n.col_span == 1)
            {
                return (g.index, c.row, c.col, c.row, c.col + 1, g.cell_count);
            }
            if g.cells
                .iter()
                .any(|n| n.col == c.col && n.row == c.row + 1 && n.row_span == 1 && n.col_span == 1)
            {
                return (g.index, c.row, c.col, c.row + 1, c.col, g.cell_count);
            }
        }
    }
    panic!("병합 가능한 1×1 인접 셀이 없다");
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-merge-{tag}-{}-{}.hwp",
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
fn merge_cells_reduces_count() {
    let src = sample();
    let (idx, row, col, end_row, end_col, before) = first_merge_range(&src);
    let out = temp("out");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "merge-cells",
            src.as_str(),
            "--table",
            &idx.to_string(),
            "--row",
            &row.to_string(),
            "--col",
            &col.to_string(),
            "--end-row",
            &end_row.to_string(),
            "--end-col",
            &end_col.to_string(),
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    let after = table_cells(&out, idx);
    assert!(
        after < before,
        "병합 후 셀 수가 줄어야 한다: {before} -> {after}"
    );
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = sample();
    let (idx, row, col, end_row, end_col, _) = first_merge_range(&src);
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "merge-cells",
            src.as_str(),
            "--table",
            &idx.to_string(),
            "--row",
            &row.to_string(),
            "--col",
            &col.to_string(),
            "--end-row",
            &end_row.to_string(),
            "--end-col",
            &end_col.to_string(),
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
