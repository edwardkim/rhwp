//! 필드 id 로 현재 값.

use crate::envelope::{
    envelope, load_core, parse_json_string, parse_u32, print_json, write_stdout, EXIT_RUNTIME,
    EXIT_USAGE,
};
use serde_json::json;

pub fn run(args: &[String]) -> i32 {
    let usage = "rhwp-q-kit field-by-id <파일> --id <N> [--json]";
    let mut json_mode = false;
    let mut path: Option<String> = None;
    let mut id: Option<u32> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_mode = true;
                i += 1;
            }
            "--id" => {
                let Some(raw) = args.get(i + 1) else {
                    eprintln!("오류: --id 뒤에 0 이상의 정수가 필요합니다.");
                    eprintln!("사용법: {usage}");
                    return EXIT_USAGE;
                };
                id = Some(match parse_u32("--id", raw) {
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
    let (Some(path), Some(id)) = (path, id) else {
        eprintln!("오류: 파일과 --id 가 필요합니다.");
        eprintln!("사용법: {usage}");
        return EXIT_USAGE;
    };
    let core = match load_core(&path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    match core.get_field_value_by_id(id) {
        Ok(raw) => {
            if json_mode {
                let data = match parse_json_string(&raw) {
                    Ok(v) => v,
                    Err(c) => return c,
                };
                let mut payload = json!({ "source": path, "id": id, "found": true });
                absorb(&mut payload, data);
                print_json(&envelope("field-by-id", payload, &["value"]))
            } else {
                write_stdout(&raw)
            }
        }
        Err(e) if e.to_string().contains("없음") => {
            if json_mode {
                print_json(&envelope(
                    "field-by-id",
                    json!({ "source": path, "id": id, "found": false, "ok": false }),
                    &[],
                ))
            } else {
                write_stdout("missing")
            }
        }
        Err(e) => {
            eprintln!("오류: 필드 값을 읽지 못했습니다 - {e}");
            EXIT_RUNTIME
        }
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
