//! `edit move-table` 계약.
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

fn first_top_table(path: &str) -> (usize, i32) {
    let bytes = std::fs::read(path).expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let g = extract_tables(doc.document())
        .into_iter()
        .find(|g| g.container_path.is_empty())
        .expect("본문 최상위 표");
    let h = match &doc.document().sections[g.section].paragraphs[g.paragraph].controls[g.control] {
        Control::Table(t) => t.common.horizontal_offset as i32,
        _ => panic!("표 컨트롤"),
    };
    (g.index, h)
}

fn table_h_offset(path: &Path, index: usize) -> i32 {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    let g = extract_tables(doc.document())
        .into_iter()
        .find(|g| g.index == index && g.container_path.is_empty())
        .expect("표");
    match &doc.document().sections[g.section].paragraphs[g.paragraph].controls[g.control] {
        Control::Table(t) => t.common.horizontal_offset as i32,
        _ => panic!("표 컨트롤"),
    }
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-movetbl-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn move_dx_changes_offset() {
    let src = sample();
    let (idx, before) = first_top_table(&src);
    let out = temp("out");
    let dx = 1200i32;
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "move-table",
            src.as_str(),
            "--table",
            &idx.to_string(),
            "--dx",
            &dx.to_string(),
            "--dy",
            "0",
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    let after = table_h_offset(&out, idx);
    assert_eq!(
        after,
        before.wrapping_add(dx),
        "before={before} after={after}"
    );
    HwpDocument::from_bytes(&std::fs::read(&out).unwrap()).expect("산출물 재파싱");
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = sample();
    let (idx, _) = first_top_table(&src);
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "move-table",
            src.as_str(),
            "--table",
            &idx.to_string(),
            "--dx",
            "100",
            "--dy",
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
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = sample();
    let (idx, _) = first_top_table(&src);
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "move-table",
            src.as_str(),
            "--table",
            &idx.to_string(),
            "--dx",
            "1",
            "--dy",
            "0",
            "--nope",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
}
