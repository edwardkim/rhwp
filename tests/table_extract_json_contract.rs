//! [#3278] `export-tables` 출력 계약 회귀 테스트.
//!
//! 계약: `--json` 의 stdout 은 순수 JSON 한 덩어리이고 `schemaVersion` 을 포함한다.
//! 핵심 가치는 **병합(rowSpan/colSpan) 보존**과 **중첩 표 표현** — 이 둘이 없으면
//! export-markdown 과 다를 바 없다(병합 소실로 소비자가 빈 칸을 별개 열로 오독).
//! 종료 코드는 #2707 계약(0/1/2)을 따른다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 19행×9열, 셀 131개. 가로 병합(colSpan=3)과 세로 병합(rowSpan=3)을 모두 가진다.
const SAMPLE_MERGED: &str = "samples/table-001.hwp";
/// 표 셀 문단 안에 또 다른 표가 들어 있는 중첩 표 문서.
const SAMPLE_NESTED: &str = "samples/inner-table-01.hwp";
/// 본문 최상위가 아닌 **컨테이너(글상자 등) 안에 표가 있는** 문서.
/// 최상위만 훑는 `info` 의 표 열거는 1개만 보지만 실제로는 3개가 있다.
const SAMPLE_CONTAINER: &str = "samples/basic/treatise sample.hwp";

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

fn parse_stdout_json(args: &[&str], output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout 이 순수 JSON 이 아닙니다 ({e}).\n{}",
            describe(args, output)
        )
    })
}

