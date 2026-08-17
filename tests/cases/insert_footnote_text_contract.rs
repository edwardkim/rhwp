//! [#5110] `edit insert-footnote-text` 계약.
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
        "rhwp-fntext-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn fixture_with_footnote() -> (PathBuf, usize) {
    let bytes = std::fs::read(sample()).unwrap();
    let mut doc = HwpDocument::from_bytes(&bytes).unwrap();
    doc.insert_footnote_native(0, 0, 0).expect("각주 삽입");
    let ctrl = doc.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .rposition(|c| matches!(c, Control::Footnote(_)))
        .expect("각주 컨트롤");
    let out = temp("fx");
    std::fs::write(&out, doc.export_hwp().expect("export")).unwrap();
    (out, ctrl)
}

fn footnote_text(path: &Path, ctrl: usize) -> String {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    match &doc.document().sections[0].paragraphs[0].controls[ctrl] {
        Control::Footnote(n) => n.paragraphs[0].text.clone(),
        other => panic!("각주가 아님: {other:?}"),
    }
}

#[test]
fn insert_text_is_visible() {
    let (src, ctrl) = fixture_with_footnote();
    let before = footnote_text(&src, ctrl);
    let out = temp("out");
    let ctrl_s = ctrl.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "insert-footnote-text",
            src.to_str().unwrap(),
            "--ctrl",
            &ctrl_s,
            "--text",
            "각주본문",
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["ctrl"], ctrl);
    assert_eq!(v["text"], "각주본문");
    let after = footnote_text(&out, ctrl);
    assert!(
        after.contains("각주본문"),
        "각주 텍스트가 저장본에 없다 before={before:?} after={after:?}"
    );
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
            "insert-footnote-text",
            src.to_str().unwrap(),
            "--ctrl",
            &ctrl_s,
            "--text",
            "미리보기",
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
    assert_eq!(v["text"], "미리보기");
    let _ = std::fs::remove_file(&src);
}

#[test]
fn unknown_flag_empty_stdout() {
    let (src, ctrl) = fixture_with_footnote();
    let ctrl_s = ctrl.to_string();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "insert-footnote-text",
            src.to_str().unwrap(),
            "--ctrl",
            &ctrl_s,
            "--text",
            "x",
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
        .any(|t| t["name"] == "hwp_insert_footnote_text"));
}
