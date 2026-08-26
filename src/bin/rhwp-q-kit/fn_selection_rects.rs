//! 각주 내부 선택 영역의 줄별 사각형.

use crate::envelope::{
    envelope, load_core, parse_json_string, parse_u32, parse_usize, print_json, write_stdout,
    EXIT_RUNTIME, EXIT_USAGE,
};
use serde_json::json;

pub fn run(args: &[String]) -> i32 {
    let usage = "rhwp-q-kit fn-selection-rects <파일> --page <N> [--json]";
    let mut json_mode = false;
    let mut path: Option<String> = None;
    let mut page: Option<u32> = None;
    let mut index: usize = 0;
    let mut start_para: usize = 0;
    let mut start_pos: usize = 0;
    let mut end_para: usize = 0;
    let mut end_pos: usize = 0;
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
                    eprintln!("사용법: {usage}");
                    return EXIT_USAGE;
                };
                page = Some(match parse_u32("--page", raw) {
                    Ok(v) => v,
                    Err(c) => return c,
                });
                i += 2;
            }
            "--index" => {
                let Some(raw) = args.get(i + 1) else {
                    eprintln!("오류: --index 뒤에 0 이상의 정수가 필요합니다.");
                    eprintln!("사용법: {usage}");
                    return EXIT_USAGE;
                };
                index = match parse_usize("--index", raw) {
                    Ok(v) => v,
                    Err(c) => return c,
                };
                i += 2;
            }
            "--start-para" => {
                let Some(raw) = args.get(i + 1) else {
                    eprintln!("오류: --start-para 뒤에 0 이상의 정수가 필요합니다.");
                    eprintln!("사용법: {usage}");
                    return EXIT_USAGE;
                };
                start_para = match parse_usize("--start-para", raw) {
                    Ok(v) => v,
                    Err(c) => return c,
                };
                i += 2;
            }
            "--start-pos" => {
                let Some(raw) = args.get(i + 1) else {
                    eprintln!("오류: --start-pos 뒤에 0 이상의 정수가 필요합니다.");
                    eprintln!("사용법: {usage}");
                    return EXIT_USAGE;
                };
                start_pos = match parse_usize("--start-pos", raw) {
                    Ok(v) => v,
                    Err(c) => return c,
                };
                i += 2;
            }
            "--end-para" => {
                let Some(raw) = args.get(i + 1) else {
                    eprintln!("오류: --end-para 뒤에 0 이상의 정수가 필요합니다.");
                    eprintln!("사용법: {usage}");
                    return EXIT_USAGE;
                };
                end_para = match parse_usize("--end-para", raw) {
                    Ok(v) => v,
                    Err(c) => return c,
                };
                i += 2;
            }
            "--end-pos" => {
                let Some(raw) = args.get(i + 1) else {
                    eprintln!("오류: --end-pos 뒤에 0 이상의 정수가 필요합니다.");
                    eprintln!("사용법: {usage}");
                    return EXIT_USAGE;
                };
                end_pos = match parse_usize("--end-pos", raw) {
                    Ok(v) => v,
                    Err(c) => return c,
                };
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
    let core = match load_core(&path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let raw = match core.get_selection_rects_in_footnote_native(
        page, index, start_para, start_pos, end_para, end_pos,
    ) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("오류: 각주 선택 사각형을 읽지 못했습니다 - {e}");
            return EXIT_RUNTIME;
        }
    };
    if json_mode {
        let rects = match parse_json_string(&raw) {
            Ok(v) => v,
            Err(c) => return c,
        };
        let payload = json!({
            "source": path,
            "page": page,
            "index": index,
            "startPara": start_para,
            "startPos": start_pos,
            "endPara": end_para,
            "endPos": end_pos,
            "rects": rects,
        });
        print_json(&envelope("fn-selection-rects", payload, &["rects"]))
    } else {
        write_stdout(&raw)
    }
}
