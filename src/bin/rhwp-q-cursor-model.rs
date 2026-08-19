//! 한글 커서 좌표계(`GetPos`/`SetPos`/`MovePos`)의 리스트 지도를 조회한다.
//!
//! 기존 읽기 전용 질의 [`DocumentCore::get_cursor_model_json`] 만 부른다. 문서 편집 API 는 쓰지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;
use serde_json::{json, Value};
use std::io::Write;
use std::process;

const EXIT_OK: i32 = 0;
const EXIT_RUNTIME: i32 = 1;
const EXIT_USAGE: i32 = 2;
const TOOL: &str = "rhwp-q-cursor-model";
const COMMAND: &str = "cursor-model";
const UNTRUSTED_FIELDS: &[&str] = &["source", "root", "lists"];
const USAGE: &str = "사용법: rhwp-q-cursor-model <파일> [--json]";

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
                    eprintln!("오류: 파일은 하나만 받습니다 - {other}");
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

fn load_model(path: &str) -> Result<Value, i32> {
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
    let raw = core.get_cursor_model_json();
    match serde_json::from_str::<Value>(&raw) {
        Ok(Value::Object(map)) => Ok(Value::Object(map)),
        Ok(_) => {
            eprintln!("오류: 커서 모델 JSON 이 객체가 아닙니다.");
            Err(EXIT_RUNTIME)
        }
        Err(e) => {
            eprintln!("오류: 커서 모델 JSON 을 파싱할 수 없습니다 - {e}");
            Err(EXIT_RUNTIME)
        }
    }
}

fn envelope(path: &str, model: Value) -> Value {
    let mut out = json!({
        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
        "tool": TOOL,
        "command": COMMAND,
        "version": rhwp::version(),
        "untrustedContent": true,
        "untrustedFields": UNTRUSTED_FIELDS,
        "source": path,
    });
    if let (Some(dst), Some(src)) = (out.as_object_mut(), model.as_object()) {
        for (key, value) in src {
            dst.insert(key.clone(), value.clone());
        }
    }
    out
}

fn format_root(root: &Value) -> String {
    let para_count = root
        .get("paraCount")
        .and_then(Value::as_u64)
        .map(|n| n.to_string())
        .unwrap_or_else(|| "-".to_string());
    let top_pos = root
        .get("topPos")
        .and_then(Value::as_u64)
        .map(|n| n.to_string())
        .unwrap_or_else(|| "-".to_string());
    let end_para = root
        .get("endPara")
        .and_then(Value::as_u64)
        .map(|n| n.to_string())
        .unwrap_or_else(|| "-".to_string());
    let end_pos = root
        .get("endPos")
        .and_then(Value::as_u64)
        .map(|n| n.to_string())
        .unwrap_or_else(|| "-".to_string());
    format!("paraCount={para_count} topPos={top_pos} endPara={end_para} endPos={end_pos}")
}

fn format_list(item: &Value) -> String {
    let list_id = item
        .get("listId")
        .and_then(Value::as_u64)
        .map(|n| n.to_string())
        .unwrap_or_else(|| "-".to_string());
    let is_cell = item
        .get("isCell")
        .and_then(Value::as_bool)
        .map(|b| b.to_string())
        .unwrap_or_else(|| "-".to_string());
    let host_list_id = item
        .get("hostListId")
        .and_then(Value::as_u64)
        .map(|n| n.to_string())
        .unwrap_or_else(|| "-".to_string());
    let section_index = item
        .get("sectionIndex")
        .and_then(Value::as_u64)
        .map(|n| n.to_string())
        .unwrap_or_else(|| "-".to_string());
    let host_para = item
        .get("hostPara")
        .and_then(Value::as_u64)
        .map(|n| n.to_string())
        .unwrap_or_else(|| "-".to_string());
    let control_index = item
        .get("controlIndex")
        .and_then(Value::as_u64)
        .map(|n| n.to_string())
        .unwrap_or_else(|| "-".to_string());
    let cell_index = item
        .get("cellIndex")
        .and_then(Value::as_u64)
        .map(|n| n.to_string())
        .unwrap_or_else(|| "-".to_string());
    let para_count = item
        .get("paraCount")
        .and_then(Value::as_u64)
        .map(|n| n.to_string())
        .unwrap_or_else(|| "-".to_string());
    format!(
        "listId={list_id} isCell={is_cell} hostListId={host_list_id} sectionIndex={section_index} hostPara={host_para} controlIndex={control_index} cellIndex={cell_index} paraCount={para_count}"
    )
}

fn print_text(path: &str, model: &Value) {
    let list_count = model.get("listCount").and_then(Value::as_u64).unwrap_or(0);
    let root = model.get("root").cloned().unwrap_or(Value::Null);
    write_stdout(
        &format!(
            "cursor-model source={path} listCount={list_count} {}",
            format_root(&root)
        ),
        true,
    );
    if let Some(lists) = model.get("lists").and_then(Value::as_array) {
        for item in lists {
            write_stdout(&format_list(item), true);
        }
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
            "한글 커서 좌표계의 리스트 지도를 조회한다. 문서 편집 API 는 쓰지 않는다.",
            true,
        );
        write_stdout("종료 코드: 0 성공 · 1 실행 오류 · 2 사용법 오류", true);
        return EXIT_OK;
    }
    if opts.version {
        write_stdout(&format!("{TOOL} v{}", rhwp::version()), true);
        return EXIT_OK;
    }
    let path = opts.path.as_deref().expect("parse_args 가 경로를 검증한다");
    let model = match load_model(path) {
        Ok(model) => model,
        Err(code) => return code,
    };
    if opts.json {
        print_json(&envelope(path, model));
    } else {
        print_text(path, &model);
    }
    EXIT_OK
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    process::exit(run(&args));
}
