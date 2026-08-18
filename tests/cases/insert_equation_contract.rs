//! `edit insert-equation` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

use rhwp::model::control::Control;
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
        "rhwp-eq-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn eq_count(path: &Path) -> usize {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    doc.document()
        .sections
        .iter()
        .flat_map(|s| s.paragraphs.iter())
        .map(|p| {
            p.controls
                .iter()
                .filter(|c| matches!(c, Control::Equation(_)))
                .count()
        })
        .sum()
}

fn first_script(path: &Path) -> String {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    for s in &doc.document().sections {
        for p in &s.paragraphs {
            for c in &p.controls {
                if let Control::Equation(eq) = c {
                    return eq.script.clone();
                }
            }
        }
    }
    String::new()
}

#[test]
fn insert_equation_adds_control() {
    let src = sample();
    let before = eq_count(Path::new(&src));
    let out = temp("out");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "insert-equation",
            src.as_str(),
            "--script",
            "a+b",
            "--offset",
            "0",
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert_eq!(eq_count(&out), before + 1);
    assert_eq!(first_script(&out), "a+b");
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = sample();
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "insert-equation",
            src.as_str(),
            "--script",
            "a+b",
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
fn unknown_flag_empty_stdout() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "insert-equation",
            src.as_str(),
            "--script",
            "a+b",
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
        .any(|t| t["name"] == "hwp_insert_equation"));
}
