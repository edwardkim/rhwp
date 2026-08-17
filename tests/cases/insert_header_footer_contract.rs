//! [#5036] `edit insert-header-footer` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

use rhwp::model::control::Control;
use rhwp::model::header_footer::HeaderFooterApply;
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
        "rhwp-hf-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn footer_odd_count(path: &Path) -> usize {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    doc.document()
        .sections
        .iter()
        .flat_map(|s| s.paragraphs.iter())
        .flat_map(|p| p.controls.iter())
        .filter(|c| matches!(c, Control::Footer(f) if f.apply_to == HeaderFooterApply::Odd))
        .count()
}

#[test]
fn insert_footer_odd_appears() {
    let src = sample();
    let before = footer_odd_count(Path::new(&src));
    let out = temp("out");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "insert-header-footer",
            src.as_str(),
            "--footer",
            "--apply-to",
            "2",
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert_eq!(footer_odd_count(&out), before + 1);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = sample();
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "insert-header-footer",
            src.as_str(),
            "--footer",
            "--apply-to",
            "2",
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
fn mcp_declared() {
    let output = Command::new(rhwp_bin())
        .args(["capabilities", "--mcp"])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let tool = v["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "hwp_insert_header_footer")
        .expect("hwp_insert_header_footer 도구");
    let one_of = tool["inputSchema"]["oneOf"]
        .as_array()
        .expect("header/footer 배타 선택 oneOf");
    assert_eq!(one_of.len(), 2, "{tool}");
    assert_eq!(one_of[0]["required"], serde_json::json!(["header"]));
    assert_eq!(one_of[0]["properties"]["header"]["const"], true);
    assert_eq!(one_of[1]["required"], serde_json::json!(["footer"]));
    assert_eq!(one_of[1]["properties"]["footer"]["const"], true);
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "insert-header-footer",
            src.as_str(),
            "--footer",
            "--nope",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
}
