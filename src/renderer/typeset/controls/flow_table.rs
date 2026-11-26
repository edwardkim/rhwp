//! 일반/TAC 표 하나의 포맷·배치 선택과 후행 표 지연 등록 조정.
//! 배치 알고리즘은 기존 엔진 경로를 호출하며 각주 수집·반복 중단은 부모가 소유한다.
use super::super::paragraph::metrics::FormattedParagraph;
use super::super::{preceding_stored_vpos, FormattedTable, TypesetEngine, TypesetState};
use crate::model::{
    control::Control,
    paragraph::{ColumnBreakType, Paragraph},
    shape::VertAlign,
    table::{Table, TablePageBreak},
};
use crate::renderer::{
    composer::ComposedParagraph,
    float_placement::{
        coanchored_float_overlaps_preceding_horizontally, is_para_topbottom_float, signed_hwpunit,
        FloatLaneSet, ParagraphFloatFlow, ParagraphFloatPlacement,
    },
    height_measurer::MeasuredTable,
    hwpunit_to_px,
    pagination::PageItem,
    style_resolver::ResolvedStyleSet,
};

/// 소유 문단의 기존 입력만 빌린다. 상태와 float lane은 별도 가변 인자로 전달한다.
pub(in crate::renderer::typeset) struct FlowTableInput<'a> {
    pub para_idx: usize,
    pub ctrl_idx: usize,
    pub para: &'a Paragraph,
    pub table: &'a Table,
    pub fmt: &'a FormattedParagraph,
    pub measured_tables: &'a [MeasuredTable],
    pub styles: &'a ResolvedStyleSet,
    pub composed: Option<&'a ComposedParagraph>,
    pub next_para: Option<&'a Paragraph>,
    pub tac_count: usize,
    pub first_placed_table: Option<usize>,
    pub last_placed_table: Option<usize>,
    pub para_start_height: f64,
    pub paragraphs_all: &'a [Paragraph],
    pub composed_all: &'a [ComposedParagraph],
    pub ctrl_order: &'a [usize],
    pub order_pos: usize,
}

/// strict fit 경로의 선행 배타 영역 소비에 필요한 높이만 조회한다.
fn natural_top_lead(table: &Table, ft: &FormattedTable, dpi: f64) -> Option<f64> {
    if ft.strict_following_plain_text_fit {
        Some(
            hwpunit_to_px(signed_hwpunit(table.common.vertical_offset).max(0), dpi)
                + hwpunit_to_px(table.outer_margin_top as i32, dpi),
        )
    } else {
        None
    }
}

impl TypesetEngine {
    pub(in crate::renderer::typeset) fn stored_leading_float_frame(
        &self,
        st: &TypesetState,
        paragraphs: &[Paragraph],
        para_idx: usize,
        measured: &[MeasuredTable],
        styles: &ResolvedStyleSet,
    ) -> Option<crate::renderer::float_placement::stored_empty_host_frame::StoredLeadingFloatFrame>
    {
        if st.col_count != 1
            || !st.profile.hwp5_stored_pagination_layout()
            || st.profile.session_edited()
            || st.current_start_height != 0.0
            || st.current_zone_y_offset != 0.0
            || st.available_height() != st.base_available_height()
        {
            return None;
        }
        use crate::renderer::float_placement::stored_empty_host_frame::StoredLeadingFloatFrame;
        let frame = StoredLeadingFloatFrame::resolve(
            paragraphs.get(para_idx.checked_sub(1)?)?,
            paragraphs.get(para_idx)?,
            |index, table| {
                if self.render_normalization.table_text_reflowed(table)
                    || !crate::renderer::pagination::table_cell_footnotes(table).is_empty()
                {
                    return None;
                }
                measured
                    .iter()
                    .find(|entry| entry.para_index == para_idx && entry.control_index == index)
                    .map(|entry| entry.total_height)
            },
            st.available_height(),
            styles,
            self.dpi,
        )?;
        // 저장 좌표와 현재 단의 흐름 끝이 같은 HWPUNIT을 가리키는지
        // 확인한다. 지난 쪽 제목과 이미 놓인 첫 표의 소유권도 구분한다.
        let same_hu =
            |a: f64, b: f64| (a * 7200.0 / self.dpi).round() == (b * 7200.0 / self.dpi).round();
        let previous_is_last = |items: &[crate::renderer::pagination::PageItem]| {
            items.last().and_then(super::super::page_item_para_index) == para_idx.checked_sub(1)
        };
        let matches_flow = if let Some(first) = st.paragraph_float_placements.get(&(para_idx, 0)) {
            same_hu(first.table_top, frame.placements[0].table_top)
                && same_hu(first.occupied_bottom, frame.placements[0].occupied_bottom)
                && same_hu(st.current_height, first.occupied_bottom)
        } else if st.current_items.is_empty() {
            frame.restart
                && st.current_height == 0.0
                && st
                    .pages
                    .iter()
                    .rev()
                    .nth(1)
                    .filter(|page| page.section_index == st.section_index)
                    .and_then(|page| page.column_contents.last())
                    .is_some_and(|column| {
                        previous_is_last(&column.items)
                            && same_hu(column.used_height, frame.previous_end)
                    })
        } else {
            previous_is_last(&st.current_items) && same_hu(st.current_height, frame.previous_end)
        };
        matches_flow.then_some(frame)
    }
}

