//! `edit apply-para-format` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

use rhwp::model::style::Alignment;
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

fn first_body_para(path: &str) -> (usize, usize) {
    let bytes = std::fs::read(path).expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    for (si, sec) in doc.document().sections.iter().enumerate() {
        if let Some(pi) = sec.paragraphs.iter().position(|p| !p.text.is_empty()) {
            return (si, pi);
        }
    }
    panic!("글자가 있는 본문 문단이 없다");
}

fn para_alignment(path: &Path, section: usize, para: usize) -> Alignment {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    let psid = doc.document().sections[section].paragraphs[para].para_shape_id as usize;
    doc.document().doc_info.para_shapes[psid].alignment
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-pafmt-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn apply_center_alignment_is_visible() {
    let src = sample();
    let (section, para) = first_body_para(&src);
    assert_ne!(
        para_alignment(Path::new(&src), section, para),
        Alignment::Center,
        "샘플이 이미 가운데 정렬이라 판별이 안 된다"
    );
    let out = temp("out");
    let sec_s = section.to_string();
    let para_s = para.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-para-format",
            src.as_str(),
            "--section",
            &sec_s,
            "--para",
            &para_s,
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
    assert_eq!(v["section"], section);
    assert_eq!(v["paragraph"], para);
    assert_eq!(
        para_alignment(&out, section, para),
        Alignment::Center,
        "가운데 정렬이 저장본에 없다"
    );
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_json_has_fields_and_no_file() {
    let src = sample();
    let (section, para) = first_body_para(&src);
    let out = temp("dry");
    let sec_s = section.to_string();
    let para_s = para.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-para-format",
            src.as_str(),
            "--section",
            &sec_s,
            "--para",
            &para_s,
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
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["dryRun"], true);
    assert_eq!(v["paragraph"], para);
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-para-format",
            src.as_str(),
            "--props",
            r#"{"alignment":"center"}"#,
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
        .any(|t| t["name"] == "hwp_apply_para_format"));
}
