//! `rhwp-q-object-cycle` CLI 계약. 실제 바이너리를 실행한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SAMPLE: &str = "samples/form-01.hwp";
const TOOL: &str = "rhwp-q-object-cycle";

fn tool_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp-q-object-cycle")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp-q-object-cycle").to_string())
}

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn run(args: &[&str]) -> Output {
    Command::new(tool_bin())
        .args(args)
        .output()
        .expect("rhwp-q-object-cycle 실행 실패")
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
fn json_envelope_on_form01() {
    let path = sample(SAMPLE);
    let source = path.to_str().expect("utf-8 path");
    let args = ["--json", source];
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
    assert_eq!(v["command"], "object-cycle", "{v}");
    assert_eq!(v["untrustedContent"], true, "{v}");
    assert!(v["untrustedFields"].is_array(), "{v}");
    assert_eq!(v["source"], source, "{v}");
    let cycle = v["cycle"].as_array().expect("cycle 배열");
    assert_eq!(v["cycleCount"], cycle.len(), "{v}");
    assert!(cycle.iter().all(|item| {
        item.get("para")
            .and_then(serde_json::Value::as_u64)
            .is_some()
            && item
                .get("controlIndex")
                .and_then(serde_json::Value::as_u64)
                .is_some()
            && item
                .get("page")
                .and_then(serde_json::Value::as_u64)
                .is_some()
            && item.get("z").and_then(serde_json::Value::as_i64).is_some()
    }));
}

#[test]
fn unknown_flag_nope_is_usage() {
    let output = run(&["--nope"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&["--nope"], &output)
    );
}

#[test]
fn missing_path_is_usage() {
    let output = run(&["--json"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&["--json"], &output)
    );
}

#[test]
fn extra_file_is_usage() {
    let path = sample(SAMPLE);
    let source = path.to_str().expect("utf-8 path");
    let args = [source, source];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
}

#[test]
fn missing_file_is_runtime() {
    let path = sample("samples/__no_such_q_object_cycle__.hwp");
    let source = path.to_str().expect("utf-8 path");
    let args = [source];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        describe(&args, &output)
    );
}

#[test]
fn unparseable_file_is_runtime() {
    let path = sample("README.md");
    let source = path.to_str().expect("utf-8 path");
    let args = [source];
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
    let src = include_str!("../../src/bin/rhwp-q-object-cycle.rs");
    for needle in [".apply_", ".insert_", ".delete_", ".set_"] {
        assert!(
            !src.contains(needle),
            "읽기 전용 CLI 가 {needle} 를 부르면 안 된다"
        );
    }
    assert!(src.contains("object_cycle_json"));
    assert!(src.contains("from_bytes"));
}
