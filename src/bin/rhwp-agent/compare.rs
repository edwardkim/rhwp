//! 두 문서의 쪽수·텍스트를 비교한다.

use crate::envelope::{
    envelope, load_core, page_texts, print_json, read_file, text_hash, two_files, EXIT_GATE,
    EXIT_OK, EXIT_RUNTIME,
};
use serde_json::json;

fn texts(path: &str) -> Result<Vec<String>, i32> {
    let data = read_file(path).map_err(|m| {
        eprintln!("오류: {m}");
        EXIT_RUNTIME
    })?;
    let core = load_core(&data).map_err(|fail| {
        eprintln!("오류: 문서를 열 수 없습니다 - {path}: {}", fail.message);
        EXIT_RUNTIME
    })?;
    page_texts(&core).map_err(|m| {
        eprintln!("오류: {m}");
        EXIT_RUNTIME
    })
}

pub fn run_compare_pages(args: &[String]) -> i32 {
    let usage = "rhwp-agent compare-pages <파일A> <파일B> [--json]";
    let (json_mode, a, b) = match two_files(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let ta = match texts(&a) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let tb = match texts(&b) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let equal = ta.len() == tb.len();
    let payload = json!({
        "a": a, "b": b,
        "pageCountA": ta.len(),
        "pageCountB": tb.len(),
        "equal": equal,
        "delta": ta.len() as i64 - tb.len() as i64,
    });
    if json_mode {
        print_json(&envelope("compare-pages", payload, &[]));
    } else {
        crate::outln!("{} vs {} equal={equal}", ta.len(), tb.len());
    }
    if equal {
        EXIT_OK
    } else {
        EXIT_GATE
    }
}

pub fn run_compare_text(args: &[String]) -> i32 {
    let usage = "rhwp-agent compare-text <파일A> <파일B> [--json]";
    let (json_mode, a, b) = match two_files(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let ta = match texts(&a) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let tb = match texts(&b) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let ha = text_hash(&ta);
    let hb = text_hash(&tb);
    let equal = ha == hb;
    let payload = json!({
        "a": a, "b": b,
        "textHashA": ha,
        "textHashB": hb,
        "equal": equal,
    });
    if json_mode {
        print_json(&envelope("compare-text", payload, &[]));
    } else {
        crate::outln!("{}", if equal { "equal" } else { "differ" });
    }
    if equal {
        EXIT_OK
    } else {
        EXIT_GATE
    }
}

pub fn run_same_text(args: &[String]) -> i32 {
    run_compare_text(args)
}

pub fn run_page_delta(args: &[String]) -> i32 {
    run_compare_pages(args)
}
