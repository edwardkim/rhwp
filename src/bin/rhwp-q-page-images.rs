//! 한 쪽이 그리는 원본 그림의 신원 키만 조회한다.
//!
//! 기존 `DocumentCore::get_page_source_image_keys_native` 를 호출할 뿐이며 문서를
//! 고치지 않는다. `src/bin/*.rs` 자동 인식이라 Cargo.toml·본 CLI 를 건드리지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;
use serde_json::{json, Value};
use std::io::Write;
use std::process;

const EXIT_OK: i32 = 0;
const EXIT_RUNTIME: i32 = 1;
const EXIT_USAGE: i32 = 2;
const TOOL: &str = "rhwp-q-page-images";
const COMMAND: &str = "page-images";
const USAGE: &str = "rhwp-q-page-images <파일> --page <N> [--json]";

#[derive(Debug)]
struct Options {
    json: bool,
    page: u32,
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
        &format!("{TOOL} v{} — 쪽 원본 그림 키 조회", rhwp::version()),
        true,
    );
    write_stdout(&format!("사용법: {USAGE}"), true);
    write_stdout("", true);
    write_stdout("  --page <N>   0부터 세는 쪽 번호 (필수)", true);
    write_stdout("  --json       stdout 순수 JSON 봉투", true);
    write_stdout("", true);
    write_stdout("종료 코드: 0 성공 · 1 실행 오류 · 2 사용법 오류", true);
    write_stdout("빈 keys 배열은 성공이다. 문서를 고치지 않는다.", true);
}

