//! [#5026] `edit add-bookmark` 계약.
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
        "rhwp-addbm-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn bookmark_names(path: &Path) -> Vec<String> {
    let out = Command::new(rhwp_bin())
        .args(["bookmarks", path.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{:?}", out);
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    v["bookmarks"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|b| b["name"].as_str().map(String::from))
        .collect()
}

#[test]
fn add_bookmark_appears_in_list() {
    let src = sample();
    let out = temp("out");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "add-bookmark",
            src.as_str(),
            "--name",
            "rhwp-test-bm",
            "--offset",
            "0",
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert!(bookmark_names(&out).iter().any(|n| n == "rhwp-test-bm"));
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = sample();
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "add-bookmark",
            src.as_str(),
            "--name",
            "rhwp-test-bm",
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
fn mcp_declared() {
    let output = Command::new(rhwp_bin())
        .args(["capabilities", "--mcp"])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let tool = v["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "hwp_add_bookmark")
        .expect("hwp_add_bookmark 도구");
    assert_eq!(
        tool["inputSchema"]["properties"]["name"]["pattern"], r".*\S.*",
        "MCP schema도 CLI와 같이 빈·공백 이름을 거부해야 한다: {tool}"
    );
}
