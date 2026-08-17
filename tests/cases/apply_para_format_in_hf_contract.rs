//! `edit apply-para-format-in-hf` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use rhwp::model::control::Control;
use rhwp::model::style::Alignment;
use rhwp::wasm_api::HwpDocument;

static SEQ: AtomicU64 = AtomicU64::new(0);

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn temp(tag: &str) -> PathBuf {
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "rhwp-hfpafmt-{tag}-{}-{}-{}.hwp",
        std::process::id(),
        n,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn fixture_header() -> PathBuf {
    let mut doc = HwpDocument::create_empty();
    let raw = doc
        .create_header_footer_native(0, true, 0)
        .expect("머리말 생성");
    assert!(raw.contains(r#""ok":true"#), "{raw}");
    let out = temp("fx");
    std::fs::write(&out, doc.export_hwp().expect("export")).unwrap();
    out
}

fn header_alignment(path: &Path) -> Alignment {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    for para in &doc.document().sections[0].paragraphs {
        for ctrl in &para.controls {
            if let Control::Header(h) = ctrl {
                let id = h.paragraphs[0].para_shape_id;
                return doc.document().doc_info.para_shapes[id as usize].alignment;
            }
        }
    }
    panic!("머리말 없음");
}

#[test]
fn apply_para_format_in_hf_centers() {
    let src = fixture_header();
    let out = temp("out");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-para-format-in-hf",
            src.to_str().unwrap(),
            "--header",
            "--props",
            r#"{"alignment":"center"}"#,
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["isHeader"], true);
    assert_eq!(v["applyTo"], 0);
    assert_eq!(v["paragraph"], 0);
    assert_eq!(header_alignment(&out), Alignment::Center);
    let _ = std::fs::remove_file(&src);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = fixture_header();
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-para-format-in-hf",
            src.to_str().unwrap(),
            "--header",
            "--props",
            r#"{"alignment":"center"}"#,
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
    let src = fixture_header();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-para-format-in-hf",
            src.to_str().unwrap(),
            "--header",
            "--props",
            "{}",
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
        .any(|t| t["name"] == "hwp_apply_para_format_in_hf"));
}
