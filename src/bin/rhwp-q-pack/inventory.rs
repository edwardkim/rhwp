//! 컨트롤 종류별 전수 조회.

use rhwp::model::control::Control;
use serde_json::{json, Value};

use crate::envelope::{
    envelope, load_core, parse_one_file, print_json, write_stdout, EXIT_OK, EXIT_RUNTIME,
};

fn walk_kind(
    doc: &rhwp::model::document::Document,
    pred: impl Fn(&Control) -> Option<Value>,
) -> Vec<Value> {
    let mut items = Vec::new();
    for (si, sec) in doc.sections.iter().enumerate() {
        for (pi, para) in sec.paragraphs.iter().enumerate() {
            for (ci, ctrl) in para.controls.iter().enumerate() {
                if let Some(mut v) = pred(ctrl) {
                    v["section"] = json!(si);
                    v["paragraph"] = json!(pi);
                    v["control"] = json!(ci);
                    items.push(v);
                }
            }
        }
    }
    items
}

fn emit(command: &str, path: &str, json_mode: bool, items: Vec<Value>, untrusted: &[&str]) -> i32 {
    let payload = json!({
        "source": path,
        "count": items.len(),
        "items": items,
    });
    if json_mode {
        print_json(&envelope(command, payload, untrusted))
    } else {
        write_stdout(&format!("{command}={}", items.len()))
    }
}

fn load(
    args: &[String],
    usage: &str,
) -> Result<(String, bool, rhwp::document_core::DocumentCore), i32> {
    let opts = parse_one_file(args, usage)?;
    let core = load_core(&opts.path)?;
    Ok((opts.path, opts.json, core))
}

pub fn forms_all(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack forms-all <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Form(v) => {
            Some(json!({"name": v.name, "text": v.text, "enabled": v.enabled, "width": v.width}))
        }
        _ => None,
    });
    emit(
        "forms-all",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn shapes_all(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack shapes-all <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Shape(v) => Some(json!({"width": v.common().width})),
        _ => None,
    });
    emit(
        "shapes-all",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn char_overlaps(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack char-overlaps <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::CharOverlap(v) => Some(json!({"len": v.chars.len(), "borderType": v.border_type})),
        _ => None,
    });
    emit(
        "char-overlaps",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn headers_list(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack headers-list <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Header(v) => Some(json!({"paraCount": v.paragraphs.len()})),
        _ => None,
    });
    emit(
        "headers-list",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn footers_list(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack footers-list <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Footer(v) => Some(json!({"paraCount": v.paragraphs.len()})),
        _ => None,
    });
    emit(
        "footers-list",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn footnotes_list(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack footnotes-list <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Footnote(v) => Some(json!({"paraCount": v.paragraphs.len()})),
        _ => None,
    });
    emit(
        "footnotes-list",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn endnotes_list(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack endnotes-list <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Endnote(v) => Some(json!({"paraCount": v.paragraphs.len()})),
        _ => None,
    });
    emit(
        "endnotes-list",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn new_numbers(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack new-numbers <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::NewNumber(v) => Some(json!({"ok": true})),
        _ => None,
    });
    emit(
        "new-numbers",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn page_num_ctrls(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack page-num-ctrls <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::PageNumCtrl(v) => Some(json!({"ok": true})),
        _ => None,
    });
    emit(
        "page-num-ctrls",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn page_number_pos(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack page-number-pos <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::PageNumberPos(v) => Some(json!({"ok": true})),
        _ => None,
    });
    emit(
        "page-number-pos",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn column_defs(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack column-defs <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::ColumnDef(v) => Some(json!({"ok": true})),
        _ => None,
    });
    emit(
        "column-defs",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn unknown_ctrls(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack unknown-ctrls <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Unknown(v) => Some(json!({"ok": true})),
        _ => None,
    });
    emit(
        "unknown-ctrls",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn tables_model(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack tables-model <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Table(v) => {
            Some(json!({"rows": v.row_count, "cols": v.col_count, "cells": v.cells.len()}))
        }
        _ => None,
    });
    emit(
        "tables-model",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn field_ctrls(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack field-ctrls <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Field(v) => Some(json!({"command": v.command, "fieldId": v.field_id})),
        _ => None,
    });
    emit(
        "field-ctrls",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn bookmark_names(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack bookmark-names <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Bookmark(v) => Some(json!({"name": v.name})),
        _ => None,
    });
    emit(
        "bookmark-names",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn treat_as_char(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack treat-as-char <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| {
        c.is_treat_as_char_object()
            .then(|| json!({"treatAsChar": true}))
    });
    emit("treat-as-char", &path, json_mode, items, &[])
}

