//! rhwp-q-kit 50명령 계약. 실제 바이너리를 실행한다.
//! 제품 소스에 #[cfg(test)] 를 두지 않는다 (CONTRIBUTING / rust-unit-test-tiers).
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SAMPLE: &str = "samples/form-01.hwp";
const TOOL: &str = "rhwp-q-kit";

fn bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp-q-kit")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp-q-kit").to_string())
}

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn run(args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .output()
        .expect("rhwp-q-kit 실행 실패")
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

fn assert_envelope(v: &serde_json::Value, command: &str) {
    assert!(v["schemaVersion"].is_string(), "{v}");
    assert_eq!(v["tool"], TOOL, "{v}");
    assert_eq!(v["command"], command, "{v}");
    assert!(v["untrustedContent"].is_boolean(), "{v}");
    assert!(v["untrustedFields"].is_array(), "{v}");
}

#[test]
fn help_lists_fifty_commands() {
    let output = run(&["--help"]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&["--help"], &output)
    );
    let text = String::from_utf8_lossy(&output.stdout);
    for name in [
        "empty-doc",
        "hyperlinks",
        "equations",
        "pictures",
        "search-all",
        "page-hide",
        "layer-tree",
        "chart-csv",
        "field-by-id",
        "overlay-images",
    ] {
        assert!(text.contains(name), "help 에 {name} 없음:\n{text}");
    }
}

#[test]
fn unknown_command_is_usage() {
    let output = run(&["not-a-real-command"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&["not-a-real-command"], &output)
    );
}

#[test]
fn empty_doc_json_on_form01() {
    let path = sample(SAMPLE);
    let source = path.to_str().expect("utf-8");
    let args = ["empty-doc", "--json", source];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = stdout_json(&args, &output);
    assert_envelope(&v, "empty-doc");
    assert!(v["empty"].is_boolean(), "{v}");
}

#[test]
fn hyperlinks_json_on_form01() {
    let path = sample(SAMPLE);
    let source = path.to_str().expect("utf-8");
    let args = ["hyperlinks", "--json", source];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = stdout_json(&args, &output);
    assert_envelope(&v, "hyperlinks");
    assert!(v["items"].is_array(), "{v}");
}

#[test]
fn unknown_flag_on_pictures_is_usage() {
    let path = sample(SAMPLE);
    let source = path.to_str().expect("utf-8");
    let args = ["pictures", "--nope", source];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
}
