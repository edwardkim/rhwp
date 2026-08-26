//! 머리말·꼬리말 내부 텍스트 히트테스트.

use crate::envelope::{
    envelope, load_core, parse_f64, parse_json_string, parse_u32, print_json, write_stdout,
    EXIT_RUNTIME, EXIT_USAGE,
};
use serde_json::json;

pub fn run(args: &[String]) -> i32 {
    let usage = "rhwp-q-kit hit-in-hf <파일> --page <N> --x <F> --y <F> [--json]";
    let mut json_mode = false;
    let mut path: Option<String> = None;
    let mut page: Option<u32> = None;
    let mut x: Option<f64> = None;
    let mut y: Option<f64> = None;
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
            "--x" => {
                let Some(raw) = args.get(i + 1) else {
                    eprintln!("오류: --x 뒤에 실수가 필요합니다.");
                    eprintln!("사용법: {usage}");
                    return EXIT_USAGE;
                };
                x = Some(match parse_f64("--x", raw) {
                    Ok(v) => v,
                    Err(c) => return c,
                });
                i += 2;
            }
            "--y" => {
                let Some(raw) = args.get(i + 1) else {
                    eprintln!("오류: --y 뒤에 실수가 필요합니다.");
                    eprintln!("사용법: {usage}");
                    return EXIT_USAGE;
                };
                y = Some(match parse_f64("--y", raw) {
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
    let (Some(path), Some(page), Some(x), Some(y)) = (path, page, x, y) else {
        eprintln!("오류: 파일과 --page --x --y 가 필요합니다.");
        eprintln!("사용법: {usage}");
        return EXIT_USAGE;
    };
    let is_header = is_header.unwrap_or(true);
    let core = match load_core(&path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let raw = match core.hit_test_in_header_footer_native(page, is_header, x, y) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("오류: 머리말·꼬리말 내부 히트를 하지 못했습니다 - {e}");
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
            "x": x,
            "y": y,
            "isHeader": is_header,
        });
        absorb(&mut payload, data);
        print_json(&envelope("hit-in-hf", payload, &[]))
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
