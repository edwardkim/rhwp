//! Reject a cached TAC row only when its source boxes contradict the row.
//! No font fallback measurement is used for this admission decision.
use crate::model::{
    control::Control,
    paragraph::{LineSeg, Paragraph},
};
use crate::renderer::layout::control_line_seg_index;
use unicode_width::UnicodeWidthChar;

pub(super) fn physically_stale(para: &Paragraph) -> bool {
    if !supported_host(para) {
        return false;
    }
    for (index, control) in para.controls.iter().enumerate() {
        let Control::Table(table) = control else {
            continue;
        };
        if table.common.size_protect {
            continue;
        }
        let Some(row) = control_line_seg_index(para, index) else {
            continue;
        };
        let Some(seg) = para.line_segs.get(row) else {
            continue;
        };
        if seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
            || seg.line_height <= 0
            || seg.segment_width <= 0
        {
            continue;
        }
        let width = i64::from(table.flow_width_hu())
            + i64::from(table.outer_margin_left)
            + i64::from(table.outer_margin_right);
        let start = para.line_seg_text_start(row);
        let end = if row + 1 < para.line_segs.len() {
            para.line_seg_text_start(row + 1)
        } else {
            u32::MAX
        };
        let owns_text = para.text.chars().enumerate().any(|(i, c)| {
            let pos = para.char_offsets.get(i).copied().unwrap_or(i as u32);
            pos >= start
                && pos < end
                && !c.is_whitespace()
                && c != '\u{FFFC}'
                && c.width().is_some_and(|width| width > 0)
        });
        if width >= i64::from(seg.segment_width) && owns_text {
            return true;
        }
        let outer = i64::from(table.outer_margin_top) + i64::from(table.outer_margin_bottom);
        let declared = i64::from(table.common.height) + outer;
        if declared > i64::from(seg.line_height) {
            return true;
        }
        if table.row_count != 1 || table.cells.is_empty() {
            continue;
        }
        let minimum = table
            .cells
            .iter()
            .filter_map(|cell| {
                if cell.row_span != 1 || cell.paragraphs.is_empty() {
                    return None;
                }
                let mut content = 0i64;
                for paragraph in &cell.paragraphs {
                    if !paragraph.controls.is_empty() || paragraph.line_segs.is_empty() {
                        return None;
                    }
                    for (i, line) in paragraph.line_segs.iter().enumerate() {
                        if line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                            || line.line_height <= 0
                            || line.baseline_distance < 0
                            || line.baseline_distance > line.line_height
                        {
                            return None;
                        }
                        if i > 0 && line.vertical_pos <= paragraph.line_segs[i - 1].vertical_pos {
                            return None;
                        }
                        content += i64::from(line.line_height);
                        if i + 1 < paragraph.line_segs.len() {
                            content += i64::from(line.line_spacing.max(0));
                        }
                    }
                }
                let padding = cell.effective_padding(&table.padding);
                Some(content + i64::from(padding.top.max(0)) + i64::from(padding.bottom.max(0)))
            })
            .max();
        if minimum.is_some_and(|height| height + outer > i64::from(seg.line_height)) {
            return true;
        }
    }
    false
}

fn supported_host(para: &Paragraph) -> bool {
    para.line_segs.len() == 1
        && para
            .controls
            .iter()
            .filter(|c| matches!(c,Control::Table(t) if t.common.treat_as_char))
            .count()
            == 1
        && para.controls.iter().all(|c| {
            matches!(c,Control::Table(t) if t.common.treat_as_char && t.row_count == 1)
                || matches!(
                    c,
                    Control::SectionDef(_)
                        | Control::ColumnDef(_)
                        | Control::Field(_)
                        | Control::Bookmark(_)
                )
        })
}

pub(super) fn repair(
    document: &mut crate::model::document::Document,
    styles: &crate::renderer::style_resolver::ResolvedStyleSet,
    dpi: f64,
) {
    use crate::renderer::composer::{
        paragraph_flow_end, recalculate_section_vpos, reflow_line_segs, ParagraphBox,
    };
    let hwp3 = document.layout_profile().hwp3_layout();
    for section in &mut document.sections {
        let mut candidates = Vec::new();
        let mut column_def = crate::model::page::ColumnDef::default();
        for (index, paragraph) in section.paragraphs.iter().enumerate() {
            for control in &paragraph.controls {
                if let Control::ColumnDef(value) = control {
                    column_def = value.clone();
                }
            }
            if !physically_stale(paragraph) {
                continue;
            }
            let layout = crate::renderer::page_layout::PageLayoutInfo::from_page_def(
                &section.section_def.page_def,
                &column_def,
                dpi,
            );
            // This bounded repair does not infer owners for unequal columns.
            if layout
                .column_areas
                .windows(2)
                .any(|w| w[0].width != w[1].width)
            {
                continue;
            }
            let width = layout
                .column_areas
                .first()
                .map_or(layout.body_area.width, |r| r.width);
            let box_ = ParagraphBox::body_for_style(
                width,
                styles.para_styles.get(paragraph.para_shape_id as usize),
                dpi,
            );
            if box_.is_usable() {
                candidates.push((index, box_));
            }
        }
        // Later repaired ladders retain their reset relation when an earlier row grows.
        for (index, box_) in candidates.into_iter().rev() {
            let original_end = paragraph_flow_end(&section.paragraphs[index]);
            let mut projected = section.paragraphs[index].clone();
            for control in &mut projected.controls {
                if let Control::Table(table) = control {
                    repair_cell_width_caches(
                        table,
                        styles,
                        dpi,
                        hwp3,
                        section.section_def.line_grid == 0,
                        0,
                    );
                    crate::document_core::html_table_width::reserve_content_heights(
                        table, styles, dpi,
                    );
                }
            }
            reflow_line_segs(&mut projected, box_, styles, dpi);
            if projected.line_segs.is_empty() {
                continue;
            }
            section.paragraphs[index] = projected;
            recalculate_section_vpos(
                &mut section.paragraphs,
                index,
                None,
                original_end,
                styles,
                dpi,
                hwp3,
            );
        }
    }
}

