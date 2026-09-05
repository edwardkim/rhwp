//! `rhwp-q-markdown` CLI 계약 — JSON 봉투와 종료 코드.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::process::{Command, Output};

fn bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp-q-markdown")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp-q-markdown").to_string())
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
        .expect("rhwp-q-markdown 실행 실패")
}

#[test]
fn json_envelope_on_form01_page0() {
    let src = sample();
    let out = run(&["--json", "--page", "0", src.as_str()]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("stdout JSON");
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert_eq!(v["tool"], "rhwp-q-markdown", "{v}");
    assert_eq!(v["command"], "markdown", "{v}");
    assert_eq!(v["source"], src, "{v}");
    assert_eq!(v["page"], 0, "{v}");
    let markdown = v["markdown"].as_str().expect("markdown string");
    let char_count = v["charCount"].as_u64().expect("charCount");
    assert_eq!(char_count as usize, markdown.chars().count());
    assert!(v["pageCount"].as_u64().unwrap() >= 1, "{v}");
}

#[test]
fn unknown_flag_is_usage() {
    let src = sample();
    let out = run(&["--nope", "--page", "0", src.as_str()]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
}

#[test]
fn missing_page_is_usage() {
    let out = run(&["--json", sample().as_str()]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
}

#[test]
fn missing_path_is_usage() {
    let out = run(&["--json", "--page", "0"]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
}

#[test]
fn out_of_range_page_is_runtime() {
    let src = sample();
    let out = run(&["--json", "--page", "99", src.as_str()]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
}

#[test]
fn source_never_calls_mutators() {
    let src = include_str!("../../src/bin/rhwp-q-markdown.rs");
    for needle in [".apply_", ".insert_", ".delete_", ".set_"] {
        assert!(
            !src.contains(needle),
            "읽기 전용 CLI 가 {needle} 를 부르면 안 된다"
        );
    }
    assert!(src.contains("extract_page_markdown_native"));
    assert!(src.contains("from_bytes"));
}
