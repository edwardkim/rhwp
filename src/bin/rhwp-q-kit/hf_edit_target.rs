//! 쪽의 머리말·꼬리말 편집 대상 (구역, applyTo).

use crate::envelope::{
    envelope, load_core, parse_json_string, parse_u32, print_json, write_stdout, EXIT_RUNTIME,
    EXIT_USAGE,
};
use serde_json::json;

pub fn run(args: &[String]) -> i32 {
    let usage = "rhwp-q-kit hf-edit-target <파일> --page <N> [--json]";
    let mut json_mode = false;
    let mut path: Option<String> = None;
    let mut page: Option<u32> = None;
    let mut is_header: Option<bool> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_mode = true;
                i += 1;
            }
            "--header" => {
                if is_header.replace(true).is_some() {
                    eprintln!("오류: --header 와 --footer 중 하나만 지정합니다.");
                    eprintln!("사용법: {usage}");
                    return EXIT_USAGE;
                }
                i += 1;
            }
            "--footer" => {
                if is_header.replace(false).is_some() {
                    eprintln!("오류: --header 와 --footer 중 하나만 지정합니다.");
                    eprintln!("사용법: {usage}");
                    return EXIT_USAGE;
                }
                i += 1;
            }
            "--page" => {
                let Some(raw) = args.get(i + 1) else {
                    eprintln!("오류: --page 뒤에 0 이상의 정수가 필요합니다.");
                    eprintln!("사용법: {usage}");
                    return EXIT_USAGE;
                };
                page = Some(match parse_u32("--page", raw) {
                    Ok(v) => v,
                    Err(c) => return c,
                });
                i += 2;
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
    let (Some(path), Some(page)) = (path, page) else {
        eprintln!("오류: 파일과 --page 가 필요합니다.");
        eprintln!("사용법: {usage}");
        return EXIT_USAGE;
    };
    let is_header = is_header.unwrap_or(true);
    let core = match load_core(&path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let raw = match core.get_header_footer_edit_target_native(page, is_header) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("오류: 머리말·꼬리말 편집 대상을 읽지 못했습니다 - {e}");
            return EXIT_RUNTIME;
        }
    };
    if json_mode {
        let data = match parse_json_string(&raw) {
            Ok(v) => v,
            Err(c) => return c,
        };
        let mut payload = json!({
            "source": path,
            "page": page,
            "isHeader": is_header,
        });
        absorb(&mut payload, data);
        print_json(&envelope("hf-edit-target", payload, &[]))
    } else {
        write_stdout(&raw)
    }
}

fn absorb(payload: &mut serde_json::Value, native: serde_json::Value) {
    if let (Some(map), Some(obj)) = (payload.as_object_mut(), native.as_object()) {
        for (k, v) in obj {
            map.insert(k.clone(), v.clone());
        }
    } else if let Some(map) = payload.as_object_mut() {
        map.insert("data".into(), native);
    }
}
