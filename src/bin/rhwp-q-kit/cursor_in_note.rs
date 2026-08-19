//! 노트(각주·미주) 안 캐럿 사각형.

use crate::envelope::{
    envelope, load_core, parse_json_string, parse_usize, print_json, write_stdout, EXIT_RUNTIME,
    EXIT_USAGE,
};
use serde_json::{json, Value};

const CMD: &str = "cursor-in-note";
const USAGE: &str = "rhwp-q-kit cursor-in-note <파일> --section <N> --para <N> --ci <N> --note-para <N> --offset <N> [--json]";

pub fn run(args: &[String]) -> i32 {
    let opts = match parse(args) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let core = match load_core(&opts.path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let raw = match core.get_cursor_rect_in_note_native(
        opts.section,
        opts.para,
        opts.ci,
        opts.note_para,
        opts.offset,
    ) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("오류: 노트 캐럿 사각형을 읽지 못했습니다 - {e}");
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
        "para": opts.para,
        "ci": opts.ci,
        "notePara": opts.note_para,
        "offset": opts.offset,
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
    para: usize,
    ci: usize,
    note_para: usize,
    offset: usize,
}

fn parse(args: &[String]) -> Result<Opts, i32> {
    let mut json = false;
    let mut path: Option<String> = None;
    let mut section: Option<usize> = None;
    let mut para: Option<usize> = None;
    let mut ci: Option<usize> = None;
    let mut note_para: Option<usize> = None;
    let mut offset: Option<usize> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json = true;
                i += 1;
            }
            "--section" => section = Some(take_usize(args, &mut i, "--section")?),
            "--para" => para = Some(take_usize(args, &mut i, "--para")?),
            "--ci" => ci = Some(take_usize(args, &mut i, "--ci")?),
            "--note-para" => note_para = Some(take_usize(args, &mut i, "--note-para")?),
            "--offset" => offset = Some(take_usize(args, &mut i, "--offset")?),
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
        para: require(para, "--para")?,
        ci: require(ci, "--ci")?,
        note_para: require(note_para, "--note-para")?,
        offset: require(offset, "--offset")?,
    })
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
