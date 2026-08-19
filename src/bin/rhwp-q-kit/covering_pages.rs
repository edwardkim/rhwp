//! 문단이 걸친 쪽 — `DocumentCore::pages_covering_paragraphs`.

use crate::envelope::{envelope, load_core, parse_usize, print_json, write_stdout, EXIT_USAGE};
use serde_json::json;

const USAGE: &str = "rhwp-q-kit covering-pages <파일> --section <N> --para <N> [--json]";

pub fn run(args: &[String]) -> i32 {
    let mut json_mode = false;
    let mut path: Option<String> = None;
    let mut section: Option<usize> = None;
    let mut para: Option<usize> = None;
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
    let (Some(path), Some(section), Some(para)) = (path, section, para) else {
        eprintln!("오류: 파일과 --section --para 가 필요합니다.");
        eprintln!("사용법: {USAGE}");
        return EXIT_USAGE;
    };
    let mut core = match load_core(&path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let pages = core.pages_covering_paragraphs(&[(section, para)]);
    let covered = pages.is_some();
    let payload = json!({
        "source": path,
        "section": section,
        "para": para,
        "covered": covered,
        "pages": pages.clone(),
    });
    if json_mode {
        print_json(&envelope("covering-pages", payload, &[]))
    } else if let Some(list) = pages {
        let joined = list
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(",");
        write_stdout(&format!("pages={joined}"))
    } else {
        write_stdout("covered=false")
    }
}
