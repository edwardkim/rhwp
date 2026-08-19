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

pub fn para_empty(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more para-empty <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let mut items = Vec::new();
    for (si, sec) in doc.sections.iter().enumerate() {
        for (pi, para) in sec.paragraphs.iter().enumerate() {
            if para.text.is_empty() && para.controls.is_empty() {
                items.push(json!({"section": si, "paragraph": pi}));
            }
        }
    }
    emit("para-empty", &path, json_mode, items, &[])
}

pub fn para_has_ctrl(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more para-has-ctrl <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let mut items = Vec::new();
    for (si, sec) in doc.sections.iter().enumerate() {
        for (pi, para) in sec.paragraphs.iter().enumerate() {
            if !para.controls.is_empty() {
                items.push(
                    json!({"section": si, "paragraph": pi, "ctrlCount": para.controls.len()}),
                );
            }
        }
    }
    emit("para-has-ctrl", &path, json_mode, items, &[])
}

pub fn section_para_lens(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more section-para-lens <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items: Vec<Value> = doc
        .sections
        .iter()
        .enumerate()
        .map(|(si, sec)| json!({"section": si, "paraCount": sec.paragraphs.len()}))
        .collect();
    emit("section-para-lens", &path, json_mode, items, &[])
}

pub fn body_text_len(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more body-text-len <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let mut items = Vec::new();
    for (si, sec) in doc.sections.iter().enumerate() {
        let n: usize = sec.paragraphs.iter().map(|p| p.text.chars().count()).sum();
        items.push(json!({"section": si, "chars": n}));
    }
    emit("body-text-len", &path, json_mode, items, &[])
}

pub fn ctrl_per_para(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more ctrl-per-para <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let mut items = Vec::new();
    for (si, sec) in doc.sections.iter().enumerate() {
        for (pi, para) in sec.paragraphs.iter().enumerate() {
            items.push(json!({"section": si, "paragraph": pi, "ctrlCount": para.controls.len()}));
        }
    }
    emit("ctrl-per-para", &path, json_mode, items, &[])
}

