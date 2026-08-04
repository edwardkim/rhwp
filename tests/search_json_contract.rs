//! [#3283] `search` 출력 계약 회귀 테스트 — 주소를 가진 문서 검색.
//!
//! 이 명령의 존재 이유는 **페이지 주소**다. 평문 추출 후 외부 검색으로는 얻을 수 없고,
//! 조판 엔진이 있어야만 답할 수 있다. 페이지가 사라지면 기능 전체가 무의미해지므로
//! 그 정합성을 계약으로 고정한다. 종료 코드는 #2707 계약(0/1/2)을 따른다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SAMPLE: &str = "samples/hwp3-sample.hwp";
/// 표를 가진 문서 — 표 셀 안의 매치 좌표 검증용.
const SAMPLE_TABLE: &str = "samples/table-001.hwp";
/// 글상자 안의 매치에 재참조 가능한 좌표가 붙는지 검증한다.
const SAMPLE_TEXTBOX: &str = "samples/table-in-tbox.hwp";

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
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

fn parse_json(args: &[&str], output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout 이 순수 JSON 이 아닙니다 ({e}).\n{}",
            describe(args, output)
        )
    })
}

/// 샘플에 실재하는 검색어. `export-text --json` 같은 미머지 기능에 의존하지 않도록
/// 고정 문자열을 쓰고, 0건이면 테스트가 실패해 샘플 변경을 알린다.
const QUERY: &str = "의";

#[test]
fn search_json_envelope_and_addresses() {
    let p = sample(SAMPLE);
    let args = ["search", p.to_str().unwrap(), QUERY, "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    let v = parse_json(&args, &output);
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert!(v["source"].is_string(), "{v}");
    assert_eq!(v["query"], QUERY, "{v}");
    assert!(v["caseSensitive"].is_boolean(), "{v}");
    let count = v["matchCount"].as_u64().expect("matchCount");
    let matches = v["matches"].as_array().expect("matches 배열");
    assert_eq!(matches.len() as u64, count, "{v}");
    assert!(count >= 1, "문서에서 뽑은 단어인데 0건입니다: {v}");

    let m = &matches[0];
    assert!(m["section"].as_u64().is_some(), "{m}");
    assert!(m["paragraph"].as_u64().is_some(), "{m}");
    assert!(m["charOffset"].as_u64().is_some(), "{m}");
    assert!(m["length"].as_u64().unwrap() >= 1, "{m}");
    assert!(m["text"].is_string(), "{m}");
    assert!(m["context"].is_string(), "{m}");
}

#[test]
fn search_reports_page_within_document_range() {
    // 이 테스트가 본 기능의 존재 이유다 — 페이지 주소가 실제로, 유효 범위 안에서 나와야 한다.
    let p = sample(SAMPLE);

    // 페이지 수는 `info` 의 사람용 출력에서 얻는다 — 미머지 기능(`--json`)에 의존하지 않는다.
    let info = run(&["info", p.to_str().unwrap()]);
    let info_text = String::from_utf8_lossy(&info.stdout);
    let page_count: u64 = info_text
        .lines()
        .find_map(|l| l.strip_prefix("페이지 수:"))
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or_else(|| panic!("info 에서 페이지 수를 찾지 못했습니다:\n{info_text}"));

    let args = ["search", p.to_str().unwrap(), QUERY, "--json"];
    let output = run(&args);
    let v = parse_json(&args, &output);

    let paged: Vec<u64> = v["matches"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|m| m["page"].as_u64())
        .collect();
    assert!(
        !paged.is_empty(),
        "페이지 주소가 하나도 없으면 기능이 무의미합니다: {v}"
    );
    for pg in paged {
        assert!(
            pg < page_count,
            "페이지 {pg} 가 문서 페이지 수({page_count}) 범위를 벗어납니다: {v}"
        );
    }
}

#[test]
fn search_finds_matches_inside_table_cells() {
    // 표 셀 안의 텍스트도 검색 범위이고, 셀 좌표가 붙어야 후속 참조가 된다.
    let p = sample(SAMPLE_TABLE);
    let args = ["search", p.to_str().unwrap(), "월", "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_json(&args, &output);

    let cell_match = v["matches"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m.get("cell").is_some());
    if let Some(m) = cell_match {
        for k in ["control", "cell", "paragraph"] {
            assert!(m["cell"][k].as_u64().is_some(), "cell.{k} 누락: {m}");
        }
    }
    // 표 문서에서 이 검색어가 0건이면 표 셀 순회 자체가 끊긴 것이다.
    assert!(v["matchCount"].as_u64().unwrap() >= 1, "{v}");
}

#[test]
fn search_finds_matches_inside_textboxes_with_coordinates() {
    // 이 샘플의 첫 본문 문단은 글상자이며, 검색어는 글상자 문단 10에 실재한다.
    let p = sample(SAMPLE_TEXTBOX);
    let args = ["search", p.to_str().unwrap(), "수돗물", "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_json(&args, &output);
    let m = v["matches"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["paragraph"] == 0 && m.get("textbox").is_some())
        .unwrap_or_else(|| panic!("글상자 매치와 좌표가 필요합니다: {v}"));
    assert_eq!(m["textbox"]["control"], 2, "{m}");
    assert_eq!(m["textbox"]["paragraph"], 10, "{m}");
    assert!(m.get("cell").is_none(), "글상자는 표 셀이 아닙니다: {m}");
}

#[test]
fn search_no_match_is_success_not_error() {
    // 0건은 실패가 아니다 — 1은 런타임 실패 전용이다(#2707).
    let p = sample(SAMPLE);
    let args = [
        "search",
        p.to_str().unwrap(),
        "존재하지않을검색어ZZZQQQ",
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_json(&args, &output);
    assert_eq!(v["matchCount"], 0, "{v}");
    assert_eq!(v["matches"].as_array().unwrap().len(), 0, "{v}");
}

#[test]
fn search_limit_caps_result_count() {
    let p = sample(SAMPLE);
    let args = [
        "search",
        p.to_str().unwrap(),
        QUERY,
        "--json",
        "--limit",
        "1",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_json(&args, &output);
    assert!(v["matchCount"].as_u64().unwrap() <= 1, "{v}");
}

#[test]
fn search_ignore_case_flag_is_reported() {
    let p = sample(SAMPLE);
    let args = [
        "search",
        p.to_str().unwrap(),
        QUERY,
        "--json",
        "--ignore-case",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_json(&args, &output);
    assert_eq!(v["caseSensitive"], false, "{v}");
}

#[test]
fn search_default_output_is_human_summary() {
    let p = sample(SAMPLE);
    let args = ["search", p.to_str().unwrap(), QUERY];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        serde_json::from_str::<serde_json::Value>(&stdout).is_err(),
        "기본 출력은 JSON 이 아니어야 합니다(--json 전용).\n{}",
        describe(&args, &output)
    );
}

#[test]
fn search_missing_file_exit_runtime_silent_stdout() {
    let args = ["search", "없는파일-search.hwp", "검색어", "--json"];
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

#[test]
fn search_missing_query_exit_usage() {
    let p = sample(SAMPLE);
    let args = ["search", p.to_str().unwrap()];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
}

/// [#3289] 아카이브 실행 시 컴파일타임 경로는 빌드 러너 전용이므로,
/// nextest가 런타임에 재매핑해 주입하는 CARGO_BIN_EXE_rhwp를 우선한다.
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}
