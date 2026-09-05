//! [#4992] `edit insert-paragraph` 계약.
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
        "rhwp-inspara-{tag}-{}-{}.hwp",
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
fn insert_paragraph_increases_count() {
    let src = sample();
    let before = para_count(Path::new(&src));
    let out = temp("out");
    let args = [
        "edit",
        "insert-paragraph",
        src.as_str(),
        "--section",
        "0",
        "--para",
        "0",
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["paragraph"], 0);
    assert_eq!(para_count(&out), before + 1);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_writes_nothing() {
    let src = sample();
    let out = temp("dry");
    let args = [
        "edit",
        "insert-paragraph",
        src.as_str(),
        "-o",
        out.to_str().unwrap(),
        "--dry-run",
        "--json",
    ];
    let output = run(&args);
    assert_eq!(output.status.code(), Some(0));
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["dryRun"], true);
    assert!(!out.exists());
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = sample();
    let args = ["edit", "insert-paragraph", src.as_str(), "--nope"];
    let output = run(&args);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
}
