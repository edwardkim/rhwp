//! `edit set-column-def` 계약.
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
        "rhwp-coldef-{tag}-{}-{}-{}.hwp",
        std::process::id(),
        n,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn fixture_empty() -> PathBuf {
    let mut doc = HwpDocument::create_empty();
    let out = temp("fx");
    std::fs::write(&out, doc.export_hwp().expect("export")).unwrap();
    out
}

fn first_column_count(path: &Path) -> u16 {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    for s in &doc.document().sections {
        for p in &s.paragraphs {
            for c in &p.controls {
                if let Control::ColumnDef(cd) = c {
                    return cd.column_count;
                }
            }
        }
    }
    panic!("단 정의가 없다");
}

#[test]
fn set_column_def_writes_count() {
    let src = fixture_empty();
    let out = temp("out");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-column-def",
            src.to_str().unwrap(),
            "--count",
            "2",
            "--same-width",
            "--spacing",
            "1134",
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert_eq!(first_column_count(&out), 2);
    let _ = std::fs::remove_file(&src);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = fixture_empty();
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-column-def",
            src.to_str().unwrap(),
            "--count",
            "2",
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
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-column-def",
            src.to_str().unwrap(),
            "--count",
            "2",
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
        .any(|t| t["name"] == "hwp_set_column_def"));
}
