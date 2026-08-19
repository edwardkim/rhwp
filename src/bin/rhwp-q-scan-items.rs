//! 한글 스캔 차례 항목 조회 — `InitScan`·`GetText`·`ReleaseScan` 이 쓰는 사슬.
//!
//! 기존 읽기 전용 [`DocumentCore::scan_items_json`] 만 부른다. 문서를 고치지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;
use serde_json::{json, Value};
use std::io::Write;
use std::process;

const EXIT_OK: i32 = 0;
const EXIT_RUNTIME: i32 = 1;
const EXIT_USAGE: i32 = 2;
const TOOL: &str = "rhwp-q-scan-items";
const COMMAND: &str = "scan-items";
const UNTRUSTED_FIELDS: &[&str] = &["source", "items[].text"];
const USAGE: &str = "사용법: rhwp-q-scan-items <파일> [--json] [--limit <N>]";

#[derive(Debug)]
struct Options {
    json: bool,
    help: bool,
    version: bool,
    path: Option<String>,
    limit: Option<usize>,
}

fn write_stdout(text: &str, newline: bool) {
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    let result = if newline {
        writeln!(lock, "{text}")
    } else {
        write!(lock, "{text}")
    };
    if let Err(e) = result {
        eprintln!("오류: stdout 쓰기 실패 - {e}");
        process::exit(EXIT_RUNTIME);
    }
}

fn print_json(value: &Value) {
    match serde_json::to_string_pretty(value) {
        Ok(s) => write_stdout(&s, true),
        Err(e) => eprintln!("오류: JSON 직렬화 실패 - {e}"),
    }
}

fn parse_args(args: &[String]) -> Result<Options, i32> {
    let mut opts = Options {
        json: false,
        help: false,
        version: false,
        path: None,
        limit: None,
    };
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                opts.help = true;
                i += 1;
            }
            "--version" | "-V" => {
                opts.version = true;
                i += 1;
            }
            "--json" => {
                opts.json = true;
                i += 1;
            }
            "--limit" => {
                let Some(raw) = args.get(i + 1) else {
                    eprintln!("오류: --limit 뒤에 1 이상의 정수가 필요합니다.");
                    eprintln!("{USAGE}");
                    return Err(EXIT_USAGE);
                };
                match raw.parse::<usize>() {
                    Ok(n) if n >= 1 => opts.limit = Some(n),
                    _ => {
                        eprintln!("오류: --limit 뒤에 1 이상의 정수가 필요합니다.");
                        eprintln!("{USAGE}");
                        return Err(EXIT_USAGE);
                    }
                }
                i += 2;
            }
            other if other.starts_with('-') => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {other}");
                eprintln!("{USAGE}");
                return Err(EXIT_USAGE);
            }
            other => {
                if opts.path.is_some() {
                    eprintln!("오류: 파일이 너무 많습니다 - {other}");
                    eprintln!("{USAGE}");
                    return Err(EXIT_USAGE);
                }
                opts.path = Some(other.to_string());
                i += 1;
            }
        }
    }
    if opts.help || opts.version {
        return Ok(opts);
    }
    if opts.path.is_none() {
        eprintln!("오류: 파일 경로가 필요합니다.");
        eprintln!("{USAGE}");
        return Err(EXIT_USAGE);
    }
    Ok(opts)
}

fn load_items(path: &str) -> Result<Vec<Value>, i32> {
    let data = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {path}: {e}");
            return Err(EXIT_RUNTIME);
        }
    };
    let core = match DocumentCore::from_bytes(&data) {
        Ok(core) => core,
        Err(e) => {
            eprintln!("오류: 문서를 열 수 없습니다 - {path}: {e}");
            return Err(EXIT_RUNTIME);
        }
    };
    let raw = core.scan_items_json();
    match serde_json::from_str::<Value>(&raw) {
        Ok(Value::Array(items)) => Ok(items),
        Ok(_) => {
            eprintln!("오류: 스캔 항목 JSON 이 배열이 아닙니다.");
            Err(EXIT_RUNTIME)
        }
        Err(e) => {
            eprintln!("오류: 스캔 항목 JSON 이 깨졌습니다 - {e}");
            Err(EXIT_RUNTIME)
        }
    }
}

fn apply_limit(mut items: Vec<Value>, limit: Option<usize>) -> (Vec<Value>, usize, bool) {
    let total = items.len();
    let truncated = match limit {
        Some(n) if total > n => {
            items.truncate(n);
            true
        }
        _ => false,
    };
    (items, total, truncated)
}

fn envelope(
    path: &str,
    items: Vec<Value>,
    total: usize,
    truncated: bool,
    limit: Option<usize>,
) -> Value {
    json!({
        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
        "tool": TOOL,
        "command": COMMAND,
        "version": rhwp::version(),
        "untrustedContent": true,
        "untrustedFields": UNTRUSTED_FIELDS,
        "source": path,
        "itemCount": items.len(),
        "totalCount": total,
        "limit": limit,
        "truncated": truncated,
        "items": items,
    })
}

