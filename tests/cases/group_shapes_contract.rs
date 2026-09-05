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

fn first_two_shape_targets(path: &Path) -> (usize, String) {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    let mut found = Vec::new();
    for (si, s) in doc.document().sections.iter().enumerate() {
        for (pi, p) in s.paragraphs.iter().enumerate() {
            for (ci, c) in p.controls.iter().enumerate() {
                if matches!(c, Control::Shape(_) | Control::Picture(_)) {
                    found.push((si, pi, ci));
                }
            }
        }
    }
    assert!(
        found.len() >= 2,
        "도형/그림이 2개 이상 있어야 한다: {found:?}"
    );
    let section = found[0].0;
    assert!(
        found.iter().all(|t| t.0 == section),
        "묶을 도형이 같은 구역이어야 한다: {found:?}"
    );
    (
        section,
        format!(
            "{},{};{},{}",
            found[0].1, found[0].2, found[1].1, found[1].2
        ),
    )
}

#[test]
fn group_shapes_merges_two_rectangles() {
    let src = fixture_two_shapes();
    assert_eq!(shape_counts(&src), (2, 0));
    let (section, targets) = first_two_shape_targets(&src);
    let out = temp("out");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "group-shapes",
            src.to_str().unwrap(),
            "--section",
            &section.to_string(),
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
    let (section, targets) = first_two_shape_targets(&src);
    let parts: Vec<&str> = targets.split(';').collect();
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "group-shapes",
            src.to_str().unwrap(),
            "--section",
            &section.to_string(),
            "--target",
            parts[0],
            "--target",
            parts[1],
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
    assert_eq!(v["count"], 2);
    let _ = std::fs::remove_file(&src);
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = fixture_two_shapes();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "group-shapes",
            src.to_str().unwrap(),
            "--targets",
            "0,0;0,1",
            "--nope",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    let _ = std::fs::remove_file(&src);
}
