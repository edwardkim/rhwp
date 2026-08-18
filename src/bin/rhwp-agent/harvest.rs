//! 실무 예제집 시나리오 1·2·17 — 필드 값·표 CSV·날짜/금액 수확. 읽기 전용.

use crate::envelope::{
    envelope, load_core, one_file, print_json, read_file, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE,
};
use rhwp::document_core::queries::extract_data::DataKind;
use rhwp::document_core::queries::table_csv::grid_to_csv;
use rhwp::document_core::queries::table_extract::extract_tables;
use serde_json::json;

fn core_of(path: &str) -> Result<rhwp::document_core::DocumentCore, i32> {
    let data = read_file(path).map_err(|m| {
        eprintln!("오류: {m}");
        EXIT_RUNTIME
    })?;
    load_core(&data).map_err(|fail| {
        eprintln!("오류: 문서를 열 수 없습니다 - {path}: {}", fail.message);
        EXIT_RUNTIME
    })
}

pub fn run_extract_data(args: &[String]) -> i32 {
    let usage =
        "rhwp-agent extract-data <파일> [--kind date|amount|number|all] [--json] [--limit <N>]";
    let mut json_mode = false;
    let mut path: Option<String> = None;
    let mut kind = "all".to_string();
    let mut limit: usize = 200;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_mode = true;
                i += 1;
            }
            "--kind" => {
                let Some(v) = args.get(i + 1) else {
                    eprintln!("오류: --kind 뒤에 date|amount|number|all 이 필요합니다.");
                    return EXIT_USAGE;
                };
                kind = v.clone();
                i += 2;
            }
            "--limit" => {
                let Some(v) = args.get(i + 1).and_then(|s| s.parse().ok()) else {
                    eprintln!("오류: --limit 뒤에 숫자가 필요합니다.");
                    return EXIT_USAGE;
                };
                limit = v;
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
    let kinds: Vec<DataKind> = if kind == "all" {
        DataKind::ALL.to_vec()
    } else {
        let Some(k) = DataKind::parse(&kind) else {
            eprintln!("오류: --kind 는 date|amount|number|all 이어야 합니다.");
            return EXIT_USAGE;
        };
        vec![k]
    };
    let core = match core_of(&path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let mut items = core.extract_data(&kinds);
    let total = items.len();
    let truncated = items.len() > limit;
    items.truncate(limit);
    let payload = json!({
        "source": path,
        "kind": kind,
        "totalItemCount": total,
        "emittedCount": items.len(),
        "truncated": truncated,
        "items": items,
    });
    if json_mode {
        print_json(&envelope(
            "extract-data",
            payload,
            &["items[].raw", "items[].normalized"],
        ));
    } else {
        crate::outln!("items={total}");
    }
    EXIT_OK
}

pub fn run_field_values(args: &[String]) -> i32 {
    let usage = "rhwp-agent field-values <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let core = match core_of(&opts.path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let fields = core.collect_all_fields();
    let rows: Vec<serde_json::Value> = fields
        .iter()
        .map(|f| {
            let name = f
                .field
                .ctrl_data_name
                .clone()
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| f.field.command.clone());
            json!({ "name": name, "value": f.value })
        })
        .collect();
    let empty = rows
        .iter()
        .filter(|r| r["value"].as_str().unwrap_or("").is_empty())
        .count();
    let payload = json!({
        "source": opts.path,
        "fieldCount": rows.len(),
        "emptyCount": empty,
        "fields": rows,
    });
    if opts.json {
        print_json(&envelope(
            "field-values",
            payload,
            &["fields[].name", "fields[].value"],
        ));
    } else {
        for r in &rows {
            crate::outln!(
                "{}\t{}",
                r["name"].as_str().unwrap_or(""),
                r["value"].as_str().unwrap_or("")
            );
        }
    }
    EXIT_OK
}

pub fn run_table_csv(args: &[String]) -> i32 {
    let usage = "rhwp-agent table-csv <파일> [--table <N>] [--json]";
    let mut json_mode = false;
    let mut path: Option<String> = None;
    let mut table: usize = 0;
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
                    return EXIT_USAGE;
                };
                table = v;
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
    let core = match core_of(&path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let tables = extract_tables(core.document());
    if tables.is_empty() {
        eprintln!("오류: 표가 없습니다.");
        return EXIT_RUNTIME;
    }
    if table >= tables.len() {
        eprintln!(
            "오류: --table {table} 이 tableCount={} 이상입니다.",
            tables.len()
        );
        return EXIT_USAGE;
    }
    let csv = grid_to_csv(&tables[table]);
    let payload = json!({
        "source": path,
        "table": table,
        "tableCount": tables.len(),
        "rows": tables[table].rows,
        "cols": tables[table].cols,
        "csv": csv,
    });
    if json_mode {
        print_json(&envelope("table-csv", payload, &["csv"]));
    } else {
        crate::outp!("{csv}");
    }
    EXIT_OK
}

pub fn run_form_ready(args: &[String]) -> i32 {
    let usage = "rhwp-agent form-ready <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let core = match core_of(&opts.path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let n = core.collect_all_fields().len();
    let ready = n > 0;
    let payload = json!({
        "source": opts.path,
        "fieldCount": n,
        "ready": ready,
        "next": if ready { "edit fill-fields" } else { "edit set-cell" },
    });
    if opts.json {
        print_json(&envelope("form-ready", payload, &[]));
    } else {
        crate::outln!("{}", if ready { "fields" } else { "cells" });
    }
    if ready {
        EXIT_OK
    } else {
        crate::envelope::EXIT_GATE
    }
}
