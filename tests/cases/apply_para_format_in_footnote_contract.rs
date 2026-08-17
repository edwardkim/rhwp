//! `edit apply-para-format-in-footnote` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

use rhwp::model::control::Control;
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

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-pafmtfn-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn first_footnote(path: &Path) -> Option<(usize, usize, usize)> {
    let bytes = std::fs::read(path).ok()?;
    let doc = HwpDocument::from_bytes(&bytes).ok()?;
    for (si, sec) in doc.document().sections.iter().enumerate() {
        for (pi, p) in sec.paragraphs.iter().enumerate() {
            for (ci, c) in p.controls.iter().enumerate() {
                if matches!(c, Control::Footnote(_) | Control::Endnote(_)) {
                    return Some((si, pi, ci));
                }
            }
        }
    }
    None
}

fn insert_footnote(src: &str) -> PathBuf {
    let out = temp("ins");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "insert-footnote",
            src,
            "--section",
            "0",
            "--para",
            "0",
            "--offset",
            "0",
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    out
}

fn footnote_alignment(path: &Path, section: usize, para: usize, ctrl: usize) -> Alignment {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    let c = &doc.document().sections[section].paragraphs[para].controls[ctrl];
    let fn_para = match c {
        Control::Footnote(f) => &f.paragraphs[0],
        Control::Endnote(f) => &f.paragraphs[0],
        _ => panic!("각주 아님"),
    };
    doc.document().doc_info.para_shapes[fn_para.para_shape_id as usize].alignment
}

#[test]
fn apply_center_after_insert_and_reparse() {
    let src = sample();
    let with_fn = match first_footnote(Path::new(&src)) {
        Some(_) => PathBuf::from(&src),
        None => insert_footnote(&src),
    };
    let (section, para, ctrl) = first_footnote(&with_fn).expect("각주 컨트롤");
    let before = footnote_alignment(&with_fn, section, para, ctrl);
    assert_ne!(
        before,
        Alignment::Center,
        "각주가 이미 가운데라 판별이 안 된다"
    );
    let out = temp("out");
    let sec_s = section.to_string();
    let para_s = para.to_string();
    let ctrl_s = ctrl.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-para-format-in-footnote",
            with_fn.to_str().unwrap(),
            "--section",
            &sec_s,
            "--para",
            &para_s,
            "--ctrl",
            &ctrl_s,
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
    assert_eq!(v["ctrl"], ctrl);
    assert!(out.exists());
    assert!(HwpDocument::from_bytes(&std::fs::read(&out).unwrap()).is_ok());
    assert_eq!(
        footnote_alignment(&out, section, para, ctrl),
        Alignment::Center
    );
    if with_fn != Path::new(&src) {
        let _ = std::fs::remove_file(&with_fn);
    }
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = sample();
    let with_fn = match first_footnote(Path::new(&src)) {
        Some(_) => PathBuf::from(&src),
        None => insert_footnote(&src),
    };
    let (section, para, ctrl) = first_footnote(&with_fn).expect("각주 컨트롤");
    let out = temp("dry");
    let sec_s = section.to_string();
    let para_s = para.to_string();
    let ctrl_s = ctrl.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-para-format-in-footnote",
            with_fn.to_str().unwrap(),
            "--section",
            &sec_s,
            "--para",
            &para_s,
            "--ctrl",
            &ctrl_s,
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
    if with_fn != Path::new(&src) {
        let _ = std::fs::remove_file(&with_fn);
    }
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-para-format-in-footnote",
            src.as_str(),
            "--section",
            "0",
            "--para",
            "0",
            "--ctrl",
            "0",
            "--props",
            "{}",
            "--nope",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(out.stdout.len(), 0);
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
        .any(|t| t["name"] == "hwp_apply_para_format_in_footnote"));
}
