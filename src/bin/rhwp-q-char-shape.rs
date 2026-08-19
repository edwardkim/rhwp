//! 커서 자리 CharShape를 조회하는 읽기 전용 CLI.
//!
//! 이미 있는 `DocumentCore::char_shape_set_json`를 부를 뿐이며 문서를
//! 고치지 않는다. `src/bin/*.rs` 자동 인식이라 Cargo.toml·본 CLI 는 그대로다.

use rhwp::document_core::DocumentCore;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;
use serde_json::{json, Value};
use std::io::Write;
use std::process;

const EXIT_OK: i32 = 0;
const EXIT_RUNTIME: i32 = 1;
const EXIT_USAGE: i32 = 2;
const TOOL: &str = "rhwp-q-char-shape";
const COMMAND: &str = "char-shape";
const UNTRUSTED_FIELDS: &[&str] = &["source", "charShape"];
const USAGE: &str = "rhwp-q-char-shape <파일> --list <N> --para <N> --pos <N> [--json]";

#[derive(Debug)]
struct Options {
    json: bool,
    list: u32,
    para: usize,
    pos: usize,
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
            "{TOOL} v{} — 커서 자리 CharShape를 조회하는 읽기 전용 CLI",
            rhwp::version()
        ),
        true,
    );
    write_stdout(&format!("사용법: {USAGE}"), true);
    write_stdout("", true);
    write_stdout("  --list <N>  0부터 세는 리스트 번호 (필수)", true);
    write_stdout("  --para <N>  0부터 세는 문단 번호 (필수)", true);
    write_stdout("  --pos <N>   0부터 세는 스트림 위치 (필수)", true);
    write_stdout("  --json      stdout 순수 JSON 봉투", true);
    write_stdout("", true);
    write_stdout("종료 코드: 0 성공 · 1 실행 오류 · 2 사용법 오류", true);
    write_stdout("문서를 고치지 않는다. 편집 API 를 부르지 않는다.", true);
    write_stdout("없는 자리의 셋은 빈 객체다.", true);
}

