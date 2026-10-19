//! 빈 문자열과 실제 저장 줄의 점유를 구분한다. 입력 캐시는 수정하지 않는다.

use super::InlineFlowPlan;
use crate::model::{
    control::Control,
    paragraph::{ColumnBreakType, LineSeg, Paragraph},
    shape::TextWrap,
};
use crate::renderer::{hwpunit_to_px, px_to_hwpunit, style_resolver::ResolvedStyleSet};

/// 다음 실제 줄의 원점이 증언하는 정상 저장 부동 개체 호스트 예약.
pub(crate) fn plan_stored_empty_square_host(
    host: &Paragraph,
    next: &Paragraph,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<InlineFlowPlan> {
    if host.text.chars().any(|c| c > '\u{001f}' && c != '\u{fffc}')
        || host.stored_text_partition_is_dirty()
        || host.cell_format_vpos_dirty
        || next.stored_text_partition_is_dirty()
        || next.cell_format_vpos_dirty
        || !matches!(next.column_type, ColumnBreakType::None)
    {
        return None;
    }
    let mut saw_float = false;
    for control in &host.controls {
        let common = match control {
            Control::Picture(p) if p.caption.is_none() => &p.common,
            Control::Shape(s) => s.common(),
            Control::SectionDef(_)
            | Control::ColumnDef(_)
            | Control::PageNumberPos(_)
            | Control::PageNumCtrl(_) => continue,
            _ => return None,
        };
        if common.treat_as_char
            || !matches!(
                common.text_wrap,
                TextWrap::Square | TextWrap::InFrontOfText | TextWrap::BehindText
            )
        {
            return None;
        }
        saw_float = true;
    }
    if !saw_float {
        return None;
    }
    let first = host.line_segs.first()?;
    let following = next.line_segs.first()?;
    let synthetic = LineSeg::TAG_IMPLEMENTATION_PROPERTY;
    if following.tag & synthetic != 0
        || following.line_height <= 0
        || first.line_height <= 0
        || host.line_segs.iter().any(|s| {
            s.tag & synthetic != 0
                || s.vertical_pos != first.vertical_pos
                || s.line_height <= 0
                || i64::from(s.line_height) + i64::from(s.line_spacing) <= 0
        })
        || host
            .line_segs
            .iter()
            .skip(1)
            .any(|s| s.column_start == first.column_start)
    {
        return None;
    }
    let style = styles.para_styles.get(host.para_shape_id as usize)?;
    let following_style = styles.para_styles.get(next.para_shape_id as usize)?;
    if following_style.page_break_before {
        return None;
    }
    let mut row = first.clone();
    row.line_height = host.line_segs.iter().map(|s| s.line_height).max()?;
    row.line_spacing = host.line_segs.iter().map(|s| s.line_spacing).max()?;
    let delta = i64::from(following.vertical_pos) - i64::from(first.vertical_pos);
    let expected = i64::from(row.line_height)
        + i64::from(row.line_spacing)
        + i64::from(px_to_hwpunit(style.spacing_after, dpi))
        + i64::from(px_to_hwpunit(following_style.spacing_before, dpi));
    // HWPUNIT 변환의 저장 반올림만 허용한다. 위치를 px 범위로 골라내지 않는다.
    if delta <= 0 || (delta - expected).abs() > 1 {
        return None;
    }
    row.vertical_pos = 0;
    // 음수 줄간격은 줄 상자의 높이와 다음 원점의 전진량을 다르게 만든다.
    // 양수인 실제 전진량이 다음 저장 원점과 일치할 때만 그대로 소비한다.
    let end = style.spacing_before
        + hwpunit_to_px(row.line_height.saturating_add(row.line_spacing), dpi)
        + style.spacing_after;
    Some(InlineFlowPlan {
        stored_empty_host: true,
        stored_float_host: false,
        text_rows: Some(vec![row]),
        square_host_control: None,
        text_spacing_before: Some(style.spacing_before),
        square_host_placement: None,
        square_host_exclusion: None,
        start: 0.0,
        end,
        boxes: Vec::new(),
        rows: Vec::new(),
        carved: false,
        next_row_top: end,
        fallback_font_size: 0.0,
    })
}

/// 실제 저장 첫 원점이 절대 전폭 표의 배제 하단일 때 앞 간격은 이미 트림됐다.
/// 저장 줄의 원점과 끝을 측정·부분 본문 paint가 같은 계획으로 소비한다.
#[allow(clippy::too_many_arguments)]
pub(crate) fn plan_stored_body_wide_host(
    host: &Paragraph,
    next: &Paragraph,
    styles: &ResolvedStyleSet,
    body: &crate::renderer::page_layout::LayoutRect,
    page_height: f64,
    dpi: f64,
    para_index: usize,
    measured_tables: &[crate::renderer::height_measurer::MeasuredTable],
) -> Option<InlineFlowPlan> {
    use crate::model::shape::VertRelTo;
    if host.stored_text_partition_is_dirty()
        || host.cell_format_vpos_dirty
        || next.stored_text_partition_is_dirty()
        || next.cell_format_vpos_dirty
        || !host
            .text
            .chars()
            .any(|ch| ch > '\u{001f}' && !ch.is_whitespace() && ch != '\u{fffc}')
        || !matches!(next.column_type, ColumnBreakType::None)
    {
        return None;
    }
    let mut bottom = None;
    for (control_index, control) in host.controls.iter().enumerate() {
        let table = match control {
            Control::Table(table)
                if !table.common.treat_as_char
                    && table.common.text_wrap == TextWrap::TopAndBottom
                    && matches!(table.common.vert_rel_to, VertRelTo::Paper | VertRelTo::Page)
                    && table.caption.is_none() =>
            {
                table
            }
            Control::SectionDef(_) | Control::ColumnDef(_) => continue,
            _ => return None,
        };
        if bottom.is_some() || hwpunit_to_px(table.common.width as i32, dpi) < body.width * 0.8 {
            return None;
        }
        // 셀 편집으로 내용이 선언 프레임보다 커졌으면 낡은 저장 배제 하단을 재사용하지 않는다.
        // 다른 문단의 편집 여부 대신 이 표의 실제 측정과 현재 선언 크기를 대조한다.
        let measured = measured_tables.iter().find(|measured| {
            measured.para_index == para_index && measured.control_index == control_index
        })?;
        if !measured.total_height.is_finite()
            || measured.total_height
                > hwpunit_to_px(table.common.height as i32, dpi) + hwpunit_to_px(1, dpi)
        {
            return None;
        }
        let (ref_y, ref_h) = if table.common.vert_rel_to == VertRelTo::Paper {
            (0.0, page_height)
        } else {
            (body.y, body.height)
        };
        let (_, end) = crate::renderer::float_placement::absolute_table_vertical_geometry(
            table,
            ref_y,
            ref_h,
            hwpunit_to_px(table.common.height as i32, dpi),
            dpi,
        );
        bottom = Some(end - body.y);
    }
    let start = bottom.filter(|v| *v > 0.0)?;
    let first = host.line_segs.first()?;
    let last = host.line_segs.last()?;
    let following = next.line_segs.first()?;
    let synthetic = LineSeg::TAG_IMPLEMENTATION_PROPERTY;
    if (i64::from(first.vertical_pos) - i64::from(px_to_hwpunit(start, dpi))).abs() > 1
        || host
            .line_segs
            .iter()
            .chain(std::iter::once(following))
            .any(|line| {
                line.tag & synthetic != 0
                    || line.line_height <= 0
                    || i64::from(line.line_height) + i64::from(line.line_spacing) <= 0
            })
        || host.line_segs.windows(2).any(|pair| {
            pair[1].vertical_pos
                != pair[0]
                    .vertical_pos
                    .saturating_add(pair[0].line_height)
                    .saturating_add(pair[0].line_spacing)
        })
    {
        return None;
    }
    let after = styles
        .para_styles
        .get(host.para_shape_id as usize)?
        .spacing_after;
    let next_style = styles.para_styles.get(next.para_shape_id as usize)?;
    let expected = i64::from(last.vertical_pos)
        + i64::from(last.line_height)
        + i64::from(last.line_spacing)
        + i64::from(px_to_hwpunit(after + next_style.spacing_before, dpi));
    if next_style.page_break_before || (expected - i64::from(following.vertical_pos)).abs() > 1 {
        return None;
    }
    let mut rows = host.line_segs.clone();
    for row in &mut rows {
        row.vertical_pos = row.vertical_pos.saturating_sub(first.vertical_pos);
    }
    let end = start
        + hwpunit_to_px(
            last.vertical_pos
                .saturating_sub(first.vertical_pos)
                .saturating_add(last.line_height)
                .saturating_add(last.line_spacing),
            dpi,
        )
        + after;
    Some(InlineFlowPlan {
        stored_empty_host: false,
        stored_float_host: true,
        text_rows: Some(rows),
        square_host_control: None,
        text_spacing_before: Some(0.0),
        square_host_placement: None,
        square_host_exclusion: None,
        start,
        end,
        boxes: Vec::new(),
        rows: Vec::new(),
        carved: false,
        next_row_top: end,
        fallback_font_size: 0.0,
    })
}
