//! `edit merge-table` 계약.
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

fn first_splittable(path: &str) -> (usize, u16, usize) {
    let bytes = std::fs::read(path).expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tops: Vec<_> = extract_tables(doc.document())
        .into_iter()
        .filter(|g| g.container_path.is_empty())
        .collect();
    let top_count = tops.len();
    for g in tops {
        if g.rows < 2 {
            continue;
        }
        for at in 1..g.rows {
            let spans = g
                .cells
                .iter()
                .any(|c| c.row < at && c.row.saturating_add(c.row_span) > at);
            if !spans {
                return (g.index, at, top_count);
            }
        }
    }
    panic!("나눌 수 있는 본문 최상위 표가 없다");
}

fn top_count(path: &Path) -> usize {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    extract_tables(doc.document())
        .into_iter()
        .filter(|g| g.container_path.is_empty())
        .count()
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-mergetbl-{tag}-{}-{}.hwpx",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn merge_after_split_restores_count() {
    let src = sample();
    let (idx, at, before_count) = first_splittable(&src);
    let split_out = temp("split");
    let idx_s = idx.to_string();
    let at_s = at.to_string();
    let split = Command::new(rhwp_bin())
        .args([
            "edit",
            "split-table",
            src.as_str(),
            "--table",
            &idx_s,
            "--row",
            &at_s,
            "-o",
            split_out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(split.status.code(), Some(0), "{:?}", split);
    assert_eq!(top_count(&split_out), before_count + 1);

    let merged = temp("out");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "merge-table",
            split_out.to_str().unwrap(),
            "--table",
            &idx_s,
            "-o",
            merged.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert_eq!(top_count(&merged), before_count);
    let _ = std::fs::remove_file(&split_out);
    let _ = std::fs::remove_file(&merged);
}

#[test]
fn dry_run_no_file() {
    let src = sample();
    let (idx, _, _) = first_splittable(&src);
    let out = temp("dry");
    let idx_s = idx.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "merge-table",
            src.as_str(),
            "--table",
            &idx_s,
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
            "merge-table",
            src.as_str(),
            "--table",
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
        .any(|t| t["name"] == "hwp_merge_table"));
}