fn print_text(path: &str, items: &[Value], total: usize, truncated: bool) {
    write_stdout(
        &format!(
            "scan-items source={path} itemCount={} totalCount={total} truncated={truncated}",
            items.len()
        ),
        true,
    );
    for item in items {
        let state = item.get("state").and_then(Value::as_u64).unwrap_or(0);
        let kind = item.get("kind").and_then(Value::as_u64).unwrap_or(0);
        let text = item.get("text").and_then(Value::as_str).unwrap_or("");
        let one_line = text.replace('\r', "\\r").replace('\n', "\\n");
        write_stdout(&format!("{state}\t{kind}\t{one_line}"), true);
    }
}

fn run(args: &[String]) -> i32 {
    let opts = match parse_args(args) {
        Ok(opts) => opts,
        Err(code) => return code,
    };
    if opts.help {
        write_stdout(USAGE, true);
        write_stdout(
            "한글 스캔 차례 항목을 조회한다. 문서를 고치지 않는다.",
            true,
        );
        write_stdout("종료 코드: 0 성공 · 1 실행 오류 · 2 사용법 오류", true);
        return EXIT_OK;
    }
    if opts.version {
        write_stdout(&format!("{TOOL} v{}", rhwp::version()), true);
        return EXIT_OK;
    }
    let path = opts.path.as_deref().expect("parse_args 가 경로를 확인한다");
    let loaded = match load_items(path) {
        Ok(items) => items,
        Err(code) => return code,
    };
    let (items, total, truncated) = apply_limit(loaded, opts.limit);
    if opts.json {
        print_json(&envelope(path, items, total, truncated, opts.limit));
    } else {
        print_text(path, &items, total, truncated);
    }
    EXIT_OK
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    process::exit(run(&args));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample() -> String {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("samples")
            .join("form-01.hwp")
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn unknown_flag_is_usage() {
        let err = parse_args(&["--nope".into(), sample()]).unwrap_err();
        assert_eq!(err, EXIT_USAGE);
    }

    #[test]
    fn missing_path_is_usage() {
        let err = parse_args(&["--json".into()]).unwrap_err();
        assert_eq!(err, EXIT_USAGE);
    }

    #[test]
    fn extra_path_is_usage() {
        let err = parse_args(&[sample(), "other.hwp".into()]).unwrap_err();
        assert_eq!(err, EXIT_USAGE);
    }

    #[test]
    fn limit_without_value_is_usage() {
        let err = parse_args(&[sample(), "--limit".into()]).unwrap_err();
        assert_eq!(err, EXIT_USAGE);
    }

    #[test]
    fn limit_zero_is_usage() {
        let err = parse_args(&[sample(), "--limit".into(), "0".into()]).unwrap_err();
        assert_eq!(err, EXIT_USAGE);
    }

    #[test]
    fn limit_non_integer_is_usage() {
        let err = parse_args(&[sample(), "--limit".into(), "x".into()]).unwrap_err();
        assert_eq!(err, EXIT_USAGE);
    }

    #[test]
    fn flags_may_surround_path() {
        let opts = parse_args(&["--json".into(), "--limit".into(), "20".into(), sample()]).unwrap();
        assert!(opts.json);
        assert_eq!(opts.limit, Some(20));
        assert_eq!(opts.path.as_deref(), Some(sample().as_str()));
    }

    #[test]
    fn form_sample_emits_scan_items() {
        let items = load_items(&sample()).expect("form-01.hwp 를 열 수 있어야 한다");
        assert!(!items.is_empty());
        let first = &items[0];
        assert!(first.get("state").and_then(Value::as_u64).is_some());
        assert!(first.get("kind").and_then(Value::as_u64).is_some());
        assert!(first.get("text").and_then(Value::as_str).is_some());
    }

    #[test]
    fn limit_truncates_and_marks_envelope() {
        let loaded = load_items(&sample()).expect("form-01.hwp 를 열 수 있어야 한다");
        assert!(
            loaded.len() > 20,
            "제한을 검증하려면 항목이 20개를 넘어야 한다"
        );
        let (items, total, truncated) = apply_limit(loaded, Some(20));
        assert!(truncated);
        assert_eq!(items.len(), 20);
        assert!(total > 20);
        let env = envelope(&sample(), items, total, truncated, Some(20));
        assert_eq!(env["tool"], TOOL);
        assert_eq!(env["command"], COMMAND);
        assert_eq!(env["truncated"], true);
        assert_eq!(env["itemCount"], 20);
        assert_eq!(env["totalCount"], total);
        assert_eq!(env["limit"], 20);
        assert_eq!(env["untrustedFields"], json!(["source", "items[].text"]));
        assert_eq!(env["items"].as_array().map(|a| a.len()), Some(20));
    }

    #[test]
    fn source_never_calls_mutators() {
        let src = include_str!("rhwp-q-scan-items.rs");
        let code = src.split("#[cfg(test)]").next().unwrap();
        for needle in [".apply_", ".insert_", ".delete_", ".set_"] {
            assert!(
                !code.contains(needle),
                "읽기 전용 CLI 가 {needle} 를 부르면 안 된다"
            );
        }
        assert!(code.contains("scan_items_json"));
        assert!(code.contains("from_bytes"));
    }
}
