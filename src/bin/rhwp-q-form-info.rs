//! 양식 개체 정보를 조회하는 읽기 전용 CLI.
//!
//! 이미 있는 `DocumentCore::get_form_object_info_native` 를 부를 뿐이며
//! 문서를 고치지 않는다. `src/bin/*.rs` 자동 인식이라 Cargo.toml·본 CLI 는
//! 그대로다.

use rhwp::document_core::DocumentCore;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;
use serde_json::{json, Value};
use std::io::Write;
use std::process;

const EXIT_OK: i32 = 0;
const EXIT_RUNTIME: i32 = 1;
const EXIT_USAGE: i32 = 2;
const TOOL: &str = "rhwp-q-form-info";
const COMMAND: &str = "form-info";
const UNTRUSTED_FIELDS: &[&str] = &["source", "form"];
const USAGE: &str = "rhwp-q-form-info <파일> --section <N> --para <N> --ci <N> [--json]";

#[derive(Debug)]
struct Options {
    json: bool,
    section: usize,
    para: usize,
    ci: usize,
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
        &format!(
            "{TOOL} v{} — 양식 개체 정보를 조회하는 읽기 전용 CLI",
            rhwp::version()
        ),
        true,
    );
    write_stdout(&format!("사용법: {USAGE}"), true);
    write_stdout("", true);
    write_stdout("  --section <N>  0부터 세는 구역 번호 (필수)", true);
    write_stdout("  --para <N>     0부터 세는 문단 번호 (필수)", true);
    write_stdout("  --ci <N>       0부터 세는 컨트롤 인덱스 (필수)", true);
    write_stdout("  --json         stdout 순수 JSON 봉투", true);
    write_stdout("", true);
    write_stdout("종료 코드: 0 성공 · 1 실행 오류 · 2 사용법 오류", true);
    write_stdout(
        "양식 개체가 없으면 found=false 로 종료 코드 0 이다. 게이트(3)를 쓰지 않는다.",
        true,
    );
    write_stdout("문서를 고치지 않는다. 편집 API 를 부르지 않는다.", true);
}

fn parse_cli(args: &[String]) -> Result<Cli, i32> {
    let mut json = false;
    let mut section: Option<usize> = None;
    let mut para: Option<usize> = None;
    let mut ci: Option<usize> = None;
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
                section = Some(parse_required_usize(args, i, "--section")?);
                i += 2;
            }
            other if other.starts_with("--section=") => {
                section = Some(parse_equals_usize(other, "--section")?);
                i += 1;
            }
            "--para" => {
                para = Some(parse_required_usize(args, i, "--para")?);
                i += 2;
            }
            other if other.starts_with("--para=") => {
                para = Some(parse_equals_usize(other, "--para")?);
                i += 1;
            }
            "--ci" => {
                ci = Some(parse_required_usize(args, i, "--ci")?);
                i += 2;
            }
            other if other.starts_with("--ci=") => {
                ci = Some(parse_equals_usize(other, "--ci")?);
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
    let Some(para) = para else {
        eprintln!("오류: --para 가 필요합니다.");
        eprintln!("사용법: {USAGE}");
        return Err(EXIT_USAGE);
    };
    let Some(ci) = ci else {
        eprintln!("오류: --ci 가 필요합니다.");
        eprintln!("사용법: {USAGE}");
        return Err(EXIT_USAGE);
    };
    Ok(Cli::Run(Options {
        json,
        section,
        para,
        ci,
        path,
    }))
}

fn parse_required_usize(args: &[String], i: usize, flag: &str) -> Result<usize, i32> {
    let Some(raw) = args.get(i + 1) else {
        eprintln!("오류: {flag} 뒤에 0부터 세는 번호가 필요합니다.");
        eprintln!("사용법: {USAGE}");
        return Err(EXIT_USAGE);
    };
    parse_usize_value(raw, flag)
}

fn parse_equals_usize(arg: &str, flag: &str) -> Result<usize, i32> {
    let raw = &arg[flag.len() + 1..];
    parse_usize_value(raw, flag)
}

fn parse_usize_value(raw: &str, flag: &str) -> Result<usize, i32> {
    match raw.parse::<usize>() {
        Ok(n) => Ok(n),
        Err(_) => {
            eprintln!("오류: {flag} 뒤에 0부터 세는 번호가 필요합니다.");
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
    let envelope = match form_info_envelope(&opts.path, opts.section, opts.para, opts.ci, &core) {
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
        print_text(&envelope);
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

fn form_info_envelope(
    source: &str,
    section: usize,
    para: usize,
    ci: usize,
    core: &DocumentCore,
) -> Result<Value, i32> {
    let raw = match core.get_form_object_info_native(section, para, ci) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("오류: 양식 개체 정보를 읽지 못했습니다 - {e}");
            return Err(EXIT_RUNTIME);
        }
    };
    let form: Value = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("오류: 양식 개체 JSON 이 깨졌습니다 - {e}");
            return Err(EXIT_RUNTIME);
        }
    };
    if !form.is_object() {
        eprintln!("오류: 양식 개체 JSON 이 객체가 아닙니다.");
        return Err(EXIT_RUNTIME);
    }
    let found = form.get("ok").and_then(Value::as_bool).unwrap_or(false);
    Ok(json!({
        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
        "tool": TOOL,
        "command": COMMAND,
        "version": rhwp::version(),
        "untrustedContent": true,
        "untrustedFields": UNTRUSTED_FIELDS,
        "source": source,
        "section": section,
        "para": para,
        "ci": ci,
        "found": found,
        "form": form,
    }))
}

fn print_text(envelope: &Value) {
    let found = envelope["found"].as_bool().unwrap_or(false);
    if !found {
        write_stdout(
            &format!(
                "section={} para={} ci={} found=false",
                envelope["section"], envelope["para"], envelope["ci"]
            ),
            true,
        );
        return;
    }
    let form_type = display_json(&envelope["form"]["formType"]);
    let name = display_json(&envelope["form"]["name"]);
    let value = display_json(&envelope["form"]["value"]);
    let text = display_json(&envelope["form"]["text"]);
    let caption = display_json(&envelope["form"]["caption"]);
    let enabled = display_json(&envelope["form"]["enabled"]);
    write_stdout(
        &format!(
            "section={} para={} ci={} found=true formType={} name={} value={} text={} caption={} enabled={}",
            envelope["section"],
            envelope["para"],
            envelope["ci"],
            form_type,
            name,
            value,
            text,
            caption,
            enabled
        ),
        true,
    );
}

fn display_json(value: &Value) -> String {
    match value {
        Value::Null => "-".to_string(),
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
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
