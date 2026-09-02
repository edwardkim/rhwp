//! hyperlinks — `Control::Hyperlink` 열거. 읽기 전용.

use rhwp::model::control::Control;
use serde_json::json;

use crate::envelope::{emit_items, for_each_control, load_core, loc_json, parse_one_file};

pub fn run(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-kit hyperlinks <파일> [--json]";
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
        let Control::Hyperlink(hl) = at.ctrl else {
            return;
        };
        let mut item = loc_json(&at);
        item["url"] = json!(&hl.url);
        item["text"] = json!(&hl.text);
        items.push(item);
    });
    emit_items(
        "hyperlinks",
        &opts.path,
        opts.json,
        items,
        &["items[].url", "items[].text"],
    )
}
