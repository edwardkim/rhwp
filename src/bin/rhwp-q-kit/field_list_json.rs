//! 한글 필드 리스트 JSON.

use crate::envelope::{
    envelope, load_core, parse_json_string, print_json, write_stdout, EXIT_USAGE,
};
use serde_json::json;

pub fn run(args: &[String]) -> i32 {
    let usage = "rhwp-q-kit field-list-json <파일> [--json]";
    let mut json_mode = false;
    let mut path: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_mode = true;
                i += 1;
            }
            other if other.starts_with('-') => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {other}");
                eprintln!("사용법: {usage}");
                return EXIT_USAGE;
            }
            other => {
                if path.is_some() {
                    eprintln!("오류: 파일이 너무 많습니다.");
                    eprintln!("사용법: {usage}");
                    return EXIT_USAGE;
                }
                path = Some(other.to_string());
                i += 1;
            }
        }
    }
    let Some(path) = path else {
        eprintln!("오류: 파일 경로가 필요합니다.");
        eprintln!("사용법: {usage}");
        return EXIT_USAGE;
    };
    let core = match load_core(&path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let raw = core.get_field_list_json();
    if json_mode {
        let fields = match parse_json_string(&raw) {
            Ok(v) => v,
            Err(c) => return c,
        };
        let count = fields.as_array().map(|a| a.len()).unwrap_or(0);
        print_json(&envelope(
            "field-list-json",
            json!({ "source": path, "count": count, "fields": fields }),
            &[
                "fields[].name",
                "fields[].guide",
                "fields[].command",
                "fields[].value",
            ],
        ))
    } else {
        write_stdout(&raw)
    }
}
