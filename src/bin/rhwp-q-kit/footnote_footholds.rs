//! 쪽에 각주 발판이 있는지.

use crate::envelope::{envelope, load_core, parse_u32, print_json, write_stdout, EXIT_USAGE};
use serde_json::json;

const USAGE: &str = "rhwp-q-kit footnote-footholds <파일> --page <N> [--json]";

pub fn run(args: &[String]) -> i32 {
    let opts = match parse(args) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let core = match load_core(&opts.path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let has = core.page_has_footnote_footholds_native(opts.page);
    let payload = json!({
        "source": opts.path,
        "page": opts.page,
        "hasFootholds": has,
    });
    if opts.json {
        print_json(&envelope("footnote-footholds", payload, &[]))
    } else {
        write_stdout(if has { "true" } else { "false" })
    }
}

struct Opts {
    json: bool,
    path: String,
    page: u32,
}

fn parse(args: &[String]) -> Result<Opts, i32> {
    let mut json = false;
    let mut path: Option<String> = None;
    let mut page: Option<u32> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json = true;
                i += 1;
            }
            "--page" => page = Some(take_u32(args, &mut i, "--page")?),
            other if other.starts_with('-') => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {other}");
                eprintln!("사용법: {USAGE}");
                return Err(EXIT_USAGE);
            }
            other => {
                if path.replace(other.to_string()).is_some() {
                    eprintln!("오류: 파일이 너무 많습니다 - {other}");
                    eprintln!("사용법: {USAGE}");
                    return Err(EXIT_USAGE);
                }
                i += 1;
            }
        }
    }
    let Some(path) = path else {
        eprintln!("오류: 파일 경로가 필요합니다.");
        eprintln!("사용법: {USAGE}");
        return Err(EXIT_USAGE);
    };
    let Some(page) = page else {
        eprintln!("오류: --page 가 필요합니다.");
        eprintln!("사용법: {USAGE}");
        return Err(EXIT_USAGE);
    };
    Ok(Opts { json, path, page })
}

fn take_u32(args: &[String], i: &mut usize, flag: &str) -> Result<u32, i32> {
    let raw = match args.get(*i + 1) {
        Some(v) => v,
        None => {
            eprintln!("오류: {flag} 뒤에 값이 필요합니다.");
            eprintln!("사용법: {USAGE}");
            return Err(EXIT_USAGE);
        }
    };
    *i += 2;
    parse_u32(flag, raw)
}
