//! `edit merge-paragraph-in-footnote` 계약.
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
        "rhwp-fnmerge-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

/// 첫 각주/미주. (0,0,0) 을 가정하지 않는다.
fn first_note(path: &Path) -> Option<(usize, usize, usize, usize)> {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    for (si, section) in doc.document().sections.iter().enumerate() {
        for (pi, para) in section.paragraphs.iter().enumerate() {
            for (ci, ctrl) in para.controls.iter().enumerate() {
                let n = match ctrl {
                    Control::Footnote(f) => f.paragraphs.len(),
                    Control::Endnote(e) => e.paragraphs.len(),
                    _ => 0,
                };
                if n > 0 {
                    return Some((si, pi, ci, n));
                }
            }
        }
    }
    None
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

fn split_first_note(src: &Path) -> PathBuf {
    let (si, pi, ci, _) = first_note(src).expect("각주/미주");
    let out = temp("split");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "split-paragraph-in-footnote",
            src.to_str().unwrap(),
            "--section",
            &si.to_string(),
            "--para",
            &pi.to_string(),
            "--ctrl",
            &ci.to_string(),
            "--fn-para",
            "0",
            "--offset",
            "1",
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    out
}

#[test]
fn merge_paragraph_in_footnote_decreases_count() {
    let inserted = insert_footnote();
    let split = split_first_note(&inserted);
    let (si, pi, ci, before) = first_note(&split).expect("분할된 각주");
    assert!(before >= 2, "병합하려면 각주 문단이 2개 이상이어야 한다");
    let out = temp("out");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "merge-paragraph-in-footnote",
            split.to_str().unwrap(),
            "--section",
            &si.to_string(),
            "--para",
            &pi.to_string(),
            "--ctrl",
            &ci.to_string(),
            "--fn-para",
            "1",
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["fnPara"], 1);
    let after = first_note(&out).expect("각주가 남아 있어야 한다");
    assert_eq!(after.3, before - 1);
    HwpDocument::from_bytes(&std::fs::read(&out).unwrap()).expect("산출물 재파싱");
    let _ = std::fs::remove_file(&inserted);
    let _ = std::fs::remove_file(&split);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let inserted = insert_footnote();
    let split = split_first_note(&inserted);
    let (si, pi, ci, _) = first_note(&split).expect("분할된 각주");
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "merge-paragraph-in-footnote",
            split.to_str().unwrap(),
            "--section",
            &si.to_string(),
            "--para",
            &pi.to_string(),
            "--ctrl",
            &ci.to_string(),
            "--fn-para",
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
    let _ = std::fs::remove_file(&split);
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "merge-paragraph-in-footnote",
            src.as_str(),
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
        .any(|t| t["name"] == "hwp_merge_paragraph_in_footnote"));
}
