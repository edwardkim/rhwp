//! 쪽 제한을 끈 나누지 않는 float는 본문 높이 때문에 행을 자르지 않는다.

use super::BlockTableInput;
use crate::model::{shape::VertAlign, table::TablePageBreak};
use crate::renderer::{
    float_placement::{
        control_leaves_table_anchor_empty, is_para_topbottom_float, signed_hwpunit,
        ParagraphFloatFlow, ParagraphFloatPlacement,
    },
    hwpunit_to_px,
};

pub(super) fn independent_child_can_split(input: BlockTableInput<'_>, fresh_height: f64) -> bool {
    input
        .ft
        .external_flow
        .is_some_and(|flow| flow.child_splittable && flow.parent_frame_height <= fresh_height)
}

pub(super) fn owns_whole_object(
    input: BlockTableInput<'_>,
    frame_budget: f64,
    fresh_height: f64,
) -> bool {
    is_para_topbottom_float(&input.table.common)
        && !input.table.common.flow_with_text
        && input.ft.table_footnotes.is_empty()
        && ((input.table.page_break == TablePageBreak::None
            && !independent_child_can_split(input, fresh_height))
            || atomic_external_children_fit_parent_frame(input, frame_budget))
}

/// 부모의 쪽 나눔 속성은 부모 밖의 원자 개체를 분할하지 않는다.
/// 측정기가 별도로 인증한 부모 프레임이 현재 쪽에 들어갈 때만 사용한다.
/// 전체 예약 높이는 그대로 두어 밖으로 나온 자식과 후속 본문을 보호한다.
fn atomic_external_children_fit_parent_frame(
    input: BlockTableInput<'_>,
    frame_budget: f64,
) -> bool {
    let Some(measured) = input.mt else {
        return false;
    };
    let Some(frames) = measured.row_frame_heights.as_ref() else {
        return false;
    };
    let frame_height = frames.iter().sum::<f64>()
        + measured.cell_spacing * frames.len().saturating_sub(1) as f64
        + measured.caption_height
        + input.ft.host_spacing.before
        + input.ft.host_spacing.after_for_fit;
    if !frame_height.is_finite() || frame_height > frame_budget {
        return false;
    }
    let mut has_external_child = false;
    for cell in &input.table.cells {
        // 걸침 칸은 별도의 조각 원장이 물리 프레임을 결정한다.
        if cell.row_span != 1 {
            return false;
        }
        for para in &cell.paragraphs {
            for control in &para.controls {
                if let crate::model::control::Control::Table(child) = control {
                    if child.common.treat_as_char
                        || child.common.flow_with_text
                        || !is_para_topbottom_float(&child.common)
                        || child.page_break != TablePageBreak::None
                    {
                        return false;
                    }
                    has_external_child = true;
                }
            }
        }
    }
    has_external_child
}

/// 다른 경로가 인증한 원점은 호출자가 우선한다. 빈 Top 앵커의 누락 상자만
/// 전체 format 결과에서 만들고 예약과 실제 배치에 같은 값을 전달한다.
pub(super) fn missing_frame(
    input: BlockTableInput<'_>,
    origin: f64,
    dpi: f64,
    frame_budget: f64,
    fresh_height: f64,
) -> Option<ParagraphFloatPlacement> {
    if !owns_whole_object(input, frame_budget, fresh_height)
        || !input.para.text.is_empty()
        || input.table.common.vert_align != VertAlign::Top
    {
        return None;
    }
    let mut objects = input
        .para
        .controls
        .iter()
        .filter(|control| !control_leaves_table_anchor_empty(control));
    if !matches!(
        objects.next(),
        Some(crate::model::control::Control::Table(_))
    ) || objects.next().is_some()
    {
        return None;
    }
    let offset = hwpunit_to_px(signed_hwpunit(input.table.common.vertical_offset), dpi);
    let table_top = origin + input.ft.host_spacing.before + offset;
    let occupied_bottom = origin + input.ft.total_height + offset;
    if ![origin, table_top, occupied_bottom]
        .iter()
        .all(|value| value.is_finite())
    {
        return None;
    }
    Some(ParagraphFloatPlacement {
        flow: ParagraphFloatFlow::EmptyHost,
        anchor_y: origin,
        stored_host_origin: None,
        stored_successor_line_origin: None,
        table_left: None,
        table_top,
        occupied_bottom,
    })
}
