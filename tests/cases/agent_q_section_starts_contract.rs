//! `rhwp-q-section-starts` CLI 계약 — JSON 봉투와 종료 코드.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::process::{Command, Output};

fn bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp-q-section-starts")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp-q-section-starts").to_string())
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
        .expect("rhwp-q-section-starts 실행 실패")
}

#[test]
fn json_envelope_on_form01() {
    let src = sample();
    let out = run(&["--json", src.as_str()]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("stdout JSON");
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert_eq!(v["tool"], "rhwp-q-section-starts", "{v}");
    assert_eq!(v["command"], "section-starts", "{v}");
    assert_eq!(v["untrustedContent"], true, "{v}");
    assert_eq!(
        v["untrustedFields"],
        serde_json::json!(["source", "starts"]),
        "{v}"
    );
    assert_eq!(v["source"], src, "{v}");
    let starts = v["starts"]
        .as_array()
        .unwrap_or_else(|| panic!("starts 는 배열이어야 한다: {v}"));
    assert_eq!(v["startCount"], starts.len(), "{v}");
    assert!(
        !starts.is_empty(),
        "구역이 있는 문서는 시작 문단이 하나 이상이다: {v}"
    );
    assert!(
        starts.iter().all(|n| n.is_u64()),
        "starts 원소는 본문 문단 번호다: {v}"
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
        .join("samples/__no_such_q_section_starts__.hwp")
        .to_string_lossy()
        .into_owned();
    let out = run(&[missing.as_str()]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
}

#[test]
fn source_never_calls_mutators() {
    let src = include_str!("../../src/bin/rhwp-q-section-starts.rs");
    for needle in [".apply_", ".insert_", ".delete_", ".set_"] {
        assert!(
            !src.contains(needle),
            "읽기 전용 CLI 가 {needle} 를 부르면 안 된다"
        );
    }
    assert!(src.contains("section_starts_json"));
    assert!(src.contains("from_bytes"));
}
