//! `form-value` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::process::Command;

use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/form-01.hwp")
        .to_string_lossy()
        .into_owned()
}

/// 문서 트리를 훑어 본문 첫 Form 좌표를 고른다. (0,0,0) 을 가정하지 않는다.
fn first_body_form(path: &str) -> (usize, usize, usize) {
    let bytes = std::fs::read(path).expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    for (si, sec) in doc.document().sections.iter().enumerate() {
        for (pi, para) in sec.paragraphs.iter().enumerate() {
            for (ci, c) in para.controls.iter().enumerate() {
                if matches!(c, Control::Form(_)) {
                    return (si, pi, ci);
                }
            }
        }
    }
    panic!("본문 양식 컨트롤이 없다");
}

#[test]
fn form_value_json_has_form_fields() {
    let src = sample();
    let (sec, para, ci) = first_body_form(&src);
    let sec_s = sec.to_string();
    let para_s = para.to_string();
    let ci_s = ci.to_string();
    let out = Command::new(rhwp_bin())
        .args([
            "form-value",
            src.as_str(),
            "--section",
            &sec_s,
            "--para",
            &para_s,
            "--ctrl",
            &ci_s,
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{:?}", out);
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["section"], sec);
    assert_eq!(v["paragraph"], para);
    assert_eq!(v["ctrl"], ci);
    assert!(v["formType"].as_str().is_some());
    assert!(v["name"].as_str().is_some());
    assert!(v["value"].as_i64().is_some() || v["value"].as_u64().is_some());
    assert!(v["text"].as_str().is_some());
    assert!(v["caption"].as_str().is_some());
    assert!(v["enabled"].as_bool().is_some());
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = sample();
    let (sec, para, ci) = first_body_form(&src);
    let out = Command::new(rhwp_bin())
        .args([
            "form-value",
            src.as_str(),
            "--section",
            &sec.to_string(),
            "--para",
            &para.to_string(),
            "--ctrl",
            &ci.to_string(),
            "--nope",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
}

#[test]
fn mcp_declared() {
    let out = Command::new(rhwp_bin())
        .args(["capabilities", "--mcp"])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(v["tools"]
        .as_array()
        .unwrap()
        .iter()
        .any(|t| t["name"] == "hwp_form_value"));
}
