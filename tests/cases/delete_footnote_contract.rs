//! [#5017] `edit delete-footnote` 계약.
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
        "rhwp-delfn-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn first_footnote_ctrl(path: &Path) -> Option<(usize, usize)> {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    let para = &doc.document().sections[0].paragraphs[0];
    let idx = para
        .controls
        .iter()
        .position(|c| matches!(c, Control::Footnote(_)))?;
    Some((
        para.controls
            .iter()
            .filter(|c| matches!(c, Control::Footnote(_)))
            .count(),
        idx,
    ))
}

#[test]
fn delete_footnote_removes_control() {
    let src = sample();
    let inserted = temp("ins");
    let insert = Command::new(rhwp_bin())
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
    assert_eq!(insert.status.code(), Some(0), "{:?}", insert);
    let (before, ctrl) = first_footnote_ctrl(&inserted).expect("삽입한 각주 컨트롤");
    let out = temp("out");
    let ctrl_s = ctrl.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "delete-footnote",
            inserted.to_str().unwrap(),
            "--section",
            "0",
            "--para",
            "0",
            "--ctrl",
            &ctrl_s,
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    let after = first_footnote_ctrl(&out).map(|(n, _)| n).unwrap_or(0);
    assert_eq!(after, before - 1);
    let _ = std::fs::remove_file(&inserted);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = sample();
    let inserted = temp("insdry");
    let insert = Command::new(rhwp_bin())
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
    assert_eq!(insert.status.code(), Some(0), "{:?}", insert);
    let (_, ctrl) = first_footnote_ctrl(&inserted).expect("삽입한 각주 컨트롤");
    let out = temp("dry");
    let ctrl_s = ctrl.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "delete-footnote",
            inserted.to_str().unwrap(),
            "--section",
            "0",
            "--para",
            "0",
            "--ctrl",
            &ctrl_s,
            "-o",
            out.to_str().unwrap(),
            "--dry-run",
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert!(!out.exists());
    let _ = std::fs::remove_file(&inserted);
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
        .any(|t| t["name"] == "hwp_delete_footnote"));
}
