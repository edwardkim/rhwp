//! `edit delete-equation` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-deleq-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn first_eq_addr(path: &Path) -> (usize, usize, usize) {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    for (si, s) in doc.document().sections.iter().enumerate() {
        for (pi, p) in s.paragraphs.iter().enumerate() {
            for (ci, c) in p.controls.iter().enumerate() {
                if matches!(c, Control::Equation(_)) {
                    return (si, pi, ci);
                }
            }
        }
    }
    panic!("수식이 없다");
}

fn fixture_with_equation() -> PathBuf {
    let mut doc = HwpDocument::create_empty();
    doc.insert_equation_native(0, 0, 0, "a+b", 1000, 0)
        .expect("수식 삽입");
    let out = temp("fx");
    std::fs::write(&out, doc.export_hwp().expect("export")).unwrap();
    out
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

#[test]
fn delete_equation_removes_control() {
    let src = fixture_with_equation();
    assert_eq!(eq_count(&src), 1);
    let (section, para, ctrl) = first_eq_addr(&src);
    let out = temp("out");
    let section_s = section.to_string();
    let para_s = para.to_string();
    let ctrl_s = ctrl.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "delete-equation",
            src.to_str().unwrap(),
            "--section",
            &section_s,
            "--para",
            &para_s,
            "--ctrl",
            &ctrl_s,
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert_eq!(eq_count(&out), 0);
    let _ = std::fs::remove_file(&src);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = fixture_with_equation();
    let (section, para, ctrl) = first_eq_addr(&src);
    let out = temp("dry");
    let section_s = section.to_string();
    let para_s = para.to_string();
    let ctrl_s = ctrl.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "delete-equation",
            src.to_str().unwrap(),
            "--section",
            &section_s,
            "--para",
            &para_s,
            "--ctrl",
            &ctrl_s,
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
    let src = fixture_with_equation();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "delete-equation",
            src.to_str().unwrap(),
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
        .any(|t| t["name"] == "hwp_delete_equation"));
}
