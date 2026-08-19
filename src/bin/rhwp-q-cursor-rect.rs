//! 캐럿 사각형을 조회하는 읽기 전용 CLI.
//!
//! 이미 있는 `DocumentCore::get_cursor_rect_native`를 부를 뿐이며 문서를
//! 고치지 않는다. `src/bin/*.rs` 자동 인식이라 Cargo.toml·본 CLI 는 그대로다.

use rhwp::document_core::DocumentCore;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;
use serde_json::{json, Value};
use std::io::Write;
use std::process;

const EXIT_OK: i32 = 0;
const EXIT_RUNTIME: i32 = 1;
const EXIT_USAGE: i32 = 2;
const TOOL: &str = "rhwp-q-cursor-rect";
const COMMAND: &str = "cursor-rect";
const USAGE: &str = "rhwp-q-cursor-rect <파일> --section <N> --para <N> --offset <N> [--json]";

#[derive(Debug)]
struct Options {
    json: bool,
    section: usize,
    para: usize,
    offset: usize,
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
            "{TOOL} v{} — 캐럿 사각형을 조회하는 읽기 전용 CLI",
            rhwp::version()
        ),
        true,
    );
    write_stdout(&format!("사용법: {USAGE}"), true);
    write_stdout("", true);
    write_stdout("  --section <N>  0부터 세는 구역 번호 (필수)", true);
    write_stdout("  --para <N>     0부터 세는 문단 번호 (필수)", true);
    write_stdout("  --offset <N>   0부터 세는 문자 오프셋 (필수)", true);
    write_stdout("  --json         stdout 순수 JSON 봉투", true);
    write_stdout("", true);
    write_stdout("종료 코드: 0 성공 · 1 실행 오류 · 2 사용법 오류", true);
    write_stdout("문서를 고치지 않는다. 편집 API 를 부르지 않는다.", true);
}

