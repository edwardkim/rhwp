//! 표 겹침 목록 — `DocumentCore::take_table_overlaps`.
//!
//! 목록을 읽고 비우므로 방금 연 코어의 잔여값만 보고한다.

use crate::envelope::{envelope, load_core, parse_one_file, print_json, write_stdout};
use serde_json::json;

const USAGE: &str = "rhwp-q-kit table-overlaps <파일> [--json]";

pub fn run(args: &[String]) -> i32 {
    let opts = match parse_one_file(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let core = match load_core(&opts.path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let overlaps = core.take_table_overlaps();
    let rows: Vec<serde_json::Value> = overlaps
        .iter()
        .map(|o| {
            json!({
                "page": o.page_index,
                "section": o.section_index,
                "paraA": o.para_a,
                "paraB": o.para_b,
                "aY0": o.a_y0,
                "aY1": o.a_y1,
                "bY0": o.b_y0,
                "bY1": o.b_y1,
                "overlapPx": o.overlap_px,
            })
        })
        .collect();
    let count = rows.len();
    if opts.json {
        print_json(&envelope(
            "table-overlaps",
            json!({
                "source": opts.path,
                "count": count,
                "overlaps": rows,
            }),
            &[],
        ))
    } else {
        write_stdout(&format!("overlaps={count}"))
    }
}
