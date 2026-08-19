//! 문서 한 건을 열어 쪽·포맷·창·빈 쪽을 기계 판정한다. 편집 로직은 만들지 않는다.

use crate::envelope::{
    envelope, format_token, load_core, one_file, one_file_value, page_texts, print_json, read_file,
    EXIT_OK, EXIT_RUNTIME, EXIT_USAGE,
};
use serde_json::json;

fn open(path: &str) -> Result<(rhwp::document_core::DocumentCore, &'static str, u64), i32> {
    let data = match read_file(path) {
        Ok(d) => d,
        Err(m) => {
            eprintln!("오류: {m}");
            return Err(EXIT_RUNTIME);
        }
    };
    let format = format_token(rhwp::parser::detect_format(&data));
    let core = match load_core(&data) {
        Ok(c) => c,
        Err(fail) => {
            eprintln!("오류: 문서를 열 수 없습니다 - {path}: {}", fail.message);
            return Err(EXIT_RUNTIME);
        }
    };
    Ok((core, format, data.len() as u64))
}

pub fn run_info(args: &[String]) -> i32 {
    let usage = "rhwp-agent info <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let (core, format, bytes) = match open(&opts.path) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let pages = match page_texts(&core) {
        Ok(p) => p,
        Err(m) => {
            eprintln!("오류: {m}");
            return EXIT_RUNTIME;
        }
    };
    let document = core.document();
    let para_count: u64 = document
        .sections
        .iter()
        .map(|s| s.paragraphs.len() as u64)
        .sum();
    let table_count =
        rhwp::document_core::queries::table_extract::extract_tables(document).len() as u64;
    let field_count = core.collect_all_fields().len() as u64;
    let char_count: u64 = pages.iter().map(|p| p.chars().count() as u64).sum();
    let payload = json!({
        "source": opts.path,
        "format": format,
        "bytes": bytes,
        "pageCount": core.page_count(),
        "paraCount": para_count,
        "tableCount": table_count,
        "fieldCount": field_count,
        "charCount": char_count,
        "sectionCount": document.sections.len(),
    });
    if opts.json {
        print_json(&envelope("info", payload, &["source"]));
    } else {
        crate::outln!(
            "{} format={} pages={} paras={} tables={} fields={} chars={}",
            opts.path,
            format,
            core.page_count(),
            para_count,
            table_count,
            field_count,
            char_count
        );
    }
    EXIT_OK
}

pub fn run_format(args: &[String]) -> i32 {
    let usage = "rhwp-agent format <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let data = match read_file(&opts.path) {
        Ok(d) => d,
        Err(m) => {
            eprintln!("오류: {m}");
            return EXIT_RUNTIME;
        }
    };
    let format = format_token(rhwp::parser::detect_format(&data));
    let payload = json!({ "source": opts.path, "format": format, "bytes": data.len() });
    if opts.json {
        print_json(&envelope("format", payload, &[]));
    } else {
        crate::outln!("{format}");
    }
    EXIT_OK
}

pub fn run_pages(args: &[String]) -> i32 {
    let usage = "rhwp-agent pages <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let (core, _, _) = match open(&opts.path) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let pages = match page_texts(&core) {
        Ok(p) => p,
        Err(m) => {
            eprintln!("오류: {m}");
            return EXIT_RUNTIME;
        }
    };
    let rows: Vec<serde_json::Value> = pages
        .iter()
        .enumerate()
        .map(
            |(i, t)| json!({ "page": i, "chars": t.chars().count(), "empty": t.trim().is_empty() }),
        )
        .collect();
    let payload = json!({ "source": opts.path, "pageCount": pages.len(), "pages": rows });
    if opts.json {
        print_json(&envelope("pages", payload, &[]));
    } else {
        for (i, t) in pages.iter().enumerate() {
            crate::outln!("{i}\t{}", t.chars().count());
        }
    }
    EXIT_OK
}

