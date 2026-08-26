//! 쪽 각주 참조 좌표 (`sectionIdx`·`paraIdx`·`controlIdx`).

use crate::envelope::{
    envelope, load_core, parse_json_string, parse_u32, parse_usize, print_json, write_stdout,
    EXIT_RUNTIME, EXIT_USAGE,
};
use serde_json::{json, Value};

const CMD: &str = "footnote-info";
const USAGE: &str = "rhwp-q-kit footnote-info <파일> --page <N> --index <N> [--json]";

pub fn run(args: &[String]) -> i32 {
    let opts = match parse(args) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let core = match load_core(&opts.path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let raw = match core.get_page_footnote_info_native(opts.page, opts.index) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("오류: 각주 참조를 읽지 못했습니다 - {e}");
            return EXIT_RUNTIME;
        }
    };
    let native = match parse_json_string(&raw) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let mut payload = json!({
        "source": opts.path,
        "page": opts.page,
        "index": opts.index,
    });
    merge_object(&mut payload, native);
    if opts.json {
        print_json(&envelope(CMD, payload, &["source"]))
    } else {
        write_stdout(&raw)
    }
}

struct Opts {
    json: bool,
    path: String,
    page: u32,
    index: usize,
}

fn parse(args: &[String]) -> Result<Opts, i32> {
    let mut json = false;
    let mut path: Option<String> = None;
    let mut page: Option<u32> = None;
    let mut index: Option<usize> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json = true;
                i += 1;
            }
            "--page" => page = Some(take_u32(args, &mut i, "--page")?),
            "--index" => index = Some(take_usize(args, &mut i, "--index")?),
            other if other.starts_with('-') => unknown(other)?,
            other => {
                set_path(&mut path, other)?;
                i += 1;
            }
        }
    }
    Ok(Opts {
        json,
        path: require_path(path)?,
        page: require(page, "--page")?,
        index: require(index, "--index")?,
    })
}

fn take_u32(args: &[String], i: &mut usize, flag: &str) -> Result<u32, i32> {
    let raw = take_val(args, *i, flag)?;
    *i += 2;
    parse_u32(flag, raw)
}

fn take_usize(args: &[String], i: &mut usize, flag: &str) -> Result<usize, i32> {
    let raw = take_val(args, *i, flag)?;
    *i += 2;
    parse_usize(flag, raw)
}

fn take_val<'a>(args: &'a [String], i: usize, flag: &str) -> Result<&'a str, i32> {
    match args.get(i + 1) {
        Some(v) => Ok(v),
        None => {
            eprintln!("오류: {flag} 뒤에 값이 필요합니다.");
            eprintln!("사용법: {USAGE}");
            Err(EXIT_USAGE)
        }
    }
}

fn require<T>(v: Option<T>, flag: &str) -> Result<T, i32> {
    v.ok_or_else(|| {
        eprintln!("오류: {flag} 가 필요합니다.");
        eprintln!("사용법: {USAGE}");
        EXIT_USAGE
    })
}

fn require_path(path: Option<String>) -> Result<String, i32> {
    path.ok_or_else(|| {
        eprintln!("오류: 파일 경로가 필요합니다.");
        eprintln!("사용법: {USAGE}");
        EXIT_USAGE
    })
}

fn set_path(path: &mut Option<String>, other: &str) -> Result<(), i32> {
    if path.replace(other.to_string()).is_some() {
        eprintln!("오류: 파일이 너무 많습니다 - {other}");
        eprintln!("사용법: {USAGE}");
        return Err(EXIT_USAGE);
    }
    Ok(())
}

fn unknown(flag: &str) -> Result<(), i32> {
    eprintln!("오류: 알 수 없는 옵션입니다 - {flag}");
    eprintln!("사용법: {USAGE}");
    Err(EXIT_USAGE)
}

fn merge_object(payload: &mut Value, native: Value) {
    if let (Some(dst), Value::Object(src)) = (payload.as_object_mut(), native) {
        for (k, v) in src {
            dst.insert(k, v);
        }
    }
}
