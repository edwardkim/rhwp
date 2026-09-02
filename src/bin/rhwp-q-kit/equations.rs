//! equations — `Control::Equation` 열거. 읽기 전용.

use rhwp::model::control::Control;
use serde_json::json;

use crate::envelope::{emit_items, for_each_control, load_core, loc_json, parse_one_file};

pub fn run(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-kit equations <파일> [--json]";
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
        let Control::Equation(eq) = at.ctrl else {
            return;
        };
        let mut item = loc_json(&at);
        item["script"] = json!(&eq.script);
        item["fontSize"] = json!(eq.font_size);
        item["fontName"] = json!(&eq.font_name);
        item["color"] = json!(eq.color);
        item["baseline"] = json!(eq.baseline);
        item["attr"] = json!(eq.attr);
        item["width"] = json!(eq.common.width);
        item["height"] = json!(eq.common.height);
        item["treatAsChar"] = json!(eq.common.treat_as_char);
        items.push(item);
    });
    emit_items(
        "equations",
        &opts.path,
        opts.json,
        items,
        &["items[].script", "items[].fontName"],
    )
}
