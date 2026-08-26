//! 빈 문서 여부 — 웹한글컨트롤 `IsEmpty`.

use crate::envelope::{envelope, load_core, parse_one_file, print_json, write_stdout};
use serde_json::json;

const USAGE: &str = "rhwp-q-kit empty-doc <파일> [--json]";

pub fn run(args: &[String]) -> i32 {
    let opts = match parse_one_file(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let core = match load_core(&opts.path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let empty = core.is_empty_document();
    let payload = json!({ "source": opts.path, "empty": empty });
    if opts.json {
        print_json(&envelope("empty-doc", payload, &[]))
    } else {
        write_stdout(if empty { "true" } else { "false" })
    }
}
