//! [#4993] `edit insert-page-break` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use rhwp::wasm_api::HwpDocument;

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/field-01.hwp")
        .to_string_lossy()
        .into_owned()
}
fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-inspb-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin()).args(args).output().expect("rhwp")
}
fn para_count(path: &Path) -> usize {
    let bytes = std::fs::read(path).unwrap();
    HwpDocument::from_bytes(&bytes).unwrap().document().sections[0]
        .paragraphs
        .len()
}

/// 문단 중간 오프셋은 문단을 가르고 새 문단에 쪽 나눔을 건다.
/// (offset 0 = 문단 시작은 가르지 않는다 — `issue_7218_page_break_at_paragraph_start`.)
#[test]
fn insert_page_break_splits_paragraph() {
    let src = sample();
    let doc = HwpDocument::from_bytes(&std::fs::read(&src).unwrap()).unwrap();
    let (para, _) = doc.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .find(|(_, p)| p.text.chars().count() >= 2)
        .expect("글자 2개 이상인 문단");
    let before = para_count(Path::new(&src));
    let out = temp("out");
    let para_arg = para.to_string();
    let args = [
        "edit",
        "insert-page-break",
        src.as_str(),
        "--para",
        para_arg.as_str(),
        "--offset",
        "1",
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert_eq!(para_count(&out), before + 1);
    let envelope: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["paragraphDelta"], 1);
    assert_eq!(envelope["pageBreakParagraph"], para + 1);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = sample();
    let out = temp("dry");
    let args = [
        "edit",
        "insert-page-break",
        src.as_str(),
        "-o",
        out.to_str().unwrap(),
        "--dry-run",
        "--json",
    ];
    let output = run(&args);
    assert_eq!(output.status.code(), Some(0));
    assert!(!out.exists());
}

#[test]
fn mcp_declared() {
    let output = run(&["capabilities", "--mcp"]);
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(v["tools"]
        .as_array()
        .unwrap()
        .iter()
        .any(|t| t["name"] == "hwp_insert_page_break"));
}