pub fn table_border_fill(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more table-border-fill <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Table(v) => Some(
            json!({"rows": v.row_count, "cols": v.col_count, "cells": v.cells.len(), "attr": v.attr, "borderFillId": v.border_fill_id, "zones": v.zones.len(), "grid": v.cell_grid.len()}),
        ),
        _ => None,
    });
    emit(
        "table-border-fill",
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

pub fn table_spacing(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more table-spacing <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Table(v) => Some(
            json!({"rows": v.row_count, "cols": v.col_count, "cells": v.cells.len(), "attr": v.attr, "borderFillId": v.border_fill_id, "zones": v.zones.len(), "grid": v.cell_grid.len()}),
        ),
        _ => None,
    });
    emit(
        "table-spacing",
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

pub fn table_attr(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more table-attr <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Table(v) => Some(
            json!({"rows": v.row_count, "cols": v.col_count, "cells": v.cells.len(), "attr": v.attr, "borderFillId": v.border_fill_id, "zones": v.zones.len(), "grid": v.cell_grid.len()}),
        ),
        _ => None,
    });
    emit(
        "table-attr",
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

pub fn picture_border_width(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more picture-border-width <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Picture(v) => Some(
            json!({"instanceId": v.instance_id, "lock": v.lock, "reverse": v.reverse, "cropL": v.crop.left, "borderWidth": v.border_width, "opacity": v.border_opacity, "hrefSet": v.href.is_some()}),
        ),
        _ => None,
    });
    emit(
        "picture-border-width",
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

pub fn picture_opacity(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more picture-opacity <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Picture(v) => Some(
            json!({"instanceId": v.instance_id, "lock": v.lock, "reverse": v.reverse, "cropL": v.crop.left, "borderWidth": v.border_width, "opacity": v.border_opacity, "hrefSet": v.href.is_some()}),
        ),
        _ => None,
    });
    emit(
        "picture-opacity",
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

pub fn picture_href_set(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more picture-href-set <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Picture(v) => Some(
            json!({"instanceId": v.instance_id, "lock": v.lock, "reverse": v.reverse, "cropL": v.crop.left, "borderWidth": v.border_width, "opacity": v.border_opacity, "hrefSet": v.href.is_some()}),
        ),
        _ => None,
    });
    emit(
        "picture-href-set",
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

pub fn equation_baseline(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more equation-baseline <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Equation(v) => Some(
            json!({"script": v.script, "font": v.font_name, "fontSize": v.font_size, "baseline": v.baseline, "color": v.color, "attr": v.attr, "version": v.version_info}),
        ),
        _ => None,
    });
    emit(
        "equation-baseline",
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

pub fn equation_color(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more equation-color <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Equation(v) => Some(
            json!({"script": v.script, "font": v.font_name, "fontSize": v.font_size, "baseline": v.baseline, "color": v.color, "attr": v.attr, "version": v.version_info}),
        ),
        _ => None,
    });
    emit(
        "equation-color",
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

pub fn equation_attr(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more equation-attr <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Equation(v) => Some(
            json!({"script": v.script, "font": v.font_name, "fontSize": v.font_size, "baseline": v.baseline, "color": v.color, "attr": v.attr, "version": v.version_info}),
        ),
        _ => None,
    });
    emit(
        "equation-attr",
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

pub fn form_height(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more form-height <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Form(v) => Some(
            json!({"name": v.name, "text": v.text, "enabled": v.enabled, "width": v.width, "height": v.height, "caption": v.caption, "fore": v.fore_color, "back": v.back_color}),
        ),
        _ => None,
    });
    emit(
        "form-height",
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

pub fn form_caption(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more form-caption <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Form(v) => Some(
            json!({"name": v.name, "text": v.text, "enabled": v.enabled, "width": v.width, "height": v.height, "caption": v.caption, "fore": v.fore_color, "back": v.back_color}),
        ),
        _ => None,
    });
    emit(
        "form-caption",
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

pub fn form_fore_color(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more form-fore-color <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Form(v) => Some(
            json!({"name": v.name, "text": v.text, "enabled": v.enabled, "width": v.width, "height": v.height, "caption": v.caption, "fore": v.fore_color, "back": v.back_color}),
        ),
        _ => None,
    });
    emit(
        "form-fore-color",
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

pub fn field_properties(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more field-properties <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Field(v) => Some(
            json!({"command": v.command, "fieldId": v.field_id, "properties": v.properties, "ctrlId": v.ctrl_id, "extra": v.extra_properties}),
        ),
        _ => None,
    });
    emit(
        "field-properties",
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

pub fn field_ctrl_id(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more field-ctrl-id <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Field(v) => Some(
            json!({"command": v.command, "fieldId": v.field_id, "properties": v.properties, "ctrlId": v.ctrl_id, "extra": v.extra_properties}),
        ),
        _ => None,
    });
    emit(
        "field-ctrl-id",
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

pub fn ruby_align(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more ruby-align <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Ruby(v) => Some(
            json!({"main": v.main_text, "ruby": v.ruby_text, "ratio": v.sz_ratio, "align": v.align, "pos": v.pos_type}),
        ),
        _ => None,
    });
    emit(
        "ruby-align",
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

pub fn ruby_pos(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more ruby-pos <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Ruby(v) => Some(
            json!({"main": v.main_text, "ruby": v.ruby_text, "ratio": v.sz_ratio, "align": v.align, "pos": v.pos_type}),
        ),
        _ => None,
    });
    emit(
        "ruby-pos",
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

pub fn pagehide_border(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more pagehide-border <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::PageHide(v) => Some(
            json!({"hideHeader": v.hide_header, "hideFooter": v.hide_footer, "hideBorder": v.hide_border, "hideFill": v.hide_fill, "hideMaster": v.hide_master_page}),
        ),
        _ => None,
    });
    emit(
        "pagehide-border",
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

pub fn pagehide_fill(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more pagehide-fill <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::PageHide(v) => Some(
            json!({"hideHeader": v.hide_header, "hideFooter": v.hide_footer, "hideBorder": v.hide_border, "hideFill": v.hide_fill, "hideMaster": v.hide_master_page}),
        ),
        _ => None,
    });
    emit(
        "pagehide-fill",
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

pub fn pagehide_master(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more pagehide-master <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::PageHide(v) => Some(
            json!({"hideHeader": v.hide_header, "hideFooter": v.hide_footer, "hideBorder": v.hide_border, "hideFill": v.hide_fill, "hideMaster": v.hide_master_page}),
        ),
        _ => None,
    });
    emit(
        "pagehide-master",
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

pub fn autonumber_super(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more autonumber-super <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::AutoNumber(v) => {
            Some(json!({"number": v.number, "format": v.format, "super": v.superscript}))
        }
        _ => None,
    });
    emit(
        "autonumber-super",
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

pub fn char_overlap_border(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more char-overlap-border <파일> [--json]";
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
        "char-overlap-border",
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

pub fn hyperlink_text_len(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more hyperlink-text-len <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Hyperlink(v) => Some(
            json!({"url": v.url, "text": v.text, "urlLen": v.url.len(), "textLen": v.text.len()}),
        ),
        _ => None,
    });
    emit(
        "hyperlink-text-len",
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

pub fn bookmark_empty_name(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more bookmark-empty-name <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Bookmark(v) => Some(json!({"name": v.name, "emptyName": v.name.is_empty()})),
        _ => None,
    });
    emit(
        "bookmark-empty-name",
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

pub fn header_nonempty(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more header-nonempty <파일> [--json]";
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
        "header-nonempty",
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

pub fn footer_nonempty(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more footer-nonempty <파일> [--json]";
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
        "footer-nonempty",
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

pub fn footnote_nonempty(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more footnote-nonempty <파일> [--json]";
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
        "footnote-nonempty",
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

pub fn endnote_nonempty(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more endnote-nonempty <파일> [--json]";
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
        "endnote-nonempty",
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

pub fn hidden_nonempty(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more hidden-nonempty <파일> [--json]";
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
        "hidden-nonempty",
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

pub fn shape_height(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more shape-height <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Shape(v) => Some(json!({"width": v.common().width, "height": v.common().height})),
        _ => None,
    });
    emit(
        "shape-height",
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

pub fn table_zones(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more table-zones <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Table(v) => Some(
            json!({"rows": v.row_count, "cols": v.col_count, "cells": v.cells.len(), "attr": v.attr, "borderFillId": v.border_fill_id, "zones": v.zones.len(), "grid": v.cell_grid.len()}),
        ),
        _ => None,
    });
    emit(
        "table-zones",
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

pub fn table_grid_len(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more table-grid-len <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Table(v) => Some(
            json!({"rows": v.row_count, "cols": v.col_count, "cells": v.cells.len(), "attr": v.attr, "borderFillId": v.border_fill_id, "zones": v.zones.len(), "grid": v.cell_grid.len()}),
        ),
        _ => None,
    });
    emit(
        "table-grid-len",
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

pub fn field_extra_props(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more field-extra-props <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Field(v) => Some(
            json!({"command": v.command, "fieldId": v.field_id, "properties": v.properties, "ctrlId": v.ctrl_id, "extra": v.extra_properties}),
        ),
        _ => None,
    });
    emit(
        "field-extra-props",
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

pub fn form_back_color(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more form-back-color <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Form(v) => Some(
            json!({"name": v.name, "text": v.text, "enabled": v.enabled, "width": v.width, "height": v.height, "caption": v.caption, "fore": v.fore_color, "back": v.back_color}),
        ),
        _ => None,
    });
    emit(
        "form-back-color",
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

pub fn equation_version(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more equation-version <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Equation(v) => Some(
            json!({"script": v.script, "font": v.font_name, "fontSize": v.font_size, "baseline": v.baseline, "color": v.color, "attr": v.attr, "version": v.version_info}),
        ),
        _ => None,
    });
    emit(
        "equation-version",
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

pub fn index_both_keys(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more index-both-keys <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::IndexMark(v) => Some(
            json!({"first": v.first_key, "second": v.second_key, "both": v.first_key.len() + v.second_key.len()}),
        ),
        _ => None,
    });
    emit(
        "index-both-keys",
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

pub fn para_char_count(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more para-char-count <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let mut items = Vec::new();
    for (si, sec) in doc.sections.iter().enumerate() {
        for (pi, para) in sec.paragraphs.iter().enumerate() {
            items.push(json!({"section": si, "paragraph": pi, "chars": para.text.chars().count()}));
        }
    }
    emit("para-char-count", &path, json_mode, items, &[])
}

pub fn section_ctrl_total(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more section-ctrl-total <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items: Vec<Value> = doc
        .sections
        .iter()
        .enumerate()
        .map(|(si, sec)| {
            let n: usize = sec.paragraphs.iter().map(|p| p.controls.len()).sum();
            json!({"section": si, "ctrlCount": n})
        })
        .collect();
    emit("section-ctrl-total", &path, json_mode, items, &[])
}

pub fn caption_para_count(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more caption-para-count <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Table(v) => Some(
            json!({"rows": v.row_count, "cols": v.col_count, "cells": v.cells.len(), "attr": v.attr, "borderFillId": v.border_fill_id, "zones": v.zones.len(), "grid": v.cell_grid.len()}),
        ),
        _ => None,
    });
    emit(
        "caption-para-count",
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

pub fn enabled_forms_only(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more enabled-forms-only <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Form(v) => Some(
            json!({"name": v.name, "text": v.text, "enabled": v.enabled, "width": v.width, "height": v.height, "caption": v.caption, "fore": v.fore_color, "back": v.back_color}),
        ),
        _ => None,
    });
    emit(
        "enabled-forms-only",
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

pub fn nonempty_urls(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more nonempty-urls <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Hyperlink(v) => Some(
            json!({"url": v.url, "text": v.text, "urlLen": v.url.len(), "textLen": v.text.len()}),
        ),
        _ => None,
    });
    emit(
        "nonempty-urls",
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

pub fn nonempty_scripts(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more nonempty-scripts <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Equation(v) => Some(
            json!({"script": v.script, "font": v.font_name, "fontSize": v.font_size, "baseline": v.baseline, "color": v.color, "attr": v.attr, "version": v.version_info}),
        ),
        _ => None,
    });
    emit(
        "nonempty-scripts",
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

pub fn lock_pictures_only(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more lock-pictures-only <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Picture(v) => Some(
            json!({"instanceId": v.instance_id, "lock": v.lock, "reverse": v.reverse, "cropL": v.crop.left, "borderWidth": v.border_width, "opacity": v.border_opacity, "hrefSet": v.href.is_some()}),
        ),
        _ => None,
    });
    emit(
        "lock-pictures-only",
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

pub fn picture_crop_left(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more picture-crop-left <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Picture(v) => Some(
            json!({"instanceId": v.instance_id, "lock": v.lock, "reverse": v.reverse, "cropL": v.crop.left, "borderWidth": v.border_width, "opacity": v.border_opacity, "hrefSet": v.href.is_some()}),
        ),
        _ => None,
    });
    emit(
        "picture-crop-left",
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

pub fn form_name_len(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more form-name-len <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Form(v) => Some(
            json!({"name": v.name, "text": v.text, "enabled": v.enabled, "width": v.width, "height": v.height, "caption": v.caption, "fore": v.fore_color, "back": v.back_color}),
        ),
        _ => None,
    });
    emit(
        "form-name-len",
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

pub fn field_command_len(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more field-command-len <파일> [--json]";
    let (path, json_mode, core) = match load(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let doc = core.document();
    let items = walk_kind(doc, |c| match c {
        Control::Field(v) => Some(
            json!({"command": v.command, "fieldId": v.field_id, "properties": v.properties, "ctrlId": v.ctrl_id, "extra": v.extra_properties}),
        ),
        _ => None,
    });
    emit(
        "field-command-len",
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
