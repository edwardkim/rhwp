//! 문단 캐럿 경계.

use crate::envelope::{
    envelope, load_core, parse_json_string, parse_u32, parse_usize, print_json, write_stdout,
    EXIT_USAGE,
};
use serde_json::json;

pub fn run(args: &[String]) -> i32 {
    let usage = "rhwp-q-kit para-bounds <파일> --list <N> --para <N> [--json]";
    let mut json_mode = false;
    let mut path: Option<String> = None;
    let mut list: Option<u32> = None;
    let mut para: Option<usize> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_mode = true;
                i += 1;
            }
            "--list" => {
                let Some(raw) = args.get(i + 1) else {
                    eprintln!("오류: --list 뒤에 0 이상의 정수가 필요합니다.");
                    eprintln!("사용법: {usage}");
                    return EXIT_USAGE;
                };
                list = Some(match parse_u32("--list", raw) {
                    Ok(v) => v,
                    Err(c) => return c,
                });
                i += 2;
            }
            "--para" => {
                let Some(raw) = args.get(i + 1) else {
                    eprintln!("오류: --para 뒤에 0 이상의 정수가 필요합니다.");
                    eprintln!("사용법: {usage}");
                    return EXIT_USAGE;
                };
                para = Some(match parse_usize("--para", raw) {
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
    let (Some(path), Some(list), Some(para)) = (path, list, para) else {
        eprintln!("오류: 파일과 --list --para 가 필요합니다.");
        eprintln!("사용법: {usage}");
        return EXIT_USAGE;
    };
    let core = match load_core(&path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let raw = core.para_bounds_json(list, para);
    if json_mode {
        let data = match parse_json_string(&raw) {
            Ok(v) => v,
            Err(c) => return c,
        };
        let mut payload = json!({ "source": path, "list": list, "para": para });
        absorb(&mut payload, data);
        print_json(&envelope("para-bounds", payload, &[]))
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
