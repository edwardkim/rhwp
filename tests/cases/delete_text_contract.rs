//! [#5011] `edit delete-text` 계약.
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
        "rhwp-deltxt-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

/// field-01.hwp 첫 문단은 비어 있다. 글자가 있는 문단을 고른다.
fn first_nonempty_para(path: &str) -> (usize, usize, usize) {
    let bytes = std::fs::read(path).expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    for (si, section) in doc.document().sections.iter().enumerate() {
        for (pi, para) in section.paragraphs.iter().enumerate() {
            let n = para.text.chars().count();
            if n >= 1 {
                return (si, pi, n);
            }
        }
    }
    panic!("삭제할 글자가 있는 문단이 없다");
}

fn para_len(path: &Path, section: usize, para: usize) -> usize {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    doc.document().sections[section].paragraphs[para]
        .text
        .chars()
        .count()
}

#[test]
fn delete_text_shortens_paragraph() {
    let src = sample();
    let (section, para, before) = first_nonempty_para(&src);
    let out = temp("out");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "delete-text",
            src.as_str(),
            "--section",
            &section.to_string(),
            "--para",
            &para.to_string(),
            "--offset",
            "0",
            "--count",
            "1",
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert_eq!(para_len(&out, section, para), before - 1);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = sample();
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "delete-text",
            src.as_str(),
            "--count",
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
        .any(|t| t["name"] == "hwp_delete_text"));
}
