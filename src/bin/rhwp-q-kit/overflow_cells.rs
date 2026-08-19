//! 넘친 셀 줄 수 — `DocumentCore::take_overflow_cell_lines`.
//!
//! 카운터를 읽고 리셋하므로 방금 연 코어의 잔여값만 보고한다.

use crate::envelope::{envelope, load_core, parse_one_file, print_json, write_stdout};
use serde_json::json;

const USAGE: &str = "rhwp-q-kit overflow-cells <파일> [--json]";

pub fn run(args: &[String]) -> i32 {
    let opts = match parse_one_file(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let core = match load_core(&opts.path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let count = core.take_overflow_cell_lines();
    if opts.json {
        print_json(&envelope(
            "overflow-cells",
            json!({
                "source": opts.path,
                "overflowCellLines": count,
            }),
            &[],
        ))
    } else {
        write_stdout(&format!("overflowCellLines={count}"))
    }
}
