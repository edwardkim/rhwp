//! Document expectation verification query adapter.

use std::fs;

use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;

use crate::{collect_field_records, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

const USAGE: &str = "사용법: rhwp verify <파일.hwp|파일.hwpx> [--expect-pages N] \
[--expect-min-pages N] [--expect-max-pages N] [--expect-min-chars N] \
[--expect-min-tables N] [--expect-table-count N] \
[--expect-contains 문자열]... [--expect-not-contains 문자열]... [--expect-field 이름=값]... \
[--expect-format hwp5|hwpx|hwp3|hml] [--json] — 기대 조건이 최소 1개 필요합니다";

struct VerifyArgs {
    path: String,
    json_mode: bool,
    expect_pages: Option<u64>,
    expect_min_pages: Option<u64>,
    expect_max_pages: Option<u64>,
    expect_min_chars: Option<u64>,
    expect_min_tables: Option<u64>,
    expect_table_count: Option<u64>,
    expect_format: Option<String>,
    expect_contains: Vec<String>,
    expect_not_contains: Vec<String>,
    expect_fields: Vec<(String, String)>,
}

impl VerifyArgs {
    fn parse(args: &[String]) -> Result<Self, i32> {
        let mut parsed = Self {
            path: String::new(),
            json_mode: false,
            expect_pages: None,
            expect_min_pages: None,
            expect_max_pages: None,
            expect_min_chars: None,
            expect_min_tables: None,
            expect_table_count: None,
            expect_format: None,
            expect_contains: Vec::new(),
            expect_not_contains: Vec::new(),
            expect_fields: Vec::new(),
        };
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--json" => parsed.json_mode = true,
                flag @ ("--expect-pages"
                | "--expect-min-pages"
                | "--expect-max-pages"
                | "--expect-min-chars"
                | "--expect-min-tables"
                | "--expect-table-count") => {
                    i += 1;
                    let Some(n) = args.get(i).and_then(|value| value.parse::<u64>().ok()) else {
                        eprintln!("오류: {flag} 뒤에 숫자가 필요합니다.");
                        eprintln!("{USAGE}");
                        return Err(EXIT_USAGE);
                    };
                    *match flag {
                        "--expect-pages" => &mut parsed.expect_pages,
                        "--expect-min-pages" => &mut parsed.expect_min_pages,
                        "--expect-max-pages" => &mut parsed.expect_max_pages,
                        "--expect-min-chars" => &mut parsed.expect_min_chars,
                        "--expect-min-tables" => &mut parsed.expect_min_tables,
                        _ => &mut parsed.expect_table_count,
                    } = Some(n);
                }
                "--expect-contains" => {
                    i += 1;
                    let Some(value) = args.get(i) else {
                        eprintln!("오류: --expect-contains 뒤에 문자열이 필요합니다.");
                        eprintln!("{USAGE}");
                        return Err(EXIT_USAGE);
                    };
                    parsed.expect_contains.push(value.clone());
                }
                "--expect-not-contains" => {
                    i += 1;
                    let Some(value) = args.get(i) else {
                        eprintln!("오류: --expect-not-contains 뒤에 문자열이 필요합니다.");
                        eprintln!("{USAGE}");
                        return Err(EXIT_USAGE);
                    };
                    parsed.expect_not_contains.push(value.clone());
                }
                "--expect-field" => {
                    i += 1;
                    let Some((name, value)) = args.get(i).and_then(|value| value.split_once('='))
                    else {
                        eprintln!("오류: --expect-field 는 이름=값 형식입니다.");
                        eprintln!("{USAGE}");
                        return Err(EXIT_USAGE);
                    };
                    if name.is_empty() {
                        eprintln!("오류: --expect-field 는 이름=값 형식입니다.");
                        eprintln!("{USAGE}");
                        return Err(EXIT_USAGE);
                    }
                    parsed
                        .expect_fields
                        .push((name.to_string(), value.to_string()));
                }
                "--expect-format" => {
                    i += 1;
                    match args.get(i).map(String::as_str) {
                        Some(value @ ("hwp5" | "hwpx" | "hwp3" | "hml")) => {
                            parsed.expect_format = Some(value.to_string());
                        }
                        Some(value) => {
                            eprintln!(
                                "오류: --expect-format 은 hwp5|hwpx|hwp3|hml 중 하나입니다 - {value}"
                            );
                            eprintln!("{USAGE}");
                            return Err(EXIT_USAGE);
                        }
                        None => {
                            eprintln!("오류: --expect-format 뒤에 형식이 필요합니다.");
                            eprintln!("{USAGE}");
                            return Err(EXIT_USAGE);
                        }
                    }
                }
                other if other.starts_with('-') => {
                    eprintln!("오류: 알 수 없는 옵션입니다 - {other}");
                    eprintln!("{USAGE}");
                    return Err(EXIT_USAGE);
                }
                other if parsed.path.is_empty() => parsed.path = other.to_string(),
                other => {
                    eprintln!("오류: 파일 경로는 하나여야 합니다 - {other}");
                    eprintln!("{USAGE}");
                    return Err(EXIT_USAGE);
                }
            }
            i += 1;
        }
        if parsed.path.is_empty() {
            eprintln!("오류: 파일 경로가 필요합니다.");
            eprintln!("{USAGE}");
            return Err(EXIT_USAGE);
        }
        if parsed.expectation_count() == 0 {
            eprintln!("오류: 기대 조건이 없습니다 — --expect-* 로 최소 1개를 지정하세요.");
            eprintln!("{USAGE}");
            return Err(EXIT_USAGE);
        }
        Ok(parsed)
    }

    fn expectation_count(&self) -> usize {
        usize::from(self.expect_pages.is_some())
            + usize::from(self.expect_min_pages.is_some())
            + usize::from(self.expect_max_pages.is_some())
            + usize::from(self.expect_min_chars.is_some())
            + usize::from(self.expect_min_tables.is_some())
            + usize::from(self.expect_table_count.is_some())
            + usize::from(self.expect_format.is_some())
            + self.expect_contains.len()
            + self.expect_not_contains.len()
            + self.expect_fields.len()
    }
}

/// [#4113 / #3918 승격 2호] `verify` — 편집 파이프라인의 독립 사후검증 게이트.
///
/// 기대 조건 집합을 문서 실측과 대조해 전부 만족이면 exit 0, 하나라도 어긋나면
/// **봉투를 먼저 내고** exit 3(판정 — #2707) — 판정은 데이터다(규칙 3). 실행
/// 실패는 stdout 을 비우고 exit 1, 조립 오류는 exit 2. 실측은 전부 기존 코어
/// 재사용이다: `page_count`·`grep`·`collect_field_records`·`detect_format`(규칙 2).
pub(crate) fn run(args: &[String]) -> i32 {
    let parsed = match VerifyArgs::parse(args) {
        Ok(parsed) => parsed,
        Err(exit) => return exit,
    };
    let expectation_count = parsed.expectation_count();
    let VerifyArgs {
        path,
        json_mode,
        expect_pages,
        expect_min_pages,
        expect_max_pages,
        expect_min_chars,
        expect_min_tables,
        expect_table_count,
        expect_format,
        expect_contains,
        expect_not_contains,
        expect_fields,
    } = parsed;

    let data = match fs::read(&path) {
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
        println!("{}", envelope);
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
