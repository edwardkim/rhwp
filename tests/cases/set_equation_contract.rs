//! `edit set-equation` 계약.
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
        "rhwp-seteq-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn fixture_with_equation() -> PathBuf {
    let mut doc = HwpDocument::create_empty();
    doc.insert_equation_native(0, 0, 0, "a+b", 1000, 0)
        .expect("수식 삽입");
    let out = temp("fx");
    std::fs::write(&out, doc.export_hwp().expect("export")).unwrap();
    out
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

fn first_eq_script(path: &Path) -> String {
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
    panic!("수식이 없다");
}

#[test]
fn set_equation_rewrites_scanned_script() {
    let src = fixture_with_equation();
    let (section, para, ctrl) = first_eq_addr(&src);
    let before = first_eq_script(&src);
    let want = if before == "x^2+1" { "y^2+2" } else { "x^2+1" };
    let props = format!(r#"{{"script":"{want}"}}"#);
    let out = temp("out");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-equation",
            src.to_str().unwrap(),
            "--section",
            &section.to_string(),
            "--para",
            &para.to_string(),
            "--ctrl",
            &ctrl.to_string(),
            "--props",
            &props,
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert_eq!(first_eq_script(&out), want);
    HwpDocument::from_bytes(&std::fs::read(&out).unwrap()).expect("산출물 재파싱");
    let _ = std::fs::remove_file(&src);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = fixture_with_equation();
    let (section, para, ctrl) = first_eq_addr(&src);
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-equation",
            src.to_str().unwrap(),
            "--section",
            &section.to_string(),
            "--para",
            &para.to_string(),
            "--ctrl",
            &ctrl.to_string(),
            "--props",
            r#"{"script":"x^2"}"#,
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
    let (section, para, ctrl) = first_eq_addr(&src);
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-equation",
            src.to_str().unwrap(),
            "--section",
            &section.to_string(),
            "--para",
            &para.to_string(),
            "--ctrl",
            &ctrl.to_string(),
            "--props",
            r#"{"script":"x^2"}"#,
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
        .any(|t| t["name"] == "hwp_set_equation"));
}
