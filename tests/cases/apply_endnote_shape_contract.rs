//! `edit apply-endnote-shape` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

use rhwp::wasm_api::HwpDocument;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-enshape-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn fixture_with_endnote() -> PathBuf {
    let mut doc = HwpDocument::create_empty();
    doc.insert_endnote_native(0, 0, 0).expect("미주 삽입");
    let out = temp("fx");
    std::fs::write(&out, doc.export_hwp().expect("export")).unwrap();
    out
}

fn start_number(path: &Path) -> u64 {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    let raw = doc.get_endnote_shape_native(0).expect("미주 모양");
    let v: serde_json::Value = serde_json::from_str(&raw).expect("JSON");
    v["startNumber"].as_u64().expect("startNumber")
}

#[test]
fn apply_endnote_shape_sets_start() {
    let src = fixture_with_endnote();
    let out = temp("out");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-endnote-shape",
            src.to_str().unwrap(),
            "--props",
            r#"{"startNumber":5}"#,
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert_eq!(start_number(&out), 5);
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["section"], 0);
    let _ = std::fs::remove_file(&src);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = fixture_with_endnote();
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-endnote-shape",
            src.to_str().unwrap(),
            "--props",
            r#"{"startNumber":3}"#,
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
    let src = fixture_with_endnote();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-endnote-shape",
            src.to_str().unwrap(),
            "--props",
            "{}",
            "--nope",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    let _ = std::fs::remove_file(&src);
}
