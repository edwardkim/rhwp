//! 책갈피·차트·발췌·빈 누름틀·병합 표. 조회만 하고 문서를 고치지 않는다.

use crate::envelope::{
    envelope, field_display_name, hex_hash, one_file, open_core, page_texts, print_json, EXIT_GATE,
    EXIT_OK, EXIT_RUNTIME, EXIT_USAGE,
};
use rhwp::document_core::queries::chart_extract::collect_charts;
use rhwp::document_core::queries::table_extract::extract_tables;
use serde_json::json;

pub fn run_bookmarks(args: &[String]) -> i32 {
    let usage = "rhwp-agent bookmarks <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let core = match open_core(&opts.path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let raw = match core.get_bookmarks_native() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("오류: 책갈피를 읽지 못했습니다 - {e}");
            return EXIT_RUNTIME;
        }
    };
    let items: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("오류: 책갈피 JSON 이 깨졌습니다 - {e}");
            return EXIT_RUNTIME;
        }
    };
    let count = items.as_array().map(|a| a.len()).unwrap_or(0);
    let payload = json!({
        "source": opts.path,
        "bookmarkCount": count,
        "bookmarks": items,
    });
    if opts.json {
        print_json(&envelope("bookmarks", payload, &["bookmarks[].name"]));
    } else {
        crate::outln!("bookmarks={count}");
    }
    EXIT_OK
}

pub fn run_charts(args: &[String]) -> i32 {
    let usage = "rhwp-agent charts <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let core = match open_core(&opts.path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let charts = collect_charts(core.document());
    let items = serde_json::to_value(&charts).unwrap_or_else(|_| json!([]));
    let payload = json!({
        "source": opts.path,
        "chartCount": charts.len(),
        "charts": items,
    });
    if opts.json {
        print_json(&envelope("charts", payload, &[]));
    } else {
        crate::outln!("charts={}", charts.len());
    }
    EXIT_OK
}

pub fn run_digest(args: &[String]) -> i32 {
    let usage = "rhwp-agent digest <파일> [--max-chars <N>] [--json]";
    let mut json_mode = false;
    let mut path: Option<String> = None;
    let mut max_chars: usize = 240;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_mode = true;
                i += 1;
            }
            "--max-chars" => {
                let Some(v) = args
                    .get(i + 1)
                    .and_then(|s| s.parse().ok())
                    .filter(|n| *n > 0)
                else {
                    eprintln!("오류: --max-chars 뒤에 1 이상 숫자가 필요합니다.");
                    return EXIT_USAGE;
                };
                max_chars = v;
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
    let core = match open_core(&path) {
        Ok(c) => c,
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
        .map(|(i, text)| {
            let first_line = text
                .lines()
                .map(str::trim)
                .find(|l| !l.is_empty())
                .unwrap_or("")
                .to_string();
            let char_count = text.chars().count();
            let truncated = char_count > max_chars;
            let excerpt: String = text.chars().take(max_chars).collect();
            json!({
                "page": i,
                "charCount": char_count,
                "firstLine": first_line,
                "excerpt": excerpt,
                "truncated": truncated,
            })
        })
        .collect();
    let payload = json!({
        "source": path,
        "pageCount": pages.len(),
        "maxChars": max_chars,
        "pages": rows,
    });
    if json_mode {
        print_json(&envelope(
            "digest",
            payload,
            &["pages[].firstLine", "pages[].excerpt"],
        ));
    } else {
        for row in &rows {
            crate::outln!(
                "p{} chars={} {}",
                row["page"],
                row["charCount"],
                row["firstLine"].as_str().unwrap_or("")
            );
        }
    }
    EXIT_OK
}

pub fn run_page_hashes(args: &[String]) -> i32 {
    let usage = "rhwp-agent page-hashes <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let core = match open_core(&opts.path) {
        Ok(c) => c,
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
        .map(|(i, text)| {
            json!({
                "page": i,
                "charCount": text.chars().count(),
                "hash": hex_hash(text.as_bytes()),
            })
        })
        .collect();
    let payload = json!({
        "source": opts.path,
        "pageCount": pages.len(),
        "pages": rows,
    });
    if opts.json {
        print_json(&envelope("page-hashes", payload, &[]));
    } else {
        for row in &rows {
            crate::outln!(
                "p{}\t{}\t{}",
                row["page"],
                row["charCount"],
                row["hash"].as_str().unwrap_or("")
            );
        }
    }
    EXIT_OK
}

pub fn run_empty_fields(args: &[String]) -> i32 {
    let usage = "rhwp-agent empty-fields <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let core = match open_core(&opts.path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let fields = core.collect_all_fields();
    let empty: Vec<serde_json::Value> = fields
        .iter()
        .filter(|f| f.value.trim().is_empty())
        .map(|f| json!({ "name": field_display_name(f) }))
        .collect();
    let payload = json!({
        "source": opts.path,
        "fieldCount": fields.len(),
        "emptyCount": empty.len(),
        "empty": empty,
    });
    if opts.json {
        print_json(&envelope("empty-fields", payload, &["empty[].name"]));
    } else {
        crate::outln!("empty={}/{}", empty.len(), fields.len());
    }
    EXIT_OK
}

pub fn run_merged_tables(args: &[String]) -> i32 {
    let usage = "rhwp-agent merged-tables <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let core = match open_core(&opts.path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let tables = extract_tables(core.document());
    let merged: Vec<serde_json::Value> = tables
        .iter()
        .enumerate()
        .filter(|(_, t)| t.cells.iter().any(|c| c.row_span > 1 || c.col_span > 1))
        .map(|(i, t)| {
            let merge_count = t
                .cells
                .iter()
                .filter(|c| c.row_span > 1 || c.col_span > 1)
                .count();
            json!({
                "index": i,
                "rows": t.rows,
                "cols": t.cols,
                "mergeCount": merge_count,
            })
        })
        .collect();
    let payload = json!({
        "source": opts.path,
        "tableCount": tables.len(),
        "mergedCount": merged.len(),
        "tables": merged,
    });
    if opts.json {
        print_json(&envelope("merged-tables", payload, &[]));
    } else {
        crate::outln!("merged={}/{}", merged.len(), tables.len());
    }
    EXIT_OK
}

pub fn run_encrypted(args: &[String]) -> i32 {
    let usage = "rhwp-agent encrypted <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let core = match open_core(&opts.path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let encrypted = core.document().header.encrypted;
    let payload = json!({
        "source": opts.path,
        "encrypted": encrypted,
    });
    if opts.json {
        print_json(&envelope("encrypted", payload, &[]));
    } else {
        crate::outln!("{}", if encrypted { "yes" } else { "no" });
    }
    if encrypted {
        EXIT_GATE
    } else {
        EXIT_OK
    }
}

pub fn run_outline_nav(args: &[String]) -> i32 {
    let usage = "rhwp-agent outline-nav <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let core = match open_core(&opts.path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let raw = match core.get_outline_navigation_native() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("오류: 개요 번호를 읽지 못했습니다 - {e}");
            return EXIT_RUNTIME;
        }
    };
    let nav: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("오류: 개요 JSON 이 깨졌습니다 - {e}");
            return EXIT_RUNTIME;
        }
    };
    let count = nav
        .get("outline")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    let payload = json!({
        "source": opts.path,
        "outlineCount": count,
        "navigation": nav,
    });
    if opts.json {
        print_json(&envelope(
            "outline-nav",
            payload,
            &["navigation.outline[].title", "navigation.outline[].number"],
        ));
    } else {
        crate::outln!("outline={count}");
    }
    EXIT_OK
}

