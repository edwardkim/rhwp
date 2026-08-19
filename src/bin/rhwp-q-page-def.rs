//! 한 구역의 용지 설정(폭·높이·여백, HWPUNIT)만 조회한다.
//!
//! 기존 읽기 전용 `DocumentCore::get_page_def_native` 를 호출할 뿐이며 문서를
//! 고치지 않는다. `src/bin/*.rs` 자동 인식이라 Cargo.toml·본 CLI 를 건드리지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;
use serde_json::{json, Value};
use std::io::Write;
use std::process;

const EXIT_OK: i32 = 0;
const EXIT_RUNTIME: i32 = 1;
const EXIT_USAGE: i32 = 2;
const TOOL: &str = "rhwp-q-page-def";
const COMMAND: &str = "page-def";
const USAGE: &str = "rhwp-q-page-def <파일> --section <N> [--json]";

#[derive(Debug)]
struct Options {
    json: bool,
    section: usize,
    path: String,
}

#[derive(Debug)]
enum Cli {
    Help,
    Version,
    Run(Options),
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match parse_cli(&args) {
        Ok(Cli::Help) => {
            print_help();
            EXIT_OK
        }
        Ok(Cli::Version) => {
            write_stdout(&format!("{TOOL} v{}", rhwp::version()), true);
            EXIT_OK
        }
        Ok(Cli::Run(opts)) => run(&opts),
        Err(code) => code,
    };
    process::exit(code);
}

fn print_help() {
    write_stdout(
        &format!("{TOOL} v{} — 구역 용지 설정 조회", rhwp::version()),
        true,
    );
    write_stdout(&format!("사용법: {USAGE}"), true);
    write_stdout("", true);
    write_stdout("  --section <N>  0부터 세는 구역 번호 (필수)", true);
    write_stdout("  --json         stdout 순수 JSON 봉투", true);
    write_stdout("", true);
    write_stdout("종료 코드: 0 성공 · 1 실행 오류 · 2 사용법 오류", true);
    write_stdout("폭·높이·여백은 HWPUNIT 이다. 문서를 고치지 않는다.", true);
}

fn parse_cli(args: &[String]) -> Result<Cli, i32> {
    let mut json = false;
    let mut section: Option<usize> = None;
    let mut path: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "--help" | "-h" => return Ok(Cli::Help),
            "--version" | "-V" => return Ok(Cli::Version),
            "--json" => {
                json = true;
                i += 1;
            }
            "--section" => {
                let Some(raw) = args.get(i + 1) else {
                    eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다.");
                    eprintln!("사용법: {USAGE}");
                    return Err(EXIT_USAGE);
                };
                let n = parse_section(raw)?;
                if section.is_some() {
                    eprintln!("오류: --section 를 두 번 지정했습니다.");
                    eprintln!("사용법: {USAGE}");
                    return Err(EXIT_USAGE);
                }
                section = Some(n);
                i += 2;
            }
            other if other.starts_with("--section=") => {
                let raw = &other["--section=".len()..];
                let n = parse_section(raw)?;
                if section.is_some() {
                    eprintln!("오류: --section 를 두 번 지정했습니다.");
                    eprintln!("사용법: {USAGE}");
                    return Err(EXIT_USAGE);
                }
                section = Some(n);
                i += 1;
            }
            other if other.starts_with('-') => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {other}");
                eprintln!("사용법: {USAGE}");
                return Err(EXIT_USAGE);
            }
            other => {
                if path.is_some() {
                    eprintln!("오류: 파일이 너무 많습니다 - {other}");
                    eprintln!("사용법: {USAGE}");
                    return Err(EXIT_USAGE);
                }
                path = Some(other.to_string());
                i += 1;
            }
        }
    }
    let Some(path) = path else {
        eprintln!("오류: 파일 경로가 필요합니다.");
        eprintln!("사용법: {USAGE}");
        return Err(EXIT_USAGE);
    };
    let Some(section) = section else {
        eprintln!("오류: --section 가 필요합니다.");
        eprintln!("사용법: {USAGE}");
        return Err(EXIT_USAGE);
    };
    Ok(Cli::Run(Options {
        json,
        section,
        path,
    }))
}

fn parse_section(raw: &str) -> Result<usize, i32> {
    match raw.parse::<usize>() {
        Ok(n) => Ok(n),
        Err(_) => {
            eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다.");
            eprintln!("사용법: {USAGE}");
            Err(EXIT_USAGE)
        }
    }
}

fn run(opts: &Options) -> i32 {
    let core = match open_core(&opts.path) {
        Ok(core) => core,
        Err(code) => return code,
    };
    let envelope = match page_def_envelope(&opts.path, opts.section, &core) {
        Ok(value) => value,
        Err(code) => return code,
    };
    if opts.json {
        match serde_json::to_string_pretty(&envelope) {
            Ok(text) => write_stdout(&text, true),
            Err(e) => {
                eprintln!("오류: JSON 직렬화 실패 - {e}");
                return EXIT_RUNTIME;
            }
        }
    } else {
        write_stdout(
            &format!(
                "section={} width={} height={} marginLeft={} marginRight={} marginTop={} marginBottom={}",
                opts.section,
                envelope["width"],
                envelope["height"],
                envelope["marginLeft"],
                envelope["marginRight"],
                envelope["marginTop"],
                envelope["marginBottom"]
            ),
            true,
        );
    }
    EXIT_OK
}

fn open_core(path: &str) -> Result<DocumentCore, i32> {
    let data = match std::fs::read(path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {path}: {e}");
            return Err(EXIT_RUNTIME);
        }
    };
    DocumentCore::from_bytes(&data).map_err(|e| {
        eprintln!("오류: 문서를 열 수 없습니다 - {path}: {e}");
        EXIT_RUNTIME
    })
}

fn page_def_envelope(source: &str, section: usize, core: &DocumentCore) -> Result<Value, i32> {
    let raw = match core.get_page_def_native(section) {
        Ok(raw) => raw,
        Err(e) => {
            eprintln!("오류: 구역 {section} 용지 설정을 읽지 못했습니다 - {e}");
            return Err(EXIT_RUNTIME);
        }
    };
    let native: Value = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("오류: 구역 용지 설정 JSON 이 깨졌습니다 - {e}");
            return Err(EXIT_RUNTIME);
        }
    };
    if native.get("width").and_then(Value::as_u64).is_none()
        || native.get("height").and_then(Value::as_u64).is_none()
    {
        eprintln!("오류: 구역 용지 설정 JSON 이 깨졌습니다 - width/height 가 없습니다");
        return Err(EXIT_RUNTIME);
    }
    let mut envelope = json!({
        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
        "tool": TOOL,
        "command": COMMAND,
        "version": rhwp::version(),
        "untrustedContent": true,
        "untrustedFields": [
            "source",
            "width",
            "height",
            "marginLeft",
            "marginRight",
            "marginTop",
            "marginBottom",
            "marginHeader",
            "marginFooter",
            "marginGutter",
            "landscape",
            "binding"
        ],
        "source": source,
        "section": section,
        "sectionCount": core.document().sections.len(),
        "units": "HWPUNIT",
    });
    if let (Some(obj), Some(native_obj)) = (envelope.as_object_mut(), native.as_object()) {
        for (key, value) in native_obj {
            obj.insert(key.clone(), value.clone());
        }
    }
    Ok(envelope)
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
