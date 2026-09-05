//! rhwp-q-page-images CLI 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp-q-page-images")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp-q-page-images").to_string())
}

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/form-01.hwp")
}

fn run(args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .output()
        .expect("rhwp-q-page-images 실행 실패")
}

fn stdout_json(output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout JSON 아님 ({e})\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

#[test]
fn form01_json_envelope() {
    let path = sample();
    let source = path.to_str().expect("utf-8 path");
    let out = run(&[source, "--page", "0", "--json"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_eq!(v["tool"], "rhwp-q-page-images");
    assert_eq!(v["command"], "page-images");
    assert_eq!(
        v["schemaVersion"],
        rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION
    );
    assert_eq!(v["page"], 0);
    assert_eq!(v["source"], source);
    let keys = v["keys"].as_array().expect("keys array");
    assert_eq!(v["keyCount"], keys.len());
    assert!(v["cacheable"].as_bool().is_some());
    assert!(v["pageCount"].as_u64().is_some());
}

#[test]
fn unknown_nope_is_usage() {
    let path = sample();
    let source = path.to_str().expect("utf-8 path");
    let out = run(&[source, "--page", "0", "--nope"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
}

#[test]
fn missing_page_is_usage() {
    let path = sample();
    let source = path.to_str().expect("utf-8 path");
    let out = run(&[source]);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
}

#[test]
fn help_exits_ok() {
    let out = run(&["--help"]);
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("rhwp-q-page-images"));
    assert!(text.contains("--json"));
}

#[test]
fn source_never_calls_mutators() {
    let src = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/bin/rhwp-q-page-images.rs"
    ));
    for needle in [".apply_", ".insert_", ".delete_", ".set_"] {
        assert!(
            !src.contains(needle),
            "읽기 전용 CLI 가 {needle} 를 부르면 안 된다"
        );
    }
    assert!(!src.contains("#[cfg(test)]"));
    assert!(src.contains("get_page_source_image_keys_native"));
    assert!(src.contains("from_bytes"));
}
