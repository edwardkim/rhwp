//! rhwp-q-page-caret CLI 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::process::{Command, Output};

fn bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp-q-page-caret")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp-q-page-caret").to_string())
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
        .expect("rhwp-q-page-caret 실행 실패")
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

fn assert_envelope(v: &serde_json::Value) {
    assert!(v["schemaVersion"].as_str().is_some(), "{v}");
    assert_eq!(v["tool"], "rhwp-q-page-caret", "{v}");
    assert_eq!(v["command"], "page-caret", "{v}");
}

#[test]
fn form_sample_json_emits_envelope() {
    let src = sample();
    let out = run(&[src.as_str(), "--json"]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let v = stdout_json(&out);
    assert_envelope(&v);
    let pages = v["pages"].as_array().expect("pages");
    assert!(!pages.is_empty(), "{v}");
    assert_eq!(v["pageCount"], pages.len(), "{v}");
    assert!(pages[0]["list"].as_u64().is_some(), "{v}");
    assert!(pages[0]["para"].as_u64().is_some(), "{v}");
    assert!(pages[0]["pos"].as_u64().is_some(), "{v}");
}

#[test]
fn flags_may_surround_path() {
    let src = sample();
    let out = run(&["--json", src.as_str()]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let v = stdout_json(&out);
    assert_envelope(&v);
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
    let out = run(&[src.as_str(), "other.hwp"]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
}

#[test]
fn missing_file_on_disk_is_runtime() {
    let missing =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/this-file-does-not-exist.hwp");
    let out = run(&[missing.to_str().expect("utf-8"), "--json"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
}
