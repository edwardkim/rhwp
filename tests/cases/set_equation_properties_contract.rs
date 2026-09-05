//! `edit set-equation-properties` 계약.
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
        "rhwp-eqprop-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn fixture_with_equation() -> (PathBuf, usize, usize, usize) {
    let bytes = std::fs::read(sample()).unwrap();
    let mut doc = HwpDocument::from_bytes(&bytes).unwrap();
    let (section, para) = {
        let mut found = None;
        for (si, sec) in doc.document().sections.iter().enumerate() {
            for (pi, p) in sec.paragraphs.iter().enumerate() {
                if p.text.chars().count() >= 2 {
                    found = Some((si, pi));
                    break;
                }
            }
            if found.is_some() {
                break;
            }
        }
        found.expect("본문 문단")
    };
    let raw = doc
        .insert_equation_native(section, para, 0, "a+b", 1000, 0)
        .expect("수식 삽입");
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let ctrl = v["controlIdx"].as_u64().unwrap_or_else(|| {
        doc.document().sections[section].paragraphs[para]
            .controls
            .iter()
            .rposition(|c| matches!(c, Control::Equation(_)))
            .expect("수식 컨트롤") as u64
    }) as usize;
    let out = temp("fx");
    let exported = doc.export_hwp().expect("export");
    std::fs::write(&out, exported).unwrap();
    (out, section, para, ctrl)
}

fn eq_script(path: &Path, section: usize, para: usize, ctrl: usize) -> String {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    match &doc.document().sections[section].paragraphs[para].controls[ctrl] {
        Control::Equation(eq) => eq.script.clone(),
        _ => panic!("수식 컨트롤"),
    }
}

#[test]
fn set_script_is_visible() {
    let (inserted, section, para, ctrl) = fixture_with_equation();
    let out = temp("out");
    let sec_s = section.to_string();
    let para_s = para.to_string();
    let ctrl_s = ctrl.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-equation-properties",
            inserted.to_str().unwrap(),
            "--section",
            &sec_s,
            "--para",
            &para_s,
            "--ctrl",
            &ctrl_s,
            "--props",
            r#"{"script":"x^2"}"#,
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
    assert_eq!(
        eq_script(&out, section, para, ctrl),
        "x^2",
        "수식 스크립트가 저장본에 없다"
    );
    let _ = std::fs::remove_file(&inserted);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_json_has_fields_and_no_file() {
    let (inserted, section, para, ctrl) = fixture_with_equation();
    let out = temp("dry");
    let sec_s = section.to_string();
    let para_s = para.to_string();
    let ctrl_s = ctrl.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-equation-properties",
            inserted.to_str().unwrap(),
            "--section",
            &sec_s,
            "--para",
            &para_s,
            "--ctrl",
            &ctrl_s,
            "--props",
            r#"{"script":"x^2"}"#,
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
    assert_eq!(v["ctrl"], ctrl);
    let _ = std::fs::remove_file(&inserted);
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-equation-properties",
            src.as_str(),
            "--section",
            "0",
            "--para",
            "0",
            "--ctrl",
            "0",
            "--props",
            r#"{"script":"x"}"#,
            "--nope",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
}
