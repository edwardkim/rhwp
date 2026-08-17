//! [#5031] `edit delete-table` 계약.
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
        .join("samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx")
        .to_string_lossy()
        .into_owned()
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-deltbl-{tag}-{}-{}.hwpx",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn first_table_ctrl(path: &Path) -> Option<(usize, usize, usize)> {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    for (si, section) in doc.document().sections.iter().enumerate() {
        for (pi, para) in section.paragraphs.iter().enumerate() {
            if let Some(ci) = para
                .controls
                .iter()
                .position(|c| matches!(c, Control::Table(_)))
            {
                return Some((si, pi, ci));
            }
        }
    }
    None
}

fn table_count(path: &Path) -> usize {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    doc.document()
        .sections
        .iter()
        .flat_map(|s| s.paragraphs.iter())
        .map(|p| {
            p.controls
                .iter()
                .filter(|c| matches!(c, Control::Table(_)))
                .count()
        })
        .sum()
}

#[test]
fn delete_table_removes_control() {
    let src = sample();
    let (section, para, ctrl) = first_table_ctrl(Path::new(&src)).expect("표 컨트롤");
    let before = table_count(Path::new(&src));
    let out = temp("out");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "delete-table",
            src.as_str(),
            "--section",
            &section.to_string(),
            "--para",
            &para.to_string(),
            "--ctrl",
            &ctrl.to_string(),
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert_eq!(table_count(&out), before - 1);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = sample();
    let (section, para, ctrl) = first_table_ctrl(Path::new(&src)).expect("표 컨트롤");
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "delete-table",
            src.as_str(),
            "--section",
            &section.to_string(),
            "--para",
            &para.to_string(),
            "--ctrl",
            &ctrl.to_string(),
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
        .any(|t| t["name"] == "hwp_delete_table"));
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "delete-table",
            src.as_str(),
            "--section",
            "0",
            "--para",
            "0",
            "--ctrl",
            "0",
            "--nope",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
}
