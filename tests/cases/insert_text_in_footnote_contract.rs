//! `edit insert-text-in-footnote` 계약.
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
        "rhwp-fnins-{tag}-{}-{}-{}.hwp",
        std::process::id(),
        n,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn fixture_with_footnote() -> (PathBuf, usize) {
    let mut doc = HwpDocument::create_empty();
    doc.insert_text_native(0, 0, 0, "본문")
        .expect("본문 텍스트");
    let raw = doc.insert_footnote_native(0, 0, 0).expect("각주 삽입");
    assert!(raw.contains(r#""ok":true"#), "{raw}");
    let out = temp("fx");
    std::fs::write(&out, doc.export_hwp().expect("export")).unwrap();
    let bytes = std::fs::read(&out).unwrap();
    let reloaded = HwpDocument::from_bytes(&bytes).unwrap();
    let ctrl = first_footnote_ctrl(&reloaded).expect("각주 컨트롤");
    (out, ctrl)
}

fn first_footnote_ctrl(doc: &HwpDocument) -> Option<usize> {
    doc.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .position(|c| matches!(c, Control::Footnote(_)))
}

fn footnote_text(path: &Path, ctrl: usize) -> String {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    match &doc.document().sections[0].paragraphs[0].controls[ctrl] {
        Control::Footnote(f) => f
            .paragraphs
            .iter()
            .map(|p| p.text.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
        other => panic!("각주가 아님: {other:?}"),
    }
}

#[test]
fn insert_text_in_footnote_writes() {
    let (src, ctrl) = fixture_with_footnote();
    let before = footnote_text(&src, ctrl);
    let out = temp("out");
    let ctrl_s = ctrl.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "insert-text-in-footnote",
            src.to_str().unwrap(),
            "--ctrl",
            &ctrl_s,
            "--text",
            "가",
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["ctrl"], ctrl as u64);
    assert_eq!(v["text"], "가");
    let after = footnote_text(&out, ctrl);
    assert!(after.contains('가'), "before={before:?} after={after:?}");
    assert_ne!(after, before);
    let _ = std::fs::remove_file(&src);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let (src, ctrl) = fixture_with_footnote();
    let out = temp("dry");
    let ctrl_s = ctrl.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "insert-text-in-footnote",
            src.to_str().unwrap(),
            "--ctrl",
            &ctrl_s,
            "--text",
            "가",
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
    let (src, ctrl) = fixture_with_footnote();
    let ctrl_s = ctrl.to_string();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "insert-text-in-footnote",
            src.to_str().unwrap(),
            "--ctrl",
            &ctrl_s,
            "--text",
            "가",
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
        .any(|t| t["name"] == "hwp_insert_text_in_footnote"));
}
