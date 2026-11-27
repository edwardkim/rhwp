//! Hancom imports shape-free equation flows with a fallback object frame.

use crate::model::paragraph::ColumnBreakType;
use crate::model::shape::{CommonObjAttr, HorzRelTo, TextWrap, VertRelTo};

use super::reader::{HmlControl, HmlSource};

/// The compatibility case is identified by the missing elements on the
/// styles actually used by the flow. Attributes omitted inside a present
/// element still use their documented defaults.
pub(super) fn apply(source: &mut HmlSource) {
    if source.version != "2.91" || !uses_implicit_flow(source) {
        return;
    }

    for section in &mut source.sections {
        for paragraph in &mut section.paragraphs {
            source.para_shapes[paragraph.para_shape_id as usize].line_spacing = 0;
            for run in &paragraph.char_shapes {
                let style = &mut source.char_shapes[run.char_shape_id as usize];
                style.ratios = [0; 7];
                style.relative_sizes = [0; 7];
            }
            for control in &mut paragraph.controls {
                if let HmlControl::Equation(equation) = control {
                    // This is the fallback frame Hancom writes when the
                    // source supplies a script but no shape dimensions.
                    if equation.font_name.is_empty() {
                        equation.font_name = "HYhwpEQ".to_owned();
                    }
                    equation.implicit_float = Some(fallback_frame());
                }
            }
        }
    }
}

pub(crate) fn fallback_frame() -> CommonObjAttr {
    let side = crate::renderer::HWPUNIT_PER_INCH as u32;
    CommonObjAttr {
        ctrl_id: crate::parser::tags::CTRL_EQUATION,
        flow_with_text: true,
        width: side,
        height: side,
        vert_rel_to: VertRelTo::Para,
        horz_rel_to: HorzRelTo::Column,
        text_wrap: TextWrap::TopAndBottom,
        ..Default::default()
    }
}

/// The elements omitted by the HML writer reconstruct only this placement.
pub(crate) fn uses_fallback_placement(frame: &CommonObjAttr) -> bool {
    let fallback = fallback_frame();
    frame.treat_as_char == fallback.treat_as_char
        && frame.flow_with_text == fallback.flow_with_text
        && frame.vert_rel_to == fallback.vert_rel_to
        && frame.horz_rel_to == fallback.horz_rel_to
        && frame.text_wrap == fallback.text_wrap
        && frame.vert_align == fallback.vert_align
        && frame.horz_align == fallback.horz_align
        && frame.vertical_offset == 0
        && frame.horizontal_offset == 0
        && !frame.allow_overlap
}

fn uses_implicit_flow(source: &HmlSource) -> bool {
    let mut has_equation = false;
    for section in &source.sections {
        for (index, paragraph) in section.paragraphs.iter().enumerate() {
            if !(paragraph.column_type == ColumnBreakType::None
                || (index == 0 && paragraph.column_type == ColumnBreakType::Section))
                || source
                    .para_margin_elements
                    .get(&(paragraph.para_shape_id as usize))
                    != Some(&false)
                || (!paragraph.text.is_empty() && paragraph.char_shapes.is_empty())
                || !paragraph.char_shapes.iter().all(|run| {
                    source
                        .char_layout_elements
                        .get(&(run.char_shape_id as usize))
                        .is_some_and(|layout| !layout.ratio && !layout.relative_size)
                })
            {
                return false;
            }
            match paragraph.controls.as_slice() {
                [HmlControl::Equation(equation)] if !equation.declares_shape_object => {
                    has_equation = true;
                }
                [HmlControl::ColumnDef(columns)] if index == 0 && columns.column_count == 1 => {}
                [] => {}
                _ => return false,
            }
        }
    }
    has_equation
}