// The page's accepted opening table may be whole or the last part of a split.
// The sibling's closed box and the host line are consumed in that order.
#[allow(clippy::too_many_arguments)]
fn try_place_closed_terminal_sibling(
    engine: &TypesetEngine,
    st: &mut TypesetState,
    para_idx: usize,
    para: &Paragraph,
    fmt: &FormattedParagraph,
    tail_format: &FormattedTable,
    next: Option<&Paragraph>,
    styles: &ResolvedStyleSet,
) -> bool {
    let [Control::Table(first), Control::Table(tail)] = para.controls.as_slice() else {
        return false;
    };
    let (Some(next), [line]) = (next, para.line_segs.as_slice()) else {
        return false;
    };
    if !st.profile.hwp5_stored_pagination_layout()
        || st.profile.session_edited()
        || st.col_count != 1
        || st.current_zone_y_offset != 0.0
        || para.empty_control_stream_position(0).is_none()
        || para.column_type != ColumnBreakType::None
        || line.vertical_pos <= 0
        || fmt.computed_host_lines.is_some()
        || fmt.spacing_before != 0.0
        || signed_hwpunit(first.common.vertical_offset) <= 0
        || signed_hwpunit(tail.common.vertical_offset)
            <= signed_hwpunit(first.common.vertical_offset)
        || first.row_count <= 1
        || tail.row_count != 1
        || tail.cell_spacing != 0
        || tail.repeat_header
        || tail_format.external_flow.is_some()
        || tail_format.table_footnote_count != 0
        || !coanchored_float_overlaps_preceding_horizontally(para, 1)
        || [first, tail].iter().any(|table| {
            !is_para_topbottom_float(&table.common)
                || !table.common.flow_with_text
                || table.common.allow_overlap
                || table.common.vert_align != VertAlign::Top
                || table.layout_page_break() != TablePageBreak::RowBreak
                || table.caption.is_some()
                || table.outer_margin_top < 0
                || table.outer_margin_bottom < 0
                || engine.render_normalization.table_text_reflowed(table)
        })
    {
        return false;
    }
    let Some(next_style) = styles.para_styles.get(next.para_shape_id as usize) else {
        return false;
    };
    if next_style.page_break_before {
        return false;
    }
    let query = super::deferred::CoanchoredTableQuery::new(para, fmt, engine.tac_flow_query());
    let Some(advance) = query.stored_host_line_advance(next, next_style.spacing_before) else {
        return false;
    };
    let (_, items) = st.coanchored_table_page();
    if !matches!(items.last(), Some(PageItem::Table { para_index, control_index: 0 }
        | PageItem::PartialTable { para_index, control_index: 0, is_continuation: true, .. })
        if *para_index == para_idx)
    {
        return false;
    }
    let dpi = engine.dpi;
    let hu = |value| hwpunit_to_px(value, dpi);
    let anchor = st.current_height;
    let top = anchor + hu(i32::from(tail.outer_margin_top));
    let bottom = top + tail_format.effective_height + hu(i32::from(tail.outer_margin_bottom));
    let end = bottom + advance;
    let column_anchor = hu(st.vpos_page_base.unwrap_or(0));
    if tail_format.effective_height <= 0.0
        || (bottom - (hu(line.vertical_pos) - column_anchor)).abs() > hu(1)
        || end + next_style.spacing_before > st.available_height()
    {
        return false;
    }

    st.record_paragraph_float_placement(
        (para_idx, 1),
        ParagraphFloatPlacement {
            flow: ParagraphFloatFlow::NextLine,
            anchor_y: anchor,
            stored_host_origin: Some(bottom),
            stored_successor_line_origin: Some(end + next_style.spacing_before),
            table_left: None,
            table_top: top,
            occupied_bottom: bottom,
        },
    );
    st.append_item(PageItem::Table {
        para_index: para_idx,
        control_index: 1,
    });
    st.align_flow_to(end);
    st.record_stored_paragraph_flow_end(para_idx, end);
    true
}

