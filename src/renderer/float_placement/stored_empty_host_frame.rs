//! 다음 저장 줄이 닫는 빈 호스트 표의 프레임을 측정·배치에서 공유한다.

use crate::model::{
    control::Control,
    paragraph::{ColumnBreakType, LineSeg, Paragraph},
    shape::{CommonObjAttr, HorzRelTo, TextWrap, VertAlign, VertRelTo},
    table::Table,
};
use crate::renderer::{hwpunit_to_px, px_to_hwpunit, stored_float_anchor};

use super::{is_para_topbottom_float, signed_hwpunit, ParagraphFloatFlow, ParagraphFloatPlacement};

/// 종이·쪽에 고정된 장식은 보이더라도 호스트의 글줄 흐름을 밀지 않는다.
fn detached_overlay(common: &CommonObjAttr) -> bool {
    !common.treat_as_char
        && !common.flow_with_text
        && !common.affect_line_spacing
        && matches!(
            common.text_wrap,
            TextWrap::InFrontOfText | TextWrap::BehindText
        )
        && matches!(common.vert_rel_to, VertRelTo::Paper | VertRelTo::Page)
        && matches!(common.horz_rel_to, HorzRelTo::Paper | HorzRelTo::Page)
}

fn single_flow_table(para: &Paragraph) -> bool {
    para.controls
        .iter()
        .filter(|c| matches!(c, Control::Table(_)))
        .count()
        == 1
        && para.controls.iter().all(|control| match control {
            Control::Table(_) => true,
            Control::Shape(shape) => detached_overlay(shape.common()),
            Control::Picture(picture) => detached_overlay(&picture.common),
            _ => false,
        })
}

fn stored_line(para: &Paragraph) -> Option<i32> {
    let line = para.line_segs.first()?;
    (line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
        && !para.stored_text_partition_is_dirty()
        && !para.cell_format_vpos_dirty)
        .then_some(line.vertical_pos)
}

/// 저장 호스트 앞 간격을 제외한 원점부터 다음 줄까지가 선언 본체·오프셋·
/// 바깥 여백을 정확히 포함할 때만 그 상자를 수용한다. 선언 높이가 같다는
/// 사실만으로 문단 뒤 간격이나 호스트 줄 높이를 추가하지 않는다.
pub(super) fn from_stored_ladder(
    para: &Paragraph,
    next: &Paragraph,
    table: &Table,
    origin: f64,
    spacing_before: f64,
    measured_table_height: f64,
    dpi: f64,
) -> Option<ParagraphFloatPlacement> {
    let paragraph_square = !table.common.treat_as_char
        && table.common.text_wrap == TextWrap::Square
        && table.common.vert_rel_to == VertRelTo::Para;
    if !single_flow_table(para)
        || para
            .text
            .chars()
            .any(|c| !c.is_whitespace() && !c.is_control() && c != '\u{FFFC}')
        || !(is_para_topbottom_float(&table.common) || paragraph_square)
        || next.column_type != ColumnBreakType::None
        || table.common.vert_align != VertAlign::Top
        || signed_hwpunit(table.common.vertical_offset) < 0
        || table.caption.is_some()
        || !origin.is_finite()
        || !spacing_before.is_finite()
        || spacing_before < 0.0
        || !measured_table_height.is_finite()
    {
        return None;
    }
    let declared = hwpunit_to_px(table.common.height.min(i32::MAX as u32) as i32, dpi);
    if (measured_table_height - declared).abs() > 1.0 {
        return None;
    }
    let host_vpos = stored_line(para)?;
    let next_vpos = stored_line(next)?;
    let physical_hu = i64::from(signed_hwpunit(table.common.vertical_offset))
        + i64::from(table.outer_margin_top)
        + i64::from(table.common.height.min(i32::MAX as u32))
        + i64::from(table.outer_margin_bottom);
    let spacing_before_hu = px_to_hwpunit(spacing_before, dpi) as i64;
    let delta = i64::from(next_vpos) - i64::from(host_vpos);
    if delta <= 0 || (delta - (physical_hu - spacing_before_hu)).abs() > 2 {
        return None;
    }
    // 점유 하단은 그림 원점이 아니라 다음 저장 줄이 닫는 전체 프레임이다.
    // 비대칭 여백의 본체 원점도 기존 저장 앵커와 같은 규칙으로 읽는다.
    let table_top = origin
        + (i64::from(signed_hwpunit(table.common.vertical_offset))
            + stored_float_anchor::stored_outer_box_table_top_hu(table)) as f64
            * dpi
            / 7200.0;
    let occupied_bottom = origin + physical_hu as f64 * dpi / 7200.0;
    Some(ParagraphFloatPlacement {
        flow: ParagraphFloatFlow::Exclusion,
        anchor_y: origin,
        stored_host_origin: None,
        stored_successor_line_origin: Some(occupied_bottom),
        table_left: None,
        table_top,
        occupied_bottom,
    })
}
