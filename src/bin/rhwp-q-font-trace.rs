//! 쪽 글꼴 결정 추적 조회 — 기존 읽기 전용 `get_font_decision_trace_native` 만 부른다.
//!
//! 문서를 고치지 않는다. `--page` 는 0부터 센다.

use rhwp::document_core::DocumentCore;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;
use rhwp::HwpError;
use serde_json::{json, Value};
use std::io::Write;
use std::process;

const EXIT_OK: i32 = 0;
const EXIT_RUNTIME: i32 = 1;
const EXIT_USAGE: i32 = 2;
const TOOL: &str = "rhwp-q-font-trace";
const COMMAND: &str = "font-trace";
const UNTRUSTED_FIELDS: &[&str] = &["source", "trace"];
const DEFAULT_MAX_CHARACTERS: usize = 1024;
const MAX_CHARACTERS: usize = 4096;
const USAGE: &str =
    "사용법: rhwp-q-font-trace <파일> --page <N> [--max-characters <1..=4096>] [--json]";

#[derive(Debug)]
struct Options {
    json: bool,
    help: bool,
    version: bool,
    path: Option<String>,
    page: Option<u32>,
    max_characters: usize,
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

fn parse_page_value(raw: &str) -> Result<u32, i32> {
    match raw.parse::<u32>() {
        Ok(n) => Ok(n),
        Err(_) => {
            eprintln!("오류: --page 뒤에 0 이상의 정수가 필요합니다.");
            eprintln!("{USAGE}");
            Err(EXIT_USAGE)
        }
    }
}

fn parse_max_characters_value(raw: &str) -> Result<usize, i32> {
    match raw.parse::<usize>() {
        Ok(n) if (1..=MAX_CHARACTERS).contains(&n) => Ok(n),
        _ => {
            eprintln!("오류: --max-characters 뒤에 1..={MAX_CHARACTERS} 정수가 필요합니다.");
            eprintln!("{USAGE}");
            Err(EXIT_USAGE)
        }
    }
}

fn parse_args(args: &[String]) -> Result<Options, i32> {
    let mut opts = Options {
        json: false,
        help: false,
        version: false,
        path: None,
        page: None,
        max_characters: DEFAULT_MAX_CHARACTERS,
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
            "--page" => {
                let Some(raw) = args.get(i + 1) else {
                    eprintln!("오류: --page 뒤에 0 이상의 정수가 필요합니다.");
                    eprintln!("{USAGE}");
                    return Err(EXIT_USAGE);
                };
                opts.page = Some(parse_page_value(raw)?);
                i += 2;
            }
            "--max-characters" => {
                let Some(raw) = args.get(i + 1) else {
                    eprintln!(
                        "오류: --max-characters 뒤에 1..={MAX_CHARACTERS} 정수가 필요합니다."
                    );
                    eprintln!("{USAGE}");
                    return Err(EXIT_USAGE);
                };
                opts.max_characters = parse_max_characters_value(raw)?;
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
    if opts.page.is_none() {
        eprintln!("오류: --page 가 필요합니다.");
        eprintln!("{USAGE}");
        return Err(EXIT_USAGE);
    }
    Ok(opts)
}

fn load_trace(path: &str, page: u32, max_characters: usize) -> Result<Value, i32> {
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
    let options = json!({ "maxCharacters": max_characters }).to_string();
    let raw = match core.get_font_decision_trace_native(page, &options) {
        Ok(raw) => raw,
        Err(HwpError::PageOutOfRange(n)) => {
            eprintln!("오류: 페이지 {n}을(를) 찾을 수 없습니다");
            return Err(EXIT_RUNTIME);
        }
        Err(e) => {
            eprintln!("오류: 글꼴 결정 추적을 만들지 못했습니다 - {path}: {e}");
            return Err(EXIT_RUNTIME);
        }
    };
    match serde_json::from_str::<Value>(&raw) {
        Ok(trace) => Ok(trace),
        Err(e) => {
            eprintln!("오류: 글꼴 결정 추적 JSON 이 깨졌습니다 - {e}");
            Err(EXIT_RUNTIME)
        }
    }
}

fn envelope(path: &str, page: u32, trace: Value) -> Value {
    json!({
        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
        "tool": TOOL,
        "command": COMMAND,
        "version": rhwp::version(),
        "untrustedContent": true,
        "untrustedFields": UNTRUSTED_FIELDS,
        "source": path,
        "page": page,
        "trace": trace,
    })
}

fn print_text(path: &str, page: u32, trace: &Value) {
    let status = trace.get("status").and_then(Value::as_str).unwrap_or("?");
    let records = trace
        .get("counts")
        .and_then(|c| c.get("recordsEmitted"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let seen = trace
        .get("counts")
        .and_then(|c| c.get("charactersSeen"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    write_stdout(
        &format!(
            "font-trace source={path} page={page} status={status} recordsEmitted={records} charactersSeen={seen}"
        ),
        true,
    );
}

fn run(args: &[String]) -> i32 {
    let opts = match parse_args(args) {
        Ok(opts) => opts,
        Err(code) => return code,
    };
    if opts.help {
        write_stdout(USAGE, true);
        write_stdout(
            "쪽의 글꼴 결정 추적을 조회한다. 문서를 고치지 않는다. --page 는 0부터 세며 --max-characters 기본값은 1024다.",
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
    let page = opts.page.expect("parse_args 가 --page 를 확인한다");
    let trace = match load_trace(path, page, opts.max_characters) {
        Ok(trace) => trace,
        Err(code) => return code,
    };
    if opts.json {
        print_json(&envelope(path, page, trace));
    } else {
        print_text(path, page, &trace);
    }
    EXIT_OK
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    process::exit(run(&args));
}
