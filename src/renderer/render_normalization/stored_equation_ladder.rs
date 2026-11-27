//! Reissue the vertical axis of a saved equation flow with zero text advance.

use crate::model::control::{Control, Equation};
use crate::model::document::DocInfo;
use crate::model::page::PageDef;
use crate::model::paragraph::{ColumnBreakType, LineSeg, Paragraph};
use crate::model::shape::{TextWrap, VertAlign, VertRelTo};
use crate::model::style::LineSpacingType;
use crate::storage::Storage;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum EquationFlowSource {
    SavedHwp,
    SavedHwpx,
    ShapeFreeHml,
}

/// The saved text line follows its float's declared frame. When the whole
/// ladder advances solely by those frames, place each equation at the
/// opening of its frame. Saved HWP and shape-free HML advance by content;
/// the saved HWPX lines and equations use the declared extents. Preserve the
/// declared equation frame for paint and leave the original IR available to Save.
pub(crate) fn normalize_with_storage<S: Storage>(
    paragraphs: &[Paragraph],
    styles: &DocInfo,
    page: &PageDef,
    source: EquationFlowSource,
    storage: &S,
) -> Result<Option<Vec<Paragraph>>, S::Error> {
    if !has_equation_frame_ladder(paragraphs, styles, source) {
        return Ok(None);
    }

    let body = crate::model::page::PageAreas::from_page_def(page).body_area;
    if body.top < 0 || body.bottom <= body.top {
        return Ok(None);
    }
    let mut projected = storage.vec(paragraphs.len())?;
    let mut next_top = body.top;
    for paragraph in paragraphs {
        let mut copy = storage.clone_paragraph(paragraph)?;
        if let Some((index, equation)) = equation(paragraph) {
            let height = if source == EquationFlowSource::SavedHwpx {
                equation.common.height
            } else {
                crate::renderer::equation::intrinsic_size_hwp(&equation.script, equation.font_size)
                    .1
            };
            let Some(bottom) = i32::try_from(height)
                .ok()
                .and_then(|height| next_top.checked_add(height))
                .filter(|&bottom| bottom <= body.bottom)
            else {
                return Ok(None);
            };
            let Control::Equation(equation) = &mut copy.controls[index] else {
                unreachable!("the render copy has the same control order");
            };
            equation.common.vert_rel_to = VertRelTo::Paper;
            equation.common.vertical_offset = next_top as u32;
            next_top = bottom;
        }
        if source == EquationFlowSource::SavedHwp {
            copy.line_segs[0].vertical_pos = next_top - body.top;
            copy.source_line_seg_vertical_pos = None;
        }
        storage.push(&mut projected, copy)?;
    }
    Ok(Some(projected))
}

fn equation(paragraph: &Paragraph) -> Option<(usize, &Equation)> {
    paragraph
        .controls
        .iter()
        .enumerate()
        .find_map(|(index, control)| match control {
            Control::Equation(equation) => Some((index, equation.as_ref())),
            _ => None,
        })
}

fn has_equation_frame_ladder(
    paragraphs: &[Paragraph],
    styles: &DocInfo,
    source: EquationFlowSource,
) -> bool {
    let mut declared_end = 0;
    for (index, paragraph) in paragraphs.iter().enumerate() {
        let first = index == 0;
        if paragraph.stored_text_partition_is_dirty()
            || paragraph.cell_format_vpos_dirty
            || !(paragraph.column_type == ColumnBreakType::None
                || (first && paragraph.column_type == ColumnBreakType::Section))
            || !styles
                .para_shapes
                .get(paragraph.para_shape_id as usize)
                .is_some_and(|style| {
                    style.line_spacing_type == LineSpacingType::Percent
                        && style.line_spacing == 0
                        && style.spacing_before == 0
                        && style.spacing_after == 0
                })
            || (!paragraph.text.is_empty()
                && (paragraph.char_shapes.is_empty()
                    || !paragraph.char_shapes.iter().all(|run| {
                        styles
                            .char_shapes
                            .get(run.char_shape_id as usize)
                            .is_some_and(|style| style.relative_sizes == [0; 7])
                    })))
        {
            return false;
        }
        match paragraph.controls.as_slice() {
            [Control::Equation(equation)] if is_unprotected_paragraph_float(equation) => {
                declared_end += i64::from(equation.common.height);
            }
            [] => {}
            controls
                if first
                    && controls.iter().all(|control| match control {
                        Control::SectionDef(_) => true,
                        Control::ColumnDef(columns) => columns.column_count == 1,
                        _ => false,
                    }) => {}
            _ => return false,
        }
        if source != EquationFlowSource::ShapeFreeHml
            && !has_closed_original_line(paragraph, declared_end)
        {
            return false;
        }
    }
    declared_end > 0
}

fn has_closed_original_line(paragraph: &Paragraph, declared_end: i64) -> bool {
    let [line] = paragraph.line_segs.as_slice() else {
        return false;
    };
    line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
        && line.line_height > 0
        && line.line_height.checked_add(line.line_spacing) == Some(0)
        && i64::from(line.vertical_pos) == declared_end
}

fn is_unprotected_paragraph_float(equation: &Equation) -> bool {
    let frame = &equation.common;
    !frame.treat_as_char
        && frame.flow_with_text
        && !frame.allow_overlap
        && !frame.size_protect
        && frame.vert_rel_to == VertRelTo::Para
        && frame.vert_align == VertAlign::Top
        && frame.text_wrap == TextWrap::TopAndBottom
        && frame.vertical_offset == 0
        && frame.height > 0
        && equation.font_size > 0
        && !equation.script.trim().is_empty()
}
