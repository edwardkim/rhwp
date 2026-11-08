//! 본문과 바탕쪽의 단 구분선에 같은 영역 계산·선 스타일을 적용한다.

use super::border_rendering::border_width_to_px;
use crate::model::header_footer::MasterPage;
use crate::model::page::ColumnDef;
use crate::renderer::page_layout::{calculate_column_areas, LayoutRect, PageLayoutInfo};
use crate::renderer::render_tree::LineNode;
use crate::renderer::{hwpunit_to_px, LineStyle, StrokeDash};

/// 바탕쪽의 저장 텍스트 프레임 전체를 점유하는 배경 구분선이다.
/// 본문 내용 끝이나 본문의 단 설정으로 바탕쪽 정의를 대신하지 않는다.
pub(super) fn master_page_lines(
    master: &MasterPage,
    definition: &ColumnDef,
    page: &PageLayoutInfo,
    dpi: f64,
) -> Vec<LineNode> {
    if master.text_width == 0 || master.text_height == 0 {
        return Vec::new();
    }
    let frame = LayoutRect {
        x: page.body_area.x,
        y: page.body_area.y,
        width: hwpunit_to_px(master.text_width as i32, dpi),
        height: hwpunit_to_px(master.text_height as i32, dpi),
    };
    let mut layout = page.clone();
    layout.body_area = frame;
    layout.column_areas = calculate_column_areas(&frame, definition, dpi);
    layout.separator_type = definition.separator_type;
    layout.separator_width = definition.separator_width;
    layout.separator_color = definition.separator_color;
    separator_lines(&layout, frame.y, frame.y + frame.height)
}

/// 본문 zone과 바탕쪽에서 소비할 실제 선 기하를 한 번 계산한다.
pub(super) fn separator_lines(layout: &PageLayoutInfo, y_start: f64, y_end: f64) -> Vec<LineNode> {
    // 본문 zone의 기존 물리 프레임 하단 제한을 보존한다.
    let y_end = y_end.min(layout.body_area.y + layout.body_area.height);
    if layout.column_areas.len() < 2 || layout.separator_type == 0 || y_end <= y_start {
        return Vec::new();
    }
    let width = border_width_to_px(layout.separator_width).max(0.5);
    let dash = match layout.separator_type {
        2 | 6 => StrokeDash::Dash,
        3 | 7 => StrokeDash::Dot,
        4 => StrokeDash::DashDot,
        5 => StrokeDash::DashDotDot,
        _ => StrokeDash::Solid,
    };
    layout
        .column_areas
        .windows(2)
        .map(|pair| {
            let (left, right) = if pair[0].x <= pair[1].x {
                (&pair[0], &pair[1])
            } else {
                (&pair[1], &pair[0])
            };
            let x = (left.x + left.width + right.x) / 2.0;
            LineNode::new(
                x,
                y_start,
                x,
                y_end,
                LineStyle {
                    color: layout.separator_color,
                    width,
                    dash,
                    ..Default::default()
                },
            )
        })
        .collect()
}
