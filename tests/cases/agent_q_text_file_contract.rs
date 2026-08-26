//! rhwp-q-text-file CLI 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::process::{Command, Output};

fn bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp-q-text-file")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp-q-text-file").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/form-01.hwp")
        .to_string_lossy()
        .into_owned()
}

fn run(args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .output()
        .expect("rhwp-q-text-file 실행 실패")
}

fn stdout_json(output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout JSON 아님 ({e})\nstdout:\n{}",
            String::from_utf8_lossy(&output.stdout)
        )
    })
}

fn assert_envelope(v: &serde_json::Value) {
    assert!(v["schemaVersion"].as_str().is_some(), "{v}");
    assert_eq!(v["tool"], "rhwp-q-text-file", "{v}");
    assert_eq!(v["command"], "text-file", "{v}");
    assert_eq!(v["untrustedContent"], true, "{v}");
    assert_eq!(
        v["untrustedFields"],
        serde_json::json!(["source", "text"]),
        "{v}"
    );
}

#[test]
fn form_sample_json_emits_envelope() {
    let src = sample();
    let out = run(&[src.as_str(), "--json"]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let v = stdout_json(&out);
    assert_envelope(&v);
    assert_eq!(v["format"], "UNICODE", "{v}");
    assert_eq!(v["cp949"], false, "{v}");
    let text = v["text"].as_str().expect("text");
    assert!(!text.is_empty(), "{v}");
    assert_eq!(v["charCount"], text.chars().count(), "{v}");
}

#[test]
fn form_sample_cp949_json_emits_envelope() {
    let src = sample();
    let out = run(&[src.as_str(), "--cp949", "--json"]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let v = stdout_json(&out);
    assert_envelope(&v);
    assert_eq!(v["format"], "TEXT", "{v}");
    assert_eq!(v["cp949"], true, "{v}");
    let text = v["text"].as_str().expect("text");
    assert!(!text.is_empty(), "{v}");
    assert_eq!(v["charCount"], text.chars().count(), "{v}");
}

#[test]
fn unknown_flag_is_usage() {
    let src = sample();
    let out = run(&["--nope", src.as_str()]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
}

#[test]
fn missing_path_is_usage() {
    let out = run(&["--json"]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
}

#[test]
fn extra_path_is_usage() {
    let src = sample();
    let out = run(&[src.as_str(), "other.hwp", "--json"]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
}

#[test]
fn missing_file_on_disk_is_runtime() {
    let missing =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/this-file-does-not-exist.hwp");
    let out = run(&[missing.to_str().expect("utf-8"), "--json"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
}
