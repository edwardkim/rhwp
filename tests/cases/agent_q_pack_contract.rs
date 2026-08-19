//! rhwp-q-pack 계약. src 에 #[cfg(test)] 없음.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SAMPLE: &str = "samples/form-01.hwp";

fn bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp-q-pack")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp-q-pack").to_string())
}

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn run(args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .output()
        .expect("rhwp-q-pack")
}

#[test]
fn help_lists_pack_commands() {
    let out = run(&["--help"]);
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    for name in [
        "forms-all",
        "shapes-all",
        "char-overlaps",
        "headers-list",
        "footers-list",
        "footnotes-list",
        "endnotes-list",
        "new-numbers",
        "page-num-ctrls",
        "page-number-pos",
        "column-defs",
        "unknown-ctrls",
    ] {
        assert!(text.contains(name), "{name} missing from help:\n{text}");
    }
}

#[test]
fn unknown_command_is_usage() {
    let out = run(&["not-a-command"]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn forms_all_json() {
    let p = sample();
    let src = p.to_str().unwrap();
    let args = ["forms-all", "--json", src];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{:?}", out);
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["tool"], "rhwp-q-pack");
    assert_eq!(v["command"], "forms-all");
    assert!(v["count"].is_number());
}

#[test]
fn volume_probe_slot0() {
    let p = sample();
    let src = p.to_str().unwrap();
    let args = ["volume-probe", "--json", "--slot", "0", src];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{:?}", out);
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["command"], "volume-probe");
    assert!(v["acc"].is_number());
}
