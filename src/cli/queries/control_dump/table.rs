use rhwp::model::control::Control;
use rhwp::model::table::Table;

use super::shape::{horz_str, vert_str, wrap_str};
use crate::{hu_to_mm, hu_to_mm_i};

pub(super) fn dump(table: &Table, prefix: &str) {
    println!(
        "{}표: {}행×{}열, 셀={}, 쪽나눔={:?} (attr=0x{:08x}), padding=({},{},{},{}), cs={}",
        prefix,
        table.row_count,
        table.col_count,
        table.cells.len(),
        table.page_break,
        table.raw_table_record_attr,
        table.padding.left,
        table.padding.right,
        table.padding.top,
        table.padding.bottom,
        table.cell_spacing
    );
    for (index, zone) in table.zones.iter().enumerate() {
        println!(
            "{}  zone[{}] row={}..{} col={}..{} bf={}",
            prefix,
            index,
            zone.start_row,
            zone.end_row,
            zone.start_col,
            zone.end_col,
            zone.border_fill_id
        );
    }

    let common = &table.common;
    println!(
        "{}  [common] treat_as_char={}, wrap={}, vert={}({}={:.1}mm), horz={}({}={:.1}mm)",
        prefix,
        common.treat_as_char,
        wrap_str(&common.text_wrap),
        vert_str(&common.vert_rel_to),
        common.vertical_offset,
        hu_to_mm(common.vertical_offset),
        horz_str(&common.horz_rel_to),
        common.horizontal_offset,
        hu_to_mm(common.horizontal_offset)
    );
    println!(
        "{}  [common] size={}×{}({:.1}×{:.1}mm), valign={:?}, halign={:?}",
        prefix,
        common.width,
        common.height,
        hu_to_mm(common.width),
        hu_to_mm(common.height),
        common.vert_align,
        common.horz_align
    );
    println!(
        "{}  [outer_margin] left={:.1}mm({}) right={:.1}mm({}) top={:.1}mm({}) bottom={:.1}mm({})",
        prefix,
        hu_to_mm_i(table.outer_margin_left as i32),
        table.outer_margin_left,
        hu_to_mm_i(table.outer_margin_right as i32),
        table.outer_margin_right,
        hu_to_mm_i(table.outer_margin_top as i32),
        table.outer_margin_top,
        hu_to_mm_i(table.outer_margin_bottom as i32),
        table.outer_margin_bottom
    );
    if table.raw_ctrl_data.len() >= 20 {
        println!(
            "{}  [raw] {:02X?}",
            prefix,
            &table.raw_ctrl_data[..20.min(table.raw_ctrl_data.len())]
        );
    }

    dump_deep(table, &format!("{}  ", prefix), 0);
}

fn dump_deep(table: &Table, indent: &str, depth: usize) {
    for (cell_index, cell) in table.cells.iter().enumerate() {
        let text_preview = cell
            .paragraphs
            .iter()
            .map(|paragraph| paragraph.text.chars().take(30).collect::<String>())
            .collect::<Vec<_>>()
            .join("|");
        println!("{}셀[{}] r={},c={} rs={},cs={} h={} w={} pad=({},{},{},{}) valign={:?} aim={} hdr={} bf={} paras={} text=\"{}\"",
            indent, cell_index, cell.row, cell.col, cell.row_span, cell.col_span, cell.height,
            cell.width, cell.padding.left, cell.padding.right, cell.padding.top, cell.padding.bottom,
            cell.vertical_align, cell.apply_inner_margin, cell.is_header, cell.border_fill_id,
            cell.paragraphs.len(), text_preview);
        if let Some(field_name) = &cell.field_name {
            println!("{}  field=\"{}\"", indent, field_name);
        }
        for (paragraph_index, paragraph) in cell.paragraphs.iter().enumerate() {
            dump_cell_paragraph(paragraph, paragraph_index, indent);
            if depth < 3 {
                dump_nested_tables(paragraph, paragraph_index, indent, depth);
            }
        }
    }
}

fn dump_cell_paragraph(
    paragraph: &rhwp::model::paragraph::Paragraph,
    paragraph_index: usize,
    indent: &str,
) {
    if !paragraph.line_segs.is_empty() || !paragraph.controls.is_empty() {
        let line_info = paragraph
            .line_segs
            .iter()
            .enumerate()
            .map(|(line_index, line)| {
                format!(
                    "ls[{}] vpos={} lh={} ls={}",
                    line_index, line.vertical_pos, line.line_height, line.line_spacing
                )
            })
            .collect::<Vec<_>>();
        println!(
            "{}  p[{}] ps_id={} ctrls={} text_len={} {}",
            indent,
            paragraph_index,
            paragraph.para_shape_id,
            paragraph.controls.len(),
            paragraph.text.len(),
            line_info.join(", ")
        );
    }
    for (control_index, control) in paragraph.controls.iter().enumerate() {
        match control {
            Control::Picture(picture) => {
                println!("{}    ctrl[{}] 그림: bin_id={}, w={} h={} ({:.1}×{:.1}mm), tac={}, wrap={:?}, vert={:?}(off={}), horz={:?}(off={}), orig={}×{}, cur={}×{}, crop=({},{},{},{})",
                    indent, control_index, picture.image_attr.bin_data_id, picture.common.width,
                    picture.common.height, picture.common.width as f64 / 7200.0 * 25.4,
                    picture.common.height as f64 / 7200.0 * 25.4, picture.common.treat_as_char,
                    picture.common.text_wrap, picture.common.vert_rel_to, picture.common.vertical_offset,
                    picture.common.horz_rel_to, picture.common.horizontal_offset,
                    picture.shape_attr.original_width, picture.shape_attr.original_height,
                    picture.shape_attr.current_width, picture.shape_attr.current_height,
                    picture.crop.left, picture.crop.top, picture.crop.right, picture.crop.bottom);
                println!("{}      [image_attr] effect={:?} brightness={} contrast={} watermark={}",
                    indent, picture.image_attr.effect, picture.image_attr.brightness,
                    picture.image_attr.contrast, picture.image_attr.watermark_preset().unwrap_or("none"));
            }
            Control::Shape(shape) => println!(
                "{}    ctrl[{}] {}: tac={}, wrap={:?}",
                indent,
                control_index,
                shape.shape_name(),
                shape.common().treat_as_char,
                shape.common().text_wrap
            ),
            Control::PageHide(page_hide) => println!("{}    ctrl[{}] PageHide: header={} footer={} master={} border={} fill={} page_num={}",
                indent, control_index, page_hide.hide_header, page_hide.hide_footer,
                page_hide.hide_master_page, page_hide.hide_border, page_hide.hide_fill,
                page_hide.hide_page_num),
            _ => {}
        }
    }
}

fn dump_nested_tables(
    paragraph: &rhwp::model::paragraph::Paragraph,
    paragraph_index: usize,
    indent: &str,
    depth: usize,
) {
    for control in &paragraph.controls {
        let Control::Table(inner) = control else {
            continue;
        };
        println!(
            "{}  p[{}] 내부표: {}행×{}열, 셀={}, cs={}, pad=({},{},{},{})",
            indent,
            paragraph_index,
            inner.row_count,
            inner.col_count,
            inner.cells.len(),
            inner.cell_spacing,
            inner.padding.left,
            inner.padding.right,
            inner.padding.top,
            inner.padding.bottom
        );
        dump_deep(inner, &format!("{}    ", indent), depth + 1);
    }
}
