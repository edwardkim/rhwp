//! `edit set-form-value` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use rhwp::model::control::{Control, FormType};
use rhwp::wasm_api::HwpDocument;

static SEQ: AtomicU64 = AtomicU64::new(0);

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/form-01.hwp")
        .to_string_lossy()
        .into_owned()
}

fn temp(tag: &str) -> PathBuf {
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "rhwp-formval-{tag}-{}-{}-{}.hwp",
        std::process::id(),
        n,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

/// 문서 트리를 훑어 본문 첫 Form 좌표를 고른다. (0,0,0) 을 가정하지 않는다.
fn first_body_text_form(path: &str) -> (usize, usize, usize) {
    let bytes = std::fs::read(path).expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    for (si, sec) in doc.document().sections.iter().enumerate() {
        for (pi, para) in sec.paragraphs.iter().enumerate() {
            for (ci, c) in para.controls.iter().enumerate() {
                if let Control::Form(form) = c {
                    if matches!(form.form_type, FormType::ComboBox | FormType::Edit) {
                        return (si, pi, ci);
                    }
                }
            }
        }
    }
    panic!("text를 지원하는 본문 양식 컨트롤이 없다");
}

fn form_text(path: &Path, sec: usize, para: usize, ci: usize) -> String {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    match doc.get_form_value_native(sec, para, ci) {
        Ok(raw) => {
            let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
            v["text"].as_str().unwrap_or("").to_string()
        }
        Err(e) => panic!("get_form_value_native: {e}"),
    }
}

#[test]
fn set_form_value_writes_text() {
    let src = sample();
    let (sec, para, ci) = first_body_text_form(&src);
    let out = temp("out");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-form-value",
            src.as_str(),
            "--section",
            &sec.to_string(),
            "--para",
            &para.to_string(),
            "--ctrl",
            &ci.to_string(),
            "--value",
            r#"{"text":"계약값"}"#,
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert!(out.exists());
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["section"], sec);
    assert_eq!(v["paragraph"], para);
    assert_eq!(v["ctrl"], ci);
    assert_eq!(form_text(&out, sec, para, ci), "계약값");
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = sample();
    let (sec, para, ci) = first_body_text_form(&src);
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-form-value",
            src.as_str(),
            "--section",
            &sec.to_string(),
            "--para",
            &para.to_string(),
            "--ctrl",
            &ci.to_string(),
            "--value",
            r#"{"text":"x"}"#,
            "-o",
            out.to_str().unwrap(),
            "--dry-run",
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert!(!out.exists());
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = sample();
    let (sec, para, ci) = first_body_text_form(&src);
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-form-value",
            src.as_str(),
            "--section",
            &sec.to_string(),
            "--para",
            &para.to_string(),
            "--ctrl",
            &ci.to_string(),
            "--value",
            r#"{"text":"x"}"#,
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
        .any(|t| t["name"] == "hwp_set_form_value"));
}
