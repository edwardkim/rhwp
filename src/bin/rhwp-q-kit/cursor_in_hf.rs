//! 머리말·꼬리말 캐럿 사각형.

use crate::envelope::{
    envelope, load_core, parse_json_string, parse_u32, parse_usize, print_json, write_stdout,
    EXIT_RUNTIME, EXIT_USAGE,
};
use serde_json::{json, Value};

const CMD: &str = "cursor-in-hf";
const USAGE: &str = "rhwp-q-kit cursor-in-hf <파일> --section <N> --header|--footer --apply-to <N> --para <N> --offset <N> [--page <N>] [--json]";

pub fn run(args: &[String]) -> i32 {
    let opts = match parse(args) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let core = match load_core(&opts.path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let raw = match core.get_cursor_rect_in_header_footer_native(
        opts.section,
        opts.is_header,
        opts.apply_to,
        opts.para,
        opts.offset,
        opts.preview_page_hint,
    ) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("오류: 머리말·꼬리말 캐럿 사각형을 읽지 못했습니다 - {e}");
            return EXIT_RUNTIME;
        }
    };
    let native = match parse_json_string(&raw) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let mut payload = json!({
        "source": opts.path,
        "section": opts.section,
        "isHeader": opts.is_header,
        "applyTo": opts.apply_to,
        "para": opts.para,
        "offset": opts.offset,
        // 기존 JSON 소비자를 위해 키 이름은 유지한다. 값의 의미는 대표 페이지 힌트다.
        "preferredPage": opts.preview_page_hint,
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
    section: usize,
    is_header: bool,
    apply_to: u8,
    para: usize,
    offset: usize,
    preview_page_hint: i32,
}

fn parse(args: &[String]) -> Result<Opts, i32> {
    let mut json = false;
    let mut path: Option<String> = None;
    let mut section: Option<usize> = None;
    let mut is_header: Option<bool> = None;
    let mut apply_to: Option<u8> = None;
    let mut para: Option<usize> = None;
    let mut offset: Option<usize> = None;
    let mut page: Option<u32> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json = true;
                i += 1;
            }
            "--header" => {
                if is_header.replace(true).is_some() {
                    eprintln!("오류: --header 와 --footer 중 하나만 지정합니다.");
                    eprintln!("사용법: {USAGE}");
                    return Err(EXIT_USAGE);
                }
                i += 1;
            }
            "--footer" => {
                if is_header.replace(false).is_some() {
                    eprintln!("오류: --header 와 --footer 중 하나만 지정합니다.");
                    eprintln!("사용법: {USAGE}");
                    return Err(EXIT_USAGE);
                }
                i += 1;
            }
            "--section" => section = Some(take_usize(args, &mut i, "--section")?),
            "--apply-to" => apply_to = Some(take_apply_to(args, &mut i)?),
            "--para" => para = Some(take_usize(args, &mut i, "--para")?),
            "--offset" => offset = Some(take_usize(args, &mut i, "--offset")?),
            "--page" => page = Some(take_u32(args, &mut i, "--page")?),
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
        section: require(section, "--section")?,
        is_header: require(is_header, "--header|--footer")?,
        apply_to: require(apply_to, "--apply-to")?,
        para: require(para, "--para")?,
        offset: require(offset, "--offset")?,
        preview_page_hint: page.map(|p| p as i32).unwrap_or(-1),
    })
}

fn take_apply_to(args: &[String], i: &mut usize) -> Result<u8, i32> {
    let raw = take_val(args, *i, "--apply-to")?;
    *i += 2;
    let v = parse_u32("--apply-to", raw)?;
    if v > 2 {
        eprintln!("오류: --apply-to 는 0(양쪽)·1(짝수)·2(홀수) 만 허용합니다 - {raw}");
        return Err(EXIT_USAGE);
    }
    Ok(v as u8)
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
