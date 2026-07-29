//! [#3353] `search --limit` 절단 가시성 회귀 테스트.
//!
//! 계약: `matchCount` 는 반환된 매치 수(= `matches.len()`)이고, `totalMatchCount` 는
//! 문서 전체 매치 수, `truncated` 는 절단 여부다. `--limit` 을 써도 에이전트가
//! "정확히 N건"과 "N건만 표시(실제 그 이상)"를 구별할 수 있어야 한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// search_json_contract.rs 와 같은 샘플·검색어 — 매치가 여러 건 실재한다.
const SAMPLE: &str = "samples/hwp3-sample.hwp";
const QUERY: &str = "의";

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn describe(args: &[&str], output: &Output) -> String {
    format!(
        "명령: rhwp {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn search_json(extra: &[&str]) -> serde_json::Value {
    let p = sample(SAMPLE);
    let mut args = vec!["search", p.to_str().unwrap(), QUERY, "--json"];
    args.extend_from_slice(extra);
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout 이 순수 JSON 이 아닙니다 ({e}).\n{}",
            describe(&args, &output)
        )
    })
}

/// 절단 시: matchCount 는 limit, totalMatchCount 는 전체, truncated=true.
#[test]
fn limit_reports_total_and_truncated() {
    let full = search_json(&[]);
    let total = full["matchCount"].as_u64().expect("matchCount");
    assert!(total >= 2, "샘플에 매치가 2건 이상이어야 합니다: {full}");

    let limited = search_json(&["--limit", "1"]);
    assert_eq!(limited["matchCount"], 1, "{limited}");
    assert_eq!(
        limited["matches"].as_array().map(Vec::len),
        Some(1),
        "{limited}"
    );
    assert_eq!(
        limited["totalMatchCount"].as_u64(),
        Some(total),
        "절단돼도 문서 전체 매치 수를 보고해야 합니다: {limited}"
    );
    assert_eq!(limited["truncated"], true, "{limited}");
}

/// 미절단 시: totalMatchCount == matchCount, truncated=false.
#[test]
fn no_limit_reports_not_truncated() {
    let v = search_json(&[]);
    assert_eq!(v["totalMatchCount"], v["matchCount"], "{v}");
    assert_eq!(v["truncated"], false, "{v}");
}

/// limit 이 전체 이상이면 절단이 아니다.
#[test]
fn limit_at_or_above_total_is_not_truncated() {
    let full = search_json(&[]);
    let total = full["matchCount"].as_u64().expect("matchCount");
    let big = (total + 10).to_string();
    let v = search_json(&["--limit", &big]);
    assert_eq!(v["matchCount"].as_u64(), Some(total), "{v}");
    assert_eq!(v["totalMatchCount"].as_u64(), Some(total), "{v}");
    assert_eq!(v["truncated"], false, "{v}");
}

/// 비-JSON 출력도 절단 시 전체 건수를 알린다.
#[test]
fn human_output_reports_total_when_truncated() {
    let p = sample(SAMPLE);
    let args = ["search", p.to_str().unwrap(), QUERY, "--limit", "1"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("건 중 1건 표시"),
        "절단 시 '전체 N건 중 1건 표시' 안내가 나와야 합니다.\n{}",
        describe(&args, &output)
    );
}
