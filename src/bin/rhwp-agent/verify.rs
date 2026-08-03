//! [#3918] `verify` — 산출물 사후 검증 게이트.
//!
//! 에이전트 루프의 빠진 반쪽이다: 편집·변환은 있는데 "그 산출물이 기대를
//! 만족하는가"를 종료 코드로 판정하는 범용 표면이 없었다(`convert --verify` 는
//! 변환 왕복 전용). `--expect-*` 를 하나 이상 주면 전부 평가하고, 하나라도
//! 어긋나면 exit 3 이다. 평가 자체가 불가능하면(파일 없음·파싱 실패) 실행 오류 1 —
//! "위반"과 "판정 불능"을 스크립트가 구별할 수 있어야 한다.

use crate::envelope::{
    envelope, format_token, load_core, page_texts, print_json, read_file, EXIT_GATE, EXIT_OK,
    EXIT_RUNTIME, EXIT_USAGE,
};
use serde_json::{json, Value};

/// 기대 하나 — 파싱 단계에서 모아 두고 적재 후 일괄 평가한다.
enum Expect {
    Format(String),
    Pages(u32),
    MinPages(u32),
    MaxPages(u32),
    MinChars(u64),
    Contains(String),
    NotContains(String),
    TableCount(u64),
    MinTables(u64),
    /// (이름, 기대값 — None 이면 존재만 검사)
    Field(String, Option<String>),
}

impl Expect {
    fn name(&self) -> &'static str {
        match self {
            Expect::Format(_) => "format",
            Expect::Pages(_) => "pages",
            Expect::MinPages(_) => "min-pages",
            Expect::MaxPages(_) => "max-pages",
            Expect::MinChars(_) => "min-chars",
            Expect::Contains(_) => "contains",
            Expect::NotContains(_) => "not-contains",
            Expect::TableCount(_) => "table-count",
            Expect::MinTables(_) => "min-tables",
            Expect::Field(_, _) => "field",
        }
    }

    /// 포맷 검사만이면 문서를 열 필요가 없다.
    fn needs_parse(&self) -> bool {
        !matches!(self, Expect::Format(_))
    }
}

fn parse_u32(args: &[String], i: usize, flag: &str) -> Result<u32, String> {
    args.get(i + 1)
        .and_then(|v| v.parse::<u32>().ok())
        .ok_or_else(|| format!("{flag} 뒤에 0 이상의 정수가 필요합니다."))
}

fn parse_u64(args: &[String], i: usize, flag: &str) -> Result<u64, String> {
    args.get(i + 1)
        .and_then(|v| v.parse::<u64>().ok())
        .ok_or_else(|| format!("{flag} 뒤에 0 이상의 정수가 필요합니다."))
}

fn parse_text(args: &[String], i: usize, flag: &str) -> Result<String, String> {
    match args.get(i + 1) {
        Some(v) if !v.is_empty() => Ok(v.clone()),
        _ => Err(format!("{flag} 뒤에 비어 있지 않은 값이 필요합니다.")),
    }
}

