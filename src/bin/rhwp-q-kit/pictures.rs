//! pictures — `Control::Picture` 열거. 읽기 전용.

use rhwp::model::control::Control;
use serde_json::json;

use crate::envelope::{emit_items, for_each_control, load_core, loc_json, parse_one_file};

pub fn run(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-kit pictures <파일> [--json]";
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
        let Control::Picture(pic) = at.ctrl else {
            return;
        };
        let mut item = loc_json(&at);
        item["href"] = json!(&pic.href);
        item["binDataId"] = json!(pic.image_attr.bin_data_id);
        item["instanceId"] = json!(pic.instance_id);
        item["width"] = json!(pic.common.width);
        item["height"] = json!(pic.common.height);
        item["treatAsChar"] = json!(pic.common.treat_as_char);
        item["reverse"] = json!(pic.reverse);
        item["lock"] = json!(pic.lock);
        item["imgDim"] = json!([pic.img_dim.0, pic.img_dim.1]);
        item["transparency"] = json!(pic.image_attr.transparency);
        item["externalPath"] = json!(&pic.image_attr.external_path);
        items.push(item);
    });
    emit_items(
        "pictures",
        &opts.path,
        opts.json,
        items,
        &["items[].href", "items[].externalPath"],
    )
}
