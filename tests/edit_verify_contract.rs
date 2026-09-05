//! [#3702] 편집 --verify 내장 — 저장 직후 자기검증 봉투 (#3630 P2).
//! 봉투-exit 정합: identical=false 면 봉투 출력 후 exit 3 (판정은 데이터).
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SAMPLE: &str = "samples/field-01.hwp";

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn temp_path(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-editverify-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ))
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .output()
        .expect("rhwp")
}

#[test]
fn fill_verify_reports_and_exit_matches() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out = temp_path("fill");
    let args = [
        "edit",
        "fill-fields",
        p.to_str().unwrap(),
        "--data",
        r#"{"회사명":"검증사"}"#,
        "-o",
        out.to_str().unwrap(),
        "--verify",
        "--json",
    ];
    let output = run(&args);
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("envelope");
    let identical = v["verify"]["identical"]
        .as_bool()
        .unwrap_or_else(|| panic!("verify.identical 필요: {v}"));
    let expected = if identical { 0 } else { 3 };
    assert_eq!(output.status.code(), Some(expected), "봉투-exit 모순: {v}");
    assert!(out.exists(), "판정과 무관하게 산출물은 남는다");
    let _ = std::fs::remove_file(&out);
}

#[test]
fn without_verify_field_is_null_and_exit_0() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out = temp_path("novf");
    let args = [
        "edit",
        "fill-fields",
        p.to_str().unwrap(),
        "--data",
        r#"{"회사명":"A"}"#,
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(output.status.code(), Some(0));
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(v["verify"].is_null(), "미요청 시 null: {v}");
    let _ = std::fs::remove_file(&out);
}

#[test]
fn set_cell_and_replace_accept_verify() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let table_sample = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx");
    if !table_sample.exists() {
        eprintln!("표 샘플 없음 — 건너뜀");
        return;
    }
    let _ = p;
    // 좌표는 하드코딩하지 않는다 — export-tables 재독으로 실존 최상위 표·셀을 고른다.
    let listing = run(&["export-tables", table_sample.to_str().unwrap(), "--json"]);
    assert_eq!(listing.status.code(), Some(0));
    let tables: serde_json::Value = serde_json::from_slice(&listing.stdout).expect("tables");
    let table = tables["tables"]
        .as_array()
        .expect("tables")
        .iter()
        .find(|t| t.get("containerPath").is_none())
        .expect("본문 최상위 표");
    let (ts, rs, cs) = (
        table["index"].as_u64().expect("index").to_string(),
        table["cells"][0]["row"].as_u64().expect("row").to_string(),
        table["cells"][0]["col"].as_u64().expect("col").to_string(),
    );
    let out = temp_path("cell").with_extension("hwpx");
    let out_s = out.to_str().unwrap().to_string();
    let args = [
        "edit",
        "set-cell",
        table_sample.to_str().unwrap(),
        "--table",
        &ts,
        "--row",
        &rs,
        "--col",
        &cs,
        "--text",
        "V",
        "-o",
        &out_s,
        "--verify",
        "--json",
    ];
    let output = run(&args);
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("envelope");
    assert!(v["verify"]["identical"].is_boolean(), "{v}");
    let _ = std::fs::remove_file(&out);
}