pub fn run_field_locate(args: &[String]) -> i32 {
    use rhwp::document_core::queries::field_query::NestedEntry;

    let usage = "rhwp-agent field-locate <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let core = match open_core(&opts.path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let fields = core.collect_all_fields();
    let rows: Vec<serde_json::Value> = fields
        .iter()
        .map(|f| {
            let nested: Vec<serde_json::Value> = f
                .location
                .nested_path
                .iter()
                .map(|n| match n {
                    NestedEntry::TableCell {
                        control_index,
                        cell_index,
                        para_index,
                    } => json!({
                        "kind": "tableCell",
                        "control": control_index,
                        "cell": cell_index,
                        "paragraph": para_index,
                    }),
                    NestedEntry::TextBox {
                        control_index,
                        para_index,
                    } => json!({
                        "kind": "textbox",
                        "control": control_index,
                        "paragraph": para_index,
                    }),
                })
                .collect();
            json!({
                "name": field_display_name(f),
                "value": f.value,
                "section": f.location.section_index,
                "paragraph": f.location.para_index,
                "listId": f.list_id,
                "startPos": f.start_pos,
                "endPos": f.end_pos,
                "nested": nested,
            })
        })
        .collect();
    let payload = json!({
        "source": opts.path,
        "fieldCount": rows.len(),
        "fields": rows,
    });
    if opts.json {
        print_json(&envelope(
            "field-locate",
            payload,
            &["fields[].name", "fields[].value"],
        ));
    } else {
        for r in &rows {
            crate::outln!(
                "{}\tsec={}\tpara={}\tlist={}",
                r["name"].as_str().unwrap_or(""),
                r["section"],
                r["paragraph"],
                r["listId"]
            );
        }
    }
    EXIT_OK
}

pub fn run_captions(args: &[String]) -> i32 {
    let usage = "rhwp-agent captions <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let core = match open_core(&opts.path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let tables = extract_tables(core.document());
    let caps: Vec<serde_json::Value> = tables
        .iter()
        .enumerate()
        .filter_map(|(i, t)| {
            t.caption.as_ref().map(|c| {
                json!({
                    "table": i,
                    "caption": c,
                })
            })
        })
        .collect();
    let payload = json!({
        "source": opts.path,
        "tableCount": tables.len(),
        "captionCount": caps.len(),
        "captions": caps,
    });
    if opts.json {
        print_json(&envelope("captions", payload, &["captions[].caption"]));
    } else {
        crate::outln!("captions={}/{}", caps.len(), tables.len());
    }
    EXIT_OK
}

pub fn run_headers_footers(args: &[String]) -> i32 {
    let usage = "rhwp-agent headers-footers <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let core = match open_core(&opts.path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let raw = match core.get_header_footer_list_native(0, true, 0) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("오류: 머리말·꼬리말을 읽지 못했습니다 - {e}");
            return EXIT_RUNTIME;
        }
    };
    let list: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("오류: 머리말 JSON 이 깨졌습니다 - {e}");
            return EXIT_RUNTIME;
        }
    };
    let count = list
        .get("items")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    let payload = json!({
        "source": opts.path,
        "itemCount": count,
        "list": list,
    });
    if opts.json {
        print_json(&envelope(
            "headers-footers",
            payload,
            &["list.items[].label"],
        ));
    } else {
        crate::outln!("headersFooters={count}");
    }
    EXIT_OK
}
