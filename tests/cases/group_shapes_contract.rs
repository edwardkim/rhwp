//! `edit group-shapes` 계약.
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
        "rhwp-group-{tag}-{}-{}-{}.hwp",
        std::process::id(),
        n,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn add_rectangle(doc: &mut HwpDocument, horz: u32, vert: u32) {
    doc.create_shape_control_native(
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
}

fn fixture_two_shapes() -> PathBuf {
    let mut doc = HwpDocument::create_empty();
    add_rectangle(&mut doc, 0, 0);
    add_rectangle(&mut doc, 2000, 2000);
    let out = temp("fx");
    std::fs::write(&out, doc.export_hwp().expect("export")).unwrap();
    out
}

fn shape_addrs(path: &Path) -> Vec<(usize, usize, usize)> {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    let mut addrs = Vec::new();
    for (si, s) in doc.document().sections.iter().enumerate() {
        for (pi, p) in s.paragraphs.iter().enumerate() {
            for (ci, c) in p.controls.iter().enumerate() {
                if matches!(c, Control::Shape(_)) {
                    addrs.push((si, pi, ci));
                }
            }
        }
    }
    addrs
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

#[test]
fn group_shapes_makes_group() {
    let src = fixture_two_shapes();
    let addrs = shape_addrs(&src);
    assert_eq!(addrs.len(), 2);
    assert_eq!(group_count(&src), 0);
    let targets = format!(
        r#"[{{"paraIdx":{},"controlIdx":{}}},{{"paraIdx":{},"controlIdx":{}}}]"#,
        addrs[0].1, addrs[0].2, addrs[1].1, addrs[1].2
    );
    let out = temp("out");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "group-shapes",
            src.to_str().unwrap(),
            "--section",
            &addrs[0].0.to_string(),
            "--targets",
            &targets,
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert_eq!(group_count(&out), 1);
    HwpDocument::from_bytes(&std::fs::read(&out).unwrap()).expect("산출물 재파싱");
    let _ = std::fs::remove_file(&src);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = fixture_two_shapes();
    let addrs = shape_addrs(&src);
    let targets = format!(
        r#"[{{"paraIdx":{},"controlIdx":{}}},{{"paraIdx":{},"controlIdx":{}}}]"#,
        addrs[0].1, addrs[0].2, addrs[1].1, addrs[1].2
    );
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "group-shapes",
            src.to_str().unwrap(),
            "--targets",
            &targets,
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
    let src = fixture_two_shapes();
    let addrs = shape_addrs(&src);
    let targets = format!(
        r#"[{{"paraIdx":{},"controlIdx":{}}},{{"paraIdx":{},"controlIdx":{}}}]"#,
        addrs[0].1, addrs[0].2, addrs[1].1, addrs[1].2
    );
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "group-shapes",
            src.to_str().unwrap(),
            "--targets",
            &targets,
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
        .any(|t| t["name"] == "hwp_group_shapes"));
}
