//! 한글 구역 시작 문단 조회 — `MoveSectionUp`·`MoveSectionDown` 이 딛는 본문 문단 번호.
//!
//! 기존 읽기 전용 [`DocumentCore::section_starts_json`] 만 부른다. 문서를 고치지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;
use serde_json::{json, Value};
use std::io::Write;
use std::process;

const EXIT_OK: i32 = 0;
const EXIT_RUNTIME: i32 = 1;
const EXIT_USAGE: i32 = 2;
const TOOL: &str = "rhwp-q-section-starts";
const COMMAND: &str = "section-starts";
const UNTRUSTED_FIELDS: &[&str] = &["source", "starts"];
const USAGE: &str = "사용법: rhwp-q-section-starts <파일> [--json]";

#[derive(Debug)]
struct Options {
    json: bool,
    help: bool,
    version: bool,
    path: Option<String>,
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
    };
    for arg in args {
        match arg.as_str() {
            "--help" | "-h" => opts.help = true,
            "--version" | "-V" => opts.version = true,
            "--json" => opts.json = true,
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

fn load_starts(path: &str) -> Result<Vec<Value>, i32> {
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
    let raw = core.section_starts_json();
    match serde_json::from_str::<Value>(&raw) {
        Ok(Value::Array(starts)) => Ok(starts),
        Ok(_) => {
            eprintln!("오류: 구역 시작 JSON 이 배열이 아닙니다.");
            Err(EXIT_RUNTIME)
        }
        Err(e) => {
            eprintln!("오류: 구역 시작 JSON 이 깨졌습니다 - {e}");
            Err(EXIT_RUNTIME)
        }
    }
}

fn envelope(path: &str, starts: Vec<Value>) -> Value {
    json!({
        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
        "tool": TOOL,
        "command": COMMAND,
        "version": rhwp::version(),
        "untrustedContent": true,
        "untrustedFields": UNTRUSTED_FIELDS,
        "source": path,
        "startCount": starts.len(),
        "starts": starts,
    })
}

fn print_text(path: &str, starts: &[Value]) {
    write_stdout(
        &format!("section-starts source={path} startCount={}", starts.len()),
        true,
    );
    for start in starts {
        let index = start
            .as_u64()
            .map(|n| n.to_string())
            .unwrap_or_else(|| start.to_string());
        write_stdout(&index, true);
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
            "한글 구역 시작 본문 문단 번호를 조회한다. 문서를 고치지 않는다.",
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
    let starts = match load_starts(path) {
        Ok(starts) => starts,
        Err(code) => return code,
    };
    if opts.json {
        print_json(&envelope(path, starts));
    } else {
        print_text(path, &starts);
    }
    EXIT_OK
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    process::exit(run(&args));
}
