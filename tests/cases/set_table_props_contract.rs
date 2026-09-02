//! `edit set-table-props` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

use rhwp::document_core::queries::table_extract::extract_tables;
use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-tblprop-{tag}-{}-{}.hwp",
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

fn first_table_spacing(path: &Path) -> i16 {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    for s in &doc.document().sections {
        for p in &s.paragraphs {
            for c in &p.controls {
                if let Control::Table(t) = c {
                    return t.cell_spacing;
                }
            }
        }
    }
    panic!("표가 없다");
}

#[test]
fn set_table_props_writes_scanned_spacing() {
    let src = fixture_plain_table();
    let idx = first_top_index(&src);
    let before = first_table_spacing(&src);
    let want: i16 = if before == 200 { 400 } else { 200 };
    let props = format!(r#"{{"cellSpacing":{want}}}"#);
    let out = temp("out");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-table-props",
            src.to_str().unwrap(),
            "--table",
            &idx.to_string(),
            "--props",
            &props,
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert_eq!(first_table_spacing(&out), want);
    HwpDocument::from_bytes(&std::fs::read(&out).unwrap()).expect("산출물 재파싱");
    let _ = std::fs::remove_file(&src);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = fixture_plain_table();
    let idx = first_top_index(&src);
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-table-props",
            src.to_str().unwrap(),
            "--table",
            &idx.to_string(),
            "--props",
            r#"{"cellSpacing":200}"#,
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
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-table-props",
            src.to_str().unwrap(),
            "--table",
            "0",
            "--props",
            r#"{"cellSpacing":200}"#,
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
        .any(|t| t["name"] == "hwp_set_table_props"));
}
