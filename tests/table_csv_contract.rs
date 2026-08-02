//! [#3719 §6-7] `table-to-csv` / `csv-to-table` 계약 회귀 테스트.
//!
//! 이 축의 값은 두 가지다.
//!
//! 1. **병합 격자 채움** — `Table.cells` 는 앵커 셀만 담고 덮인 좌표는 목록에 없다.
//!    앵커를 그대로 이어 붙이면 병합이 있는 행에서 열이 밀린 CSV 가 나오고, 그것은
//!    오류 없이 틀린 데이터다. 그래서 모든 레코드의 필드 수가 `colCount` 와 같아야 한다.
//! 2. **조용한 절삭 금지** — CSV 의 행·열 수가 표와 다르면 한 칸도 쓰지 않고 `invalid[]`
//!    로 보고하며 사용법 오류(2)로 끝난다. 잘라 쓰면 "표는 그럴듯한데 뒤쪽 데이터가
//!    통째로 사라진" 보고서가 남는다.
//!
//! 종료 코드는 #2707 계약(0/1/2)을, 봉투는 `schemaVersion` 규약을 따른다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 표 53개(머리말 표 포함)·병합 다수의 실물 지자체 보고서 양식.
/// **본문 최상위 표 번호가 0 에서 시작하지 않는다** — 0 번은 머리말 안의 표다.
const SAMPLE: &str = "samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx";

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
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

fn temp_path(label: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "rhwp-table-csv-{label}-{}-{nonce}",
        std::process::id()
    ))
}

/// 병합이 있는 **본문 최상위** 표 하나를 실제 문서에서 고른다.
///
/// 표 번호를 상수로 박으면 샘플이 바뀔 때 조용히 다른 표를 검사하게 된다 —
/// `export-tables` 가 보고하는 실제 `index` 를 쓴다(containerPath 가 없는 것만).
fn pick_merged_top_level_table() -> (usize, u64, u64) {
    let p = sample();
    let args = ["export-tables", p.to_str().unwrap(), "--json"];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    let v = parse_stdout_json(&args, &out);
    let tables = v["tables"].as_array().expect("tables 배열");
    let picked = tables
        .iter()
        .filter(|t| t["containerPath"].is_null())
        .find(|t| {
            t["cells"]
                .as_array()
                .map(|cells| {
                    cells
                        .iter()
                        .any(|c| c["rowSpan"].as_u64() > Some(1) || c["colSpan"].as_u64() > Some(1))
                })
                .unwrap_or(false)
        })
        .unwrap_or_else(|| panic!("병합 있는 최상위 표를 찾지 못했습니다"));
    (
        picked["index"].as_u64().expect("index") as usize,
        picked["rows"].as_u64().expect("rows"),
        picked["cols"].as_u64().expect("cols"),
    )
}

/// RFC 4180 판독기 — 계약이 말하는 인용 규칙을 테스트 쪽에서 **독립 구현**해 대조한다.
/// 산출과 같은 코드를 재사용하면 둘 다 틀렸을 때 통과하는 순환 검증이 된다.
fn read_csv(input: &str) -> Vec<Vec<String>> {
    let chars: Vec<char> = input.chars().collect();
    let mut records = Vec::new();
    let mut record: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut i = 0usize;
    let mut quoted = false;
    while i < chars.len() {
        let c = chars[i];
        if quoted {
            if c == '"' {
                if chars.get(i + 1) == Some(&'"') {
                    field.push('"');
                    i += 1;
                } else {
                    quoted = false;
                }
            } else {
                field.push(c);
            }
        } else if c == '"' && field.is_empty() {
            quoted = true;
        } else if c == ',' {
            record.push(std::mem::take(&mut field));
        } else if c == '\n' {
            record.push(std::mem::take(&mut field));
            records.push(std::mem::take(&mut record));
        } else if c != '\r' {
            field.push(c);
        }
        i += 1;
    }
    if !field.is_empty() || !record.is_empty() {
        record.push(field);
        records.push(record);
    }
    records
}

/// 레코드 배열을 다시 CSV 로 조립한다 (테스트 입력을 만드는 쪽 — 역시 독립 구현).
fn write_csv(records: &[Vec<String>]) -> String {
    let mut out = String::new();
    for record in records {
        let line: Vec<String> = record
            .iter()
            .map(|f| {
                if f.contains([',', '"', '\r', '\n']) {
                    format!("\"{}\"", f.replace('"', "\"\""))
                } else {
                    f.clone()
                }
            })
            .collect();
        out.push_str(&line.join(","));
        out.push_str("\r\n");
    }
    out
}

