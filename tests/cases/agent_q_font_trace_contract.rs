//! rhwp-q-font-trace CLI 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::process::{Command, Output};

fn bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp-q-font-trace")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp-q-font-trace").to_string())
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
        .expect("rhwp-q-font-trace 실행 실패")
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
    assert_eq!(v["tool"], "rhwp-q-font-trace", "{v}");
    assert_eq!(v["command"], "font-trace", "{v}");
}

#[test]
fn form_sample_json_emits_envelope() {
    let src = sample();
    let out = run(&[src.as_str(), "--page", "0", "--json"]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let v = stdout_json(&out);
    assert_envelope(&v);
    assert_eq!(v["page"], 0, "{v}");
    assert!(v["trace"].is_object(), "{v}");
    assert_eq!(v["trace"]["scope"]["pageIndex"], 0, "{v}");
    assert!(v["trace"]["status"].as_str().is_some(), "{v}");
    assert!(v["trace"]["records"].is_array(), "{v}");
}

#[test]
fn bounded_max_characters_is_forwarded_to_the_query() {
    let src = sample();
    let out = run(&[
        src.as_str(),
        "--page",
        "0",
        "--max-characters",
        "1",
        "--json",
    ]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let v = stdout_json(&out);
    assert_envelope(&v);
    assert_eq!(v["trace"]["scope"]["requestedLimits"]["maxCharacters"], 1);
    assert_eq!(v["trace"]["scope"]["appliedLimits"]["maxCharacters"], 1);
    assert_eq!(v["trace"]["counts"]["recordsEmitted"], 1);
}

#[test]
fn max_characters_outside_the_core_bound_is_usage() {
    let src = sample();
    for value in ["0", "4097", "nope"] {
        let out = run(&[
            src.as_str(),
            "--page",
            "0",
            "--max-characters",
            value,
            "--json",
        ]);
        assert_eq!(out.status.code(), Some(2), "value={value} {out:?}");
    }
}

#[test]
fn unknown_flag_is_usage() {
    let src = sample();
    let out = run(&["--nope", src.as_str(), "--page", "0"]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
}

#[test]
fn missing_path_is_usage() {
    let out = run(&["--json", "--page", "0"]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
}

#[test]
fn missing_page_is_usage() {
    let src = sample();
    let out = run(&[src.as_str(), "--json"]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
}

#[test]
fn extra_path_is_usage() {
    let src = sample();
    let out = run(&[src.as_str(), "other.hwp", "--page", "0"]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
}

#[test]
fn page_out_of_range_is_runtime() {
    let src = sample();
    let out = run(&[src.as_str(), "--page", "9999", "--json"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
}
