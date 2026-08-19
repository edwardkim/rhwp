//! 쪽 안 개체 순환 순서를 조회하는 읽기 전용 CLI.
//!
//! `src/bin/*.rs` 자동 인식이라 Cargo.toml 을 건드리지 않는다. 본 CLI(`src/main.rs`)와
//! 경합하지 않도록 공개 조회 API 만 부른다 — `apply_`/`insert_`/`delete_`/`set_*` 금지.

use rhwp::document_core::DocumentCore;
use serde_json::{json, Value};
use std::io::Write;

const EXIT_OK: i32 = 0;
const EXIT_RUNTIME: i32 = 1;
const EXIT_USAGE: i32 = 2;
const USAGE: &str = "rhwp-q-object-cycle <파일> [--json]";
const UNTRUSTED: &[&str] = &["source", "cycle"];

fn envelope(command: &str, payload: Value, untrusted: &[&str]) -> Value {
    let mut out = json!({
        "schemaVersion": rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION,
        "tool": "rhwp-q-object-cycle",
        "command": command,
        "version": rhwp::version(),
        "untrustedContent": !untrusted.is_empty(),
        "untrustedFields": untrusted,
    });
    if let (Some(dst), Some(src)) = (out.as_object_mut(), payload.as_object()) {
        for (key, value) in src {
            dst.insert(key.clone(), value.clone());
        }
    }
    out
}

fn write_stdout(text: &str) -> i32 {
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    if let Err(e) = writeln!(lock, "{text}") {
        eprintln!("오류: stdout 쓰기 실패 - {e}");
        return EXIT_RUNTIME;
    }
    EXIT_OK
}

fn parse_args(args: &[String]) -> Result<(bool, String), i32> {
    let mut json = false;
    let mut path: Option<String> = None;
    for arg in args {
        match arg.as_str() {
            "--json" => json = true,
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
            }
        }
    }
    let Some(path) = path else {
        eprintln!("오류: 파일 경로가 필요합니다.");
        eprintln!("사용법: {USAGE}");
        return Err(EXIT_USAGE);
    };
    Ok((json, path))
}

fn inspect(path: &str) -> Result<Value, i32> {
    let data = std::fs::read(path).map_err(|e| {
        eprintln!("오류: 파일을 읽을 수 없습니다 - {path}: {e}");
        EXIT_RUNTIME
    })?;
    let core = DocumentCore::from_bytes(&data).map_err(|e| {
        eprintln!("오류: 문서를 열 수 없습니다 - {path}: {e}");
        EXIT_RUNTIME
    })?;
    let cycle: Value = serde_json::from_str(&core.object_cycle_json().map_err(|e| {
        eprintln!("오류: 개체 순환을 만들지 못했습니다 - {path}: {e}");
        EXIT_RUNTIME
    })?)
    .map_err(|e| {
        eprintln!("오류: 개체 순환 JSON 이 깨졌습니다 - {e}");
        EXIT_RUNTIME
    })?;
    Ok(envelope(
        "object-cycle",
        json!({
            "source": path,
            "cycleCount": cycle.as_array().map(Vec::len).unwrap_or(0),
            "cycle": cycle,
        }),
        UNTRUSTED,
    ))
}

fn print_text(report: &Value) -> i32 {
    let source = report["source"].as_str().unwrap_or("");
    let cycle = report["cycle"].as_array();
    let mut lines = Vec::new();
    lines.push(source.to_string());
    lines.push(format!("cycle\t{}", cycle.map(Vec::len).unwrap_or(0)));
    if let Some(items) = cycle {
        for (i, item) in items.iter().enumerate() {
            lines.push(format!(
                "{i}\tpara={}\tctrl={}\tpage={}\tz={}",
                item["para"], item["controlIndex"], item["page"], item["z"]
            ));
        }
    }
    write_stdout(&lines.join("\n"))
}

fn run(args: Vec<String>) -> i32 {
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        return write_stdout(USAGE);
    }
    let (json, path) = match parse_args(&args) {
        Ok(v) => v,
        Err(code) => return code,
    };
    let report = match inspect(&path) {
        Ok(v) => v,
        Err(code) => return code,
    };
    if json {
        match serde_json::to_string_pretty(&report) {
            Ok(text) => write_stdout(&text),
            Err(e) => {
                eprintln!("오류: JSON 직렬화 실패 - {e}");
                EXIT_RUNTIME
            }
        }
    } else {
        print_text(&report)
    }
}

fn main() {
    std::process::exit(run(std::env::args().skip(1).collect()));
}
