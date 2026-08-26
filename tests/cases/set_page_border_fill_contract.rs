//! `edit set-page-border-fill` 계약.
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

fn first_section(path: &str) -> usize {
    let bytes = std::fs::read(path).expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    doc.document()
        .sections
        .iter()
        .enumerate()
        .map(|(i, _)| i)
        .next()
        .expect("구역이 없다")
}

fn spacing_left(path: &Path, section: usize) -> i64 {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    let raw = doc
        .get_page_border_fill_native(section)
        .expect("page border fill");
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    v["spacingLeft"].as_i64().expect("spacingLeft")
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-pgborder-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn spacing_left_is_visible() {
    let src = sample();
    let section = first_section(&src);
    let before = spacing_left(Path::new(&src), section);
    let target = before + 80;
    let props = format!(r#"{{"spacingLeft":{target}}}"#);
    let out = temp("out");
    let sec_s = section.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-page-border-fill",
            src.as_str(),
            "--section",
            &sec_s,
            "--props",
            &props,
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["section"], section);
    assert_eq!(
        spacing_left(&out, section),
        target,
        "쪽 테두리 spacingLeft 가 저장본에 없다"
    );
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_json_has_fields_and_no_file() {
    let src = sample();
    let section = first_section(&src);
    let out = temp("dry");
    let sec_s = section.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-page-border-fill",
            src.as_str(),
            "--section",
            &sec_s,
            "--props",
            r#"{"spacingLeft":120}"#,
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
    assert_eq!(v["section"], section);
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-page-border-fill",
            src.as_str(),
            "--props",
            r#"{"spacingLeft":120}"#,
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
        .any(|t| t["name"] == "hwp_set_page_border_fill"));
}
