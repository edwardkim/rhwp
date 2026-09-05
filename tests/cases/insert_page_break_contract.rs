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

#[test]
fn insert_page_break_splits_paragraph() {
    let src = sample();
    let before = para_count(Path::new(&src));
    let out = temp("out");
    let args = [
        "edit",
        "insert-page-break",
        src.as_str(),
        "--offset",
        "0",
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert!(para_count(&out) > before);
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
