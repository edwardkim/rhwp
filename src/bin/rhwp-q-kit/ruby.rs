//! ruby — `Control::Ruby` 열거. 읽기 전용.

use rhwp::model::control::Control;
use serde_json::json;

use crate::envelope::{emit_items, for_each_control, load_core, loc_json, parse_one_file};

pub fn run(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-kit ruby <파일> [--json]";
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
        let Control::Ruby(ruby) = at.ctrl else {
            return;
        };
        let mut item = loc_json(&at);
        item["mainText"] = json!(&ruby.main_text);
        item["rubyText"] = json!(&ruby.ruby_text);
        item["posType"] = json!(ruby.pos_type);
        item["align"] = json!(ruby.align);
        item["szRatio"] = json!(ruby.sz_ratio);
        item["option"] = json!(ruby.option);
        item["styleIdRef"] = json!(ruby.style_id_ref);
        items.push(item);
    });
    emit_items(
        "ruby",
        &opts.path,
        opts.json,
        items,
        &["items[].mainText", "items[].rubyText"],
    )
}
