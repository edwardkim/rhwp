//! 문서가 무엇인지 한 봉투로. 기존 조회 개수만 모으며 문서를 고치지 않는다.

use crate::envelope::{
    envelope, format_token, load_core, one_file, print_json, read_file, EXIT_OK, EXIT_RUNTIME,
};
use rhwp::document_core::queries::chart_extract::collect_charts;
use rhwp::document_core::queries::table_extract::extract_tables;
use serde_json::json;

fn open(path: &str) -> Result<(rhwp::document_core::DocumentCore, &'static str), i32> {
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
    Ok((core, format))
}

pub fn run_explain(args: &[String]) -> i32 {
    let usage = "rhwp-agent explain <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let (core, format) = match open(&opts.path) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let document = core.document();
    let para_count: usize = document.sections.iter().map(|s| s.paragraphs.len()).sum();
    let tables = extract_tables(document);
    let field_count = core.collect_all_fields().len();
    let notes = rhwp::document_core::queries::explain::count_notes(document);
    let chart_count = collect_charts(document).len();
    let page_count = core.page_count();
    let summary = format!(
        "{format} · {page_count}쪽 · 문단 {para_count} · 표 {} · 누름틀 {field_count} · 각주 {} · 미주 {}",
        tables.len(),
        notes.footnote_count,
        notes.endnote_count
    );
    let payload = json!({
        "source": opts.path,
        "format": format,
        "pageCount": page_count,
        "paraCount": para_count,
        "sectionCount": document.sections.len(),
        "tableCount": tables.len(),
        "fieldCount": field_count,
        "chartCount": chart_count,
        "footnoteCount": notes.footnote_count,
        "endnoteCount": notes.endnote_count,
        "encrypted": document.header.encrypted,
        "summary": summary,
    });
    if opts.json {
        print_json(&envelope("explain", payload, &["summary"]));
    } else {
        crate::outln!("{summary}");
    }
    EXIT_OK
}

pub fn run_notes(args: &[String]) -> i32 {
    let usage = "rhwp-agent notes <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let (core, _) = match open(&opts.path) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let notes = rhwp::document_core::queries::explain::count_notes(core.document());
    let payload = json!({
        "source": opts.path,
        "footnoteCount": notes.footnote_count,
        "endnoteCount": notes.endnote_count,
        "noteCount": notes.footnote_count + notes.endnote_count,
    });
    if opts.json {
        print_json(&envelope("notes", payload, &[]));
    } else {
        crate::outln!(
            "footnotes={} endnotes={}",
            notes.footnote_count,
            notes.endnote_count
        );
    }
    EXIT_OK
}
