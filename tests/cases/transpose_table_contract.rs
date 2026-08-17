//! `edit transpose-table` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use rhwp::document_core::queries::table_extract::extract_tables;
use rhwp::wasm_api::HwpDocument;

static SEQ: AtomicU64 = AtomicU64::new(0);

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn temp(tag: &str) -> PathBuf {
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "rhwp-trptbl-{tag}-{}-{}-{}.hwp",
        std::process::id(),
        n,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn fixture_2x3() -> PathBuf {
    let mut doc = HwpDocument::create_empty();
    let raw = doc.create_table_native(0, 0, 0, 2, 3).expect("표 생성");
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let para = v["paraIdx"].as_u64().unwrap() as usize;
    let ctrl = v["controlIdx"].as_u64().unwrap() as usize;
    for (idx, ch) in ["A", "B", "C", "D", "E", "F"].iter().enumerate() {
        doc.insert_text_in_cell_native(0, para, ctrl, idx, 0, 0, ch)
            .expect("셀 채움");
    }
    let out = temp("fx");
    std::fs::write(&out, doc.export_hwp().expect("export")).unwrap();
    out
}

fn grid(path: &std::path::Path) -> (u16, u16, Vec<(u16, u16, String)>) {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    let t = extract_tables(doc.document())
        .into_iter()
        .find(|g| g.container_path.is_empty())
        .expect("본문 최상위 표");
    let mut cells: Vec<(u16, u16, String)> = t
        .cells
        .into_iter()
        .map(|c| (c.row, c.col, c.text))
        .collect();
    cells.sort_by_key(|(r, c, _)| (*r, *c));
    (t.rows, t.cols, cells)
}

#[test]
fn transpose_table_swaps_axes() {
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
    let (rows, cols, cells) = grid(&out);
    assert_eq!((rows, cols), (3, 2));
    assert_eq!(
        cells,
        vec![
            (0, 0, "A".into()),
            (0, 1, "D".into()),
            (1, 0, "B".into()),
            (1, 1, "E".into()),
            (2, 0, "C".into()),
            (2, 1, "F".into()),
        ]
    );
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
