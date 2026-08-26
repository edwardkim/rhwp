//! Issue #4967: same-snapshot font trace, line frame and stored-row evidence.
//!
//! The command is read-only. `--page` is zero-based and the character bound is
//! shared with `rhwp-q-font-trace`.

use rhwp::document_core::DocumentCore;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;
use rhwp::HwpError;
use serde_json::{json, Value};
use std::io::Write;
use std::process;

const EXIT_OK: i32 = 0;
const EXIT_RUNTIME: i32 = 1;
const EXIT_USAGE: i32 = 2;
const TOOL: &str = "rhwp-q-font-layout-evidence";
const COMMAND: &str = "font-layout-evidence";
const DEFAULT_MAX_CHARACTERS: usize = 1024;
const MAX_CHARACTERS: usize = 4096;
const USAGE: &str =
    "사용법: rhwp-q-font-layout-evidence <파일> --page <N> [--max-characters <1..=4096>] [--json]";

#[derive(Debug)]
struct Options {
    json: bool,
    help: bool,
    version: bool,
    path: Option<String>,
    page: Option<u32>,
    max_characters: usize,
}

fn parse_args(args: &[String]) -> Result<Options, i32> {
    let mut options = Options {
        json: false,
        help: false,
        version: false,
        path: None,
        page: None,
        max_characters: DEFAULT_MAX_CHARACTERS,
    };
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" => {
                options.help = true;
                index += 1;
            }
            "--version" | "-V" => {
                options.version = true;
                index += 1;
            }
            "--json" => {
                options.json = true;
                index += 1;
            }
            "--page" => {
                let Some(raw) = args.get(index + 1) else {
                    return usage_error("--page 뒤에 0 이상의 정수가 필요합니다.");
                };
                options.page =
                    Some(raw.parse::<u32>().map_err(|_| {
                        usage_error_code("--page 뒤에 0 이상의 정수가 필요합니다.")
                    })?);
                index += 2;
            }
            "--max-characters" => {
                let Some(raw) = args.get(index + 1) else {
                    return usage_error("--max-characters 뒤에 1..=4096 정수가 필요합니다.");
                };
                let value = raw.parse::<usize>().map_err(|_| {
                    usage_error_code("--max-characters 뒤에 1..=4096 정수가 필요합니다.")
                })?;
                if !(1..=MAX_CHARACTERS).contains(&value) {
                    return usage_error("--max-characters 뒤에 1..=4096 정수가 필요합니다.");
                }
                options.max_characters = value;
                index += 2;
            }
            unknown if unknown.starts_with('-') => {
                return usage_error(&format!("알 수 없는 옵션입니다 - {unknown}"));
            }
            path => {
                if options.path.is_some() {
                    return usage_error(&format!("파일이 너무 많습니다 - {path}"));
                }
                options.path = Some(path.to_string());
                index += 1;
            }
        }
    }
    if !options.help && !options.version {
        if options.path.is_none() {
            return usage_error("파일 경로가 필요합니다.");
        }
        if options.page.is_none() {
            return usage_error("--page 가 필요합니다.");
        }
    }
    Ok(options)
}

fn usage_error(message: &str) -> Result<Options, i32> {
    Err(usage_error_code(message))
}

fn usage_error_code(message: &str) -> i32 {
    eprintln!("오류: {message}");
    eprintln!("{USAGE}");
    EXIT_USAGE
}

fn load_evidence(path: &str, page: u32, max_characters: usize) -> Result<Value, i32> {
    let bytes = std::fs::read(path).map_err(|error| {
        eprintln!("오류: 파일을 읽을 수 없습니다 - {path}: {error}");
        EXIT_RUNTIME
    })?;
    let core = DocumentCore::from_bytes(&bytes).map_err(|error| {
        eprintln!("오류: 문서를 열 수 없습니다 - {path}: {error}");
        EXIT_RUNTIME
    })?;
    let options = json!({ "maxCharacters": max_characters }).to_string();
    let raw = core
        .get_font_layout_evidence_native(page, &options)
        .map_err(|error| {
            match error {
                HwpError::PageOutOfRange(number) => {
                    eprintln!("오류: 페이지 {number}을(를) 찾을 수 없습니다")
                }
                other => eprintln!("오류: 결합 증거를 만들지 못했습니다 - {path}: {other}"),
            }
            EXIT_RUNTIME
        })?;
    serde_json::from_str(&raw).map_err(|error| {
        eprintln!("오류: 결합 증거 JSON 이 깨졌습니다 - {error}");
        EXIT_RUNTIME
    })
}

fn write_stdout(value: &str) {
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    if let Err(error) = writeln!(lock, "{value}") {
        eprintln!("오류: stdout 쓰기 실패 - {error}");
        process::exit(EXIT_RUNTIME);
    }
}

fn run(args: &[String]) -> i32 {
    let options = match parse_args(args) {
        Ok(options) => options,
        Err(code) => return code,
    };
    if options.help {
        write_stdout(USAGE);
        write_stdout("한 번 만든 쪽 tree에서 글꼴 추적·줄 frame·stored-row 판정을 함께 조회한다. 문서를 고치지 않는다.");
        write_stdout("종료 코드: 0 성공 · 1 실행 오류 · 2 사용법 오류");
        return EXIT_OK;
    }
    if options.version {
        write_stdout(&format!("{TOOL} v{}", rhwp::version()));
        return EXIT_OK;
    }
    let path = options.path.as_deref().expect("경로 검증 완료");
    let page = options.page.expect("쪽 검증 완료");
    let evidence = match load_evidence(path, page, options.max_characters) {
        Ok(evidence) => evidence,
        Err(code) => return code,
    };
    if options.json {
        let envelope = json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "tool": TOOL,
            "command": COMMAND,
            "version": rhwp::version(),
            "untrustedContent": true,
            "untrustedFields": ["source", "evidence.trace", "evidence.lines"],
            "source": path,
            "page": page,
            "evidence": evidence,
        });
        match serde_json::to_string_pretty(&envelope) {
            Ok(text) => write_stdout(&text),
            Err(error) => {
                eprintln!("오류: JSON 직렬화 실패 - {error}");
                return EXIT_RUNTIME;
            }
        }
    } else {
        write_stdout(&format!(
            "font-layout-evidence source={path} page={page} status={} lines={} runs={} unframedRuns={}",
            evidence["status"].as_str().unwrap_or("?"),
            evidence["counts"]["lines"].as_u64().unwrap_or(0),
            evidence["counts"]["runs"].as_u64().unwrap_or(0),
            evidence["counts"]["unframedRuns"].as_u64().unwrap_or(0),
        ));
    }
    EXIT_OK
}

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    process::exit(run(&args));
}
