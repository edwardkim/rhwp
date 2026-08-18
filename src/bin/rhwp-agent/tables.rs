//! 표 치수 조회. 표를 고치지 않는다.

use crate::envelope::{
    envelope, load_core, one_file, print_json, read_file, EXIT_OK, EXIT_RUNTIME,
};
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

pub fn run_tables(args: &[String]) -> i32 {
    let usage = "rhwp-agent tables <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let core = match core_of(&opts.path) {
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
    let core = match core_of(&opts.path) {
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
