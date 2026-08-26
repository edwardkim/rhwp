//! 단어 끝 오프셋 — `DocumentCore::word_end_json`.

use crate::envelope::{
    envelope, load_core, parse_json_string, parse_u32, parse_usize, print_json, write_stdout,
    EXIT_USAGE,
};
use serde_json::json;

const USAGE: &str = "rhwp-q-kit word-end <파일> --list <N> --para <N> --pos <N> [--json]";

pub fn run(args: &[String]) -> i32 {
    let mut json_mode = false;
    let mut path: Option<String> = None;
    let mut list: Option<u32> = None;
    let mut para: Option<usize> = None;
    let mut pos: Option<usize> = None;
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
                    eprintln!("사용법: {USAGE}");
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
                    eprintln!("사용법: {USAGE}");
                    return EXIT_USAGE;
                };
                para = Some(match parse_usize("--para", raw) {
                    Ok(v) => v,
                    Err(c) => return c,
                });
                i += 2;
            }
            "--pos" => {
                let Some(raw) = args.get(i + 1) else {
                    eprintln!("오류: --pos 뒤에 0 이상의 정수가 필요합니다.");
                    eprintln!("사용법: {USAGE}");
                    return EXIT_USAGE;
                };
                pos = Some(match parse_usize("--pos", raw) {
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
    let (Some(path), Some(list), Some(para), Some(pos)) = (path, list, para, pos) else {
        eprintln!("오류: 파일과 --list --para --pos 가 필요합니다.");
        eprintln!("사용법: {USAGE}");
        return EXIT_USAGE;
    };
    let core = match load_core(&path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let end = match parse_json_string(&core.word_end_json(list, para, pos)) {
        Ok(v) => v,
        Err(c) => return c,
    };
    if json_mode {
        print_json(&envelope(
            "word-end",
            json!({
                "source": path,
                "list": list,
                "para": para,
                "pos": pos,
                "end": end,
            }),
            &["end"],
        ))
    } else {
        write_stdout(&end.to_string())
    }
}
