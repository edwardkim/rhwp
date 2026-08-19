//! 쪽 조판 항목 조회 — 기존 읽기 전용 `dump_page_items_json` 만 부른다.
//!
//! `--page` 를 생략하면 API 의 기본 모양대로 모든 쪽을 덤프한다 (`page_filter:
//! None`). 문서를 고치지 않는다. `--page` 는 0부터 센다.

use rhwp::document_core::DocumentCore;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;
use serde_json::{json, Value};
use std::io::Write;
use std::process;

const EXIT_OK: i32 = 0;
const EXIT_RUNTIME: i32 = 1;
const EXIT_USAGE: i32 = 2;
const TOOL: &str = "rhwp-q-page-items";
const COMMAND: &str = "page-items";
const UNTRUSTED_FIELDS: &[&str] = &["source", "pages"];
const USAGE: &str = "사용법: rhwp-q-page-items <파일> [--page <N>] [--json]";

#[derive(Debug)]
struct Options {
    json: bool,
    help: bool,
    version: bool,
    path: Option<String>,
    page: Option<u32>,
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

fn parse_page_value(raw: &str) -> Result<u32, i32> {
    match raw.parse::<u32>() {
        Ok(n) => Ok(n),
        Err(_) => {
            eprintln!("오류: --page 뒤에 0 이상의 정수가 필요합니다.");
            eprintln!("{USAGE}");
            Err(EXIT_USAGE)
        }
    }
}

fn parse_args(args: &[String]) -> Result<Options, i32> {
    let mut opts = Options {
        json: false,
        help: false,
        version: false,
        path: None,
        page: None,
    };
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                opts.help = true;
                i += 1;
            }
            "--version" | "-V" => {
                opts.version = true;
                i += 1;
            }
            "--json" => {
                opts.json = true;
                i += 1;
            }
            "--page" => {
                let Some(raw) = args.get(i + 1) else {
                    eprintln!("오류: --page 뒤에 0 이상의 정수가 필요합니다.");
                    eprintln!("{USAGE}");
                    return Err(EXIT_USAGE);
                };
                if opts.page.is_some() {
                    eprintln!("오류: --page 를 두 번 지정했습니다.");
                    eprintln!("{USAGE}");
                    return Err(EXIT_USAGE);
                }
                opts.page = Some(parse_page_value(raw)?);
                i += 2;
            }
            other if other.starts_with("--page=") => {
                let raw = &other["--page=".len()..];
                if opts.page.is_some() {
                    eprintln!("오류: --page 를 두 번 지정했습니다.");
                    eprintln!("{USAGE}");
                    return Err(EXIT_USAGE);
                }
                opts.page = Some(parse_page_value(raw)?);
                i += 1;
            }
            other if other.starts_with('-') => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {other}");
                eprintln!("{USAGE}");
                return Err(EXIT_USAGE);
            }
            other => {
                if opts.path.is_some() {
                    eprintln!("오류: 파일이 너무 많습니다 - {other}");
                    eprintln!("{USAGE}");
                    return Err(EXIT_USAGE);
                }
                opts.path = Some(other.to_string());
                i += 1;
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

fn open_core(path: &str) -> Result<DocumentCore, i32> {
    let data = match std::fs::read(path) {
        Ok(bytes) => bytes,
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

fn load_pages(path: &str, page: Option<u32>) -> Result<(u32, Value), i32> {
    let core = open_core(path)?;
    let page_count = core.page_count();
    if let Some(p) = page {
        if p >= page_count {
            eprintln!(
                "오류: 페이지 번호가 범위를 벗어났습니다 (0~{})",
                page_count.saturating_sub(1)
            );
            return Err(EXIT_RUNTIME);
        }
    }
    Ok((page_count, core.dump_page_items_json(page)))
}

fn envelope(path: &str, page_count: u32, page_filter: Option<u32>, pages: Value) -> Value {
    json!({
        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
        "tool": TOOL,
        "command": COMMAND,
        "version": rhwp::version(),
        "untrustedContent": true,
        "untrustedFields": UNTRUSTED_FIELDS,
        "source": path,
        "pageCount": page_count,
        "pageFilter": page_filter,
        "pages": pages,
    })
}

fn column_item_count(page: &Value) -> u64 {
    page.get("columns")
        .and_then(Value::as_array)
        .map(|cols| {
            cols.iter()
                .map(|c| {
                    c.get("itemCount")
                        .and_then(Value::as_u64)
                        .or_else(|| {
                            c.get("items")
                                .and_then(Value::as_array)
                                .map(|a| a.len() as u64)
                        })
                        .unwrap_or(0)
                })
                .sum()
        })
        .unwrap_or(0)
}

fn print_text(path: &str, page_count: u32, page_filter: Option<u32>, pages: &Value) {
    let filter = match page_filter {
        Some(n) => n.to_string(),
        None => "all".to_string(),
    };
    let arr = pages.as_array();
    let dumped = arr.map(Vec::len).unwrap_or(0);
    write_stdout(
        &format!(
            "page-items source={path} pageCount={page_count} pageFilter={filter} dumped={dumped}"
        ),
        true,
    );
    let Some(arr) = arr else {
        return;
    };
    for page in arr {
        let idx = page
            .get("pageIndex")
            .and_then(Value::as_u64)
            .map(|n| n.to_string())
            .unwrap_or_else(|| "-".to_string());
        let section = page
            .get("section")
            .and_then(Value::as_u64)
            .map(|n| n.to_string())
            .unwrap_or_else(|| "-".to_string());
        let columns = page
            .get("columns")
            .and_then(Value::as_array)
            .map(|a| a.len())
            .unwrap_or(0);
        let extras = page
            .get("extras")
            .and_then(Value::as_array)
            .map(|a| a.len())
            .unwrap_or(0);
        let items = column_item_count(page);
        write_stdout(
            &format!("{idx}\tsection={section} columns={columns} extras={extras} items={items}"),
            true,
        );
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
            "쪽의 조판 항목을 조회한다. 문서를 고치지 않는다. --page 는 0부터 센다.",
            true,
        );
        write_stdout(
            "--page 를 생략하면 dump_page_items_json(None) 으로 모든 쪽을 덤프한다.",
            true,
        );
        write_stdout("종료 코드: 0 성공 · 1 실행 오류 · 2 사용법 오류", true);
        return EXIT_OK;
    }
    if opts.version {
        write_stdout(&format!("{TOOL} v{}", rhwp::version()), true);
        return EXIT_OK;
    }
    let path = opts.path.as_deref().expect("parse_args 가 경로를 확인한다");
    let (page_count, pages) = match load_pages(path, opts.page) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    if opts.json {
        print_json(&envelope(path, page_count, opts.page, pages));
    } else {
        print_text(path, page_count, opts.page, &pages);
    }
    EXIT_OK
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    process::exit(run(&args));
}
