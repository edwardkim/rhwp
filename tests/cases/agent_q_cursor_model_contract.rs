//! `rhwp-q-cursor-model` CLI 계약 — JSON 봉투와 종료 코드.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::process::{Command, Output};

fn bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp-q-cursor-model")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp-q-cursor-model").to_string())
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
        .expect("rhwp-q-cursor-model 실행 실패")
}

#[test]
fn json_envelope_on_form01() {
    let src = sample();
    let out = run(&["--json", src.as_str()]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("stdout JSON");
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert_eq!(v["tool"], "rhwp-q-cursor-model", "{v}");
    assert_eq!(v["command"], "cursor-model", "{v}");
    assert_eq!(v["untrustedContent"], true, "{v}");
    assert_eq!(
        v["untrustedFields"],
        serde_json::json!(["source", "root", "lists"]),
        "{v}"
    );
    assert_eq!(v["source"], src, "{v}");
    assert_eq!(v["listCount"], 2, "{v}");
    assert!(v["root"].is_object(), "{v}");
    assert_eq!(v["root"]["paraCount"], 13, "{v}");
    assert_eq!(
        v["lists"].as_array().map(Vec::len),
        Some(0),
        "form-01 은 본문 리스트만 있다: {v}"
    );
}

#[test]
fn unknown_flag_is_usage() {
    let out = run(&["--nope", sample().as_str()]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
}

#[test]
fn missing_path_is_usage() {
    let out = run(&["--json"]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
}

#[test]
fn missing_file_is_runtime() {
    let missing = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/__no_such_q_cursor_model__.hwp")
        .to_string_lossy()
        .into_owned();
    let out = run(&[missing.as_str()]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
}

#[test]
fn source_never_calls_mutators() {
    let src = include_str!("../../src/bin/rhwp-q-cursor-model.rs");
    for needle in [".apply_", ".insert_", ".delete_", ".set_"] {
        assert!(
            !src.contains(needle),
            "읽기 전용 CLI 가 {needle} 를 부르면 안 된다"
        );
    }
    assert!(src.contains("get_cursor_model_json"));
    assert!(src.contains("from_bytes"));
}
