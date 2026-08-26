//! auto-numbers — `Control::AutoNumber` 열거. 읽기 전용.

use rhwp::model::control::Control;
use serde_json::json;

use crate::envelope::{emit_items, for_each_control, load_core, loc_json, parse_one_file};

pub fn run(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-kit auto-numbers <파일> [--json]";
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
        let Control::AutoNumber(an) = at.ctrl else {
            return;
        };
        let mut item = loc_json(&at);
        item["numberType"] = json!(an.number_type);
        item["format"] = json!(an.format);
        item["superscript"] = json!(an.superscript);
        item["assignedNumber"] = json!(an.assigned_number);
        item["number"] = json!(an.number);
        item["userSymbol"] = json!(an.user_symbol.to_string());
        item["prefixChar"] = json!(an.prefix_char.to_string());
        item["suffixChar"] = json!(an.suffix_char.to_string());
        items.push(item);
    });
    emit_items(
        "auto-numbers",
        &opts.path,
        opts.json,
        items,
        &[
            "items[].userSymbol",
            "items[].prefixChar",
            "items[].suffixChar",
        ],
    )
}
