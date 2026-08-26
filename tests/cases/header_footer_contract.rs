//! `header-footer` 조회 계약.
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
        "rhwp-hfget-{tag}-{}-{}.hwp",
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
    doc.insert_text_in_header_footer_native(0, false, 2, 0, 0, "FOOT")
        .expect("텍스트");
    let out = temp("fx");
    let exported = doc.export_hwp().expect("export");
    std::fs::write(&out, exported).unwrap();
    out
}

#[test]
fn missing_header_exists_false() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args(["header-footer", src.as_str(), "--header", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{:?}", out);
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["exists"], false);
    assert_eq!(v["isHeader"], true);
    assert_eq!(v["section"], 0);
    assert_eq!(v["applyTo"], 0);
}

#[test]
fn existing_footer_reports_text() {
    let inserted = fixture_with_odd_footer();
    let out = Command::new(rhwp_bin())
        .args([
            "header-footer",
            inserted.to_str().unwrap(),
            "--footer",
            "--apply-to",
            "2",
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{:?}", out);
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["exists"], true);
    assert_eq!(v["isHeader"], false);
    assert_eq!(v["applyTo"], 2);
    assert_eq!(v["text"], "FOOT");
    let _ = std::fs::remove_file(&inserted);
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args(["header-footer", src.as_str(), "--nope"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
}

#[test]
fn mcp_declared() {
    let out = Command::new(rhwp_bin())
        .args(["capabilities", "--mcp"])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(v["tools"]
        .as_array()
        .unwrap()
        .iter()
        .any(|t| t["name"] == "hwp_header_footer"));
}