pub fn run(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp-agent verify <파일> [--json] --expect-... (하나 이상 필수, 'rhwp-agent capabilities' 참고)";

    let mut json_mode = false;
    let mut file: Option<String> = None;
    let mut expects: Vec<Expect> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let flag = args[i].as_str();
        let mut two = true;
        let parsed: Result<Option<Expect>, String> = match flag {
            "--json" => {
                two = false;
                json_mode = true;
                Ok(None)
            }
            "--expect-format" => parse_text(args, i, flag).and_then(|v| {
                if matches!(v.as_str(), "hwp5" | "hwpx" | "hwp3" | "hml") {
                    Ok(Some(Expect::Format(v)))
                } else {
                    Err("--expect-format 값은 hwp5|hwpx|hwp3|hml 중 하나여야 합니다.".to_string())
                }
            }),
            "--expect-pages" => parse_u32(args, i, flag).map(|n| Some(Expect::Pages(n))),
            "--expect-min-pages" => parse_u32(args, i, flag).map(|n| Some(Expect::MinPages(n))),
            "--expect-max-pages" => parse_u32(args, i, flag).map(|n| Some(Expect::MaxPages(n))),
            "--expect-min-chars" => parse_u64(args, i, flag).map(|n| Some(Expect::MinChars(n))),
            "--expect-contains" => parse_text(args, i, flag).map(|v| Some(Expect::Contains(v))),
            "--expect-not-contains" => {
                parse_text(args, i, flag).map(|v| Some(Expect::NotContains(v)))
            }
            "--expect-table-count" => parse_u64(args, i, flag).map(|n| Some(Expect::TableCount(n))),
            "--expect-min-tables" => parse_u64(args, i, flag).map(|n| Some(Expect::MinTables(n))),
            "--expect-field" => parse_text(args, i, flag).map(|v| {
                let (name, value) = match v.split_once('=') {
                    Some((n, val)) => (n.to_string(), Some(val.to_string())),
                    None => (v, None),
                };
                Some(Expect::Field(name, value))
            }),
            other if other.starts_with('-') => Err(format!("알 수 없는 옵션입니다 - {other}")),
            positional => {
                two = false;
                if file.is_some() {
                    Err(format!("파일은 하나만 지정할 수 있습니다 - {positional}"))
                } else {
                    file = Some(positional.to_string());
                    Ok(None)
                }
            }
        };
        match parsed {
            Ok(Some(expect)) => expects.push(expect),
            Ok(None) => {}
            Err(message) => {
                eprintln!("오류: {message}");
                eprintln!("{USAGE}");
                return EXIT_USAGE;
            }
        }
        i += if two { 2 } else { 1 };
    }

    let Some(path) = file else {
        eprintln!("오류: 대상 파일을 지정해주세요.");
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if expects.is_empty() {
        eprintln!("오류: --expect-* 기대를 하나 이상 지정해주세요.");
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    }

    // ── 적재 (필요한 만큼만) ────────────────────────────────────────────────
    let data = match read_file(&path) {
        Ok(d) => d,
        Err(message) => {
            eprintln!("오류: {message}");
            return EXIT_RUNTIME;
        }
    };
    let magic = format_token(rhwp::parser::detect_format(&data));

    let needs_parse = expects.iter().any(Expect::needs_parse);
    let mut page_count = 0u32;
    let mut full_text = String::new();
    let mut char_count = 0u64;
    let mut table_count = 0u64;
    let mut fields: Vec<(String, String)> = Vec::new();
    if needs_parse {
        let core = match load_core(&data) {
            Ok(c) => c,
            Err(fail) => {
                eprintln!("오류: 문서를 열 수 없습니다 - {path}: {}", fail.message);
                return EXIT_RUNTIME;
            }
        };
        let pages = match page_texts(&core) {
            Ok(p) => p,
            Err(message) => {
                eprintln!("오류: {path}: {message}");
                return EXIT_RUNTIME;
            }
        };
        page_count = core.page_count();
        char_count = pages.iter().map(|p| p.chars().count() as u64).sum();
        full_text = pages.join("\n");
        table_count = rhwp::document_core::queries::table_extract::extract_tables(core.document())
            .len() as u64;
        fields = core
            .collect_all_fields()
            .into_iter()
            .map(|f| {
                let name = f
                    .field
                    .ctrl_data_name
                    .clone()
                    .filter(|n| !n.is_empty())
                    .unwrap_or_else(|| f.field.command.clone());
                (name, f.value)
            })
            .collect();
    }

    // ── 평가 ───────────────────────────────────────────────────────────────
    let mut assertions: Vec<Value> = Vec::new();
    let mut failed = 0usize;
    for expect in &expects {
        let (expected, actual, ok) = match expect {
            Expect::Format(want) => (json!(want), json!(magic), magic == want),
            Expect::Pages(n) => (json!(n), json!(page_count), page_count == *n),
            Expect::MinPages(n) => (json!(n), json!(page_count), page_count >= *n),
            Expect::MaxPages(n) => (json!(n), json!(page_count), page_count <= *n),
            Expect::MinChars(n) => (json!(n), json!(char_count), char_count >= *n),
            Expect::Contains(needle) => {
                let found = full_text.contains(needle.as_str());
                (json!(needle), json!({ "found": found }), found)
            }
            Expect::NotContains(needle) => {
                let found = full_text.contains(needle.as_str());
                (json!(needle), json!({ "found": found }), !found)
            }
            Expect::TableCount(n) => (json!(n), json!(table_count), table_count == *n),
            Expect::MinTables(n) => (json!(n), json!(table_count), table_count >= *n),
            Expect::Field(name, want) => {
                let hit = fields.iter().find(|(n, _)| n == name);
                match (hit, want) {
                    (None, _) => (
                        json!({ "name": name, "value": want }),
                        json!({ "exists": false }),
                        false,
                    ),
                    (Some((_, value)), None) => (
                        json!({ "name": name }),
                        json!({ "exists": true, "value": value }),
                        true,
                    ),
                    (Some((_, value)), Some(expected_value)) => (
                        json!({ "name": name, "value": expected_value }),
                        json!({ "exists": true, "value": value }),
                        value == expected_value,
                    ),
                }
            }
        };
        if !ok {
            failed += 1;
        }
        assertions.push(json!({
            "name": expect.name(),
            "expected": expected,
            "actual": actual,
            "ok": ok,
        }));
    }

    let all_ok = failed == 0;
    if json_mode {
        let payload = json!({
            "source": path,
            "ok": all_ok,
            "failed": failed,
            "total": assertions.len(),
            "assertions": assertions,
        });
        // `--expect-field` 의 actual.value 는 문서 파생이다 — 보수적으로 항상 선언.
        print_json(&envelope("verify", payload, &["assertions[].actual"]));
    } else {
        crate::outln!("rhwp-agent verify — {path}");
        for assertion in &assertions {
            crate::outln!(
                "  [{}] {} (기대 {}, 실제 {})",
                if assertion["ok"].as_bool() == Some(true) {
                    "통과"
                } else {
                    "위반"
                },
                assertion["name"].as_str().unwrap_or("?"),
                assertion["expected"],
                assertion["actual"],
            );
        }
        crate::outln!(
            "결과: {} ({}건 중 위반 {failed}건)",
            if all_ok {
                "전부 통과"
            } else {
                "위반 있음"
            },
            assertions.len(),
        );
    }

    if all_ok {
        EXIT_OK
    } else {
        EXIT_GATE
    }
}
