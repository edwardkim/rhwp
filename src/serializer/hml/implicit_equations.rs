//! 한컴이 누락된 HML 요소에서 복원한 수식 상자를 같은 방법으로 왕복한다.

use std::collections::BTreeSet;

use crate::model::control::Control;
use crate::model::document::Document;
use crate::model::paragraph::ColumnBreakType;
use crate::model::style::LineSpacingType;
use crate::parser::hml::implicit_equation_print::{fallback_frame, uses_fallback_placement};

#[derive(Default)]
pub(super) struct ImplicitEquationStyles {
    pub char_shapes: BTreeSet<usize>,
    pub para_shapes: BTreeSet<usize>,
}

/// reader가 다시 복원할 수 있는 문서 전체의 형식에 한해 요소를 생략한다.
pub(super) fn implicit_equation_styles(document: &Document) -> Option<ImplicitEquationStyles> {
    let mut used = ImplicitEquationStyles::default();
    let mut has_equation = false;
    let fallback = fallback_frame();
    for section in &document.sections {
        for (index, paragraph) in section.paragraphs.iter().enumerate() {
            let para_id = paragraph.para_shape_id as usize;
            let shape = document.doc_info.para_shapes.get(para_id)?;
            if !(paragraph.column_type == ColumnBreakType::None
                || (index == 0 && paragraph.column_type == ColumnBreakType::Section))
                || shape.line_spacing_type != LineSpacingType::Percent
                || [
                    shape.line_spacing,
                    shape.margin_left,
                    shape.margin_right,
                    shape.indent,
                    shape.spacing_before,
                    shape.spacing_after,
                ] != [0; 6]
                || (!paragraph.text.is_empty() && paragraph.char_shapes.is_empty())
            {
                return None;
            }
            used.para_shapes.insert(para_id);
            for run in &paragraph.char_shapes {
                let char_id = run.char_shape_id as usize;
                let shape = document.doc_info.char_shapes.get(char_id)?;
                if shape.ratios != [0; 7] || shape.relative_sizes != [0; 7] {
                    return None;
                }
                used.char_shapes.insert(char_id);
            }
            match paragraph.controls.as_slice() {
                [Control::Equation(equation)] => {
                    let frame = &equation.common;
                    if (frame.width, frame.height) != (fallback.width, fallback.height)
                        || !uses_fallback_placement(frame)
                    {
                        return None;
                    }
                    has_equation = true;
                }
                [Control::ColumnDef(columns)] if index == 0 && columns.column_count == 1 => {}
                [] => {}
                _ => return None,
            }
        }
    }
    has_equation.then_some(used)
}
