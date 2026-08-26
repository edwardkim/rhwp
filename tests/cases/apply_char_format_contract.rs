//! `edit apply-char-format` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

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
        for (pi, p) in sec.paragraphs.iter().enumerate() {
            if p.text.chars().count() >= 2 {
                return (si, pi);
            }
        }
    }
    panic!("글자 2개 이상인 본문 문단이 없다");
}

fn para_has_superscript(path: &Path, section: usize, para: usize) -> bool {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    let shapes = &doc.document().doc_info.char_shapes;
    doc.document().sections[section].paragraphs[para]
        .char_shapes
        .iter()
        .any(|cs| {
            shapes
                .get(cs.char_shape_id as usize)
                .is_some_and(|s| s.superscript)
        })
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-chfmt-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn apply_superscript_is_visible() {
    let src = sample();
    let (section, para) = first_body_para(&src);
    assert!(
        !para_has_superscript(Path::new(&src), section, para),
        "샘플이 이미 위첨자라 판별이 안 된다"
    );
    let out = temp("out");
    let sec_s = section.to_string();
    let para_s = para.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-char-format",
            src.as_str(),
            "--section",
            &sec_s,
            "--para",
            &para_s,
            "--offset",
            "0",
            "--count",
            "2",
            "--props",
            r#"{"superscript":true}"#,
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
    assert_eq!(v["offset"], 0);
    assert_eq!(v["count"], 2);
    assert!(
        para_has_superscript(&out, section, para),
        "위첨자가 저장본에 없다"
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
            "apply-char-format",
            src.as_str(),
            "--section",
            &sec_s,
            "--para",
            &para_s,
            "--offset",
            "0",
            "--count",
            "2",
            "--props",
            r#"{"superscript":true}"#,
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
    assert_eq!(v["count"], 2);
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-char-format",
            src.as_str(),
            "--props",
            r#"{"superscript":true}"#,
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
        .any(|t| t["name"] == "hwp_apply_char_format"));
}