pub fn logical_inline(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack logical-inline <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| {
        c.is_logical_inline()
            .then(|| json!({"logicalInline": true}))
    });
    emit("logical-inline", &path, json_mode, items, &[])
}

pub fn picture_crops(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack picture-crops <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Picture(v) => Some(
            json!({"instanceId": v.instance_id, "lock": v.lock, "reverse": v.reverse, "cropL": v.crop.left}),
        ),
        _ => None,
    });
    emit(
        "picture-crops",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn equation_scripts(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack equation-scripts <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Equation(v) => {
            Some(json!({"script": v.script, "font": v.font_name, "fontSize": v.font_size}))
        }
        _ => None,
    });
    emit(
        "equation-scripts",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn form_types(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack form-types <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Form(v) => {
            Some(json!({"name": v.name, "text": v.text, "enabled": v.enabled, "width": v.width}))
        }
        _ => None,
    });
    emit(
        "form-types",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn hyperlink_hosts(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack hyperlink-hosts <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Hyperlink(v) => Some(json!({"url": v.url, "text": v.text})),
        _ => None,
    });
    emit(
        "hyperlink-hosts",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn ruby_mains(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack ruby-mains <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Ruby(v) => {
            Some(json!({"main": v.main_text, "ruby": v.ruby_text, "ratio": v.sz_ratio}))
        }
        _ => None,
    });
    emit(
        "ruby-mains",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn pagehide_headers(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack pagehide-headers <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::PageHide(v) => {
            Some(json!({"hideHeader": v.hide_header, "hideFooter": v.hide_footer}))
        }
        _ => None,
    });
    emit(
        "pagehide-headers",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn autonumber_nums(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack autonumber-nums <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::AutoNumber(v) => Some(json!({"number": v.number, "format": v.format})),
        _ => None,
    });
    emit(
        "autonumber-nums",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn index_second_keys(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack index-second-keys <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::IndexMark(v) => Some(json!({"first": v.first_key, "second": v.second_key})),
        _ => None,
    });
    emit(
        "index-second-keys",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn hidden_comment_len(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack hidden-comment-len <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::HiddenComment(v) => Some(json!({"paraCount": v.paragraphs.len()})),
        _ => None,
    });
    emit(
        "hidden-comment-len",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn table_rows(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack table-rows <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Table(v) => {
            Some(json!({"rows": v.row_count, "cols": v.col_count, "cells": v.cells.len()}))
        }
        _ => None,
    });
    emit(
        "table-rows",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn table_cells(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack table-cells <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Table(v) => {
            Some(json!({"rows": v.row_count, "cols": v.col_count, "cells": v.cells.len()}))
        }
        _ => None,
    });
    emit(
        "table-cells",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn shape_sizes(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack shape-sizes <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Shape(v) => Some(json!({"width": v.common().width})),
        _ => None,
    });
    emit(
        "shape-sizes",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn header_paras(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack header-paras <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Header(v) => Some(json!({"paraCount": v.paragraphs.len()})),
        _ => None,
    });
    emit(
        "header-paras",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn footer_paras(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack footer-paras <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Footer(v) => Some(json!({"paraCount": v.paragraphs.len()})),
        _ => None,
    });
    emit(
        "footer-paras",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn footnote_paras(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack footnote-paras <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Footnote(v) => Some(json!({"paraCount": v.paragraphs.len()})),
        _ => None,
    });
    emit(
        "footnote-paras",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn endnote_paras(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack endnote-paras <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Endnote(v) => Some(json!({"paraCount": v.paragraphs.len()})),
        _ => None,
    });
    emit(
        "endnote-paras",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn picture_locks(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack picture-locks <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Picture(v) => Some(
            json!({"instanceId": v.instance_id, "lock": v.lock, "reverse": v.reverse, "cropL": v.crop.left}),
        ),
        _ => None,
    });
    emit(
        "picture-locks",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn picture_reverse(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack picture-reverse <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Picture(v) => Some(
            json!({"instanceId": v.instance_id, "lock": v.lock, "reverse": v.reverse, "cropL": v.crop.left}),
        ),
        _ => None,
    });
    emit(
        "picture-reverse",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn equation_fonts(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack equation-fonts <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Equation(v) => {
            Some(json!({"script": v.script, "font": v.font_name, "fontSize": v.font_size}))
        }
        _ => None,
    });
    emit(
        "equation-fonts",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn form_enabled(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack form-enabled <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Form(v) => {
            Some(json!({"name": v.name, "text": v.text, "enabled": v.enabled, "width": v.width}))
        }
        _ => None,
    });
    emit(
        "form-enabled",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn field_commands(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack field-commands <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Field(v) => Some(json!({"command": v.command, "fieldId": v.field_id})),
        _ => None,
    });
    emit(
        "field-commands",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn field_ids(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack field-ids <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Field(v) => Some(json!({"command": v.command, "fieldId": v.field_id})),
        _ => None,
    });
    emit(
        "field-ids",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn form_sizes(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack form-sizes <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Form(v) => {
            Some(json!({"name": v.name, "text": v.text, "enabled": v.enabled, "width": v.width}))
        }
        _ => None,
    });
    emit(
        "form-sizes",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn section_defs(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack section-defs <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::SectionDef(v) => Some(json!({"ok": true})),
        _ => None,
    });
    emit(
        "section-defs",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn caption_tables(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack caption-tables <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Table(v) => {
            Some(json!({"rows": v.row_count, "cols": v.col_count, "cells": v.cells.len()}))
        }
        _ => None,
    });
    emit(
        "caption-tables",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn ctrl_kinds(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack ctrl-kinds <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| {
        Some(json!({"treatAsChar": c.is_treat_as_char_object()}))
    });
    emit("ctrl-kinds", &path, json_mode, items, &[])
}