pub fn run_page_window(args: &[String]) -> i32 {
    let usage = "rhwp-agent page-window <파일> --from <N> --to <N> [--json]";
    let mut json_mode = false;
    let mut path: Option<String> = None;
    let mut from: Option<u32> = None;
    let mut to: Option<u32> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_mode = true;
                i += 1;
            }
            "--from" => {
                let Some(v) = args.get(i + 1).and_then(|s| s.parse().ok()) else {
                    eprintln!("오류: --from 뒤에 쪽 번호가 필요합니다.");
                    return EXIT_USAGE;
                };
                from = Some(v);
                i += 2;
            }
            "--to" => {
                let Some(v) = args.get(i + 1).and_then(|s| s.parse().ok()) else {
                    eprintln!("오류: --to 뒤에 쪽 번호가 필요합니다.");
                    return EXIT_USAGE;
                };
                to = Some(v);
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
                    return EXIT_USAGE;
                }
                path = Some(other.to_string());
                i += 1;
            }
        }
    }
    let Some(path) = path else {
        eprintln!("오류: 파일 경로가 필요합니다.");
        eprintln!("사용법: {usage}");
        return EXIT_USAGE;
    };
    let Some(from) = from else {
        eprintln!("오류: --from 이 필요합니다.");
        return EXIT_USAGE;
    };
    let Some(to) = to else {
        eprintln!("오류: --to 가 필요합니다.");
        return EXIT_USAGE;
    };
    if to < from {
        eprintln!("오류: --to 는 --from 이상이어야 합니다.");
        return EXIT_USAGE;
    }
    let (core, _, _) = match open(&path) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let pages = match page_texts(&core) {
        Ok(p) => p,
        Err(m) => {
            eprintln!("오류: {m}");
            return EXIT_RUNTIME;
        }
    };
    let n = pages.len() as u32;
    if from >= n || to >= n {
        eprintln!("오류: 쪽 범위가 pageCount={n} 을 벗어납니다.");
        return EXIT_USAGE;
    }
    let slice: Vec<String> = pages[from as usize..=to as usize].to_vec();
    let payload = json!({
        "source": path,
        "from": from,
        "to": to,
        "emittedCount": slice.len(),
        "pageCount": n,
        "chars": slice.iter().map(|s| s.chars().count()).sum::<usize>(),
    });
    if json_mode {
        print_json(&envelope("page-window", payload, &[]));
    } else {
        for (offset, text) in slice.iter().enumerate() {
            crate::outln!("--- page {} ---", from as usize + offset);
            crate::outp!("{text}");
        }
    }
    EXIT_OK
}

pub fn run_empty_pages(args: &[String]) -> i32 {
    let usage = "rhwp-agent empty-pages <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let (core, _, _) = match open(&opts.path) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let pages = match page_texts(&core) {
        Ok(p) => p,
        Err(m) => {
            eprintln!("오류: {m}");
            return EXIT_RUNTIME;
        }
    };
    let empty: Vec<u32> = pages
        .iter()
        .enumerate()
        .filter(|(_, t)| t.trim().is_empty())
        .map(|(i, _)| i as u32)
        .collect();
    let payload = json!({
        "source": opts.path,
        "pageCount": pages.len(),
        "emptyCount": empty.len(),
        "emptyPages": empty,
    });
    if opts.json {
        print_json(&envelope("empty-pages", payload, &[]));
    } else {
        crate::outln!("{}", empty.len());
    }
    EXIT_OK
}

pub fn run_char_count(args: &[String]) -> i32 {
    let usage = "rhwp-agent char-count <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let (core, _, _) = match open(&opts.path) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let pages = match page_texts(&core) {
        Ok(p) => p,
        Err(m) => {
            eprintln!("오류: {m}");
            return EXIT_RUNTIME;
        }
    };
    let n: u64 = pages.iter().map(|p| p.chars().count() as u64).sum();
    let payload = json!({ "source": opts.path, "charCount": n, "pageCount": pages.len() });
    if opts.json {
        print_json(&envelope("char-count", payload, &[]));
    } else {
        crate::outln!("{n}");
    }
    EXIT_OK
}

pub fn run_para_count(args: &[String]) -> i32 {
    let usage = "rhwp-agent para-count <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let (core, _, _) = match open(&opts.path) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let n: u64 = core
        .document()
        .sections
        .iter()
        .map(|s| s.paragraphs.len() as u64)
        .sum();
    let payload = json!({ "source": opts.path, "paraCount": n, "sectionCount": core.document().sections.len() });
    if opts.json {
        print_json(&envelope("para-count", payload, &[]));
    } else {
        crate::outln!("{n}");
    }
    EXIT_OK
}

