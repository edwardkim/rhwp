//! 구역 쪽 테두리·배경 — `DocumentCore::get_page_border_fill_native`.

use crate::envelope::{
    envelope, load_core, parse_json_string, parse_usize, print_json, write_stdout, EXIT_RUNTIME,
    EXIT_USAGE,
};
use serde_json::json;

const USAGE: &str = "rhwp-q-kit page-border-fill <파일> --section <N> [--json]";

pub fn run(args: &[String]) -> i32 {
    let mut json_mode = false;
    let mut path: Option<String> = None;
    let mut section: Option<usize> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_mode = true;
                i += 1;
            }
            "--section" => {
                let Some(raw) = args.get(i + 1) else {
                    eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다.");
                    eprintln!("사용법: {USAGE}");
                    return EXIT_USAGE;
                };
                section = Some(match parse_usize("--section", raw) {
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
    let (Some(path), Some(section)) = (path, section) else {
        eprintln!("오류: 파일과 --section 이 필요합니다.");
        eprintln!("사용법: {USAGE}");
        return EXIT_USAGE;
    };
    let core = match load_core(&path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let raw = match core.get_page_border_fill_native(section) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("오류: 쪽 테두리·배경을 읽지 못했습니다 - {e}");
            return EXIT_RUNTIME;
        }
    };
    let fill = match parse_json_string(&raw) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let payload = json!({
        "source": path,
        "section": section,
        "pageBorderFill": fill,
    });
    if json_mode {
        print_json(&envelope("page-border-fill", payload, &[]))
    } else {
        write_stdout(&raw)
    }
}
