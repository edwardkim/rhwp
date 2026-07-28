//! Issue #3480: 채운 값이 칸을 넘쳐도 경고 없이 성공을 보고한다.
//!
//! `edit set-cell`·`edit fill-fields` 는 값이 셀을 넘쳐 여러 줄로 밀리거나 셀 경계를
//! 벗어나도 `exit 0` 에 경고 하나 없었다. 에이전트는 렌더 결과를 보지 않으므로,
//! 사람이라면 제출하지 않을 문서를 완성본으로 넘긴다.
//!
//! 신호는 **조판 엔진의 실제 렌더 트리**에서 읽는다 — 폭을 따로 추정하면 측정 경로와
//! 렌더 경로가 갈라지는 알려진 함정(#2237)을 밟는다.
//!
//! 계약:
//!   1) 편집이 줄 수를 늘렸거나 글이 셀 상자를 가로로 넘으면 `overflow` 에 보고
//!   2) 들어가는 값이면 `overflow` 는 비어 있음 (무회귀)
//!   3) 채우기를 막지 않는다 — 종료코드는 그대로 0, 판단은 소비자 몫
//!   4) `--dry-run` 도 같은 보고를 한다 (파일을 만들기 전에 알 수 있어야 한다)
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

/// 한 줄짜리 기입 칸(성명)이 있는 실물 서식.
const FORM: &str = "samples/복학원서.hwp";
/// 누름틀이 표 셀 안에 있는 대형 법정 서식.
const FIELD_FORM: &str = "samples/80168_regulatory_analysis.hwp";

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn temp_path(tag: &str, ext: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-issue3480-{tag}-{}-{}.{ext}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ))
}

fn run(args: &[&str]) -> (serde_json::Value, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .output()
        .expect("rhwp 실행 실패");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let last = stdout.lines().last().unwrap_or("{}").to_string();
    let json = serde_json::from_str(&last).unwrap_or_else(|e| {
        panic!(
            "JSON 파싱 실패({e}): {last}\nstderr={}",
            String::from_utf8_lossy(&out.stderr)
        )
    });
    (json, out.status.code().unwrap_or(-1))
}

fn set_cell(text: &str, out: &Path, extra: &[&str]) -> (serde_json::Value, i32) {
    let input = sample(FORM);
    let mut args = vec![
        "edit",
        "set-cell",
        input.to_str().unwrap(),
        "--table",
        "0",
        "--row",
        "2",
        "--col",
        "3",
        "--text",
        text,
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    args.extend_from_slice(extra);
    run(&args)
}

fn overflow_of(json: &serde_json::Value) -> &Vec<serde_json::Value> {
    json["overflow"]
        .as_array()
        .unwrap_or_else(|| panic!("overflow 배열이 없다: {json}"))
}

/// 칸을 넘치는 값은 보고된다 — 그래도 채우기는 막지 않는다.
#[test]
fn long_value_reports_overflow_without_failing() {
    let out = temp_path("long", "hwp");
    let (json, code) = set_cell(
        "홍가상홍가상홍가상홍가상홍가상홍가상홍가상홍가상홍가상홍가상",
        &out,
        &[],
    );
    assert_eq!(code, 0, "넘침은 실패가 아니다 — 신호만 준다: {json}");

    let overflow = overflow_of(&json);
    assert_eq!(overflow.len(), 1, "넘침이 보고되지 않았다: {json}");
    let entry = &overflow[0];
    assert_eq!(entry["target"], "table0[2,3]");
    let lines = entry["lines"].as_u64().expect("lines");
    let lines_before = entry["linesBefore"].as_u64().expect("linesBefore");
    assert!(
        lines > lines_before,
        "편집이 줄을 늘렸을 때만 신호여야 한다: {entry}"
    );
    assert!(
        entry["cellWidthPx"].as_f64().is_some_and(|w| w > 0.0),
        "칸 폭이 실측값이어야 한다: {entry}"
    );
    let _ = std::fs::remove_file(&out);
}

/// 들어가는 값은 조용하다 — 무회귀.
#[test]
fn fitting_value_reports_no_overflow() {
    let out = temp_path("fit", "hwp");
    let (json, code) = set_cell("홍길동", &out, &[]);
    assert_eq!(code, 0);
    assert!(
        overflow_of(&json).is_empty(),
        "들어가는 값에 넘침을 보고했다: {json}"
    );
    let _ = std::fs::remove_file(&out);
}

/// `--dry-run` 도 같은 보고를 한다 — 파일을 만들기 전에 알 수 있어야 한다.
#[test]
fn dry_run_reports_the_same_overflow() {
    let out = temp_path("dry", "hwp");
    let (json, code) = set_cell(
        "홍가상홍가상홍가상홍가상홍가상홍가상홍가상홍가상홍가상홍가상",
        &out,
        &["--dry-run"],
    );
    assert_eq!(code, 0);
    assert_eq!(
        overflow_of(&json).len(),
        1,
        "--dry-run 이 넘침을 보고하지 않았다: {json}"
    );
    assert!(!out.exists(), "--dry-run 은 출력 파일을 쓰지 않아야 한다");
}

/// 누름틀 경로도 같은 신호를 준다 — 셀 폭을 가로로 넘으면 `clipped`.
#[test]
fn fill_fields_reports_overflowing_cell_field() {
    let data_path = temp_path("data", "json");
    std::fs::write(
        &data_path,
        serde_json::to_string(&serde_json::json!({ "직급": "가상직급".repeat(30) }))
            .expect("직렬화"),
    )
    .expect("data.json 쓰기");
    let out = temp_path("fill", "hwp");
    let input = sample(FIELD_FORM);
    let data_arg = format!("@{}", data_path.to_str().unwrap());
    let (json, code) = run(&[
        "edit",
        "fill-fields",
        input.to_str().unwrap(),
        "--data",
        &data_arg,
        "-o",
        out.to_str().unwrap(),
        "--json",
    ]);
    let _ = std::fs::remove_file(&data_path);
    assert_eq!(code, 0, "넘침은 실패가 아니다: {json}");
    assert_eq!(json["filledCount"].as_u64(), Some(1));

    let overflow = overflow_of(&json);
    assert_eq!(overflow.len(), 1, "누름틀 넘침이 보고되지 않았다: {json}");
    let entry = &overflow[0];
    assert_eq!(entry["target"], "field:직급");
    assert_eq!(
        entry["clipped"], true,
        "셀 폭을 크게 넘는 값인데 clipped 가 아니다: {entry}"
    );
    let cell = entry["cellWidthPx"].as_f64().expect("cellWidthPx");
    let text = entry["textWidthPx"].as_f64().expect("textWidthPx");
    assert!(text > cell, "글 폭이 칸 폭을 넘어야 한다: {entry}");
    let _ = std::fs::remove_file(&out);
}
