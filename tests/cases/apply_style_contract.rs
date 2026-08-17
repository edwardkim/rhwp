//! `edit apply-style` 계약.
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

fn first_body_para_and_other_style(path: &str) -> (usize, usize, usize, usize) {
    let bytes = std::fs::read(path).expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let styles = &doc.document().doc_info.styles;
    assert!(styles.len() >= 2, "스타일이 2개 미만이라 판별이 안 된다");
    for (si, sec) in doc.document().sections.iter().enumerate() {
        for (pi, p) in sec.paragraphs.iter().enumerate() {
            if p.text.chars().count() < 2 {
                continue;
            }
            let current = p.style_id as usize;
            let other = styles.iter().enumerate().find_map(|(id, st)| {
                if id != current && st.style_type == 0 {
                    Some(id)
                } else {
                    None
                }
            });
            if let Some(other) = other {
                return (si, pi, current, other);
            }
        }
    }
    panic!("다른 문단 스타일을 고를 본문 문단이 없다");
}

fn para_style_id(path: &Path, section: usize, para: usize) -> usize {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    doc.document().sections[section].paragraphs[para].style_id as usize
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-style-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn apply_style_is_visible() {
    let src = sample();
    let (section, para, current, style) = first_body_para_and_other_style(&src);
    assert_ne!(current, style);
    let out = temp("out");
    let sec_s = section.to_string();
    let para_s = para.to_string();
    let style_s = style.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-style",
            src.as_str(),
            "--section",
            &sec_s,
            "--para",
            &para_s,
            "--style",
            &style_s,
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
    assert_eq!(v["ctrl"], style);
    assert_eq!(
        para_style_id(&out, section, para),
        style,
        "문단 스타일이 저장본에 없다"
    );
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_json_has_fields_and_no_file() {
    let src = sample();
    let (section, para, _current, style) = first_body_para_and_other_style(&src);
    let out = temp("dry");
    let sec_s = section.to_string();
    let para_s = para.to_string();
    let style_s = style.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-style",
            src.as_str(),
            "--section",
            &sec_s,
            "--para",
            &para_s,
            "--style",
            &style_s,
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
    assert_eq!(v["ctrl"], style);
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-style",
            src.as_str(),
            "--style",
            "0",
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
        .any(|t| t["name"] == "hwp_apply_style"));
}
