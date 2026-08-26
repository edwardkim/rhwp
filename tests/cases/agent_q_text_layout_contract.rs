//! `rhwp-q-text-layout` CLI 계약. 실제 바이너리를 실행한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SAMPLE: &str = "samples/form-01.hwp";
const TOOL: &str = "rhwp-q-text-layout";

fn tool_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp-q-text-layout")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp-q-text-layout").to_string())
}

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn run(args: &[&str]) -> Output {
    Command::new(tool_bin())
        .args(args)
        .output()
        .expect("rhwp-q-text-layout 실행 실패")
}

fn describe(args: &[&str], output: &Output) -> String {
    format!(
        "명령: {TOOL} {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn stdout_json(args: &[&str], output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout 이 순수 JSON 이 아닙니다 ({e}).\n{}",
            describe(args, output)
        )
    })
}

#[test]
fn json_envelope_on_form01_page0() {
    let path = sample(SAMPLE);
    let source = path.to_str().expect("utf-8 path");
    let args = ["--json", "--page", "0", source];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = stdout_json(&args, &output);
    assert!(v["schemaVersion"].is_string(), "{v}");
    assert_eq!(v["tool"], TOOL, "{v}");
    assert_eq!(v["command"], "text-layout", "{v}");
    assert_eq!(v["untrustedContent"], true, "{v}");
    assert!(v["untrustedFields"].is_array(), "{v}");
    assert_eq!(v["page"], 0, "{v}");
    assert_eq!(v["source"], source, "{v}");
    let runs = v["runs"].as_array().expect("runs array");
    assert_eq!(v["runCount"], runs.len(), "{v}");
    assert!(v["pageCount"].as_u64().unwrap() >= 1, "{v}");
    for run in runs {
        assert!(run
            .get("text")
            .and_then(serde_json::Value::as_str)
            .is_some());
        assert!(run.get("x").is_some());
        assert!(run.get("y").is_some());
        assert!(run.get("w").is_some());
        assert!(run.get("h").is_some());
    }
}

#[test]
fn unknown_flag_nope_is_usage() {
    let path = sample(SAMPLE);
    let source = path.to_str().expect("utf-8 path");
    let args = [source, "--page", "0", "--nope"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
}

#[test]
fn missing_page_is_usage() {
    let path = sample(SAMPLE);
    let source = path.to_str().expect("utf-8 path");
    let args = [source, "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
}

#[test]
fn out_of_range_page_is_runtime() {
    let path = sample(SAMPLE);
    let source = path.to_str().expect("utf-8 path");
    let args = ["--json", "--page", "99", source];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        describe(&args, &output)
    );
}

#[test]
fn source_never_calls_mutators() {
    let src = include_str!("../../src/bin/rhwp-q-text-layout.rs");
    for needle in [".apply_", ".insert_", ".delete_", ".set_"] {
        assert!(
            !src.contains(needle),
            "읽기 전용 CLI 가 {needle} 를 부르면 안 된다"
        );
    }
    assert!(src.contains("get_page_text_layout_native"));
    assert!(src.contains("from_bytes"));
}
