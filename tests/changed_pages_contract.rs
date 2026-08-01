//! [#3712] changedPages — 편집 봉투가 눈검증 대상 페이지를 지정한다 (#3630 P3).
//! 계약: 변경 문단 ∩ 편집 반영 후 페이지네이션. 확정 불가·무산출은 null (부분 목록 금지).
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SAMPLE: &str = "samples/field-01.hwp";

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn temp_path(tag: &str, ext: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-chpages-{tag}-{}-{}.{ext}",
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

/// changedPages 배열이 0 기준 글로벌 페이지 번호 규약을 지키는지 공통 판정.
fn assert_valid_pages(v: &serde_json::Value, page_count: u64) {
    let pages = v["changedPages"].as_array().expect("changedPages 배열");
    assert!(!pages.is_empty(), "변경이 있었으니 비어 있지 않다: {v}");
    for p in pages {
        let n = p.as_u64().expect("페이지 번호는 정수");
        assert!(n < page_count, "0 기준·범위 내: {n} < {page_count}: {v}");
    }
}

fn page_count_of(path: &str) -> u64 {
    let output = run(&["info", path, "--json"]);
    assert_eq!(output.status.code(), Some(0));
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    v["pageCount"].as_u64().expect("pageCount")
}

/// fill-fields — 채운 필드가 놓인 페이지를 지정한다.
#[test]
fn fill_fields_reports_changed_pages() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out = temp_path("fill", "hwp");
    let output = run(&[
        "edit",
        "fill-fields",
        p.to_str().unwrap(),
        "--data",
        r#"{"회사명":"페이지지정사"}"#,
        "-o",
        out.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(output.status.code(), Some(0));
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("envelope");
    assert_valid_pages(&v, page_count_of(p.to_str().unwrap()));
    let _ = std::fs::remove_file(&out);
}

/// replace-text — 치환 매치가 놓인 페이지를 지정한다.
#[test]
fn replace_text_reports_changed_pages() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out = temp_path("repl", "hwp");
    let output = run(&[
        "edit",
        "replace-text",
        p.to_str().unwrap(),
        "--find",
        "마케팅",
        "--replace",
        "기획",
        "-o",
        out.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(output.status.code(), Some(0));
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("envelope");
    assert_valid_pages(&v, page_count_of(p.to_str().unwrap()));
    let _ = std::fs::remove_file(&out);
}

/// dry-run 은 산출물이 없다 — changedPages 는 null (예측 목록으로 오인 금지).
#[test]
fn dry_run_changed_pages_is_null() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let output = run(&[
        "edit",
        "fill-fields",
        p.to_str().unwrap(),
        "--data",
        r#"{"회사명":"드라이런"}"#,
        "--dry-run",
        "--json",
    ]);
    assert_eq!(output.status.code(), Some(0));
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("envelope");
    assert!(v["changedPages"].is_null(), "dry-run 은 null: {v}");
}

/// set-cell — 표 호스트 문단이 걸친 페이지(분할 표 포함 전 쪽)를 지정한다.
#[test]
fn set_cell_reports_changed_pages() {
    let table_sample = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx");
    if !table_sample.exists() {
        eprintln!("표 샘플 없음 — 건너뜀");
        return;
    }
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
    let out = temp_path("cell", "hwpx");
    let output = run(&[
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
        out.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(output.status.code(), Some(0));
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("envelope");
    assert_valid_pages(&v, page_count_of(table_sample.to_str().unwrap()));
    let _ = std::fs::remove_file(&out);
}

/// rhwp run 저널 — 전 step 합집합을 최상위 changedPages 로 보고한다.
#[test]
fn run_plan_journal_reports_changed_pages_union() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out = temp_path("plan", "hwp");
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": p.to_str().unwrap(),
        "output": out.to_str().unwrap(),
        "steps": [
            { "action": "fill_fields", "data": {"회사명": "합집합사"} },
            { "action": "replace_text", "find": "마케팅", "replace": "기획" },
        ],
        "assertions": { "verify": true },
    });
    let plan_path = temp_path("plan", "json");
    std::fs::write(&plan_path, serde_json::to_vec(&plan).unwrap()).unwrap();
    let output = run(&["run", plan_path.to_str().unwrap(), "--json"]);
    assert_eq!(output.status.code(), Some(0));
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("envelope");
    assert_valid_pages(&v, page_count_of(p.to_str().unwrap()));
    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(&plan_path);
}
