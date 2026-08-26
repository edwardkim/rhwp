//! 표 치수 조회. 표를 고치지 않는다.

use crate::envelope::{envelope, one_file, open_core, print_json, EXIT_OK};
use serde_json::json;

pub fn run_tables(args: &[String]) -> i32 {
    let usage = "rhwp-agent tables <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let core = match open_core(&opts.path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let tables = rhwp::document_core::queries::table_extract::extract_tables(core.document());
    let rows: Vec<serde_json::Value> = tables
        .iter()
        .enumerate()
        .map(|(i, t)| {
            json!({
                "index": i,
                "rows": t.rows,
                "cols": t.cols,
            })
        })
        .collect();
    let payload = json!({
        "source": opts.path,
        "tableCount": tables.len(),
        "tables": rows,
    });
    if opts.json {
        print_json(&envelope("tables", payload, &[]));
    } else {
        for t in &rows {
            crate::outln!("{}\t{}x{}", t["index"], t["rows"], t["cols"]);
        }
    }
    EXIT_OK
}

pub fn run_table_count(args: &[String]) -> i32 {
    let usage = "rhwp-agent table-count <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let core = match open_core(&opts.path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let n = rhwp::document_core::queries::table_extract::extract_tables(core.document()).len();
    let payload = json!({ "source": opts.path, "tableCount": n });
    if opts.json {
        print_json(&envelope("table-count", payload, &[]));
    } else {
        crate::outln!("{n}");
    }
    EXIT_OK
}

pub fn run_table_dims(args: &[String]) -> i32 {
    run_tables(args)
}

/// 레시피 2 1단계 — 표 격자·병합·셀 텍스트. `extract_tables` 만 부른다.
pub fn run_table_inspect(args: &[String]) -> i32 {
    let usage = "rhwp-agent table-inspect <파일> [--table <N>] [--json]";
    let mut json_mode = false;
    let mut path: Option<String> = None;
    let mut table: Option<usize> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_mode = true;
                i += 1;
            }
            "--table" => {
                let Some(v) = args.get(i + 1).and_then(|s| s.parse().ok()) else {
                    eprintln!("오류: --table 뒤에 표 번호가 필요합니다.");
                    return crate::envelope::EXIT_USAGE;
                };
                table = Some(v);
                i += 2;
            }
            other if other.starts_with('-') => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {other}");
                eprintln!("사용법: {usage}");
                return crate::envelope::EXIT_USAGE;
            }
            other => {
                if path.is_some() {
                    eprintln!("오류: 파일이 너무 많습니다.");
                    return crate::envelope::EXIT_USAGE;
                }
                path = Some(other.to_string());
                i += 1;
            }
        }
    }
    let Some(path) = path else {
        eprintln!("오류: 파일 경로가 필요합니다.");
        eprintln!("사용법: {usage}");
        return crate::envelope::EXIT_USAGE;
    };
    let core = match open_core(&path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let tables = rhwp::document_core::queries::table_extract::extract_tables(core.document());
    if let Some(idx) = table {
        if idx >= tables.len() {
            eprintln!(
                "오류: --table {idx} 이 tableCount={} 이상입니다.",
                tables.len()
            );
            return crate::envelope::EXIT_USAGE;
        }
    }
    let selected: Vec<&rhwp::document_core::queries::table_extract::TableGrid> = match table {
        Some(idx) => vec![&tables[idx]],
        None => tables.iter().collect(),
    };
    let summaries: Vec<serde_json::Value> = selected
        .iter()
        .map(|t| {
            let merge_count = t
                .cells
                .iter()
                .filter(|c| c.row_span > 1 || c.col_span > 1)
                .count();
            json!({
                "index": t.index,
                "section": t.section,
                "paragraph": t.paragraph,
                "control": t.control,
                "rows": t.rows,
                "cols": t.cols,
                "cellCount": t.cell_count,
                "mergeCount": merge_count,
                "csvReady": merge_count == 0,
                "caption": t.caption,
                "cells": t.cells,
            })
        })
        .collect();
    let payload = json!({
        "source": path,
        "tableCount": tables.len(),
        "emittedCount": summaries.len(),
        "tables": summaries,
    });
    if json_mode {
        print_json(&envelope(
            "table-inspect",
            payload,
            &["tables[].cells[].text", "tables[].caption"],
        ));
    } else {
        for t in &summaries {
            crate::outln!(
                "{}\t{}x{}\tmerges={}\tcsvReady={}",
                t["index"],
                t["rows"],
                t["cols"],
                t["mergeCount"],
                t["csvReady"]
            );
        }
    }
    EXIT_OK
}
