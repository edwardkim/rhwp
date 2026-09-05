//! `edit set-chart-data` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

use rhwp::wasm_api::HwpDocument;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue2006/1790387_prep_final_report.hwpx")
        .to_string_lossy()
        .into_owned()
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-chartdata-{tag}-{}-{}.hwpx",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn first_series_edits(path: &str) -> (String, String, String) {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    let raw = doc.get_chart_data_by_index_native(0).expect("차트 읽기");
    let mut data: serde_json::Value = serde_json::from_str(&raw).expect("JSON");
    let values = data["series"][0]["values"].as_array_mut().expect("values");
    let from = values[0].as_str().expect("값").to_string();
    let to = if from == "1" { "2" } else { "1" }.to_string();
    values[0] = serde_json::json!(to);
    let edits = serde_json::json!({
        "series": data["series"].as_array().unwrap().iter().map(|s| {
            serde_json::json!({
                "name": s["name"],
                "values": s["values"],
            })
        }).collect::<Vec<_>>(),
    });
    (edits.to_string(), from, to)
}

fn first_value(path: &Path) -> String {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    let raw = doc.get_chart_data_by_index_native(0).expect("차트 읽기");
    let data: serde_json::Value = serde_json::from_str(&raw).expect("JSON");
    data["series"][0]["values"][0]
        .as_str()
        .expect("값")
        .to_string()
}

#[test]
fn set_chart_data_writes_value() {
    let src = sample();
    let (edits, from, to) = first_series_edits(&src);
    assert_ne!(from, to);
    let out = temp("out");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-chart-data",
            src.as_str(),
            "--chart",
            "1",
            "--data",
            &edits,
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert_eq!(first_value(&out), to);
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["count"], 1);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = sample();
    let (edits, _, _) = first_series_edits(&src);
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-chart-data",
            src.as_str(),
            "--chart",
            "1",
            "--data",
            &edits,
            "-o",
            out.to_str().unwrap(),
            "--dry-run",
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert!(!out.exists());
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-chart-data",
            src.as_str(),
            "--chart",
            "1",
            "--data",
            "{}",
            "--nope",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
}
