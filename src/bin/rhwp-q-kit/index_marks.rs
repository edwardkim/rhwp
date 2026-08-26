//! index-marks — `Control::IndexMark` 열거. 읽기 전용.

use rhwp::model::control::Control;
use serde_json::json;

use crate::envelope::{emit_items, for_each_control, load_core, loc_json, parse_one_file};

pub fn run(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-kit index-marks <파일> [--json]";
    let opts = match parse_one_file(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let core = match load_core(&opts.path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let mut items = Vec::new();
    for_each_control(core.document(), |at| {
        let Control::IndexMark(im) = at.ctrl else {
            return;
        };
        let mut item = loc_json(&at);
        item["firstKey"] = json!(&im.first_key);
        item["secondKey"] = json!(&im.second_key);
        items.push(item);
    });
    emit_items(
        "index-marks",
        &opts.path,
        opts.json,
        items,
        &["items[].firstKey", "items[].secondKey"],
    )
}
