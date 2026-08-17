//! [#4999] `word-count` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
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

#[test]
fn word_count_json_has_counts() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args(["word-count", src.as_str(), "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{:?}", out);
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(v["charCount"].as_u64().unwrap() > 0);
    assert!(v["paragraphCount"].as_u64().unwrap() > 0);
    assert!(v["sectionCount"].as_u64().unwrap() >= 1);
    assert!(v["pageCount"].as_u64().is_some());
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args(["word-count", src.as_str(), "--nope"])
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
        .any(|t| t["name"] == "hwp_word_count"));
}
