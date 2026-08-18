//! 누름틀 목록. 편집하지 않는다.

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

pub fn run_fields(args: &[String]) -> i32 {
    let usage = "rhwp-agent fields <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let core = match core_of(&opts.path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let fields = core.collect_all_fields();
    let mut names: Vec<String> = fields
        .iter()
        .map(|f| {
            f.field
                .ctrl_data_name
                .clone()
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| f.field.command.clone())
        })
        .filter(|n| !n.is_empty())
        .collect();
    names.sort();
    names.dedup();
    let payload = json!({
        "source": opts.path,
        "fieldCount": fields.len(),
        "uniqueNames": names.len(),
        "names": names,
    });
    if opts.json {
        print_json(&envelope("fields", payload, &["names[]"]));
    } else {
        for n in &payload["names"].as_array().cloned().unwrap_or_default() {
            crate::outln!("{}", n.as_str().unwrap_or(""));
        }
    }
    EXIT_OK
}

pub fn run_field_count(args: &[String]) -> i32 {
    let usage = "rhwp-agent field-count <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let core = match core_of(&opts.path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let n = core.collect_all_fields().len();
    let payload = json!({ "source": opts.path, "fieldCount": n });
    if opts.json {
        print_json(&envelope("field-count", payload, &[]));
    } else {
        crate::outln!("{n}");
    }
    EXIT_OK
}

pub fn run_field_names(args: &[String]) -> i32 {
    run_fields(args)
}
