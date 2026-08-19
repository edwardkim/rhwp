//! 양식 개체 값.

use crate::envelope::{
    envelope, load_core, parse_json_string, parse_usize, print_json, write_stdout, EXIT_RUNTIME,
    EXIT_USAGE,
};
use serde_json::json;

pub fn run(args: &[String]) -> i32 {
    let usage = "rhwp-q-kit form-value <파일> --section <N> --para <N> --ci <N> [--json]";
    let mut json_mode = false;
    let mut path: Option<String> = None;
    let mut section: Option<usize> = None;
    let mut para: Option<usize> = None;
    let mut ci: Option<usize> = None;
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
                    eprintln!("사용법: {usage}");
                    return EXIT_USAGE;
                };
                section = Some(match parse_usize("--section", raw) {
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
            "--ci" => {
                let Some(raw) = args.get(i + 1) else {
                    eprintln!("오류: --ci 뒤에 0 이상의 정수가 필요합니다.");
                    eprintln!("사용법: {usage}");
                    return EXIT_USAGE;
                };
                ci = Some(match parse_usize("--ci", raw) {
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
    let (Some(path), Some(section), Some(para), Some(ci)) = (path, section, para, ci) else {
        eprintln!("오류: 파일과 --section --para --ci 가 필요합니다.");
        eprintln!("사용법: {usage}");
        return EXIT_USAGE;
    };
    let core = match load_core(&path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let raw = match core.get_form_value_native(section, para, ci) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("오류: 양식 개체 값을 읽지 못했습니다 - {e}");
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
            "section": section,
            "para": para,
            "ci": ci,
        });
        absorb(&mut payload, data);
        print_json(&envelope(
            "form-value",
            payload,
            &["name", "text", "caption"],
        ))
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
