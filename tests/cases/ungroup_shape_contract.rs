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
        "rhwp-ungrp-{tag}-{}-{}-{}.hwp",
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
    (
        section,
        format!(
            "{},{};{},{}",
            found[0].1, found[0].2, found[1].1, found[1].2
        ),
    )
}

fn first_group(path: &Path) -> (usize, usize, usize) {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    for (si, s) in doc.document().sections.iter().enumerate() {
        for (pi, p) in s.paragraphs.iter().enumerate() {
            for (ci, c) in p.controls.iter().enumerate() {
                if let Control::Shape(shape) = c {
                    if matches!(shape.as_ref(), ShapeObject::Group(_)) {
                        return (si, pi, ci);
                    }
                }
            }
        }
    }
    panic!("그룹 도형이 없다");
}

fn grouped_fixture() -> PathBuf {
    let src = fixture_two_shapes();
    let (section, targets) = first_two_shape_targets(&src);
    let grouped = temp("grp");
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
            grouped.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    let _ = std::fs::remove_file(&src);
    grouped
}

#[test]
fn ungroup_shape_restores_children() {
    let grouped = grouped_fixture();
    assert_eq!(shape_counts(&grouped), (1, 1));
    let (si, pi, ci) = first_group(&grouped);
    let out = temp("out");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "ungroup-shape",
            grouped.to_str().unwrap(),
            "--section",
            &si.to_string(),
            "--para",
            &pi.to_string(),
            "--ctrl",
            &ci.to_string(),
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert_eq!(shape_counts(&out), (2, 0));
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["section"], si);
    assert_eq!(v["paragraph"], pi);
    assert_eq!(v["ctrl"], ci);
    let _ = std::fs::remove_file(&grouped);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let grouped = grouped_fixture();
    let (si, pi, ci) = first_group(&grouped);
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "ungroup-shape",
            grouped.to_str().unwrap(),
            "--section",
            &si.to_string(),
            "--para",
            &pi.to_string(),
            "--ctrl",
            &ci.to_string(),
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
    let _ = std::fs::remove_file(&grouped);
}

#[test]
fn unknown_flag_empty_stdout() {
    let grouped = grouped_fixture();
    let (si, pi, ci) = first_group(&grouped);
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "ungroup-shape",
            grouped.to_str().unwrap(),
            "--section",
            &si.to_string(),
            "--para",
            &pi.to_string(),
            "--ctrl",
            &ci.to_string(),
            "--nope",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    let _ = std::fs::remove_file(&grouped);
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
