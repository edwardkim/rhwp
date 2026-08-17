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
        "rhwp-grpshape-{tag}-{}-{}-{}.hwp",
        std::process::id(),
        n,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn fixture_two_shapes() -> PathBuf {
    let mut doc = HwpDocument::create_empty();
    for (x, y) in [(0u32, 0u32), (12000, 0)] {
        doc.create_shape_control_native(
            0,
            0,
            0,
            4000,
            4000,
            x,
            y,
            false,
            "InFrontOfText",
            "rectangle",
            false,
            false,
            &[],
        )
        .expect("도형 삽입");
    }
    let out = temp("fx");
    std::fs::write(&out, doc.export_hwp().expect("export")).unwrap();
    out
}

fn shape_addrs(path: &Path) -> Vec<(usize, usize)> {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    let mut out = Vec::new();
    for s in &doc.document().sections {
        for (pi, p) in s.paragraphs.iter().enumerate() {
            for (ci, c) in p.controls.iter().enumerate() {
                if matches!(c, Control::Shape(_) | Control::Picture(_)) {
                    out.push((pi, ci));
                }
            }
        }
    }
    out
}

fn shape_counts(path: &Path) -> (usize, usize) {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    let mut shapes = 0usize;
    let mut groups = 0usize;
    for s in &doc.document().sections {
        for p in &s.paragraphs {
            for c in &p.controls {
                if let Control::Shape(shape) = c {
                    shapes += 1;
                    if matches!(shape.as_ref(), ShapeObject::Group(_)) {
                        groups += 1;
                    }
                }
            }
        }
    }
    (shapes, groups)
}

#[test]
fn group_shapes_merges_two_rectangles() {
    let src = fixture_two_shapes();
    assert_eq!(shape_counts(&src), (2, 0));
    let addrs = shape_addrs(&src);
    assert_eq!(addrs.len(), 2, "묶을 도형 2개가 있어야 한다: {addrs:?}");
    let targets = format!(
        "{},{};{},{}",
        addrs[0].0, addrs[0].1, addrs[1].0, addrs[1].1
    );
    let out = temp("out");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "group-shapes",
            src.to_str().unwrap(),
            "--targets",
            &targets,
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert_eq!(shape_counts(&out), (1, 1));
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["count"], 2);
    let _ = std::fs::remove_file(&src);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = fixture_two_shapes();
    let addrs = shape_addrs(&src);
    assert_eq!(addrs.len(), 2);
    let t0 = format!("{},{}", addrs[0].0, addrs[0].1);
    let t1 = format!("{},{}", addrs[1].0, addrs[1].1);
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "group-shapes",
            src.to_str().unwrap(),
            "--target",
            &t0,
            "--target",
            &t1,
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
        "{},{};{},{}",
        addrs[0].0, addrs[0].1, addrs[1].0, addrs[1].1
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
