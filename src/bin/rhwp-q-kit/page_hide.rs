//! page-hide — `Control::PageHide` 열거. 읽기 전용.

use rhwp::model::control::Control;
use serde_json::json;

use crate::envelope::{emit_items, for_each_control, load_core, loc_json, parse_one_file};

pub fn run(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-kit page-hide <파일> [--json]";
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
        let Control::PageHide(ph) = at.ctrl else {
            return;
        };
        let mut item = loc_json(&at);
        item["hideHeader"] = json!(ph.hide_header);
        item["hideFooter"] = json!(ph.hide_footer);
        item["hideMasterPage"] = json!(ph.hide_master_page);
        item["hideBorder"] = json!(ph.hide_border);
        item["hideFill"] = json!(ph.hide_fill);
        item["hidePageNum"] = json!(ph.hide_page_num);
        items.push(item);
    });
    emit_items("page-hide", &opts.path, opts.json, items, &[])
}
