//! 누름틀 목록. 편집하지 않는다.

use crate::envelope::{envelope, field_display_name, one_file, open_core, print_json, EXIT_OK};
use serde_json::json;

pub fn run_fields(args: &[String]) -> i32 {
    let usage = "rhwp-agent fields <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let core = match open_core(&opts.path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let fields = core.collect_all_fields();
    let mut names: Vec<String> = fields
        .iter()
        .map(field_display_name)
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
    let core = match open_core(&opts.path) {
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
