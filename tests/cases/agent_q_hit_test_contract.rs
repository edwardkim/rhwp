//! `rhwp-q-hit-test` CLI 계약. 실제 바이너리를 실행한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SAMPLE: &str = "samples/form-01.hwp";
const TOOL: &str = "rhwp-q-hit-test";

fn tool_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp-q-hit-test")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp-q-hit-test").to_string())
}

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn run(args: &[&str]) -> Output {
    Command::new(tool_bin())
        .args(args)
        .output()
        .expect("rhwp-q-hit-test 실행 실패")
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
fn json_envelope_on_form01_page0_at_120_120() {
    let path = sample(SAMPLE);
    let source = path.to_str().expect("utf-8 path");
    let args = ["--json", "--page", "0", "--x", "120", "--y", "120", source];
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
    assert_eq!(v["command"], "hit-test", "{v}");
    assert_eq!(v["untrustedContent"], true, "{v}");
    let fields = v["untrustedFields"].as_array().expect("untrustedFields");
    assert!(fields.iter().any(|f| f == "source"), "{v}");
    assert!(fields.iter().any(|f| f == "hit"), "{v}");
    assert_eq!(v["page"], 0, "{v}");
    assert_eq!(v["x"], 120.0, "{v}");
    assert_eq!(v["y"], 120.0, "{v}");
    let hit = v["hit"].as_object().expect("hit object");
    assert!(hit.contains_key("sectionIndex"), "hit={hit:?}");
    assert!(hit.contains_key("paragraphIndex"), "hit={hit:?}");
    assert!(hit.contains_key("charOffset"), "hit={hit:?}");
    assert!(hit.contains_key("cursorRect"), "hit={hit:?}");
    assert_eq!(hit["cursorRect"]["pageIndex"], 0);
}

#[test]
fn unknown_flag_nope_is_usage() {
    let path = sample(SAMPLE);
    let source = path.to_str().expect("utf-8 path");
    let args = [source, "--page", "0", "--x", "120", "--y", "120", "--nope"];
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
    let args = [source, "--x", "120", "--y", "120", "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
}

#[test]
fn missing_xy_is_usage() {
    let path = sample(SAMPLE);
    let source = path.to_str().expect("utf-8 path");
    let args = [source, "--page", "0", "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
}

#[test]
fn non_numeric_x_is_usage() {
    let path = sample(SAMPLE);
    let source = path.to_str().expect("utf-8 path");
    let args = [source, "--page", "0", "--x", "left", "--y", "120"];
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
    let args = ["--json", "--page", "99", "--x", "120", "--y", "120", source];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        describe(&args, &output)
    );
}
