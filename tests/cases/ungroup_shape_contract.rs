//! `edit ungroup-shape` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use rhwp::model::control::Control;
use rhwp::model::shape::ShapeObject;
use rhwp::wasm_api::HwpDocument;

static SEQ: AtomicU64 = AtomicU64::new(0);

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn temp(tag: &str) -> PathBuf {
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "rhwp-ungroup-{tag}-{}-{}-{}.hwp",
        std::process::id(),
        n,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn add_rectangle(doc: &mut HwpDocument, horz: u32, vert: u32) -> usize {
    let res = doc
        .create_shape_control_native(
            0,
            0,
            0,
            9000,
            6750,
            horz,
            vert,
            false,
            "InFrontOfText",
            "rectangle",
            false,
            false,
            &[],
        )
        .expect("rectangle");
    serde_json::from_str::<serde_json::Value>(&res)
        .ok()
        .and_then(|v| v["controlIdx"].as_u64())
        .unwrap_or(0) as usize
}

fn fixture_grouped() -> PathBuf {
    let mut doc = HwpDocument::create_empty();
    let a = add_rectangle(&mut doc, 0, 0);
    let b = add_rectangle(&mut doc, 2000, 2000);
    doc.group_shapes_native(0, &[(0, a), (0, b)])
        .expect("group");
    let out = temp("fx");
    std::fs::write(&out, doc.export_hwp().expect("export")).unwrap();
    out
}

fn first_group_addr(path: &Path) -> (usize, usize, usize) {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    for (si, s) in doc.document().sections.iter().enumerate() {
        for (pi, p) in s.paragraphs.iter().enumerate() {
            for (ci, c) in p.controls.iter().enumerate() {
                if matches!(c, Control::Shape(sh) if matches!(sh.as_ref(), ShapeObject::Group(_))) {
                    return (si, pi, ci);
                }
            }
        }
    }
    panic!("묶음이 없다");
}

fn group_count(path: &Path) -> usize {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    doc.document()
        .sections
        .iter()
        .flat_map(|s| s.paragraphs.iter())
        .flat_map(|p| p.controls.iter())
        .filter(|c| matches!(c, Control::Shape(s) if matches!(s.as_ref(), ShapeObject::Group(_))))
        .count()
}

fn shape_count(path: &Path) -> usize {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    doc.document()
        .sections
        .iter()
        .flat_map(|s| s.paragraphs.iter())
        .flat_map(|p| p.controls.iter())
        .filter(|c| matches!(c, Control::Shape(_)))
        .count()
}

#[test]
fn ungroup_shape_restores_children() {
    let src = fixture_grouped();
    assert_eq!(group_count(&src), 1);
    let (section, para, ctrl) = first_group_addr(&src);
    let out = temp("out");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "ungroup-shape",
            src.to_str().unwrap(),
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
    assert_eq!(group_count(&out), 0);
    assert!(shape_count(&out) >= 2);
    HwpDocument::from_bytes(&std::fs::read(&out).unwrap()).expect("산출물 재파싱");
    let _ = std::fs::remove_file(&src);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = fixture_grouped();
    let (section, para, ctrl) = first_group_addr(&src);
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "ungroup-shape",
            src.to_str().unwrap(),
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
    let _ = std::fs::remove_file(&src);
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = fixture_grouped();
    let (section, para, ctrl) = first_group_addr(&src);
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "ungroup-shape",
            src.to_str().unwrap(),
            "--section",
            &section.to_string(),
            "--para",
            &para.to_string(),
            "--ctrl",
            &ctrl.to_string(),
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
        .any(|t| t["name"] == "hwp_ungroup_shape"));
}