/// A rejected host can also contain cell rows from an older padding/width box.
/// Rebuild only a single ordinary row whose recorded box disagrees with the
/// current cell box; font fallback estimates do not decide this admission.
fn repair_cell_width_caches(
    table: &mut crate::model::table::Table,
    styles: &crate::renderer::style_resolver::ResolvedStyleSet,
    dpi: f64,
    hwp3: bool,
    no_line_grid: bool,
    depth: usize,
) {
    use crate::renderer::composer::{cell_inner_text_width, reflow_line_segs, ParagraphBox};
    use crate::renderer::hwpunit_to_px;
    if depth >= 32 || table.common.size_protect {
        return;
    }
    for cell in &mut table.cells {
        if cell.text_direction != 0 {
            continue;
        }
        let padding = cell.effective_padding(&table.padding);
        let inner = cell_inner_text_width(
            hwpunit_to_px(cell.width.min(i32::MAX as u32) as i32, dpi),
            hwpunit_to_px(i32::from(padding.left), dpi),
            hwpunit_to_px(i32::from(padding.right), dpi),
            dpi,
        );
        let mut first_changed = None;
        for (index, paragraph) in cell.paragraphs.iter_mut().enumerate() {
            for control in &mut paragraph.controls {
                if let Control::Table(nested) = control {
                    if nested.common.size_protect || !nested.common.treat_as_char {
                        continue;
                    }
                    repair_cell_width_caches(nested, styles, dpi, hwp3, no_line_grid, depth + 1);
                    crate::document_core::html_table_width::reserve_content_heights(
                        nested, styles, dpi,
                    );
                }
            }
            let style = styles.para_styles.get(paragraph.para_shape_id as usize);
            // A cell's first saved line is relative to the cell content origin.
            // Without a line grid or a flow object, its initial band is owned
            // by paragraph before-space and the line's internal leading.
            // Passive foreground/background objects do not create that band.
            // Reject an unsupported band before either measurement or paint
            // consumes it; the canonical cell reflow publishes both the new
            // line metrics and the subsequent paragraph ladder below.
            if index == 0
                && no_line_grid
                && !hwp3
                && unsupported_initial_cell_band(paragraph, style, dpi)
            {
                let box_ = ParagraphBox::content_for_style(inner, style, dpi);
                if box_.is_usable() {
                    // Ordinary width reflow preserves the saved first origin.
                    // This origin was rejected, so it must not seed that frame.
                    paragraph.line_segs.clear();
                    reflow_line_segs(paragraph, box_, styles, dpi);
                    first_changed.get_or_insert(index);
                }
                continue;
            }
            // Floating objects can legitimately carve a narrower stored row.
            // They need their own frame evidence, not this cell-box comparison.
            if paragraph.controls.iter().any(|control| {
                !matches!(
                    control,
                    Control::Field(_)
                        | Control::Bookmark(_)
                        | Control::SectionDef(_)
                        | Control::ColumnDef(_)
                ) && !matches!(control, Control::Table(table) if table.common.treat_as_char)
                    && !matches!(control, Control::Equation(equation) if equation.common.treat_as_char)
                    && !matches!(control, Control::Picture(picture) if picture.common.treat_as_char)
            }) {
                continue;
            }
            // Indented and carved rows have separate first/continuation boxes.
            if style.is_some_and(|style| style.indent != 0.0) {
                continue;
            }
            let box_ = ParagraphBox::content_for_style(inner, style, dpi);
            let [line] = paragraph.line_segs.as_slice() else {
                continue;
            };
            if !box_.is_usable()
                || line.column_start != 0
                || line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                || line.segment_width == box_.width_hwp()
            {
                continue;
            }
            reflow_line_segs(paragraph, box_, styles, dpi);
            first_changed.get_or_insert(index);
        }
        if let Some(first) = first_changed {
            super::super::text_editing::recalculate_cell_paragraph_vpos(
                &mut cell.paragraphs,
                first,
                None,
                styles,
                dpi,
                hwp3,
            );
        }
    }
}

fn unsupported_initial_cell_band(
    paragraph: &Paragraph,
    style: Option<&crate::renderer::style_resolver::ResolvedParaStyle>,
    dpi: f64,
) -> bool {
    use crate::model::shape::TextWrap;
    let Some(style) = style else {
        return false;
    };
    let Some(line) = paragraph.line_segs.first() else {
        return false;
    };
    if line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || line.line_height <= 0
        || line.text_height <= 0
        || line.text_height > line.line_height
        || line.baseline_distance < 0
        || line.baseline_distance > line.line_height
        || paragraph.stored_text_partition_is_dirty()
    {
        return false;
    }
    let passive = |common: &crate::model::shape::CommonObjAttr| {
        !common.treat_as_char
            && matches!(
                common.text_wrap,
                TextWrap::InFrontOfText | TextWrap::BehindText
            )
    };
    if paragraph.controls.iter().any(|control| match control {
        Control::Shape(shape) => !passive(shape.common()),
        Control::Picture(picture) => !passive(&picture.common),
        Control::Field(_) | Control::Bookmark(_) => false,
        _ => true,
    }) {
        return false;
    }
    let before_hu = crate::renderer::px_to_hwpunit(style.spacing_before.max(0.0), dpi);
    i64::from(line.vertical_pos)
        > i64::from(before_hu) + i64::from(line.line_height.saturating_sub(line.text_height))
}
