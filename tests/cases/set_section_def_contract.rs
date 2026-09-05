//! `edit set-section-def` 계약.
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

fn first_section(path: &str) -> usize {
    let bytes = std::fs::read(path).expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    doc.document()
        .sections
        .iter()
        .enumerate()
        .map(|(i, _)| i)
        .next()
        .expect("구역이 없다")
}

fn hide_header(path: &Path, section: usize) -> bool {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    let raw = doc.get_section_def_native(section).expect("section def");
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    v["hideHeader"].as_bool().expect("hideHeader")
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-secdef-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn hide_header_toggle_is_visible() {
    let src = sample();
    let section = first_section(&src);
    let current = hide_header(Path::new(&src), section);
    let target = !current;
    let props = format!(r#"{{"hideHeader":{target}}}"#);
    let out = temp("out");
    let sec_s = section.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-section-def",
            src.as_str(),
            "--section",
            &sec_s,
            "--props",
            &props,
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["section"], section);
    assert_eq!(
        hide_header(&out, section),
        target,
        "구역 hideHeader 가 저장본에 없다"
    );
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_json_has_fields_and_no_file() {
    let src = sample();
    let section = first_section(&src);
    let out = temp("dry");
    let sec_s = section.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-section-def",
            src.as_str(),
            "--section",
            &sec_s,
            "--props",
            r#"{"hideHeader":true}"#,
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
    assert_eq!(v["section"], section);
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-section-def",
            src.as_str(),
            "--props",
            r#"{"hideHeader":true}"#,
            "--nope",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
}
