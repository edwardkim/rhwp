//! `edit split-paragraph-in-footnote` 계약.
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
        .join("samples/field-01.hwp")
        .to_string_lossy()
        .into_owned()
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-fnsplit-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

/// 첫 각주/미주 중 비어 있지 않은 문단. (0,0,0) 을 가정하지 않는다.
fn first_note_text(path: &Path) -> Option<(usize, usize, usize, usize, usize)> {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    for (si, section) in doc.document().sections.iter().enumerate() {
        for (pi, para) in section.paragraphs.iter().enumerate() {
            for (ci, ctrl) in para.controls.iter().enumerate() {
                let paras = match ctrl {
                    Control::Footnote(f) => &f.paragraphs,
                    Control::Endnote(e) => &e.paragraphs,
                    _ => continue,
                };
                for (fi, fp) in paras.iter().enumerate() {
                    let n = fp.text.chars().count();
                    if n > 0 {
                        return Some((si, pi, ci, fi, n));
                    }
                }
            }
        }
    }
    None
}

fn note_para_count(path: &Path, si: usize, pi: usize, ci: usize) -> usize {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    match &doc.document().sections[si].paragraphs[pi].controls[ci] {
        Control::Footnote(f) => f.paragraphs.len(),
        Control::Endnote(e) => e.paragraphs.len(),
        _ => 0,
    }
}

fn insert_footnote() -> PathBuf {
    let src = sample();
    let inserted = temp("ins");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "insert-footnote",
            src.as_str(),
            "--offset",
            "0",
            "-o",
            inserted.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    inserted
}

#[test]
fn split_paragraph_in_footnote_increases_count() {
    let inserted = insert_footnote();
    let (si, pi, ci, fi, n) = first_note_text(&inserted).expect("삽입한 각주에 글자가 있어야 한다");
    assert!(n >= 1);
    let before = note_para_count(&inserted, si, pi, ci);
    let out = temp("out");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "split-paragraph-in-footnote",
            inserted.to_str().unwrap(),
            "--section",
            &si.to_string(),
            "--para",
            &pi.to_string(),
            "--ctrl",
            &ci.to_string(),
            "--fn-para",
            &fi.to_string(),
            "--offset",
            "1",
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["fnPara"], fi);
    assert_eq!(note_para_count(&out, si, pi, ci), before + 1);
    HwpDocument::from_bytes(&std::fs::read(&out).unwrap()).expect("산출물 재파싱");
    let _ = std::fs::remove_file(&inserted);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let inserted = insert_footnote();
    let (si, pi, ci, fi, _) = first_note_text(&inserted).expect("삽입한 각주");
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "split-paragraph-in-footnote",
            inserted.to_str().unwrap(),
            "--section",
            &si.to_string(),
            "--para",
            &pi.to_string(),
            "--ctrl",
            &ci.to_string(),
            "--fn-para",
            &fi.to_string(),
            "--offset",
            "1",
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
    let _ = std::fs::remove_file(&inserted);
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "split-paragraph-in-footnote",
            src.as_str(),
            "--nope",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
}
