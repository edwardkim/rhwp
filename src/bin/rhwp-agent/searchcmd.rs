//! 본문 검색. 조회만 하고 문서를 고치지 않는다.

use crate::envelope::{
    envelope, load_core, one_file_value, page_texts, print_json, read_file, EXIT_GATE, EXIT_OK,
    EXIT_RUNTIME, EXIT_USAGE,
};
use serde_json::json;

fn pages_of(path: &str) -> Result<Vec<String>, i32> {
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

pub fn run_search(args: &[String]) -> i32 {
    let usage = "rhwp-agent search <파일> --q <문자열> [--json]";
    let (opts, q) = match one_file_value(args, usage, "--q") {
        Ok(v) => v,
        Err(c) => return c,
    };
    let Some(q) = q.filter(|s| !s.is_empty()) else {
        eprintln!("오류: --q 검색어가 필요합니다.");
        return EXIT_USAGE;
    };
    let pages = match pages_of(&opts.path) {
        Ok(p) => p,
        Err(c) => return c,
    };
    let mut matches = Vec::new();
    for (i, text) in pages.iter().enumerate() {
        let mut start = 0usize;
        while let Some(rel) = text[start..].find(&q) {
            let at = start + rel;
            matches.push(json!({ "page": i, "offset": at }));
            start = at + q.len().max(1);
            if matches.len() >= 500 {
                break;
            }
        }
        if matches.len() >= 500 {
            break;
        }
    }
    let truncated = matches.len() >= 500;
    let payload = json!({
        "source": opts.path,
        "query": q,
        "matchCount": matches.len(),
        "truncated": truncated,
        "matches": matches,
    });
    if opts.json {
        print_json(&envelope("search", payload, &["query"]));
    } else {
        crate::outln!("{}", payload["matchCount"]);
    }
    EXIT_OK
}

pub fn run_search_count(args: &[String]) -> i32 {
    run_search(args)
}

pub fn run_contains(args: &[String]) -> i32 {
    let usage = "rhwp-agent contains <파일> --q <문자열> [--json]";
    let (opts, q) = match one_file_value(args, usage, "--q") {
        Ok(v) => v,
        Err(c) => return c,
    };
    let Some(q) = q.filter(|s| !s.is_empty()) else {
        eprintln!("오류: --q 검색어가 필요합니다.");
        return EXIT_USAGE;
    };
    let pages = match pages_of(&opts.path) {
        Ok(p) => p,
        Err(c) => return c,
    };
    let found = pages.iter().any(|t| t.contains(&q));
    let payload = json!({ "source": opts.path, "query": q, "found": found });
    if opts.json {
        print_json(&envelope("contains", payload, &["query"]));
    } else {
        crate::outln!("{}", if found { "found" } else { "missing" });
    }
    if found {
        EXIT_OK
    } else {
        EXIT_GATE
    }
}

pub fn run_grep_pages(args: &[String]) -> i32 {
    let usage = "rhwp-agent grep-pages <파일> --q <문자열> [--json]";
    let (opts, q) = match one_file_value(args, usage, "--q") {
        Ok(v) => v,
        Err(c) => return c,
    };
    let Some(q) = q.filter(|s| !s.is_empty()) else {
        eprintln!("오류: --q 검색어가 필요합니다.");
        return EXIT_USAGE;
    };
    let pages = match pages_of(&opts.path) {
        Ok(p) => p,
        Err(c) => return c,
    };
    let hits: Vec<u32> = pages
        .iter()
        .enumerate()
        .filter(|(_, t)| t.contains(&q))
        .map(|(i, _)| i as u32)
        .collect();
    let payload = json!({
        "source": opts.path,
        "query": q,
        "pageCount": pages.len(),
        "hitCount": hits.len(),
        "pages": hits,
    });
    if opts.json {
        print_json(&envelope("grep-pages", payload, &["query"]));
    } else {
        for p in hits {
            crate::outln!("{p}");
        }
    }
    EXIT_OK
}
