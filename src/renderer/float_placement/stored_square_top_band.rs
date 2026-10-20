//! 저장 줄이 상단 전폭 어울림 표 아래에서 시작할 때 공유하는 본문 예약 프레임.
use super::*;

/// 임의의 폭 비율 대신 재조판과 같은 차선 판정을 사용한다. 저장 줄이 표 아래를
/// 소유하고 표가 본문 상단을 막는 경우만 전단 공통 시작 높이를 만든다.
/// 옆 차선이 남거나 상단에 본문을 둘 수 있는 표는 기존 저장 줄 배치를 유지한다.
/// allow_overlap은 개체 간 속성이므로 글 배제 판정에 사용하지 않는다.
pub(crate) fn stored_square_top_band(
    para: &Paragraph,
    control: &Control,
    body: &LayoutRect,
    dpi: f64,
) -> Option<f64> {
    let Control::Table(table) = control else {
        return None;
    };
    let common = &table.common;
    if common.treat_as_char
        || common.text_wrap != TextWrap::Square
        || !matches!(common.vert_rel_to, VertRelTo::Page | VertRelTo::Paper)
        || common.vert_align != VertAlign::Top
        || common.horz_rel_to != HorzRelTo::Page
        || table.caption.is_some()
        || para.stored_text_partition_is_dirty()
        || para.cell_format_vpos_dirty
        || body.width <= 0.0
        || !dpi.is_finite()
        || dpi <= 0.0
    {
        return None;
    }
    let line = para.line_segs.first()?;
    if line.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || line.line_height <= 0
        || line.vertical_pos < 0
    {
        return None;
    }
    let ref_y = if common.vert_rel_to == VertRelTo::Page {
        body.y
    } else {
        0.0
    };
    let height = hwpunit_to_px(table_reserved_height_hu(table) as i32, dpi);
    let (top, bottom) = absolute_table_vertical_geometry(table, ref_y, body.height, height, dpi);
    let hu = |px| super::super::px_to_hwpunit(px, dpi);
    // 저장 원점의 비교도 원본 HWPUNIT로 한다. 같은 경계의 px 합산 오차를
    // 빈 공간 또는 줄 소유의 차이로 오인하지 않는다.
    if hu(top) > hu(body.y) || hu(bottom) <= hu(body.y) || line.vertical_pos < hu(bottom - body.y) {
        return None;
    }
    let context = FloatPlacementContext::new(*body).with_body_area(*body);
    let width = hwpunit_to_px(signed_hwpunit(common.width), dpi);
    let (left, right) = horizontal_range(common, width, context, dpi);
    let exclusion = FrameExclusion {
        horizontal: hu(left - hwpunit_to_px(i32::from(table.outer_margin_left), dpi))
            ..hu(right + hwpunit_to_px(i32::from(table.outer_margin_right), dpi)),
        vertical: hu(top)..hu(bottom),
        policy: frame_exclusion_policy(common.text_flow),
    };
    let body_top = hu(body.y);
    let mut frame = LayoutFrame::new(
        hu(body.x)..hu(body.x + body.width),
        body_top,
        vec![exclusion],
    );
    frame.carve(line.line_height);
    (frame.top > body_top).then_some(bottom)
}

/// 저장 셀의 내용·패딩과 선언 높이가 요구하는 동일한 실제 표 높이.
/// 상단 예약과 기존 TopAndBottom 예약이 이 결과를 함께 소비한다.
pub(crate) fn table_reserved_height_hu(table: &Table) -> u32 {
    let row_count = table.row_count as usize;
    if row_count == 0 {
        return table.common.height;
    }
    let mut row_heights = vec![0u32; row_count];
    for cell in &table.cells {
        if cell.row_span == 1 && (cell.row as usize) < row_count {
            let r = cell.row as usize;
            if cell.height < 0x80000000 && cell.height > row_heights[r] {
                row_heights[r] = cell.height;
            }
        }
    }
    // 셀 내 LINE_SEG 기반 콘텐츠 높이로 보정
    for cell in &table.cells {
        if cell.row_span == 1 && (cell.row as usize) < row_count {
            let r = cell.row as usize;
            let (pad_top, pad_bottom) = if cell.apply_inner_margin {
                (cell.padding.top as u32, cell.padding.bottom as u32)
            } else {
                (table.padding.top as u32, table.padding.bottom as u32)
            };
            let content_h: i32 = cell
                .paragraphs
                .iter()
                .flat_map(|p| p.line_segs.last())
                .map(|s| s.vertical_pos.saturating_add(s.line_height))
                .max()
                .unwrap_or(0);
            let required = content_h as u32 + pad_top + pad_bottom;
            if required > row_heights[r] {
                row_heights[r] = required;
            }
        }
    }
    let total: u32 = row_heights.iter().sum();
    total.max(table.common.height)
}

/// 저장된 빈 줄의 앞뒤가 줄 높이와 뒤 간격을 정확히 닫는지 확인한다.
/// 이 관계만으로 본문 흐름의 소유를 판정하지 않는다. 표 프레임의 안내 줄도
/// 같은 사다리를 가지므로 호출자가 이미 예약된 프레임의 소유를 보존해야 한다.
pub(crate) fn stored_empty_line_has_closed_source_box(
    paragraphs: &[Paragraph],
    index: usize,
) -> bool {
    let Some(current) = paragraphs.get(index) else {
        return false;
    };
    let Some(line) = stored_empty_flow_line(current) else {
        return false;
    };
    let Some(previous) = index.checked_sub(1).and_then(|i| paragraphs.get(i)) else {
        return false;
    };
    let Some(next) = paragraphs.get(index + 1) else {
        return false;
    };
    if [previous, current, next]
        .iter()
        .any(|para| para.stored_text_partition_is_dirty() || para.cell_format_vpos_dirty)
        || current.column_type != crate::model::paragraph::ColumnBreakType::None
        || next.column_type != crate::model::paragraph::ColumnBreakType::None
    {
        return false;
    }
    let Some(before) = previous.line_segs.last() else {
        return false;
    };
    let Some(after) = next.line_segs.first() else {
        return false;
    };
    let original = |line: &crate::model::paragraph::LineSeg| {
        line.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
            && line.line_height > 0
            && line.vertical_pos >= 0
            && line.line_spacing >= 0
    };
    original(before)
        && original(after)
        && before
            .vertical_pos
            .checked_add(before.line_height)
            .and_then(|end| end.checked_add(before.line_spacing))
            == Some(line.vertical_pos)
        && line
            .vertical_pos
            .checked_add(line.line_height)
            .and_then(|end| end.checked_add(line.line_spacing))
            == Some(after.vertical_pos)
}