fn run_json(rel: &str) -> serde_json::Value {
    let p = sample(rel);
    let args = ["export-tables", p.to_str().unwrap(), "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    parse_stdout_json(&args, &output)
}

#[test]
fn export_tables_json_envelope_contract() {
    let v = run_json(SAMPLE_MERGED);
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert!(v["source"].is_string(), "{v}");
    let count = v["tableCount"].as_u64().expect("tableCount");
    assert!(count >= 1, "{v}");
    let tables = v["tables"].as_array().expect("tables 배열");
    assert_eq!(
        tables.len() as u64,
        count,
        "tableCount 는 tables 길이와 같다: {v}"
    );

    let t = &tables[0];
    assert!(t["rows"].as_u64().unwrap() >= 1, "{t}");
    assert!(t["cols"].as_u64().unwrap() >= 1, "{t}");
    assert!(t["section"].as_u64().is_some(), "{t}");
    assert!(
        t["paragraph"].as_u64().is_some(),
        "표의 문단 주소가 있어야 인용된다: {t}"
    );
    assert!(
        t["control"].as_u64().is_some(),
        "같은 문단의 여러 표를 구별할 control 주소가 있어야 한다: {t}"
    );
    let cells = t["cells"].as_array().expect("cells 배열");
    assert_eq!(
        cells.len() as u64,
        t["cellCount"].as_u64().unwrap(),
        "cellCount 는 cells 길이와 같다: {t}"
    );
    let c = &cells[0];
    for k in ["row", "col", "rowSpan", "colSpan"] {
        assert!(c[k].as_u64().is_some(), "{k} 누락: {c}");
    }
    assert!(c["text"].is_string(), "{c}");
}

#[test]
fn export_tables_preserves_merged_spans() {
    // 이 테스트가 본 기능의 존재 이유다. export-markdown 은 병합을 잃어
    // "5월"(3열 병합) 뒤의 빈 칸을 별개 열처럼 보이게 한다.
    let v = run_json(SAMPLE_MERGED);
    let tables = v["tables"].as_array().unwrap();
    let cells: Vec<&serde_json::Value> = tables
        .iter()
        .flat_map(|t| t["cells"].as_array().unwrap().iter())
        .collect();

    let has_col_merge = cells
        .iter()
        .any(|c| c["colSpan"].as_u64().unwrap_or(1) >= 2);
    let has_row_merge = cells
        .iter()
        .any(|c| c["rowSpan"].as_u64().unwrap_or(1) >= 2);
    assert!(
        has_col_merge,
        "가로 병합(colSpan>=2)이 보존되어야 합니다: {v}"
    );
    assert!(
        has_row_merge,
        "세로 병합(rowSpan>=2)이 보존되어야 합니다: {v}"
    );

    // 병합 셀은 앵커 하나만 나온다 — 덮인 칸이 빈 셀로 중복 출력되면 격자가 깨진다.
    for t in tables {
        let rows = t["rows"].as_u64().unwrap();
        let cols = t["cols"].as_u64().unwrap();
        let cells = t["cells"].as_array().unwrap();
        let covered: u64 = cells
            .iter()
            .map(|c| {
                c["rowSpan"].as_u64().unwrap_or(1).max(1)
                    * c["colSpan"].as_u64().unwrap_or(1).max(1)
            })
            .sum();
        assert!(
            covered <= rows * cols,
            "병합 면적 합({covered})이 격자({rows}x{cols})를 넘습니다 — 중복 출력: {t}"
        );
    }
}

#[test]
fn export_tables_cell_addresses_are_in_bounds() {
    let v = run_json(SAMPLE_MERGED);
    for t in v["tables"].as_array().unwrap() {
        let rows = t["rows"].as_u64().unwrap();
        let cols = t["cols"].as_u64().unwrap();
        for c in t["cells"].as_array().unwrap() {
            let (r, cc) = (c["row"].as_u64().unwrap(), c["col"].as_u64().unwrap());
            assert!(r < rows, "행 주소 범위 초과: {c} (rows={rows})");
            assert!(cc < cols, "열 주소 범위 초과: {c} (cols={cols})");
        }
    }
}

#[test]
fn export_tables_expresses_nested_tables() {
    // 중첩 표(셀 안의 표)가 표현되어야 한다 — 평탄화하면 셀 소속을 잃는다.
    let v = run_json(SAMPLE_NESTED);
    let nested_found = v["tables"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|t| t["cells"].as_array().unwrap().iter())
        .any(|c| {
            c.get("nested")
                .and_then(|n| n.as_array())
                .is_some_and(|n| !n.is_empty())
        });
    assert!(nested_found, "중첩 표가 nested 로 표현되어야 합니다: {v}");
    let nested_has_cell_path = v["tables"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|t| t["cells"].as_array().unwrap().iter())
        .flat_map(|c| c["nested"].as_array().into_iter().flatten())
        .any(|t| {
            t["containerPath"]
                .as_array()
                .is_some_and(|path| path.iter().any(|e| e["kind"] == "tableCell"))
        });
    assert!(
        nested_has_cell_path,
        "중첩 표에는 부모 셀을 가리키는 containerPath 가 필요합니다: {v}"
    );
}

#[test]
fn export_tables_finds_tables_inside_containers() {
    // [#3278] 공문서는 표를 글상자·머리말·각주 안에 두는 배치가 흔하다.
    // 최상위 문단의 controls 만 훑으면(현행 info 의 표 열거가 그렇다) 통째로 누락된다.
    // 이 문서는 최상위 기준 1개지만 컨테이너까지 재귀하면 3개다.
    let v = run_json(SAMPLE_CONTAINER);
    assert!(
        v["tableCount"].as_u64().unwrap() >= 3,
        "컨테이너 안의 표까지 찾아야 합니다 (기대 3+, 최상위만 보면 1): {v}"
    );
    let container_path = v["tables"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|t| t["containerPath"].as_array())
        .unwrap_or_else(|| panic!("컨테이너 표에는 경로가 필요합니다: {v}"));
    assert!(
        container_path
            .iter()
            .any(|entry| entry["kind"].is_string() && entry["control"].is_u64()),
        "컨테이너 표의 kind·control 경로가 필요합니다: {v}"
    );
}

#[test]
fn export_tables_default_output_is_human_summary() {
    // 기본 출력은 사람용 요약 — --json 없이는 JSON 을 흘리지 않는다.
    let p = sample(SAMPLE_MERGED);
    let args = ["export-tables", p.to_str().unwrap()];
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
fn export_tables_missing_file_exit_runtime_silent_stdout() {
    let args = ["export-tables", "없는파일-tables.hwp", "--json"];
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
fn export_tables_usage_error_exit_two() {
    let args = ["export-tables"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
}
#[test]
fn export_tables_multiple_files_exit_usage() {
    let first = sample(SAMPLE_MERGED);
    let second = sample(SAMPLE_NESTED);
    let args = [
        "export-tables",
        first.to_str().unwrap(),
        second.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "여러 입력 파일을 마지막 파일로 조용히 덮어쓰면 안 됩니다.\n{}",
        describe(&args, &output)
    );
    assert!(
        output.stdout.is_empty(),
        "사용법 오류에서 stdout 은 비어야 합니다.\n{}",
        describe(&args, &output)
    );
}

/// [#3289] 아카이브 실행 시 컴파일타임 경로는 빌드 러너 전용이므로,
/// nextest가 런타임에 재매핑해 주입하는 CARGO_BIN_EXE_rhwp를 우선한다.
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}
