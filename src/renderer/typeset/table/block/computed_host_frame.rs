//! 계산한 빈 글줄과 자리차지 표의 바깥 상자를 같은 원점에서 점유한다.

use super::BlockTableInput;
use crate::renderer::{
    float_placement::{
        computed_empty_table_host, signed_hwpunit, ParagraphFloatFlow, ParagraphFloatPlacement,
    },
    hwpunit_to_px,
};

pub(super) fn resolve(
    input: BlockTableInput<'_>,
    origin: f64,
    dpi: f64,
) -> Option<ParagraphFloatPlacement> {
    let BlockTableInput {
        para,
        table,
        ft,
        fmt,
        ..
    } = input;
    // 실제 저장 줄의 원점은 다른 소유자가 검증한다. 계산 줄을 지우거나
    // 저장 줄로 승격하지 않고, 이미 format한 줄 점유를 그대로 함께 사용한다.
    if !computed_empty_table_host(para, table) {
        return None;
    }
    let table_top = origin
        + ft.host_spacing.before
        + hwpunit_to_px(signed_hwpunit(table.common.vertical_offset), dpi);
    let table_end = table_top + ft.effective_height + ft.host_spacing.after;
    let outer_box_top = table_top - hwpunit_to_px(i32::from(table.outer_margin_top), dpi);
    let line_end = origin + fmt.total_height;
    if ![origin, table_top, outer_box_top, table_end, line_end]
        .iter()
        .all(|value| value.is_finite())
    {
        return None;
    }
    Some(ParagraphFloatPlacement {
        flow: ParagraphFloatFlow::ComputedEmptyHost {
            line_advance: fmt.total_height,
            outer_box_top,
        },
        anchor_y: origin,
        stored_host_origin: None,
        stored_successor_line_origin: None,
        table_left: None,
        table_top,
        // 같은 호스트의 계산 줄과 표는 병렬 점유다. 둘의 높이를 합산하지 않는다.
        occupied_bottom: table_end.max(line_end),
    })
}
