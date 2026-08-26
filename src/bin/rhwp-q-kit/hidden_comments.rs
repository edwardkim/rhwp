//! hidden-comments — `Control::HiddenComment` 열거. 읽기 전용.

use rhwp::model::control::Control;
use serde_json::json;

use crate::envelope::{emit_items, for_each_control, load_core, loc_json, parse_one_file};

pub fn run(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-kit hidden-comments <파일> [--json]";
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
        let Control::HiddenComment(hc) = at.ctrl else {
            return;
        };
        let paragraphs: Vec<&str> = hc.paragraphs.iter().map(|p| p.text.as_str()).collect();
        let mut item = loc_json(&at);
        item["paragraphCount"] = json!(paragraphs.len());
        item["paragraphs"] = json!(paragraphs);
        items.push(item);
    });
    emit_items(
        "hidden-comments",
        &opts.path,
        opts.json,
        items,
        &["items[].paragraphs"],
    )
}
