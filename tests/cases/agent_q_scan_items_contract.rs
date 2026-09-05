//! `rhwp-q-scan-items` CLI 계약 — JSON 봉투와 종료 코드.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::process::{Command, Output};

fn bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp-q-scan-items")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp-q-scan-items").to_string())
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
        .expect("rhwp-q-scan-items 실행 실패")
}

#[test]
fn json_envelope_on_form01_with_limit() {
    let src = sample();
    let out = run(&["--json", "--limit", "20", src.as_str()]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("stdout JSON");
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert_eq!(v["tool"], "rhwp-q-scan-items", "{v}");
    assert_eq!(v["command"], "scan-items", "{v}");
    assert_eq!(v["source"], src, "{v}");
    assert_eq!(v["limit"], 20, "{v}");
    assert_eq!(v["itemCount"], 20, "{v}");
    assert_eq!(v["truncated"], true, "{v}");
    assert!(
        v["totalCount"].as_u64().is_some_and(|n| n > 20),
        "제한을 검증하려면 항목이 20개를 넘어야 한다: {v}"
    );
    assert_eq!(v["items"].as_array().map(Vec::len), Some(20), "{v}");
    let first = &v["items"][0];
    assert!(first
        .get("state")
        .and_then(serde_json::Value::as_u64)
        .is_some());
    assert!(first
        .get("kind")
        .and_then(serde_json::Value::as_u64)
        .is_some());
    assert!(first
        .get("text")
        .and_then(serde_json::Value::as_str)
        .is_some());
}

#[test]
fn unknown_flag_is_usage() {
    let out = run(&["--nope", sample().as_str()]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
}

#[test]
fn missing_path_is_usage() {
    let out = run(&["--json", "--limit", "20"]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
}

#[test]
fn limit_zero_is_usage() {
    let src = sample();
    let out = run(&[src.as_str(), "--limit", "0"]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
}

#[test]
fn source_never_calls_mutators() {
    let src = include_str!("../../src/bin/rhwp-q-scan-items.rs");
    for needle in [".apply_", ".insert_", ".delete_", ".set_"] {
        assert!(
            !src.contains(needle),
            "읽기 전용 CLI 가 {needle} 를 부르면 안 된다"
        );
    }
    assert!(src.contains("scan_items_json"));
    assert!(src.contains("from_bytes"));
}
