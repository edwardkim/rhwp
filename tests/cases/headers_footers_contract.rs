//! [#5044] `headers-footers` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

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
        "rhwp-hflist-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn list_json(path: &str) -> serde_json::Value {
    let out = Command::new(rhwp_bin())
        .args(["headers-footers", path, "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{:?}", out);
    serde_json::from_slice(&out.stdout).unwrap()
}

#[test]
fn headers_footers_json_has_array() {
    let v = list_json(&sample());
    assert!(v["headersFooters"].as_array().is_some());
    assert_eq!(
        v["count"].as_u64().unwrap(),
        v["headersFooters"].as_array().unwrap().len() as u64
    );
}

#[test]
fn insert_then_list_increases_count() {
    let src = sample();
    let before = list_json(&src)["count"].as_u64().unwrap();
    let inserted = temp("ins");
    let add = Command::new(rhwp_bin())
        .args([
            "edit",
            "insert-header-footer",
            src.as_str(),
            "--footer",
            "--apply-to",
            "2",
            "-o",
            inserted.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(add.status.code(), Some(0), "{:?}", add);
    let after = list_json(inserted.to_str().unwrap());
    assert_eq!(after["count"].as_u64().unwrap(), before + 1);
    assert!(after["headersFooters"]
        .as_array()
        .unwrap()
        .iter()
        .any(|h| h["isHeader"] == false && h["applyTo"] == 2));
    let _ = std::fs::remove_file(&inserted);
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args(["headers-footers", src.as_str(), "--nope"])
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
        .any(|t| t["name"] == "hwp_headers_footers"));
}