pub(in crate::renderer::typeset) fn place(
    engine: &TypesetEngine,
    st: &mut TypesetState,
    input: FlowTableInput<'_>,
    para_float_lanes: &mut FloatLaneSet,
) -> bool {
    let FlowTableInput {
        para_idx,
        ctrl_idx,
        para,
        table,
        fmt,
        measured_tables,
        styles,
        composed,
        next_para,
        tac_count,
        first_placed_table,
        last_placed_table,
        para_start_height,
        paragraphs_all,
        composed_all,
        ctrl_order,
        order_pos,
    } = input;
    let mut break_after_current_table = false;
    let is_column_top = st.flow_table_column_top();
    let ft = engine.format_table(
        para,
        para_idx,
        ctrl_idx,
        table,
        measured_tables,
        styles,
        composed,
        next_para,
        is_column_top,
    );

    // 두 표가 서로 다른 쪽을 소유할 때는 공백 줄까지 포함해 확정한
    // 표 상자를 그대로 발행한다.
    if let Some(frame) =
        engine.stored_leading_float_frame(st, paragraphs_all, para_idx, measured_tables, styles)
    {
        let placement = frame.placements[ctrl_idx];
        let leading_on_page = st.paragraph_float_placements.contains_key(&(para_idx, 0));
        if ctrl_idx == 0 || leading_on_page {
            if (ctrl_idx == 1 || frame.restart) && !st.current_items.is_empty() {
                st.advance_column_or_new_page();
            }
            st.record_paragraph_float_placement((para_idx, ctrl_idx), placement);
            st.append_item(crate::renderer::pagination::PageItem::Table {
                para_index: para_idx,
                control_index: ctrl_idx,
            });
            if ctrl_idx == 0 {
                st.append_item(crate::renderer::pagination::PageItem::PartialParagraph {
                    para_index: para_idx,
                    start_line: 0,
                    end_line: 1,
                });
            }
            st.align_flow_to(placement.occupied_bottom);
            return false;
        }
    }

    let issue2439_para_start_height =
        st.prepare_flow_table_anchor(natural_top_lead(table, &ft, engine.dpi), para_start_height);

    let mt = measured_tables
        .iter()
        .find(|mt| mt.para_index == para_idx && mt.control_index == ctrl_idx);
    let is_first_placed = first_placed_table == Some(ctrl_idx);
    let is_last_placed = last_placed_table == Some(ctrl_idx);
    if ctrl_idx == 1
        && is_last_placed
        && try_place_closed_terminal_sibling(
            engine, st, para_idx, para, fmt, &ft, next_para, styles,
        )
    {
        return false;
    }
    if engine.is_effective_tac_table(para, table, fmt) {
        if !engine.render_normalization.table_text_reflowed(table)
            && !engine
                .render_normalization
                .inline_host_frame_expanded(table)
        {
            if let Some(placement) = super::stored_tac::prepare_coanchored_first_line(
                ctrl_idx,
                para,
                next_para,
                next_para
                    .and_then(|next| styles.para_styles.get(next.para_shape_id as usize))
                    .map_or(0.0, |style| style.spacing_before),
                fmt,
                ft.effective_height,
                st.stored_tac_page(paragraphs_all),
                st.available_height(),
                engine.dpi,
            ) {
                st.commit_stored_tac_control(para_idx, placement);
                return false;
            }
        }
        engine.typeset_tac_table(
            st,
            para_idx,
            ctrl_idx,
            para,
            table,
            &ft,
            fmt,
            tac_count,
            is_first_placed,
            is_last_placed,
            styles,
            composed,
            next_para,
            preceding_stored_vpos(paragraphs_all, para_idx),
        );
    } else if !crate::renderer::float_placement::is_textless_coanchored_float_host(para)
        // [#7687] 양수 오프셋 표를 품은 글자 없는 다중 자리차지 host 는 lane 적층이 아니라
        // 글 있는 host 와 같은 공동 앵커 경로(`place_table_with_text`)로 간다.
        && engine.try_typeset_empty_para_float_table(
        st,
        para_idx,
        ctrl_idx,
        para,
        table,
        &ft,
        composed,
        para_idx
            .checked_sub(1)
            .and_then(|index| paragraphs_all.get(index)),
        next_para,
        styles,
        para_start_height,
        para_float_lanes,
        paragraphs_all,
    ) {
        // Empty host para-float table placed by horizontal lane reservation.
    } else {
        let pages_before_block_table = st.flow_table_page_count();
        engine.typeset_block_table(
            st,
            para_idx,
            ctrl_idx,
            para,
            table,
            &ft,
            fmt,
            mt,
            styles,
            issue2439_para_start_height,
            // [Task #1860] 비지연 경로: para_start_height 가 곧 참 para_start.
            issue2439_para_start_height,
            is_first_placed,
            is_last_placed,
            paragraphs_all,
            composed_all,
        );
        let deferred_query =
            super::deferred::CoanchoredTableQuery::new(para, fmt, engine.tac_flow_query());
        let (page_count, current_items) = st.coanchored_table_page();
        if deferred_query.should_defer_remaining_coanchored_rowbreak_tables(
            table,
            page_count,
            current_items,
            pages_before_block_table,
        ) {
            let deferred = deferred_query.remaining_controls(
                para_idx,
                ctrl_order,
                order_pos,
                first_placed_table,
                last_placed_table,
                para_start_height,
            );
            if !deferred.is_empty() {
                if let ([pending], Some(Control::Table(tail))) =
                    (deferred.as_slice(), para.controls.get(1))
                {
                    if pending.control_index == 1 && pending.is_last_placed {
                        let tail_format = engine.format_table(
                            para,
                            para_idx,
                            1,
                            tail,
                            measured_tables,
                            styles,
                            composed,
                            next_para,
                            st.flow_table_column_top(),
                        );
                        if try_place_closed_terminal_sibling(
                            engine,
                            st,
                            para_idx,
                            para,
                            fmt,
                            &tail_format,
                            next_para,
                            styles,
                        ) {
                            return true;
                        }
                    }
                }
                let page = st.stored_tac_page(paragraphs_all);
                if page.profile.hwp5_stored_pagination_layout() && !page.profile.session_edited() {
                    if let Some(advance) = next_para.and_then(|next| {
                        let before = styles
                            .para_styles
                            .get(next.para_shape_id as usize)?
                            .spacing_before;
                        deferred_query.stored_host_line_advance(next, before)
                    }) {
                        st.advance_flow_by(advance);
                        st.record_stored_paragraph_flow_end(para_idx, st.current_height);
                    }
                }
                st.enqueue_deferred_table_controls(deferred);
                break_after_current_table = true;
            }
        }
    }

    break_after_current_table
}