// ── table-to-csv ───────────────────────────────────────────────────────────

#[test]
fn table_to_csv_json_envelope_contract() {
    let p = sample();
    let args = ["table-to-csv", p.to_str().unwrap(), "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_stdout_json(&args, &output);
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert!(v["source"].is_string(), "{v}");
    assert_eq!(v["bom"], false, "기본은 BOM 없음: {v}");
    let tables = v["tables"].as_array().expect("tables 배열");
    assert!(!tables.is_empty(), "표가 0건이면 이 계약은 공허하다: {v}");
    assert_eq!(v["tableCount"].as_u64(), Some(tables.len() as u64), "{v}");
    for t in tables {
        assert!(t["index"].is_u64(), "{t}");
        assert!(t["rowCount"].is_u64(), "{t}");
        assert!(t["colCount"].is_u64(), "{t}");
        assert!(t["csv"].is_string(), "{t}");
    }
}

#[test]
fn merged_table_csv_is_a_full_rectangle() {
    // 이 축의 핵심 — 병합으로 덮인 칸을 채우지 않으면 그 행만 필드가 모자라 **열이 밀린다**.
    let (index, rows, cols) = pick_merged_top_level_table();
    let p = sample();
    let table = index.to_string();
    let args = [
        "table-to-csv",
        p.to_str().unwrap(),
        "--table",
        table.as_str(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_stdout_json(&args, &output);
    let entry = &v["tables"][0];
    assert_eq!(entry["index"].as_u64(), Some(index as u64), "{v}");
    assert_eq!(entry["rowCount"].as_u64(), Some(rows), "{v}");
    assert_eq!(entry["colCount"].as_u64(), Some(cols), "{v}");

    let records = read_csv(entry["csv"].as_str().expect("csv 문자열"));
    assert_eq!(
        records.len(),
        rows as usize,
        "CSV 레코드 수가 rowCount 와 달라 격자가 아닙니다: {v}"
    );
    for (r, record) in records.iter().enumerate() {
        assert_eq!(
            record.len(),
            cols as usize,
            "{r}행의 필드 수 {} 가 colCount {cols} 와 다릅니다 — 병합 채움이 빠지면 \
             이 행부터 열이 통째로 밀립니다: {v}",
            record.len()
        );
    }
}

#[test]
fn rfc4180_quoting_survives_a_round_trip_through_the_document() {
    // 쉼표·따옴표가 든 값을 실제 문서 셀에 넣고 다시 CSV 로 뽑아, 인용이 값 자체를
    // 바꾸지 않는지 본다. 인용이 없거나 `""` 이스케이프가 빠지면 열 수부터 어긋난다.
    let (index, rows, cols) = pick_merged_top_level_table();
    assert!(rows >= 1 && cols >= 1, "표가 비었습니다");
    let p = sample();
    let table = index.to_string();
    let edited = temp_path("quote.hwpx");
    let needle = "가,나\"다";
    let set = run(&[
        "edit",
        "set-cell",
        p.to_str().unwrap(),
        "--table",
        table.as_str(),
        "--row",
        "0",
        "--col",
        "0",
        "--text",
        needle,
        "-o",
        edited.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(
        set.status.code(),
        Some(0),
        "set-cell 실패:\n{}",
        String::from_utf8_lossy(&set.stderr)
    );

    let args = [
        "table-to-csv",
        edited.to_str().unwrap(),
        "--table",
        table.as_str(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_stdout_json(&args, &output);
    let csv = v["tables"][0]["csv"].as_str().expect("csv");
    assert!(
        csv.starts_with("\"가,나\"\"다\""),
        "RFC 4180 인용(\" → \"\")이 아닙니다: {csv:?}"
    );
    let records = read_csv(csv);
    assert_eq!(
        records[0][0], needle,
        "판독 결과가 원값과 다릅니다: {csv:?}"
    );
    assert_eq!(
        records[0].len(),
        cols as usize,
        "인용 때문에 열이 밀렸습니다"
    );
    let _ = std::fs::remove_file(&edited);
}

#[test]
fn bom_flag_only_affects_the_file_not_the_envelope() {
    // BOM 은 파일 인코딩 표식이다. 봉투의 csv 문자열에 섞으면 JSON 을 그대로 쓰는
    // 소비자가 U+FEFF 를 첫 셀 값의 일부로 읽는다.
    let (index, _, _) = pick_merged_top_level_table();
    let p = sample();
    let table = index.to_string();
    let out = temp_path("bom.csv");
    let args = [
        "table-to-csv",
        p.to_str().unwrap(),
        "--table",
        table.as_str(),
        "-o",
        out.to_str().unwrap(),
        "--bom",
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_stdout_json(&args, &output);
    assert_eq!(v["bom"], true, "{v}");
    assert_eq!(v["outputFormat"], "csv", "{v}");
    assert!(!v["tables"][0]["csv"]
        .as_str()
        .expect("csv")
        .starts_with('\u{feff}'));

    let bytes = std::fs::read(&out).expect("CSV 파일");
    assert_eq!(
        &bytes[..3],
        &[0xEF, 0xBB, 0xBF],
        "파일에는 BOM 이 있어야 합니다"
    );
    let _ = std::fs::remove_file(&out);
}

#[test]
fn unknown_top_level_table_is_a_runtime_error_with_silent_stdout() {
    let p = sample();
    let args = [
        "table-to-csv",
        p.to_str().unwrap(),
        "--table",
        "99999",
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        describe(&args, &output)
    );
    assert!(
        output.stdout.is_empty(),
        "실패 경로 stdout 은 0바이트여야 합니다: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn missing_arguments_are_usage_errors_with_silent_stdout() {
    let p = sample();
    let path = p.to_str().unwrap();
    let cases: Vec<Vec<&str>> = vec![
        vec!["table-to-csv"],
        vec!["csv-to-table"],
        vec!["csv-to-table", path],
    ];
    for args in cases {
        let output = run(&args);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{}",
            describe(&args, &output)
        );
        assert!(
            output.stdout.is_empty(),
            "사용법 오류 stdout 은 0바이트여야 합니다: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

// ── csv-to-table ───────────────────────────────────────────────────────────

/// 표 그대로의 CSV 를 얻는다 (왕복 기준선).
fn csv_of(table: usize) -> String {
    let p = sample();
    let t = table.to_string();
    let args = [
        "table-to-csv",
        p.to_str().unwrap(),
        "--table",
        t.as_str(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    parse_stdout_json(&args, &output)["tables"][0]["csv"]
        .as_str()
        .expect("csv")
        .to_string()
}

#[test]
fn identical_csv_writes_nothing_and_verifies() {
    // 같은 내용을 되쓰면 바뀔 칸이 0 이어야 한다 — 왕복(내보내기→되넣기)이 값을
    // 건드리지 않는다는 뜻이다. --verify 로 저장본 재파싱까지 함께 판정한다.
    let (index, rows, cols) = pick_merged_top_level_table();
    let csv_file = temp_path("same.csv");
    std::fs::write(&csv_file, csv_of(index)).expect("CSV 쓰기");
    let out = temp_path("same.hwpx");
    let p = sample();
    let t = index.to_string();
    let args = [
        "csv-to-table",
        p.to_str().unwrap(),
        "--csv",
        csv_file.to_str().unwrap(),
        "--table",
        t.as_str(),
        "-o",
        out.to_str().unwrap(),
        "--verify",
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_stdout_json(&args, &output);
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert_eq!(v["table"].as_u64(), Some(index as u64), "{v}");
    assert_eq!(v["rowCount"].as_u64(), Some(rows), "{v}");
    assert_eq!(v["colCount"].as_u64(), Some(cols), "{v}");
    assert_eq!(
        v["changedCount"].as_u64(),
        Some(0),
        "왕복이 값을 바꿨다: {v}"
    );
    assert_eq!(v["invalid"].as_array().map(Vec::len), Some(0), "{v}");
    assert_eq!(v["verify"]["identical"], true, "{v}");
    assert_eq!(v["outputFormat"], "hwpx", "입력 형식 보존: {v}");
    let pages = v["changedPages"].as_array().expect("changedPages 배열");
    assert!(!pages.is_empty(), "눈검증 대상 쪽이 비었습니다: {v}");
    assert!(out.exists(), "산출물이 없습니다");
    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(&csv_file);
}

#[test]
fn value_written_by_csv_is_readable_back() {
    let (index, _, cols) = pick_merged_top_level_table();
    let mut records = read_csv(&csv_of(index));
    let marker = "표값-2026";
    records[0][0] = marker.to_string();
    let body = write_csv(&records);
    let csv_file = temp_path("write.csv");
    std::fs::write(&csv_file, &body).expect("CSV 쓰기");

    let out = temp_path("write.hwpx");
    let p = sample();
    let t = index.to_string();
    let args = [
        "csv-to-table",
        p.to_str().unwrap(),
        "--csv",
        csv_file.to_str().unwrap(),
        "--table",
        t.as_str(),
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_stdout_json(&args, &output);
    assert_eq!(v["changedCount"].as_u64(), Some(1), "{v}");
    assert_eq!(v["changed"][0]["newText"], marker, "{v}");

    // 저장본을 다시 CSV 로 뽑아 값이 실제로 문서에 들어갔는지 본다.
    let back = run(&[
        "table-to-csv",
        out.to_str().unwrap(),
        "--table",
        t.as_str(),
        "--json",
    ]);
    assert_eq!(back.status.code(), Some(0));
    let bv: serde_json::Value = serde_json::from_slice(&back.stdout).expect("봉투");
    let round = read_csv(bv["tables"][0]["csv"].as_str().expect("csv"));
    assert_eq!(round[0][0], marker, "되읽은 값이 다릅니다");
    assert_eq!(round[0].len(), cols as usize, "열 수가 달라졌습니다");
    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(&csv_file);
}

#[test]
fn dry_run_writes_no_file() {
    let (index, _, _) = pick_merged_top_level_table();
    let mut records = read_csv(&csv_of(index));
    records[0][0] = "미리보기".to_string();
    let body = write_csv(&records);
    let csv_file = temp_path("dry.csv");
    std::fs::write(&csv_file, &body).expect("CSV 쓰기");
    let out = temp_path("dry.hwpx");
    let p = sample();
    let t = index.to_string();
    let args = [
        "csv-to-table",
        p.to_str().unwrap(),
        "--csv",
        csv_file.to_str().unwrap(),
        "--table",
        t.as_str(),
        "-o",
        out.to_str().unwrap(),
        "--dry-run",
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_stdout_json(&args, &output);
    assert_eq!(v["dryRun"], true, "{v}");
    assert_eq!(v["changedCount"].as_u64(), Some(1), "{v}");
    assert!(v["changedPages"].is_null(), "예측 목록으로 오인 금지: {v}");
    assert!(v["output"].is_null(), "dry-run 에는 산출물이 없다: {v}");
    assert!(!out.exists(), "dry-run 이 파일을 썼습니다");
    let _ = std::fs::remove_file(&csv_file);
}

#[test]
fn row_count_mismatch_is_invalid_and_writes_nothing() {
    // 조용한 절삭 금지 — 행이 하나 모자라면 한 칸도 쓰지 않는다.
    let (index, _, _) = pick_merged_top_level_table();
    let mut records = read_csv(&csv_of(index));
    records.pop();
    let body = write_csv(&records);
    let csv_file = temp_path("shortrow.csv");
    std::fs::write(&csv_file, &body).expect("CSV 쓰기");
    let out = temp_path("shortrow.hwpx");
    let p = sample();
    let t = index.to_string();
    let args = [
        "csv-to-table",
        p.to_str().unwrap(),
        "--csv",
        csv_file.to_str().unwrap(),
        "--table",
        t.as_str(),
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
    let v = parse_stdout_json(&args, &output);
    let invalid = v["invalid"].as_array().expect("invalid 배열");
    assert!(
        invalid.iter().any(|i| i["reason"] == "rowCountMismatch"),
        "{v}"
    );
    assert_eq!(v["changedCount"].as_u64(), Some(0), "{v}");
    assert!(!out.exists(), "invalid 인데 파일을 썼습니다");
    let _ = std::fs::remove_file(&csv_file);
}

#[test]
fn col_count_mismatch_is_invalid_and_writes_nothing() {
    let (index, _, _) = pick_merged_top_level_table();
    let mut records = read_csv(&csv_of(index));
    records[0].push("남는열".to_string());
    let body = write_csv(&records);
    let csv_file = temp_path("longcol.csv");
    std::fs::write(&csv_file, &body).expect("CSV 쓰기");
    let out = temp_path("longcol.hwpx");
    let p = sample();
    let t = index.to_string();
    let args = [
        "csv-to-table",
        p.to_str().unwrap(),
        "--csv",
        csv_file.to_str().unwrap(),
        "--table",
        t.as_str(),
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
    let v = parse_stdout_json(&args, &output);
    assert!(v["invalid"]
        .as_array()
        .expect("invalid")
        .iter()
        .any(|i| i["reason"] == "colCountMismatch"));
    assert!(!out.exists(), "invalid 인데 파일을 썼습니다");
    let _ = std::fs::remove_file(&csv_file);
}

#[test]
fn value_in_a_merged_covered_cell_is_invalid() {
    // 덮인 칸에는 쓸 수 없다. 조용히 버리면 "썼다고 보고했는데 문서엔 없는" 값이 된다.
    let (index, rows, cols) = pick_merged_top_level_table();
    let p = sample();
    let t = index.to_string();
    let args = ["export-tables", p.to_str().unwrap(), "--json"];
    let tv = parse_stdout_json(&args, &run(&args));
    let grid = tv["tables"]
        .as_array()
        .expect("tables")
        .iter()
        .find(|t| t["index"].as_u64() == Some(index as u64))
        .expect("표");
    let anchors: Vec<(u64, u64)> = grid["cells"]
        .as_array()
        .expect("cells")
        .iter()
        .map(|c| (c["row"].as_u64().unwrap(), c["col"].as_u64().unwrap()))
        .collect();
    let covered = (0..rows)
        .flat_map(|r| (0..cols).map(move |c| (r, c)))
        .find(|rc| !anchors.contains(rc))
        .expect("병합으로 덮인 칸이 있어야 합니다");

    let mut records = read_csv(&csv_of(index));
    records[covered.0 as usize][covered.1 as usize] = "덮인칸값".to_string();
    let body = write_csv(&records);
    let csv_file = temp_path("covered.csv");
    std::fs::write(&csv_file, &body).expect("CSV 쓰기");
    let out = temp_path("covered.hwpx");
    let args = [
        "csv-to-table",
        p.to_str().unwrap(),
        "--csv",
        csv_file.to_str().unwrap(),
        "--table",
        t.as_str(),
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
    let v = parse_stdout_json(&args, &output);
    assert!(v["invalid"]
        .as_array()
        .expect("invalid")
        .iter()
        .any(|i| i["reason"] == "coveredCellNotEmpty"));
    assert!(!out.exists(), "invalid 인데 파일을 썼습니다");
    let _ = std::fs::remove_file(&csv_file);
}

#[test]
fn malformed_csv_is_invalid_not_a_panic() {
    let (index, _, _) = pick_merged_top_level_table();
    let csv_file = temp_path("bad.csv");
    std::fs::write(&csv_file, "\"닫히지 않은 따옴표").expect("CSV 쓰기");
    let p = sample();
    let t = index.to_string();
    let args = [
        "csv-to-table",
        p.to_str().unwrap(),
        "--csv",
        csv_file.to_str().unwrap(),
        "--table",
        t.as_str(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
    let v = parse_stdout_json(&args, &output);
    assert!(v["invalid"]
        .as_array()
        .expect("invalid")
        .iter()
        .any(|i| i["reason"] == "csvParse"));
    let _ = std::fs::remove_file(&csv_file);
}

// ── 자기서술 정합 ───────────────────────────────────────────────────────────

#[test]
fn capabilities_and_mcp_declare_both_commands() {
    // 드리프트 가드: 명령·MCP 도구·help 세 곳이 함께 갱신돼야 에이전트가 쓸 수 있다.
    let cap = parse_stdout_json(&["capabilities"], &run(&["capabilities"]));
    let names: Vec<&str> = cap["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .filter_map(|c| c["name"].as_str())
        .collect();
    for expected in ["table-to-csv", "csv-to-table"] {
        assert!(names.contains(&expected), "capabilities 누락: {expected}");
    }

    let mcp = parse_stdout_json(&["capabilities", "--mcp"], &run(&["capabilities", "--mcp"]));
    let tools = mcp["tools"].as_array().expect("tools");
    for (tool_name, command) in [
        ("hwp_table_to_csv", "table-to-csv"),
        ("hwp_csv_to_table", "csv-to-table"),
    ] {
        let t = tools
            .iter()
            .find(|t| t["name"] == tool_name)
            .unwrap_or_else(|| panic!("MCP 도구 누락: {tool_name}"));
        assert_eq!(t["cli"]["command"], command, "{t}");
        assert_eq!(t["inputSchema"]["type"], "object", "{t}");
        assert!(t["inputSchema"]["properties"].is_object(), "{t}");
        assert!(t["inputSchema"]["required"].is_array(), "{t}");
    }

    let help = run(&["--help"]);
    let help_text = String::from_utf8_lossy(&help.stdout);
    for expected in ["  table-to-csv ", "  csv-to-table "] {
        assert!(
            help_text.contains(expected),
            "--help 에 {expected:?} 가 없습니다"
        );
    }
}
