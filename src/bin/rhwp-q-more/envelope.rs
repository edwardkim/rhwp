//! rhwp-q-more 공통 적재·봉투.

use rhwp::document_core::DocumentCore;
use serde_json::{json, Value};
use std::io::Write;

pub const EXIT_OK: i32 = 0;
pub const EXIT_RUNTIME: i32 = 1;
pub const EXIT_USAGE: i32 = 2;
pub const TOOL: &str = "rhwp-q-more";

pub fn envelope(command: &str, mut payload: Value, untrusted: &[&str]) -> Value {
    if let Some(map) = payload.as_object_mut() {
        map.insert(
            "schemaVersion".into(),
            json!(rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION),
        );
        map.insert("tool".into(), json!(TOOL));
        map.insert("command".into(), json!(command));
        map.insert("version".into(), json!(rhwp::version()));
        map.insert("untrustedContent".into(), json!(!untrusted.is_empty()));
        map.insert("untrustedFields".into(), json!(untrusted));
    }
    payload
}

pub fn write_stdout(text: &str) -> i32 {
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    if let Err(e) = writeln!(lock, "{text}") {
        eprintln!("오류: stdout 쓰기 실패 - {e}");
        return EXIT_RUNTIME;
    }
    EXIT_OK
}

pub fn print_json(value: &Value) -> i32 {
    match serde_json::to_string_pretty(value) {
        Ok(s) => write_stdout(&s),
        Err(e) => {
            eprintln!("오류: JSON 직렬화 실패 - {e}");
            EXIT_RUNTIME
        }
    }
}

pub fn load_core(path: &str) -> Result<DocumentCore, i32> {
    let data = std::fs::read(path).map_err(|e| {
        eprintln!("오류: 파일을 읽을 수 없습니다 - {path}: {e}");
        EXIT_RUNTIME
    })?;
    DocumentCore::from_bytes(&data).map_err(|e| {
        eprintln!("오류: 문서를 열 수 없습니다 - {path}: {e}");
        EXIT_RUNTIME
    })
}

pub struct FileOpts {
    pub path: String,
    pub json: bool,
}

pub fn parse_one_file(args: &[String], usage: &str) -> Result<FileOpts, i32> {
    let mut path = None;
    let mut json = false;
    for a in args {
        match a.as_str() {
            "--json" => json = true,
            other if other.starts_with('-') => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {other}");
                eprintln!("사용법: {usage}");
                return Err(EXIT_USAGE);
            }
            other => {
                if path.replace(other.to_string()).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return Err(EXIT_USAGE);
                }
            }
        }
    }
    let Some(path) = path else {
        eprintln!("오류: 파일 경로가 필요합니다.");
        eprintln!("사용법: {usage}");
        return Err(EXIT_USAGE);
    };
    Ok(FileOpts { path, json })
}

pub fn parse_slot(args: &[String], usage: &str) -> Result<(String, bool, u32), i32> {
    let mut path = None;
    let mut json = false;
    let mut slot = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json = true;
                i += 1;
            }
            "--slot" => {
                let Some(raw) = args.get(i + 1) else {
                    eprintln!("오류: --slot 뒤에 정수가 필요합니다.");
                    eprintln!("{usage}");
                    return Err(EXIT_USAGE);
                };
                match raw.parse::<u32>() {
                    Ok(n) if (n as usize) < 50 => slot = Some(n),
                    _ => {
                        eprintln!("오류: --slot 은 0..49 입니다.");
                        return Err(EXIT_USAGE);
                    }
                }
                i += 2;
            }
            other if other.starts_with('-') => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {other}");
                return Err(EXIT_USAGE);
            }
            other => {
                if path.replace(other.to_string()).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다.");
                    return Err(EXIT_USAGE);
                }
                i += 1;
            }
        }
    }
    let Some(path) = path else {
        eprintln!("오류: 파일 경로가 필요합니다.");
        return Err(EXIT_USAGE);
    };
    let Some(slot) = slot else {
        eprintln!("오류: --slot 이 필요합니다.");
        eprintln!("{usage}");
        return Err(EXIT_USAGE);
    };
    Ok((path, json, slot))
}
