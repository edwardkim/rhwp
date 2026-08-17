//! `edit apply-style` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use rhwp::wasm_api::HwpDocument;

static SEQ: AtomicU64 = AtomicU64::new(0);

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn temp(tag: &str) -> PathBuf {
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "rhwp-style-{tag}-{}-{}-{}.hwp",
        std::process::id(),
        n,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn fixture_with_extra_style() -> (PathBuf, usize) {
    let mut doc = HwpDocument::create_empty();
    let style_id =
        doc.create_style(r#"{"name":"계약","englishName":"Contract","type":0,"nextStyleId":0}"#);
    assert!(style_id >= 0, "스타일 생성");
    let sid = style_id as usize;
    let out = temp("fx");
    std::fs::write(&out, doc.export_hwp().expect("export")).unwrap();
    (out, sid)
}

fn first_style_id(path: &Path) -> u8 {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    doc.document().sections[0].paragraphs[0].style_id
}

#[test]
fn apply_style_sets_id() {
    let (src, sid) = fixture_with_extra_style();
    let out = temp("out");
    let sid_s = sid.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-style",
            src.to_str().unwrap(),
            "--style",
            &sid_s,
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert_eq!(first_style_id(&out) as usize, sid);
    let _ = std::fs::remove_file(&src);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let (src, sid) = fixture_with_extra_style();
    let out = temp("dry");
    let sid_s = sid.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-style",
            src.to_str().unwrap(),
            "--style",
            &sid_s,
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
    let (src, _) = fixture_with_extra_style();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-style",
            src.to_str().unwrap(),
            "--style",
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
        .any(|t| t["name"] == "hwp_apply_style"));
}
