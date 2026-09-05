//! `edit set-picture` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;

static SEQ: AtomicU64 = AtomicU64::new(0);

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn temp(tag: &str) -> PathBuf {
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "rhwp-setpic-{tag}-{}-{}-{}.hwp",
        std::process::id(),
        n,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn tiny_png() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/logo/logo-16.png")
}

fn first_picture_addr(path: &Path) -> (usize, usize, usize) {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    for (si, s) in doc.document().sections.iter().enumerate() {
        for (pi, p) in s.paragraphs.iter().enumerate() {
            for (ci, c) in p.controls.iter().enumerate() {
                if matches!(c, Control::Picture(_)) {
                    return (si, pi, ci);
                }
            }
        }
    }
    panic!("그림이 없다");
}

fn fixture_with_picture() -> PathBuf {
    let mut doc = HwpDocument::create_empty();
    let png = std::fs::read(tiny_png()).expect("tiny png");
    doc.insert_picture_native(
        0,
        0,
        0,
        &[],
        &png,
        1200,
        1200,
        16,
        16,
        "png",
        "logo",
        None,
        None,
    )
    .expect("그림 삽입");
    let out = temp("fx");
    std::fs::write(&out, doc.export_hwp().expect("export")).unwrap();
    out
}

fn picture_brightness(path: &Path, section: usize, para: usize, ctrl: usize) -> i8 {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    match &doc.document().sections[section].paragraphs[para].controls[ctrl] {
        Control::Picture(pic) => pic.image_attr.brightness,
        other => panic!("그림이 아니다: {other:?}"),
    }
}

#[test]
fn set_picture_applies_brightness() {
    let src = fixture_with_picture();
    let (section, para, ctrl) = first_picture_addr(&src);
    assert_eq!(picture_brightness(&src, section, para, ctrl), 0);
    let out = temp("out");
    let section_s = section.to_string();
    let para_s = para.to_string();
    let ctrl_s = ctrl.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-picture",
            src.to_str().unwrap(),
            "--section",
            &section_s,
            "--para",
            &para_s,
            "--ctrl",
            &ctrl_s,
            "--props",
            r#"{"brightness":50}"#,
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    let (section2, para2, ctrl2) = first_picture_addr(&out);
    assert_eq!(picture_brightness(&out, section2, para2, ctrl2), 50);
    let _ = std::fs::remove_file(&src);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = fixture_with_picture();
    let (section, para, ctrl) = first_picture_addr(&src);
    let out = temp("dry");
    let section_s = section.to_string();
    let para_s = para.to_string();
    let ctrl_s = ctrl.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-picture",
            src.to_str().unwrap(),
            "--section",
            &section_s,
            "--para",
            &para_s,
            "--ctrl",
            &ctrl_s,
            "--props",
            r#"{"brightness":50}"#,
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
    let src = fixture_with_picture();
    let (section, para, ctrl) = first_picture_addr(&src);
    let section_s = section.to_string();
    let para_s = para.to_string();
    let ctrl_s = ctrl.to_string();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-picture",
            src.to_str().unwrap(),
            "--section",
            &section_s,
            "--para",
            &para_s,
            "--ctrl",
            &ctrl_s,
            "--props",
            r#"{"brightness":50}"#,
            "--nope",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    let _ = std::fs::remove_file(&src);
}
