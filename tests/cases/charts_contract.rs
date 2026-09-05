//! [#5051] `charts` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::process::Command;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue2006/1790387_prep_final_report.hwpx")
        .to_string_lossy()
        .into_owned()
}

#[test]
fn charts_json_has_array() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args(["charts", src.as_str(), "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{:?}", out);
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let arr = v["charts"].as_array().expect("charts 배열");
    assert_eq!(v["count"].as_u64().unwrap(), arr.len() as u64);
    assert!(!arr.is_empty(), "차트 샘플에 차트가 있어야 한다");
    assert!(arr[0]["index"].as_u64().is_some());
    assert!(arr[0]["section"].as_u64().is_some());
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args(["charts", src.as_str(), "--nope"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
}