fn parse_cli(args: &[String]) -> Result<Cli, i32> {
    let mut json = false;
    let mut list: Option<u32> = None;
    let mut para: Option<usize> = None;
    let mut pos: Option<usize> = None;
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
            "--list" => {
                list = Some(parse_required_u32(args, i, "--list")?);
                i += 2;
            }
            other if other.starts_with("--list=") => {
                list = Some(parse_equals_u32(other, "--list")?);
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
            "--pos" => {
                pos = Some(parse_required_usize(args, i, "--pos")?);
                i += 2;
            }
            other if other.starts_with("--pos=") => {
                pos = Some(parse_equals_usize(other, "--pos")?);
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
    let Some(list) = list else {
        eprintln!("오류: --list 가 필요합니다.");
        eprintln!("사용법: {USAGE}");
        return Err(EXIT_USAGE);
    };
    let Some(para) = para else {
        eprintln!("오류: --para 가 필요합니다.");
        eprintln!("사용법: {USAGE}");
        return Err(EXIT_USAGE);
    };
    let Some(pos) = pos else {
        eprintln!("오류: --pos 가 필요합니다.");
        eprintln!("사용법: {USAGE}");
        return Err(EXIT_USAGE);
    };
    Ok(Cli::Run(Options {
        json,
        list,
        para,
        pos,
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

fn parse_required_u32(args: &[String], i: usize, flag: &str) -> Result<u32, i32> {
    let Some(raw) = args.get(i + 1) else {
        eprintln!("오류: {flag} 뒤에 0부터 세는 번호가 필요합니다.");
        eprintln!("사용법: {USAGE}");
        return Err(EXIT_USAGE);
    };
    parse_u32_value(raw, flag)
}

fn parse_equals_u32(arg: &str, flag: &str) -> Result<u32, i32> {
    let raw = &arg[flag.len() + 1..];
    parse_u32_value(raw, flag)
}

fn parse_u32_value(raw: &str, flag: &str) -> Result<u32, i32> {
    match raw.parse::<u32>() {
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
    let envelope = match char_shape_envelope(&opts.path, opts.list, opts.para, opts.pos, &core) {
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

fn char_shape_envelope(
    source: &str,
    list: u32,
    para: usize,
    pos: usize,
    core: &DocumentCore,
) -> Result<Value, i32> {
    let raw = core.char_shape_set_json(list, para, pos);
    let char_shape: Value = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("오류: CharShape JSON 이 깨졌습니다 - {e}");
            return Err(EXIT_RUNTIME);
        }
    };
    if !char_shape.is_object() {
        eprintln!("오류: CharShape JSON 이 객체가 아닙니다.");
        return Err(EXIT_RUNTIME);
    }
    Ok(json!({
        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
        "tool": TOOL,
        "command": COMMAND,
        "version": rhwp::version(),
        "untrustedContent": true,
        "untrustedFields": UNTRUSTED_FIELDS,
        "source": source,
        "list": list,
        "para": para,
        "pos": pos,
        "charShape": char_shape,
    }))
}

fn print_text(envelope: &Value) {
    let shape = envelope["charShape"].as_object();
    let empty = shape.map(|m| m.is_empty()).unwrap_or(true);
    if empty {
        write_stdout(
            &format!(
                "list={} para={} pos={} (empty)",
                envelope["list"], envelope["para"], envelope["pos"]
            ),
            true,
        );
        return;
    }
    let height = display_json(&envelope["charShape"]["Height"]);
    let bold = display_json(&envelope["charShape"]["Bold"]);
    let italic = display_json(&envelope["charShape"]["Italic"]);
    let face = display_json(&envelope["charShape"]["FaceNameHangul"]);
    write_stdout(
        &format!(
            "list={} para={} pos={} Height={} Bold={} Italic={} FaceNameHangul={}",
            envelope["list"], envelope["para"], envelope["pos"], height, bold, italic, face
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
    fn form01_list0_para0_pos0_is_success() {
        let path = sample_form01();
        let source = path.to_str().expect("utf-8 path");
        let core = open_core(source).expect("open form-01.hwp");
        let envelope =
            char_shape_envelope(source, 0, 0, 0, &core).expect("query list 0 para 0 pos 0");
        assert_eq!(envelope["tool"], TOOL);
        assert_eq!(envelope["command"], COMMAND);
        assert_eq!(envelope["schemaVersion"], ENVELOPE_SCHEMA_VERSION);
        assert_eq!(envelope["untrustedContent"], true);
        assert_eq!(envelope["untrustedFields"], json!(["source", "charShape"]));
        assert_eq!(envelope["list"], 0);
        assert_eq!(envelope["para"], 0);
        assert_eq!(envelope["pos"], 0);
        assert_eq!(envelope["source"], source);
        let shape = envelope["charShape"].as_object().expect("charShape object");
        assert!(
            !shape.is_empty(),
            "form-01 0/0/0 CharShape 가 비면 안 된다: {shape:?}"
        );
        assert!(shape.get("Height").and_then(Value::as_i64).is_some());
        assert!(shape.contains_key("Bold"));
        assert!(shape.contains_key("Italic"));
    }

    #[test]
    fn missing_cursor_is_empty_success() {
        let path = sample_form01();
        let source = path.to_str().unwrap();
        let core = open_core(source).expect("open form-01.hwp");
        let envelope =
            char_shape_envelope(source, 0, 99, 0, &core).expect("missing para is empty set");
        let shape = envelope["charShape"].as_object().expect("charShape object");
        assert!(shape.is_empty(), "없는 자리의 셋은 빈 객체다: {shape:?}");
    }

    #[test]
    fn parse_cli_accepts_file_and_flags_and_json() {
        let cli = parse_cli(&args(&[
            "--json",
            "--list",
            "0",
            "--para",
            "0",
            "--pos",
            "0",
            "samples/form-01.hwp",
        ]))
        .expect("parse");
        match cli {
            Cli::Run(opts) => {
                assert!(opts.json);
                assert_eq!(opts.list, 0);
                assert_eq!(opts.para, 0);
                assert_eq!(opts.pos, 0);
                assert_eq!(opts.path, "samples/form-01.hwp");
            }
            _ => panic!("expected Run, got help/version variant"),
        }
    }

    #[test]
    fn parse_cli_equals_form_may_follow_file() {
        let cli = parse_cli(&args(&[
            "samples/form-01.hwp",
            "--list=0",
            "--para=1",
            "--pos=2",
        ]))
        .expect("parse");
        match cli {
            Cli::Run(opts) => {
                assert!(!opts.json);
                assert_eq!(opts.list, 0);
                assert_eq!(opts.para, 1);
                assert_eq!(opts.pos, 2);
            }
            _ => panic!("expected Run"),
        }
    }

    #[test]
    fn parse_cli_missing_list_is_usage() {
        let err =
            parse_cli(&args(&["samples/form-01.hwp", "--para", "0", "--pos", "0"])).unwrap_err();
        assert_eq!(err, EXIT_USAGE);
    }

    #[test]
    fn parse_cli_missing_para_is_usage() {
        let err =
            parse_cli(&args(&["samples/form-01.hwp", "--list", "0", "--pos", "0"])).unwrap_err();
        assert_eq!(err, EXIT_USAGE);
    }

    #[test]
    fn parse_cli_missing_pos_is_usage() {
        let err = parse_cli(&args(&[
            "samples/form-01.hwp",
            "--list",
            "0",
            "--para",
            "0",
        ]))
        .unwrap_err();
        assert_eq!(err, EXIT_USAGE);
    }

    #[test]
    fn parse_cli_missing_file_is_usage() {
        let err = parse_cli(&args(&["--list", "0", "--para", "0", "--pos", "0"])).unwrap_err();
        assert_eq!(err, EXIT_USAGE);
    }

    #[test]
    fn parse_cli_unknown_flag_is_usage() {
        let err = parse_cli(&args(&[
            "samples/form-01.hwp",
            "--list",
            "0",
            "--para",
            "0",
            "--pos",
            "0",
            "--fill-fields",
        ]))
        .unwrap_err();
        assert_eq!(err, EXIT_USAGE);
    }

    #[test]
    fn parse_cli_rejects_negative_pos() {
        let err = parse_cli(&args(&[
            "samples/form-01.hwp",
            "--list",
            "0",
            "--para",
            "0",
            "--pos",
            "-1",
        ]))
        .unwrap_err();
        assert_eq!(err, EXIT_USAGE);
    }

    #[test]
    fn parse_cli_list_without_value_is_usage() {
        let err = parse_cli(&args(&["samples/form-01.hwp", "--list", "--para", "0"])).unwrap_err();
        assert_eq!(err, EXIT_USAGE);
    }

    #[test]
    fn parse_cli_extra_path_is_usage() {
        let err = parse_cli(&args(&[
            "samples/form-01.hwp",
            "other.hwp",
            "--list",
            "0",
            "--para",
            "0",
            "--pos",
            "0",
        ]))
        .unwrap_err();
        assert_eq!(err, EXIT_USAGE);
    }

    #[test]
    fn source_never_calls_mutators() {
        let src = include_str!("rhwp-q-char-shape.rs");
        let code = src.split("#[cfg(test)]").next().unwrap();
        for needle in [".apply_", ".insert_", ".delete_", ".set_"] {
            assert!(
                !code.contains(needle),
                "읽기 전용 CLI 가 {needle} 를 부르면 안 된다"
            );
        }
        assert!(code.contains("char_shape_set_json"));
        assert!(code.contains("from_bytes"));
    }
}
