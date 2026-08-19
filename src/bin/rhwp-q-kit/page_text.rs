//! 한글 스캔 쪽 텍스트 — `DocumentCore::page_text`.

use crate::envelope::{
    envelope, load_core, parse_json_string, parse_usize, print_json, write_stdout, EXIT_RUNTIME,
    EXIT_USAGE,
};
use serde_json::json;

const USAGE: &str = "rhwp-q-kit page-text <파일> --page <N> [--json]";

pub fn run(args: &[String]) -> i32 {
    let mut json_mode = false;
    let mut path: Option<String> = None;
    let mut page: Option<usize> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_mode = true;
                i += 1;
            }
            "--page" => {
                let Some(raw) = args.get(i + 1) else {
                    eprintln!("오류: --page 뒤에 0 이상의 정수가 필요합니다.");
                    eprintln!("사용법: {USAGE}");
                    return EXIT_USAGE;
                };
                page = Some(match parse_usize("--page", raw) {
                    Ok(v) => v,
                    Err(c) => return c,
                });
                i += 2;
            }
            other if other.starts_with('-') => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {other}");
                eprintln!("사용법: {USAGE}");
                return EXIT_USAGE;
            }
            other => {
                if path.is_some() {
                    eprintln!("오류: 파일이 너무 많습니다 - {other}");
                    eprintln!("사용법: {USAGE}");
                    return EXIT_USAGE;
                }
                path = Some(other.to_string());
                i += 1;
            }
        }
    }
    let (Some(path), Some(page)) = (path, page) else {
        eprintln!("오류: 파일과 --page 가 필요합니다.");
        eprintln!("사용법: {USAGE}");
        return EXIT_USAGE;
    };
    let core = match load_core(&path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let escaped = match core.page_text(page) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("오류: 쪽 텍스트를 읽지 못했습니다 - {e}");
            return EXIT_RUNTIME;
        }
    };
    // `page_text` 는 따옴표 없는 JSON 이스케이프 본문을 돌려준다.
    let text = match parse_json_string(&format!("\"{escaped}\"")) {
        Ok(v) => v.as_str().unwrap_or("").to_string(),
        Err(c) => return c,
    };
    let chars = text.chars().count();
    if json_mode {
        print_json(&envelope(
            "page-text",
            json!({
                "source": path,
                "page": page,
                "text": text,
                "chars": chars,
            }),
            &["text"],
        ))
    } else {
        write_stdout(&text)
    }
}
