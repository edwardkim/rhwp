//! 한글 GetTextFile 훑기 순서로 문서 글을 조회한다.
//!
//! 기본은 읽기 전용 `DocumentCore::text_file_unicode_json()` (`GetTextFile("UNICODE")`)
//! 이다. `--cp949` 이면 `text_file_json()` (`GetTextFile("TEXT")`) 를 부른다. 문서를
//! 고치지 않는다. `src/bin/*.rs` 자동 인식이라 Cargo.toml·본 CLI 를 건드리지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;
use serde_json::{json, Value};
use std::io::Write;
use std::process;

const EXIT_OK: i32 = 0;
const EXIT_RUNTIME: i32 = 1;
const EXIT_USAGE: i32 = 2;
const TOOL: &str = "rhwp-q-text-file";
const COMMAND: &str = "text-file";
const USAGE: &str = "rhwp-q-text-file <파일> [--json] [--cp949]";
const FORMAT_UNICODE: &str = "UNICODE";
const FORMAT_TEXT: &str = "TEXT";

#[derive(Debug)]
struct Options {
    json: bool,
    cp949: bool,
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
            "{TOOL} v{} — GetTextFile 훑기 순서 글 조회",
            rhwp::version()
        ),
        true,
    );
    write_stdout(&format!("사용법: {USAGE}"), true);
    write_stdout("", true);
    write_stdout("  --json    stdout 순수 JSON 봉투", true);
    write_stdout(
        "  --cp949   GetTextFile(\"TEXT\") — CP949 밖 글자를 &#N; 수치 참조로 바꾼다",
        true,
    );
    write_stdout("", true);
    write_stdout(
        "기본은 GetTextFile(\"UNICODE\") 원문이다. 문서를 고치지 않는다.",
        true,
    );
    write_stdout("종료 코드: 0 성공 · 1 실행 오류 · 2 사용법 오류", true);
}

fn parse_cli(args: &[String]) -> Result<Cli, i32> {
    let mut json = false;
    let mut cp949 = false;
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
            "--cp949" => {
                if cp949 {
                    eprintln!("오류: --cp949 를 두 번 지정했습니다.");
                    eprintln!("사용법: {USAGE}");
                    return Err(EXIT_USAGE);
                }
                cp949 = true;
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
    Ok(Cli::Run(Options { json, cp949, path }))
}

fn run(opts: &Options) -> i32 {
    let core = match open_core(&opts.path) {
        Ok(core) => core,
        Err(code) => return code,
    };
    let envelope = match text_file_envelope(&opts.path, opts.cp949, &core) {
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
        let format = envelope["format"].as_str().unwrap_or("");
        let char_count = envelope["charCount"].as_u64().unwrap_or(0);
        write_stdout(
            &format!(
                "format={format} cp949={} charCount={char_count}",
                opts.cp949
            ),
            true,
        );
        if let Some(text) = envelope["text"].as_str() {
            write_stdout(text, true);
        }
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

fn text_file_envelope(source: &str, cp949: bool, core: &DocumentCore) -> Result<Value, i32> {
    let raw = if cp949 {
        core.text_file_json()
    } else {
        core.text_file_unicode_json()
    };
    let text: String = match serde_json::from_str(&raw) {
        Ok(Value::String(text)) => text,
        Ok(_) => {
            eprintln!("오류: GetTextFile JSON 이 문자열이 아닙니다");
            return Err(EXIT_RUNTIME);
        }
        Err(e) => {
            eprintln!("오류: GetTextFile JSON 이 깨졌습니다 - {e}");
            return Err(EXIT_RUNTIME);
        }
    };
    let format = if cp949 { FORMAT_TEXT } else { FORMAT_UNICODE };
    let char_count = text.chars().count();
    Ok(json!({
        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
        "tool": TOOL,
        "command": COMMAND,
        "version": rhwp::version(),
        "untrustedContent": true,
        "untrustedFields": ["source", "text"],
        "source": source,
        "format": format,
        "cp949": cp949,
        "charCount": char_count,
        "text": text,
    }))
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
    fn form01_unicode_text_is_success() {
        let path = sample_form01();
        let source = path.to_str().expect("utf-8 path");
        let core = open_core(source).expect("open form-01.hwp");
        let envelope = text_file_envelope(source, false, &core).expect("query unicode");
        assert_eq!(envelope["tool"], TOOL);
        assert_eq!(envelope["command"], COMMAND);
        assert_eq!(envelope["schemaVersion"], ENVELOPE_SCHEMA_VERSION);
        assert_eq!(envelope["untrustedContent"], true);
        assert_eq!(envelope["untrustedFields"], json!(["source", "text"]));
        assert_eq!(envelope["format"], FORMAT_UNICODE);
        assert_eq!(envelope["cp949"], false);
        let text = envelope["text"].as_str().expect("text string");
        assert_eq!(envelope["charCount"], text.chars().count());
        assert!(!text.is_empty(), "form-01.hwp 본문이 비어 있다");
    }

    #[test]
    fn form01_cp949_text_is_success() {
        let path = sample_form01();
        let source = path.to_str().expect("utf-8 path");
        let core = open_core(source).expect("open form-01.hwp");
        let envelope = text_file_envelope(source, true, &core).expect("query cp949");
        assert_eq!(envelope["format"], FORMAT_TEXT);
        assert_eq!(envelope["cp949"], true);
        let text = envelope["text"].as_str().expect("text string");
        assert_eq!(envelope["charCount"], text.chars().count());
        assert!(!text.is_empty(), "form-01.hwp 본문이 비어 있다");
    }

    #[test]
    fn parse_cli_accepts_file_and_json() {
        let cli = parse_cli(&args(&["--json", "samples/form-01.hwp"])).expect("parse");
        match cli {
            Cli::Run(opts) => {
                assert!(opts.json);
                assert!(!opts.cp949);
                assert_eq!(opts.path, "samples/form-01.hwp");
            }
            _ => panic!("expected Run, got help/version variant"),
        }
    }

    #[test]
    fn parse_cli_cp949_may_follow_file() {
        let cli = parse_cli(&args(&["samples/form-01.hwp", "--cp949", "--json"])).expect("parse");
        match cli {
            Cli::Run(opts) => {
                assert!(opts.json);
                assert!(opts.cp949);
            }
            _ => panic!("expected Run"),
        }
    }

    #[test]
    fn parse_cli_missing_file_is_usage() {
        let err = parse_cli(&args(&["--json"])).unwrap_err();
        assert_eq!(err, EXIT_USAGE);
    }

    #[test]
    fn parse_cli_unknown_flag_is_usage() {
        let err = parse_cli(&args(&["samples/form-01.hwp", "--fill-fields"])).unwrap_err();
        assert_eq!(err, EXIT_USAGE);
    }

    #[test]
    fn parse_cli_rejects_duplicate_cp949() {
        let err = parse_cli(&args(&["samples/form-01.hwp", "--cp949", "--cp949"])).unwrap_err();
        assert_eq!(err, EXIT_USAGE);
    }

    #[test]
    fn missing_file_on_disk_is_runtime() {
        match open_core("samples/this-file-does-not-exist.hwp") {
            Err(err) => assert_eq!(err, EXIT_RUNTIME),
            Ok(_) => panic!("expected runtime error for missing file"),
        }
    }
}
