//! `edit insert-picture` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;

static SEQ: AtomicU64 = AtomicU64::new(0);

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn temp(tag: &str) -> PathBuf {
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "rhwp-inspic-{tag}-{}-{}-{}.hwp",
        std::process::id(),
        n,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn tiny_png() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/logo/logo-16.png")
}

fn fixture_empty() -> PathBuf {
    let mut doc = HwpDocument::create_empty();
    let out = temp("fx");
    std::fs::write(&out, doc.export_hwp().expect("export")).unwrap();
    out
}

fn picture_count(path: &Path) -> usize {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    doc.document()
        .sections
        .iter()
        .flat_map(|s| s.paragraphs.iter())
        .map(|p| {
            p.controls
                .iter()
                .filter(|c| matches!(c, Control::Picture(_)))
                .count()
        })
        .sum()
}

#[test]
fn insert_picture_writes_control() {
    let src = fixture_empty();
    let out = temp("out");
    let png = tiny_png();
    assert!(png.exists(), "tiny png missing: {}", png.display());
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "insert-picture",
            src.to_str().unwrap(),
            "--image",
            png.to_str().unwrap(),
            "--width",
            "1200",
            "--height",
            "1200",
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert_eq!(picture_count(&out), 1);
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["section"], 0);
    assert_eq!(v["paragraph"], 0);
    assert_eq!(v["offset"], 0);
    assert_eq!(v["width"], 1200);
    assert_eq!(v["height"], 1200);
    assert!(v["binDataId"].is_number(), "{v}");
    let _ = std::fs::remove_file(&src);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = fixture_empty();
    let out = temp("dry");
    let png = tiny_png();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "insert-picture",
            src.to_str().unwrap(),
            "--image",
            png.to_str().unwrap(),
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
    let src = fixture_empty();
    let png = tiny_png();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "insert-picture",
            src.to_str().unwrap(),
            "--image",
            png.to_str().unwrap(),
            "--nope",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    let _ = std::fs::remove_file(&src);
}