fn parse_cli(args: &[String]) -> Result<Cli, i32> {
    let mut json = false;
    let mut page: Option<u32> = None;
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
            "--page" => {
                let Some(raw) = args.get(i + 1) else {
                    eprintln!("오류: --page 뒤에 0 이상의 정수가 필요합니다.");
                    eprintln!("사용법: {USAGE}");
                    return Err(EXIT_USAGE);
                };
                let Ok(n) = raw.parse::<u32>() else {
                    eprintln!("오류: --page 뒤에 0 이상의 정수가 필요합니다.");
                    eprintln!("사용법: {USAGE}");
                    return Err(EXIT_USAGE);
                };
                if page.is_some() {
                    eprintln!("오류: --page 를 두 번 지정했습니다.");
                    eprintln!("사용법: {USAGE}");
                    return Err(EXIT_USAGE);
                }
                page = Some(n);
                i += 2;
            }
            other if other.starts_with("--page=") => {
                let raw = &other["--page=".len()..];
                let Ok(n) = raw.parse::<u32>() else {
                    eprintln!("오류: --page 뒤에 0 이상의 정수가 필요합니다.");
                    eprintln!("사용법: {USAGE}");
                    return Err(EXIT_USAGE);
                };
                if page.is_some() {
                    eprintln!("오류: --page 를 두 번 지정했습니다.");
                    eprintln!("사용법: {USAGE}");
                    return Err(EXIT_USAGE);
                }
                page = Some(n);
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
    let Some(page) = page else {
        eprintln!("오류: --page 가 필요합니다.");
        eprintln!("사용법: {USAGE}");
        return Err(EXIT_USAGE);
    };
    Ok(Cli::Run(Options { json, page, path }))
}

fn run(opts: &Options) -> i32 {
    let core = match open_core(&opts.path) {
        Ok(core) => core,
        Err(code) => return code,
    };
    let envelope = match page_images_envelope(&opts.path, opts.page, &core) {
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
        let cacheable = envelope["cacheable"].as_bool().unwrap_or(false);
        let key_count = envelope["keyCount"].as_u64().unwrap_or(0);
        write_stdout(
            &format!(
                "page={} cacheable={} keyCount={}",
                opts.page, cacheable, key_count
            ),
            true,
        );
        if let Some(keys) = envelope["keys"].as_array() {
            for key in keys {
                match key.as_str() {
                    Some(s) => write_stdout(s, true),
                    None if key.is_null() => write_stdout("null", true),
                    None => write_stdout(&key.to_string(), true),
                }
            }
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

fn page_images_envelope(source: &str, page: u32, core: &DocumentCore) -> Result<Value, i32> {
    let raw = match core.get_page_source_image_keys_native(page) {
        Ok(raw) => raw,
        Err(e) => {
            eprintln!("오류: 쪽 원본 그림 키를 읽지 못했습니다 - {e}");
            return Err(EXIT_RUNTIME);
        }
    };
    let native: Value = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("오류: 쪽 원본 그림 키 JSON 이 깨졌습니다 - {e}");
            return Err(EXIT_RUNTIME);
        }
    };
    let keys = match native.get("keys") {
        None => json!([]),
        Some(value) if value.is_array() => value.clone(),
        Some(_) => {
            eprintln!("오류: 쪽 원본 그림 키 JSON 이 깨졌습니다 - keys 가 배열이 아닙니다");
            return Err(EXIT_RUNTIME);
        }
    };
    let cacheable = native
        .get("cacheable")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let key_count = keys.as_array().map(Vec::len).unwrap_or(0);
    Ok(json!({
        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
        "tool": TOOL,
        "command": COMMAND,
        "version": rhwp::version(),
        "untrustedContent": true,
        "untrustedFields": ["source", "keys"],
        "source": source,
        "page": page,
        "pageCount": core.page_count(),
        "cacheable": cacheable,
        "keyCount": key_count,
        "keys": keys,
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
    fn form01_page0_empty_keys_is_success() {
        let path = sample_form01();
        let core = open_core(path.to_str().expect("utf-8 path")).expect("open form-01.hwp");
        let envelope =
            page_images_envelope(path.to_str().unwrap(), 0, &core).expect("query page 0");
        assert_eq!(envelope["tool"], TOOL);
        assert_eq!(envelope["command"], COMMAND);
        assert_eq!(envelope["schemaVersion"], ENVELOPE_SCHEMA_VERSION);
        assert_eq!(envelope["untrustedContent"], true);
        assert_eq!(envelope["untrustedFields"], json!(["source", "keys"]));
        assert_eq!(envelope["page"], 0);
        let keys = envelope["keys"].as_array().expect("keys array");
        assert_eq!(envelope["keyCount"], keys.len());
        // 그림이 없어도 빈 배열은 성공이다.
    }

    #[test]
    fn parse_cli_accepts_file_and_page_and_json() {
        let cli =
            parse_cli(&args(&["--json", "--page", "0", "samples/form-01.hwp"])).expect("parse");
        match cli {
            Cli::Run(opts) => {
                assert!(opts.json);
                assert_eq!(opts.page, 0);
                assert_eq!(opts.path, "samples/form-01.hwp");
            }
            _ => panic!("expected Run, got help/version variant"),
        }
    }

    #[test]
    fn parse_cli_page_may_follow_file() {
        let cli = parse_cli(&args(&["samples/form-01.hwp", "--page=0"])).expect("parse");
        match cli {
            Cli::Run(opts) => {
                assert!(!opts.json);
                assert_eq!(opts.page, 0);
            }
            _ => panic!("expected Run"),
        }
    }

    #[test]
    fn parse_cli_missing_page_is_usage() {
        let err = parse_cli(&args(&["samples/form-01.hwp"])).unwrap_err();
        assert_eq!(err, EXIT_USAGE);
    }

    #[test]
    fn parse_cli_missing_file_is_usage() {
        let err = parse_cli(&args(&["--page", "0"])).unwrap_err();
        assert_eq!(err, EXIT_USAGE);
    }

    #[test]
    fn parse_cli_unknown_flag_is_usage() {
        let err = parse_cli(&args(&[
            "samples/form-01.hwp",
            "--page",
            "0",
            "--fill-fields",
        ]))
        .unwrap_err();
        assert_eq!(err, EXIT_USAGE);
    }

    #[test]
    fn parse_cli_rejects_negative_page() {
        let err = parse_cli(&args(&["samples/form-01.hwp", "--page", "-1"])).unwrap_err();
        assert_eq!(err, EXIT_USAGE);
    }

    #[test]
    fn out_of_range_page_is_runtime() {
        let path = sample_form01();
        let core = open_core(path.to_str().unwrap()).expect("open form-01.hwp");
        let err = page_images_envelope(path.to_str().unwrap(), 99, &core).unwrap_err();
        assert_eq!(err, EXIT_RUNTIME);
    }
}
