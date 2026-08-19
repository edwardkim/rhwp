//! `rhwp-q-para-shape` CLI 계약. 실제 바이너리를 실행한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SAMPLE: &str = "samples/form-01.hwp";
const TOOL: &str = "rhwp-q-para-shape";

fn tool_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp-q-para-shape")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp-q-para-shape").to_string())
}

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn run(args: &[&str]) -> Output {
    Command::new(tool_bin())
        .args(args)
        .output()
        .expect("rhwp-q-para-shape 실행 실패")
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
fn json_envelope_on_form01_list0_para0() {
    let path = sample(SAMPLE);
    let source = path.to_str().expect("utf-8 path");
    let args = ["--json", "--list", "0", "--para", "0", source];
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
    assert_eq!(v["command"], "para-shape", "{v}");
    assert_eq!(v["untrustedContent"], true, "{v}");
    assert!(v["untrustedFields"].is_array(), "{v}");
    assert_eq!(v["list"], 0, "{v}");
    assert_eq!(v["para"], 0, "{v}");
    assert_eq!(v["source"], source, "{v}");
    let shape = v["paraShape"].as_object().expect("paraShape object");
    assert!(
        !shape.is_empty(),
        "form-01 0/0 ParaShape 가 비면 안 된다: {shape:?}"
    );
    assert!(shape
        .get("AlignType")
        .and_then(serde_json::Value::as_i64)
        .is_some());
    assert!(shape.contains_key("LineSpacing"));
    assert!(shape.contains_key("LeftMargin"));
}

#[test]
fn unknown_flag_nope_is_usage() {
    let path = sample(SAMPLE);
    let source = path.to_str().expect("utf-8 path");
    let args = [source, "--list", "0", "--para", "0", "--nope"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
}

#[test]
fn missing_list_is_usage() {
    let path = sample(SAMPLE);
    let source = path.to_str().expect("utf-8 path");
    let args = [source, "--para", "0", "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
}

#[test]
fn missing_cursor_is_empty_success() {
    let path = sample(SAMPLE);
    let source = path.to_str().expect("utf-8 path");
    let args = ["--json", "--list", "0", "--para", "99", source];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = stdout_json(&args, &output);
    let shape = v["paraShape"].as_object().expect("paraShape object");
    assert!(shape.is_empty(), "없는 자리의 셋은 빈 객체다: {shape:?}");
}

#[test]
fn source_never_calls_mutators() {
    let src = include_str!("../../src/bin/rhwp-q-para-shape.rs");
    for needle in [".apply_", ".insert_", ".delete_", ".set_"] {
        assert!(
            !src.contains(needle),
            "읽기 전용 CLI 가 {needle} 를 부르면 안 된다"
        );
    }
    assert!(src.contains("para_shape_set_json"));
    assert!(src.contains("from_bytes"));
}
