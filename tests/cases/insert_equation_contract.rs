//! `edit insert-equation` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

use rhwp::model::control::{Control, Equation};
use rhwp::model::shape::{HorzRelTo, TextWrap, VertRelTo};
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

fn equation_by_script(path: &Path, script: &str) -> Equation {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    doc.document()
        .sections
        .iter()
        .flat_map(|section| section.paragraphs.iter())
        .flat_map(|paragraph| paragraph.controls.iter())
        .find_map(|control| match control {
            Control::Equation(eq) if eq.script == script => Some(eq.as_ref().clone()),
            _ => None,
        })
        .expect("inserted equation must round-trip")
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

    let equation = equation_by_script(&out, "a+b");
    assert_eq!(equation.common.attr, 0x0C2A_2311);
    assert!(equation.common.treat_as_char);
    assert!(equation.common.flow_with_text);
    assert_eq!(equation.common.vert_rel_to, VertRelTo::Para);
    assert_eq!(equation.common.horz_rel_to, HorzRelTo::Para);
    assert_eq!(equation.common.text_wrap, TextWrap::TopAndBottom);
    assert_eq!(equation.common.margin.left, 56);
    assert_eq!(equation.common.margin.right, 56);
    assert_ne!(equation.common.instance_id, 0);
    assert_ne!(equation.common.instance_id & 0x4000_0000, 0);
    assert_eq!(equation.common.instance_id & 0x8000_0000, 0);
    assert_eq!(equation.common.description, "수식입니다.");
    assert_eq!(equation.baseline, 85);
    assert_eq!(equation.version_info, "Equation Version 60");
    assert_eq!(equation.font_name, "HYhwpEQ");
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