pub fn page_starts_on(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack page-starts-on <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::PageNumCtrl(v) => Some(json!({"ok": true})),
        _ => None,
    });
    emit(
        "page-starts-on",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn hidden_comment_count(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack hidden-comment-count <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::HiddenComment(v) => Some(json!({"paraCount": v.paragraphs.len()})),
        _ => None,
    });
    emit(
        "hidden-comment-count",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn ruby_ratio(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack ruby-ratio <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Ruby(v) => {
            Some(json!({"main": v.main_text, "ruby": v.ruby_text, "ratio": v.sz_ratio}))
        }
        _ => None,
    });
    emit(
        "ruby-ratio",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn char_overlap_len(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack char-overlap-len <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::CharOverlap(v) => Some(json!({"len": v.chars.len(), "borderType": v.border_type})),
        _ => None,
    });
    emit(
        "char-overlap-len",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn table_cols(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack table-cols <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Table(v) => {
            Some(json!({"rows": v.row_count, "cols": v.col_count, "cells": v.cells.len()}))
        }
        _ => None,
    });
    emit(
        "table-cols",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn picture_instance(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack picture-instance <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Picture(v) => Some(
            json!({"instanceId": v.instance_id, "lock": v.lock, "reverse": v.reverse, "cropL": v.crop.left}),
        ),
        _ => None,
    });
    emit(
        "picture-instance",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}

pub fn index_first_keys(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack index-first-keys <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::IndexMark(v) => Some(json!({"first": v.first_key, "second": v.second_key})),
        _ => None,
    });
    emit(
        "index-first-keys",
        &path,
        json_mode,
        items,
        &[
            "items[].text",
            "items[].name",
            "items[].url",
            "items[].script",
        ],
    )
}
