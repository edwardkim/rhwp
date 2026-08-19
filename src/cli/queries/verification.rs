//! Document expectation verification query adapter.

use std::fs;

use rhwp::provenance;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;

use crate::{collect_field_records, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

pub(crate) fn run(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp verify <파일.hwp|파일.hwpx> [--expect-pages N] \
[--expect-min-pages N] [--expect-max-pages N] [--expect-min-chars N] \
[--expect-min-tables N] [--expect-table-count N] \
[--expect-contains 문자열]... [--expect-not-contains 문자열]... [--expect-field 이름=값]... \
[--expect-format hwp5|hwpx|hwp3|hml] [--json] — 기대 조건이 최소 1개 필요합니다";

    let mut file_path: Option<&str> = None;
    let mut json_mode = false;
    let mut expect_pages: Option<u64> = None;
    let mut expect_min_pages: Option<u64> = None;
    let mut expect_max_pages: Option<u64> = None;
    let mut expect_min_chars: Option<u64> = None;
    let mut expect_min_tables: Option<u64> = None;
    let mut expect_table_count: Option<u64> = None;
    let mut expect_format: Option<String> = None;
    let mut expect_contains: Vec<String> = Vec::new();
    let mut expect_not_contains: Vec<String> = Vec::new();
    let mut expect_fields: Vec<(String, String)> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            flag @ ("--expect-pages"
            | "--expect-min-pages"
            | "--expect-max-pages"
            | "--expect-min-chars"
            | "--expect-min-tables"
            | "--expect-table-count") => {
                i += 1;
                let n = args.get(i).and_then(|v| v.parse::<u64>().ok());
                match n {
                    Some(n) => {
                        *match flag {
                            "--expect-pages" => &mut expect_pages,
                            "--expect-min-pages" => &mut expect_min_pages,
                            "--expect-max-pages" => &mut expect_max_pages,
                            "--expect-min-chars" => &mut expect_min_chars,
                            "--expect-min-tables" => &mut expect_min_tables,
                            _ => &mut expect_table_count,
                        } = Some(n);
                    }
                    None => {
                        eprintln!("오류: {flag} 뒤에 숫자가 필요합니다.");
                        eprintln!("{USAGE}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--expect-contains" => {
                i += 1;
                match args.get(i) {
                    Some(v) => expect_contains.push(v.clone()),
                    None => {
                        eprintln!("오류: --expect-contains 뒤에 문자열이 필요합니다.");
                        eprintln!("{USAGE}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--expect-not-contains" => {
                i += 1;
                match args.get(i) {
                    Some(v) => expect_not_contains.push(v.clone()),
                    None => {
                        eprintln!("오류: --expect-not-contains 뒤에 문자열이 필요합니다.");
                        eprintln!("{USAGE}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--expect-field" => {
                i += 1;
                match args.get(i).and_then(|v| v.split_once('=')) {
                    Some((k, val)) if !k.is_empty() => {
                        expect_fields.push((k.to_string(), val.to_string()))
                    }
                    _ => {
                        eprintln!("오류: --expect-field 는 이름=값 형식입니다.");
                        eprintln!("{USAGE}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--expect-format" => {
                i += 1;
                match args.get(i).map(String::as_str) {
                    Some(v @ ("hwp5" | "hwpx" | "hwp3" | "hml")) => {
                        expect_format = Some(v.to_string())
                    }
                    Some(v) => {
                        eprintln!(
                            "오류: --expect-format 은 hwp5|hwpx|hwp3|hml 중 하나입니다 - {v}"
                        );
                        eprintln!("{USAGE}");
                        return EXIT_USAGE;
                    }
                    None => {
                        eprintln!("오류: --expect-format 뒤에 형식이 필요합니다.");
                        eprintln!("{USAGE}");
                        return EXIT_USAGE;
                    }
                }
            }
            other if other.starts_with('-') => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {other}");
                eprintln!("{USAGE}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.is_some() {
                    eprintln!("오류: 파일 경로는 하나여야 합니다 - {other}");
                    eprintln!("{USAGE}");
                    return EXIT_USAGE;
                }
                file_path = Some(other);
            }
        }
        i += 1;
    }
    let Some(path) = file_path else {
        eprintln!("오류: 파일 경로가 필요합니다.");
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let expectation_count = usize::from(expect_pages.is_some())
        + usize::from(expect_min_pages.is_some())
        + usize::from(expect_max_pages.is_some())
        + usize::from(expect_min_chars.is_some())
        + usize::from(expect_min_tables.is_some())
        + usize::from(expect_table_count.is_some())
        + usize::from(expect_format.is_some())
        + expect_contains.len()
        + expect_not_contains.len()
        + expect_fields.len();
    if expectation_count == 0 {
        eprintln!("오류: 기대 조건이 없습니다 — --expect-* 로 최소 1개를 지정하세요.");
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    }

    let data = match fs::read(path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", path, e);
            return EXIT_RUNTIME;
        }
    };
    let actual_format = match rhwp::parser::detect_format(&data) {
        rhwp::parser::FileFormat::Hwp => "hwp5",
        rhwp::parser::FileFormat::Hwpx => "hwpx",
        rhwp::parser::FileFormat::Hwp3 => "hwp3",
        rhwp::parser::FileFormat::Hml => "hml",
        _ => "unknown",
    };
    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    let mut expectations: Vec<serde_json::Value> = Vec::new();
    let mut fail_count = 0usize;
    let mut record = |kind: &str,
                      subject: serde_json::Value,
                      expected: serde_json::Value,
                      actual: serde_json::Value,
                      pass: bool| {
        if !pass {
            fail_count += 1;
        }
        let mut e = serde_json::json!({
            "kind": kind, "expected": expected, "actual": actual, "pass": pass,
        });
        if !subject.is_null() {
            e["subject"] = subject;
        }
        expectations.push(e);
    };

    if let Some(n) = expect_pages {
        let actual = u64::from(doc.page_count());
        record(
            "pages",
            serde_json::Value::Null,
            serde_json::json!(n),
            serde_json::json!(actual),
            actual == n,
        );
    }
    if let Some(n) = expect_min_pages {
        let actual = u64::from(doc.page_count());
        record(
            "minPages",
            serde_json::Value::Null,
            serde_json::json!(n),
            serde_json::json!(actual),
            actual >= n,
        );
    }
    if let Some(n) = expect_max_pages {
        let actual = u64::from(doc.page_count());
        record(
            "maxPages",
            serde_json::Value::Null,
            serde_json::json!(n),
            serde_json::json!(actual),
            actual <= n,
        );
    }
    if let Some(n) = expect_min_chars {
        // 쪽별 추출 텍스트의 문자 수 합 — export-text 와 같은 출처를 쓴다.
        let mut actual = 0u64;
        for page in 0..doc.page_count() {
            match doc.extract_page_text_native(page) {
                Ok(text) => actual += text.chars().count() as u64,
                Err(e) => {
                    eprintln!("오류: 본문 텍스트 추출 실패 - {}쪽: {}", page, e);
                    return EXIT_RUNTIME;
                }
            }
        }
        record(
            "minChars",
            serde_json::Value::Null,
            serde_json::json!(n),
            serde_json::json!(actual),
            actual >= n,
        );
    }
    if expect_min_tables.is_some() || expect_table_count.is_some() {
        use rhwp::document_core::queries::table_extract::extract_tables;
        let actual = extract_tables(doc.document()).len() as u64;
        if let Some(n) = expect_min_tables {
            record(
                "minTables",
                serde_json::Value::Null,
                serde_json::json!(n),
                serde_json::json!(actual),
                actual >= n,
            );
        }
        if let Some(n) = expect_table_count {
            record(
                "tableCount",
                serde_json::Value::Null,
                serde_json::json!(n),
                serde_json::json!(actual),
                actual == n,
            );
        }
    }
    if let Some(f) = expect_format.as_deref() {
        record(
            "format",
            serde_json::Value::Null,
            serde_json::json!(f),
            serde_json::json!(actual_format),
            actual_format == f,
        );
    }
    for s in &expect_contains {
        let n = doc.grep(s, true, None).len();
        record(
            "contains",
            serde_json::json!(s),
            serde_json::json!(">=1"),
            serde_json::json!(n),
            n >= 1,
        );
    }
    for s in &expect_not_contains {
        let n = doc.grep(s, true, None).len();
        record(
            "notContains",
            serde_json::json!(s),
            serde_json::json!(0),
            serde_json::json!(n),
            n == 0,
        );
    }
    if !expect_fields.is_empty() {
        let records = collect_field_records(&doc);
        for (name, want) in &expect_fields {
            let actual = records
                .iter()
                .find(|r| r["name"].as_str() == Some(name.as_str()))
                .map(|r| r["value"].clone())
                .unwrap_or(serde_json::Value::Null);
            let pass = actual.as_str() == Some(want.as_str());
            record(
                "field",
                serde_json::json!(name),
                serde_json::json!(want),
                actual,
                pass,
            );
        }
    }

    let verdict = if fail_count == 0 { "pass" } else { "fail" };
    let pass_count = expectation_count - fail_count;
    if json_mode {
        let envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": path,
            "expectations": expectations,
            "passCount": pass_count,
            "failCount": fail_count,
            "verdict": verdict,
        });
        println!("{}", provenance::marked(envelope, "verify"));
    } else {
        for e in &expectations {
            let mark = if e["pass"].as_bool() == Some(true) {
                "PASS"
            } else {
                "FAIL"
            };
            let subject = e["subject"]
                .as_str()
                .map(|s| format!(" '{s}'"))
                .unwrap_or_default();
            println!(
                "{mark} {}{subject} — 기대 {} / 실측 {}",
                e["kind"].as_str().unwrap_or(""),
                e["expected"],
                e["actual"]
            );
        }
        println!("판정: {verdict} ({pass_count} 통과 / {fail_count} 불일치)");
    }
    if fail_count == 0 {
        EXIT_OK
    } else {
        3 // 판정 불일치 — #2707 의 판정 코드. 봉투는 이미 냈다.
    }
}
