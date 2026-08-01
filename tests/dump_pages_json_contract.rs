//! [#3697] `dump-pages --json` 페이지네이션 진단 기계 계약 (#3608 1-C).
//!
//! 계약: `--json` 모드의 stdout 은 순수 JSON 단건 봉투이고 `schemaVersion` 을 포함한다.
//! 필드 추가는 허용, 기존 필드의 변경·삭제는 본 테스트가 실패로 잡는다.
//! 종료 코드는 [#2707] 계약(0/1/2)을 그대로 따르고, 실패 경로 stdout 은 0바이트다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 파싱까지 성공하는 실제 샘플 (cli_json_contract.rs 와 동일).
const SAMPLE: &str = "samples/hwp3-sample.hwp";

fn sample_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
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

fn parse_stdout_json(args: &[&str], output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout 이 순수 JSON 이 아닙니다 ({e}).\n{}",
            describe(args, output)
        )
    })
}

// ── 봉투 스키마 ────────────────────────────────────────────────────────────

#[test]
fn dump_pages_json_envelope_contract() {
    let sample = sample_path();
    let args = ["dump-pages", sample.to_str().unwrap(), "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    let v = parse_stdout_json(&args, &output);
    // 스키마 고정: 아래 필드의 존재·타입이 계약이다 (필드 추가는 허용).
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert!(v["source"].is_string(), "{v}");
    assert!(v["pageCount"].as_u64().unwrap() >= 1, "{v}");
    assert!(v["pageFilter"].is_null(), "{v}");
    assert_eq!(v["respectVposReset"], false, "{v}");

    let pages = v["pages"].as_array().expect("pages 는 배열");
    assert_eq!(
        pages.len() as u64,
        v["pageCount"].as_u64().unwrap(),
        "필터 없는 pages 는 전체 페이지 수와 일치해야 한다: {v}"
    );

    let p0 = &pages[0];
    assert_eq!(p0["pageIndex"], 0, "{p0}");
    assert_eq!(p0["displayPage"], 1, "{p0}");
    assert!(p0["section"].as_u64().is_some(), "{p0}");
    assert!(p0["pageNumber"].as_u64().is_some(), "{p0}");
    for key in ["x", "y", "width", "height"] {
        assert!(
            p0["bodyArea"][key].as_f64().is_some(),
            "bodyArea.{key} 는 숫자여야 한다: {p0}"
        );
    }

    let columns = p0["columns"].as_array().expect("columns 는 배열");
    assert!(!columns.is_empty(), "{p0}");
    let c0 = &columns[0];
    assert_eq!(c0["index"], 0, "{c0}");
    assert!(c0["usedHeight"].as_f64().is_some(), "{c0}");
    let items = c0["items"].as_array().expect("items 는 배열");
    assert_eq!(
        c0["itemCount"].as_u64().unwrap(),
        items.len() as u64,
        "{c0}"
    );

    // 모든 항목은 kind 를 가지며, 문단/표/도형 항목은 paraIndex 를 가진다.
    for page in pages {
        for col in page["columns"].as_array().unwrap() {
            for item in col["items"].as_array().unwrap() {
                let kind = item["kind"].as_str().expect("kind 는 문자열");
                assert!(
                    [
                        "fullParagraph",
                        "partialParagraph",
                        "table",
                        "partialTable",
                        "shape",
                        "endnoteSeparator",
                    ]
                    .contains(&kind),
                    "알 수 없는 kind: {item}"
                );
                if kind != "endnoteSeparator" {
                    assert!(item["paraIndex"].as_u64().is_some(), "{item}");
                    assert!(item["isEndnote"].is_boolean(), "{item}");
                }
                if kind == "partialParagraph" {
                    assert!(item["startLine"].as_u64().is_some(), "{item}");
                    assert!(item["endLine"].as_u64().is_some(), "{item}");
                }
                if kind == "table" || kind == "partialTable" || kind == "shape" {
                    assert!(item["controlIndex"].as_u64().is_some(), "{item}");
                }
                if kind == "partialTable" {
                    assert!(item["startRow"].as_u64().is_some(), "{item}");
                    assert!(item["endRow"].as_u64().is_some(), "{item}");
                    assert!(item["isContinuation"].is_boolean(), "{item}");
                }
            }
        }
        assert!(page["extras"].is_array(), "{page}");
    }
}

// ── -p 필터 ────────────────────────────────────────────────────────────────

#[test]
fn dump_pages_json_page_filter_returns_single_page() {
    let sample = sample_path();
    let args = ["dump-pages", sample.to_str().unwrap(), "-p", "0", "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    let v = parse_stdout_json(&args, &output);
    assert_eq!(v["pageFilter"], 0, "{v}");
    let pages = v["pages"].as_array().expect("pages 는 배열");
    assert_eq!(pages.len(), 1, "{v}");
    assert_eq!(pages[0]["pageIndex"], 0, "{v}");
}

// ── 실패 경로: stdout 침묵 + exit 계약 ─────────────────────────────────────

#[test]
fn dump_pages_json_out_of_range_exit_usage_silent_stdout() {
    let sample = sample_path();
    let args = [
        "dump-pages",
        sample.to_str().unwrap(),
        "-p",
        "999",
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
    assert!(
        output.stdout.is_empty(),
        "실패 경로 stdout 은 비어야 합니다.\n{}",
        describe(&args, &output)
    );
}

#[test]
fn dump_pages_json_missing_file_exit_runtime_silent_stdout() {
    let args = ["dump-pages", "없는파일-dump-pages.hwp", "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        describe(&args, &output)
    );
    assert!(
        output.stdout.is_empty(),
        "실패 경로 stdout 은 비어야 합니다.\n{}",
        describe(&args, &output)
    );
}

/// [#3289] 아카이브 실행 시 컴파일타임 경로는 빌드 러너 전용이므로,
/// nextest가 런타임에 재매핑해 주입하는 CARGO_BIN_EXE_rhwp를 우선한다.
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}
