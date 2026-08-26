//! [#5059] `edit insert-field-in-hf` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

use rhwp::wasm_api::HwpDocument;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/field-01.hwp")
        .to_string_lossy()
        .into_owned()
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-hffield-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn fixture_with_odd_footer() -> PathBuf {
    let bytes = std::fs::read(sample()).unwrap();
    let mut doc = HwpDocument::from_bytes(&bytes).unwrap();
    let raw = doc
        .create_header_footer_native(0, false, 2)
        .expect("꼬리말 생성");
    assert!(raw.contains(r#""ok":true"#), "{raw}");
    let out = temp("fx");
    let exported = doc.export_hwp().expect("export");
    std::fs::write(&out, exported).unwrap();
    out
}

#[test]
fn insert_page_field_into_existing_footer() {
    let inserted = fixture_with_odd_footer();
    let out = temp("out");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "insert-field-in-hf",
            inserted.to_str().unwrap(),
            "--footer",
            "--apply-to",
            "2",
            "--field-type",
            "1",
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["isHeader"], false);
    assert_eq!(v["applyTo"], 2);
    assert_eq!(v["fieldType"], 1);
    assert!(out.exists());
    let _ = std::fs::remove_file(&inserted);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let inserted = fixture_with_odd_footer();
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "insert-field-in-hf",
            inserted.to_str().unwrap(),
            "--footer",
            "--apply-to",
            "2",
            "--field-type",
            "1",
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
    assert_eq!(v["fieldType"], 1);
    let _ = std::fs::remove_file(&inserted);
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "insert-field-in-hf",
            src.as_str(),
            "--footer",
            "--field-type",
            "1",
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
        .any(|t| t["name"] == "hwp_insert_field_in_hf"));
}