fn parse_cli(args: &[String]) -> Result<Cli, i32> {
    let mut json = false;
    let mut section: Option<usize> = None;
    let mut para: Option<usize> = None;
    let mut offset: Option<usize> = None;
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
            "--offset" => {
                offset = Some(parse_required_usize(args, i, "--offset")?);
                i += 2;
            }
            other if other.starts_with("--offset=") => {
                offset = Some(parse_equals_usize(other, "--offset")?);
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
    let Some(offset) = offset else {
        eprintln!("오류: --offset 가 필요합니다.");
        eprintln!("사용법: {USAGE}");
        return Err(EXIT_USAGE);
    };
    Ok(Cli::Run(Options {
        json,
        section,
        para,
        offset,
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
    let envelope =
        match cursor_rect_envelope(&opts.path, opts.section, opts.para, opts.offset, &core) {
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

fn cursor_rect_envelope(
    source: &str,
    section: usize,
    para: usize,
    offset: usize,
    core: &DocumentCore,
) -> Result<Value, i32> {
    let raw = match core.get_cursor_rect_native(section, para, offset) {
        Ok(raw) => raw,
        Err(e) => {
            eprintln!("오류: 캐럿 사각형을 조회하지 못했습니다 - {e}");
            return Err(EXIT_RUNTIME);
        }
    };
    let rect: Value = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("오류: 캐럿 사각형 JSON 이 깨졌습니다 - {e}");
            return Err(EXIT_RUNTIME);
        }
    };
    if !rect.is_object() {
        eprintln!("오류: 캐럿 사각형 JSON 이 객체가 아닙니다.");
        return Err(EXIT_RUNTIME);
    }
    Ok(json!({
        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
        "tool": TOOL,
        "command": COMMAND,
        "version": rhwp::version(),
        "untrustedContent": true,
        "untrustedFields": ["source", "rect"],
        "source": source,
        "section": section,
        "para": para,
        "offset": offset,
        "rect": rect,
    }))
}

fn print_text(envelope: &Value) {
    let page = envelope["rect"]["pageIndex"]
        .as_u64()
        .map(|n| n.to_string())
        .unwrap_or_else(|| "-".to_string());
    let x = envelope["rect"]["x"]
        .as_f64()
        .map(|n| format!("{n:.1}"))
        .unwrap_or_else(|| "-".to_string());
    let y = envelope["rect"]["y"]
        .as_f64()
        .map(|n| format!("{n:.1}"))
        .unwrap_or_else(|| "-".to_string());
    let height = envelope["rect"]["height"]
        .as_f64()
        .map(|n| format!("{n:.1}"))
        .unwrap_or_else(|| "-".to_string());
    write_stdout(
        &format!(
            "section={} para={} offset={} pageIndex={} x={} y={} height={}",
            envelope["section"], envelope["para"], envelope["offset"], page, x, y, height
        ),
        true,
    );
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_form01() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("samples/form-01.hwp")
    }

    fn args(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn form01_section0_para0_offset0_is_success() {
        let path = sample_form01();
        let core = open_core(path.to_str().expect("utf-8 path")).expect("open form-01.hwp");
        let envelope = cursor_rect_envelope(path.to_str().unwrap(), 0, 0, 0, &core)
            .expect("query section 0 para 0 offset 0");
        assert_eq!(envelope["tool"], TOOL);
        assert_eq!(envelope["command"], COMMAND);
        assert_eq!(envelope["schemaVersion"], ENVELOPE_SCHEMA_VERSION);
        assert_eq!(envelope["untrustedContent"], true);
        assert_eq!(envelope["untrustedFields"], json!(["source", "rect"]));
        assert_eq!(envelope["section"], 0);
        assert_eq!(envelope["para"], 0);
        assert_eq!(envelope["offset"], 0);
        let rect = envelope["rect"].as_object().expect("rect object");
        assert!(rect.get("pageIndex").and_then(Value::as_u64).is_some());
        assert!(rect.get("x").and_then(Value::as_f64).is_some());
        assert!(rect.get("y").and_then(Value::as_f64).is_some());
        assert!(rect.get("height").and_then(Value::as_f64).is_some());
    }

    #[test]
    fn parse_cli_accepts_file_and_flags_and_json() {
        let cli = parse_cli(&args(&[
            "--json",
            "--section",
            "0",
            "--para",
            "0",
            "--offset",
            "0",
            "samples/form-01.hwp",
        ]))
        .expect("parse");
        match cli {
            Cli::Run(opts) => {
                assert!(opts.json);
                assert_eq!(opts.section, 0);
                assert_eq!(opts.para, 0);
                assert_eq!(opts.offset, 0);
                assert_eq!(opts.path, "samples/form-01.hwp");
            }
            _ => panic!("expected Run, got help/version variant"),
        }
    }

    #[test]
    fn parse_cli_equals_form_may_follow_file() {
        let cli = parse_cli(&args(&[
            "samples/form-01.hwp",
            "--section=0",
            "--para=1",
            "--offset=2",
        ]))
        .expect("parse");
        match cli {
            Cli::Run(opts) => {
                assert!(!opts.json);
                assert_eq!(opts.section, 0);
                assert_eq!(opts.para, 1);
                assert_eq!(opts.offset, 2);
            }
            _ => panic!("expected Run"),
        }
    }

    #[test]
    fn parse_cli_missing_section_is_usage() {
        let err = parse_cli(&args(&[
            "samples/form-01.hwp",
            "--para",
            "0",
            "--offset",
            "0",
        ]))
        .unwrap_err();
        assert_eq!(err, EXIT_USAGE);
    }

    #[test]
    fn parse_cli_missing_para_is_usage() {
        let err = parse_cli(&args(&[
            "samples/form-01.hwp",
            "--section",
            "0",
            "--offset",
            "0",
        ]))
        .unwrap_err();
        assert_eq!(err, EXIT_USAGE);
    }

    #[test]
    fn parse_cli_missing_offset_is_usage() {
        let err = parse_cli(&args(&[
            "samples/form-01.hwp",
            "--section",
            "0",
            "--para",
            "0",
        ]))
        .unwrap_err();
        assert_eq!(err, EXIT_USAGE);
    }

    #[test]
    fn parse_cli_missing_file_is_usage() {
        let err =
            parse_cli(&args(&["--section", "0", "--para", "0", "--offset", "0"])).unwrap_err();
        assert_eq!(err, EXIT_USAGE);
    }

    #[test]
    fn parse_cli_unknown_flag_is_usage() {
        let err = parse_cli(&args(&[
            "samples/form-01.hwp",
            "--section",
            "0",
            "--para",
            "0",
            "--offset",
            "0",
            "--fill-fields",
        ]))
        .unwrap_err();
        assert_eq!(err, EXIT_USAGE);
    }

    #[test]
    fn parse_cli_rejects_negative_offset() {
        let err = parse_cli(&args(&[
            "samples/form-01.hwp",
            "--section",
            "0",
            "--para",
            "0",
            "--offset",
            "-1",
        ]))
        .unwrap_err();
        assert_eq!(err, EXIT_USAGE);
    }

    #[test]
    fn out_of_range_section_is_runtime() {
        let path = sample_form01();
        let core = open_core(path.to_str().unwrap()).expect("open form-01.hwp");
        let err = cursor_rect_envelope(path.to_str().unwrap(), 99, 0, 0, &core).unwrap_err();
        assert_eq!(err, EXIT_RUNTIME);
    }

    #[test]
    fn source_never_calls_mutators() {
        let src = include_str!("rhwp-q-cursor-rect.rs");
        let code = src.split("#[cfg(test)]").next().unwrap();
        for needle in [".apply_", ".insert_", ".delete_", ".set_"] {
            assert!(
                !code.contains(needle),
                "읽기 전용 CLI 가 {needle} 를 부르면 안 된다"
            );
        }
        assert!(code.contains("get_cursor_rect_native"));
        assert!(code.contains("from_bytes"));
    }
}
