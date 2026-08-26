//! 두 문서의 쪽수·텍스트를 비교한다.

use crate::envelope::{
    envelope, field_display_name, open_core, page_texts, print_json, text_hash, two_files,
    EXIT_GATE, EXIT_OK,
};
use serde_json::json;

fn texts(path: &str) -> Result<Vec<String>, i32> {
    let core = open_core(path)?;
    page_texts(&core).map_err(|m| {
        eprintln!("오류: {m}");
        crate::envelope::EXIT_RUNTIME
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

fn field_names(path: &str) -> Result<Vec<String>, i32> {
    let core = open_core(path)?;
    let mut names: Vec<String> = core
        .collect_all_fields()
        .iter()
        .map(field_display_name)
        .filter(|n| !n.is_empty())
        .collect();
    names.sort();
    names.dedup();
    Ok(names)
}

/// 실무 예제집 시나리오 11 — 두 서식의 누름틀 이름 차집합.
pub fn run_field_diff(args: &[String]) -> i32 {
    let usage = "rhwp-agent field-diff <파일A> <파일B> [--json]";
    let (json_mode, a, b) = match two_files(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let na = match field_names(&a) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let nb = match field_names(&b) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let only_a: Vec<&String> = na.iter().filter(|n| !nb.contains(n)).collect();
    let only_b: Vec<&String> = nb.iter().filter(|n| !na.contains(n)).collect();
    let shared: Vec<&String> = na.iter().filter(|n| nb.contains(n)).collect();
    let equal = only_a.is_empty() && only_b.is_empty();
    let payload = json!({
        "a": a,
        "b": b,
        "countA": na.len(),
        "countB": nb.len(),
        "sharedCount": shared.len(),
        "onlyInA": only_a,
        "onlyInB": only_b,
        "shared": shared,
        "equal": equal,
    });
    if json_mode {
        print_json(&envelope(
            "field-diff",
            payload,
            &["onlyInA[]", "onlyInB[]", "shared[]"],
        ));
    } else {
        crate::outln!(
            "equal={equal} onlyA={} onlyB={} shared={}",
            only_a.len(),
            only_b.len(),
            shared.len()
        );
    }
    if equal {
        EXIT_OK
    } else {
        EXIT_GATE
    }
}