pub fn run_sample_text(args: &[String]) -> i32 {
    let usage = "rhwp-agent sample-text <파일> --max-chars <N> [--json]";
    let (opts, max) = match one_file_value(args, usage, "--max-chars") {
        Ok(v) => v,
        Err(c) => return c,
    };
    let Some(max) = max.and_then(|s| s.parse::<usize>().ok()).filter(|n| *n > 0) else {
        eprintln!("오류: --max-chars 는 1 이상이어야 합니다.");
        return EXIT_USAGE;
    };
    let (core, _, _) = match open(&opts.path) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let pages = match page_texts(&core) {
        Ok(p) => p,
        Err(m) => {
            eprintln!("오류: {m}");
            return EXIT_RUNTIME;
        }
    };
    let joined = pages.join("\n");
    let truncated = joined.chars().count() > max;
    let sample: String = joined.chars().take(max).collect();
    let payload = json!({
        "source": opts.path,
        "maxChars": max,
        "truncated": truncated,
        "sample": sample,
    });
    if opts.json {
        print_json(&envelope("sample-text", payload, &["sample"]));
    } else {
        crate::outp!("{sample}");
    }
    EXIT_OK
}

pub fn run_outline(args: &[String]) -> i32 {
    let usage = "rhwp-agent outline <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let (core, _, _) = match open(&opts.path) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let pages = match page_texts(&core) {
        Ok(p) => p,
        Err(m) => {
            eprintln!("오류: {m}");
            return EXIT_RUNTIME;
        }
    };
    let lines: Vec<serde_json::Value> = pages
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let first = t
                .lines()
                .find(|l| !l.trim().is_empty())
                .unwrap_or("")
                .trim();
            json!({ "page": i, "firstLine": first })
        })
        .collect();
    let payload = json!({ "source": opts.path, "pageCount": pages.len(), "outline": lines });
    if opts.json {
        print_json(&envelope("outline", payload, &["outline[].firstLine"]));
    } else {
        for row in &lines {
            crate::outln!(
                "{}\t{}",
                row["page"],
                row["firstLine"].as_str().unwrap_or("")
            );
        }
    }
    EXIT_OK
}

pub fn run_batch_info(args: &[String]) -> i32 {
    let usage = "rhwp-agent batch-info <파일...> [--json]";
    let mut json_mode = false;
    let mut paths: Vec<String> = Vec::new();
    for arg in args {
        match arg.as_str() {
            "--json" => json_mode = true,
            other if other.starts_with('-') => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {other}");
                eprintln!("사용법: {usage}");
                return EXIT_USAGE;
            }
            other => paths.push(other.to_string()),
        }
    }
    if paths.is_empty() {
        eprintln!("오류: 파일 경로가 필요합니다.");
        eprintln!("사용법: {usage}");
        return EXIT_USAGE;
    }
    let mut files = Vec::new();
    let mut failed = 0u64;
    for path in &paths {
        match open(path) {
            Ok((core, format, bytes)) => {
                let page_count = core.page_count();
                let table_count =
                    rhwp::document_core::queries::table_extract::extract_tables(core.document())
                        .len() as u64;
                let field_count = core.collect_all_fields().len() as u64;
                files.push(json!({
                    "source": path,
                    "ok": true,
                    "format": format,
                    "bytes": bytes,
                    "pageCount": page_count,
                    "tableCount": table_count,
                    "fieldCount": field_count,
                    "sectionCount": core.document().sections.len(),
                }));
            }
            Err(_) => {
                failed += 1;
                files.push(json!({
                    "source": path,
                    "ok": false,
                }));
            }
        }
    }
    let payload = json!({
        "fileCount": paths.len(),
        "okCount": paths.len() as u64 - failed,
        "failCount": failed,
        "files": files,
    });
    if json_mode {
        print_json(&envelope("batch-info", payload, &["files[].source"]));
    } else {
        crate::outln!("ok={} fail={}", paths.len() as u64 - failed, failed);
    }
    EXIT_OK
}
