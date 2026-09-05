//! rhwp-q-page-def CLI 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp-q-page-def")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp-q-page-def").to_string())
}

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/form-01.hwp")
}

fn run(args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .output()
        .expect("rhwp-q-page-def 실행 실패")
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
    let out = run(&[source, "--section", "0", "--json"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_eq!(v["tool"], "rhwp-q-page-def");
    assert_eq!(v["command"], "page-def");
    assert_eq!(
        v["schemaVersion"],
        rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION
    );
    assert_eq!(v["section"], 0);
    assert_eq!(v["source"], source);
    assert_eq!(v["units"], "HWPUNIT");
    assert!(v["width"].as_u64().is_some(), "width 가 없다: {v}");
    assert!(v["height"].as_u64().is_some(), "height 가 없다: {v}");
    assert!(v["marginLeft"].as_u64().is_some(), "{v}");
    assert!(v["marginRight"].as_u64().is_some(), "{v}");
    assert!(v["marginTop"].as_u64().is_some(), "{v}");
    assert!(v["marginBottom"].as_u64().is_some(), "{v}");
}

#[test]
fn unknown_nope_is_usage() {
    let path = sample();
    let source = path.to_str().expect("utf-8 path");
    let out = run(&[source, "--section", "0", "--nope"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
}

#[test]
fn missing_section_is_usage() {
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
    assert!(text.contains("rhwp-q-page-def"));
    assert!(text.contains("--json"));
}

#[test]
fn source_never_calls_mutators() {
    let src = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/bin/rhwp-q-page-def.rs"
    ));
    for needle in [".apply_", ".insert_", ".delete_", ".set_"] {
        assert!(
            !src.contains(needle),
            "읽기 전용 CLI 가 {needle} 를 부르면 안 된다"
        );
    }
    assert!(!src.contains("#[cfg(test)]"));
    assert!(src.contains("get_page_def_native"));
    assert!(src.contains("from_bytes"));
}
