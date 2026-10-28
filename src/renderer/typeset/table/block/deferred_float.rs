//! 쪽 제한을 켠 나누지 않는 표와 앞쪽에 남는 호스트 글줄을 분리한다.

use super::BlockTableInput;
use crate::model::table::TablePageBreak;
use crate::renderer::float_placement::{
    computed_empty_table_host, control_leaves_table_anchor_empty, is_para_topbottom_float,
    signed_hwpunit, ParagraphFloatFlow, ParagraphFloatPlacement,
};
use crate::renderer::hwpunit_to_px;
use crate::renderer::pagination::PageItem;
use crate::renderer::typeset::{DeferredStoredFrameControl, DeferredStoredFrameKind, TypesetState};

pub(super) fn try_defer(st: &mut TypesetState, input: BlockTableInput<'_>, dpi: f64) -> bool {
    let BlockTableInput {
        para,
        table,
        ft,
        fmt,
        para_idx,
        ctrl_idx,
        ..
    } = input;
    // 나누지 않는 개체가 현재 글줄 사이에 들어가지 않으면 글줄은 현재 쪽에서
    // 계속 흐르고 개체만 다음 쪽 상단을 소유한다. 저장 좌표 대신 현재 구성을 쓴다.
    let computed_visible_host = fmt.host_composition.is_some()
        && input.is_first_placed
        && input.is_last_placed
        && fmt.computed_host_lines.as_deref().is_some_and(|lines| {
            ParagraphFloatPlacement::from_split_computed_host(
                para,
                table,
                ctrl_idx,
                st.current_height + fmt.spacing_before,
                lines,
                ft.effective_height,
                dpi,
            )
            .is_some()
        });
    if st.col_count != 1
        || table.page_break != TablePageBreak::None
        || !table.common.flow_with_text
        || !is_para_topbottom_float(&table.common)
        || table.common.vert_align != crate::model::shape::VertAlign::Top
        || !(computed_empty_table_host(para, table)
            || saved_host_matches_current_frame(st, input, dpi)
            || computed_visible_host)
        || signed_hwpunit(table.common.vertical_offset) != 0
        || table.caption.is_some()
        || !ft.table_footnotes.is_empty()
        || fmt.line_count() == 0
    {
        return false;
    }
    // 호스트의 문단 간격·빈 줄은 현재 쪽의 점유다. 다음 쪽의 표 프레임에
    // 다시 더하지 않으며, 그 쪽은 표의 바깥 여백과 실제 셀 상자를 소유한다.
    let table_top = hwpunit_to_px(i32::from(table.outer_margin_top), dpi);
    let bottom =
        table_top + ft.effective_height + hwpunit_to_px(i32::from(table.outer_margin_bottom), dpi);
    let available = st.available_height();
    if !bottom.is_finite()
        || st.current_height + bottom <= available
        || st.current_height + fmt.height_for_fit > available
    {
        return false;
    }
    if computed_visible_host {
        st.record_host_text_fragment(para_idx, 0..fmt.line_count(), para, fmt, dpi);
    }
    st.append_item(PageItem::PartialParagraph {
        para_index: para_idx,
        start_line: 0,
        end_line: fmt.line_count(),
    });
    st.advance_flow_by(fmt.total_height);
    st.mark_pre_emitted_host(para_idx);
    st.defer_stored_frame(DeferredStoredFrameControl {
        kind: DeferredStoredFrameKind::Table,
        para_index: para_idx,
        control_index: ctrl_idx,
        placement: ParagraphFloatPlacement {
            flow: ParagraphFloatFlow::EmptyHost,
            anchor_y: 0.0,
            stored_host_origin: None,
            stored_successor_line_origin: None,
            table_left: None,
            table_top,
            occupied_bottom: bottom,
        },
        following_body_para_index: None,
    });
    true
}

/// 저장 호스트와 뒤 본문이 같은 쪽의 연속 글줄임을 실제 저장 사다리로 확인한다.
/// 개체만 이월될 때에도 이 줄들은 현재 쪽에 남는다. 계산 줄을 저장 줄로 간주하지 않는다.
fn saved_host_matches_current_frame(
    st: &TypesetState,
    input: BlockTableInput<'_>,
    dpi: f64,
) -> bool {
    let para = input.para;
    if st.profile.session_edited()
        || !(st.profile.hwp5_stored_pagination_layout() || st.profile.hwpx_stored_layout())
        || !para.text.is_empty()
        || para.stored_text_partition_is_dirty()
        || para.cell_format_vpos_dirty
        || para
            .controls
            .iter()
            .filter(|c| !control_leaves_table_anchor_empty(c))
            .count()
            != 1
    {
        return false;
    }
    let ([host], Some(next)) = (
        para.line_segs.as_slice(),
        input.paragraphs_all.get(input.para_idx + 1),
    ) else {
        return false;
    };
    let Some(line) = next.line_segs.first() else {
        return false;
    };
    let real_line = |line: &crate::model::paragraph::LineSeg| {
        line.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
            && line.vertical_pos >= 0
            && line.line_height > 0
    };
    let Some(style) = input.styles.para_styles.get(next.para_shape_id as usize) else {
        return false;
    };
    let base = st.vpos_page_base.or(st.vpos_lazy_base).unwrap_or(0);
    let Some(host_vpos) = host.vertical_pos.checked_sub(base) else {
        return false;
    };
    let Some(next_bottom) = line
        .vertical_pos
        .saturating_add(line.line_height)
        .checked_sub(base)
    else {
        return false;
    };
    let host_origin = st.vpos_col_anchor + hwpunit_to_px(host_vpos, dpi);
    let next_end = st.vpos_col_anchor + hwpunit_to_px(next_bottom, dpi);
    let gap = crate::renderer::px_to_hwpunit(input.fmt.spacing_after + style.spacing_before, dpi);
    real_line(host)
        && real_line(line)
        && next.controls.is_empty()
        && !next.stored_text_partition_is_dirty()
        && !next.cell_format_vpos_dirty
        && next.column_type == crate::model::paragraph::ColumnBreakType::None
        && input.fmt.line_count() == 1
        && (host_origin - st.current_height - input.fmt.spacing_before).abs() <= dpi / 7200.0
        && (input.fmt.line_advances_sum(0..1)
            - hwpunit_to_px(host.line_height.saturating_add(host.line_spacing), dpi))
        .abs()
            <= dpi / 7200.0
        && next_end <= st.base_available_height()
        && i64::from(line.vertical_pos)
            == i64::from(host.vertical_pos)
                + i64::from(host.line_height)
                + i64::from(host.line_spacing)
                + i64::from(gap)
}
