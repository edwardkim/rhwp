//! rhwp-q-cursor-rect CLI 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp-q-cursor-rect")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp-q-cursor-rect").to_string())
}

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/form-01.hwp")
}

fn run(args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .output()
        .expect("rhwp-q-cursor-rect 실행 실패")
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
    let out = run(&[
        source,
        "--section",
        "0",
        "--para",
        "0",
        "--offset",
        "0",
        "--json",
    ]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_eq!(v["tool"], "rhwp-q-cursor-rect");
    assert_eq!(v["command"], "cursor-rect");
    assert_eq!(
        v["schemaVersion"],
        rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION
    );
    assert_eq!(v["untrustedContent"], true);
    assert_eq!(v["untrustedFields"], serde_json::json!(["source", "rect"]));
    assert_eq!(v["section"], 0);
    assert_eq!(v["para"], 0);
    assert_eq!(v["offset"], 0);
    assert_eq!(v["source"], source);
    let rect = v["rect"].as_object().expect("rect object");
    assert!(rect
        .get("pageIndex")
        .and_then(serde_json::Value::as_u64)
        .is_some());
    assert!(rect.get("x").and_then(serde_json::Value::as_f64).is_some());
    assert!(rect.get("y").and_then(serde_json::Value::as_f64).is_some());
    assert!(rect
        .get("height")
        .and_then(serde_json::Value::as_f64)
        .is_some());
}

#[test]
fn unknown_nope_is_usage() {
    let path = sample();
    let source = path.to_str().expect("utf-8 path");
    let out = run(&[
        source,
        "--section",
        "0",
        "--para",
        "0",
        "--offset",
        "0",
        "--nope",
    ]);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
}

#[test]
fn missing_offset_is_usage() {
    let path = sample();
    let source = path.to_str().expect("utf-8 path");
    let out = run(&[source, "--section", "0", "--para", "0"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
}

#[test]
fn help_exits_ok() {
    let out = run(&["--help"]);
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("rhwp-q-cursor-rect"));
    assert!(text.contains("--json"));
}

#[test]
fn source_never_calls_mutators() {
    let src = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/bin/rhwp-q-cursor-rect.rs"
    ));
    for needle in [".apply_", ".insert_", ".delete_", ".set_"] {
        assert!(
            !src.contains(needle),
            "읽기 전용 CLI 가 {needle} 를 부르면 안 된다"
        );
    }
    assert!(!src.contains("#[cfg(test)]"));
    assert!(src.contains("get_cursor_rect_native"));
    assert!(src.contains("from_bytes"));
}
