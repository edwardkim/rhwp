//! 같은 저장 글줄의 인라인 표 뒤에서도 부동 표는 문단 앵커를 공유한다.

use crate::model::{control::Control, shape::VertAlign};
use crate::renderer::{
    float_placement::{ParagraphFloatFlow, ParagraphFloatPlacement},
    typeset::{
        is_para_topbottom_float, is_synthetic_line_seg, PageItem, TypesetEngine, TypesetState,
    },
};

use super::FragmentInput;

impl TypesetEngine {
    /// 첫 원본 프레임의 행 끝은 선언 높이와 다음 쪽의 종료 위치가 함께 인증한다.
    /// 같은 호스트의 선행 인라인 개체 때문에 현재 커서가 전진해도 행을 더 소유하지 않는다.
    pub(super) fn coanchored_closed_first_fragment_row_end(
        &self,
        st: &TypesetState,
        input: FragmentInput<'_>,
        placement: Option<ParagraphFloatPlacement>,
        host_before: f64,
        vertical_offset: f64,
    ) -> Option<usize> {
        // 이 조회가 인증한 실제 동일 호스트 배치만 사용한다. 다른 float·새 쪽·편집본은
        // 기존 원점 조회에서 제외되며, 아래의 원본 행/후속 줄 계약도 모두 필요하다.
        let accepted = placement?;
        let original =
            self.coanchored_first_fragment_origin(st, input, host_before, vertical_offset)?;
        if accepted.anchor_y != original.anchor_y
            || accepted.table_top != original.table_top
            || accepted.stored_host_origin != original.stored_host_origin
            || accepted.stored_host_origin.is_none()
        {
            return None;
        }
        let table = input.source.table;
        if !std::ptr::eq(table, input.source.row_geometry_table)
            || !matches!(
                table.page_break,
                crate::model::table::TablePageBreak::RowBreak
            )
            || !input.prepared.source_next_positive_rewind
            || !input.prepared.table_footnotes.is_empty()
            || st.current_footnote_height > 0.0
            || !table.cells.iter().all(|cell| {
                cell.row_span == 1
                    && !cell.dirty_flag
                    && cell.paragraphs.iter().all(|para| {
                        !para.stored_text_partition_is_dirty()
                            && !para.cell_format_vpos_dirty
                            && !para.line_segs.is_empty()
                            && para
                                .line_segs
                                .iter()
                                .all(|line| line.line_height > 0 && !is_synthetic_line_seg(line))
                    })
            })
        {
            return None;
        }
        let next = input
            .source
            .paragraphs_all
            .get(input.source.para_index.checked_add(1)?)?;
        let [after] = next.line_segs.as_slice() else {
            return None;
        };
        if next.stored_text_partition_is_dirty()
            || next.cell_format_vpos_dirty
            || is_synthetic_line_seg(after)
            || after.vertical_pos <= 0
            || after.vertical_pos >= input.source.paragraph.line_segs.first()?.vertical_pos
        {
            return None;
        }
        let caption = table.caption.as_ref()?;
        if caption.direction != crate::model::shape::CaptionDirection::Bottom
            || caption.paragraphs.iter().any(|para| {
                para.stored_text_partition_is_dirty()
                    || para.cell_format_vpos_dirty
                    || para.line_segs.is_empty()
                    || para
                        .line_segs
                        .iter()
                        .any(|line| is_synthetic_line_seg(line))
            })
        {
            return None;
        }
        let caption_hu = crate::renderer::px_to_hwpunit(
            crate::renderer::composer::caption_height_px(&table.caption, self.dpi),
            self.dpi,
        );
        let closing_hu = i64::from(caption_hu)
            + i64::from(caption.spacing)
            + i64::from(table.outer_margin_top)
            + i64::from(table.outer_margin_bottom);
        let heights = &input.prepared.whole_row_fit_heights;
        let rows = input.prepared.row_count;
        if heights.len() != rows || rows < 2 {
            return None;
        }
        let spacing_hu = i64::from(crate::renderer::px_to_hwpunit(
            input.prepared.cell_spacing,
            self.dpi,
        ));
        let row_hu = heights
            .iter()
            .map(|height| {
                if !height.is_finite() || *height <= 0.0 {
                    return None;
                }
                let hu = crate::renderer::px_to_hwpunit(*height, self.dpi);
                ((*height - crate::renderer::hwpunit_to_px(hu, self.dpi)).abs()
                    <= self.dpi / 7200.0)
                    .then_some(i64::from(hu))
            })
            .collect::<Option<Vec<_>>>()?;
        let total_hu = row_hu.iter().sum::<i64>() + spacing_hu * (rows - 1) as i64;
        let mut prefix_hu = 0_i64;
        for (row, height) in row_hu.iter().enumerate().take(rows - 1) {
            prefix_hu += height + if row > 0 { spacing_hu } else { 0 };
            if prefix_hu == i64::from(table.common.height)
                && total_hu - prefix_hu - spacing_hu + closing_hu == i64::from(after.vertical_pos)
            {
                return Some(row + 1);
            }
        }
        None
    }

    /// 이미 이 단에 놓인 인라인 형제는 흐름 높이를 소비하지만 문단 원점을 바꾸지 않는다.
    /// 첫 부동 조각의 실제 원점을 예산과 확정 배치에 함께 전달한다. 다른 줄의 표,
    /// 아직 놓이지 않은 형제, 새 쪽의 이어받기는 같은 앵커의 근거로 사용하지 않는다.
    pub(super) fn coanchored_first_fragment_origin(
        &self,
        st: &TypesetState,
        input: FragmentInput<'_>,
        host_before: f64,
        vertical_offset: f64,
    ) -> Option<ParagraphFloatPlacement> {
        let para = input.source.paragraph;
        let table = input.source.table;
        if !st.profile.hwp5_stored_pagination_layout()
            || st.profile.session_edited()
            || input.start.is_continuation
            || input.start.cursor_row != 0
            || !input.start.start_cut.is_empty()
            || para.stored_text_partition_is_dirty()
            || para.cell_format_vpos_dirty
            || crate::renderer::typeset::para_has_non_whitespace_text(para)
            || !is_para_topbottom_float(&table.common)
            || table.common.allow_overlap
            || !table.common.flow_with_text
            || table.common.vert_align != VertAlign::Top
            // 아래 캡션은 마지막 조각의 종료 공간이며 문단 앵커를 바꾸지 않는다.
            // 위·옆 캡션은 별도 시작 프레임이 필요하므로 이 조회에서 인증하지 않는다.
            || table.caption.as_ref().is_some_and(|caption| {
                caption.direction != crate::model::shape::CaptionDirection::Bottom
            })
            || vertical_offset <= 0.0
            || self.render_normalization.table_text_reflowed(table)
            || input.prepared.host_frame
                != (
                    st.pages.len(),
                    st.current_column,
                    st.current_zone_y_offset.to_bits(),
                )
        {
            return None;
        }
        let [line] = para.line_segs.as_slice() else {
            return None;
        };
        if is_synthetic_line_seg(line) || line.line_height <= 0 {
            return None;
        }
        let PageItem::Table {
            para_index,
            control_index,
        } = st.current_items.last()?
        else {
            return None;
        };
        if *para_index != input.source.para_index || *control_index == input.source.control_index {
            return None;
        }
        let Control::Table(sibling) = para.controls.get(*control_index)? else {
            return None;
        };
        // 글자처럼 취급하는 실제 형제가 같은 원본 줄에 놓였는지 확인한다.
        // 제어 배열의 앞뒤나 같은 속성만으로 이미 소비한 줄을 추정하지 않는다.
        let owner_line =
            crate::renderer::layout::control_line_seg_index(para, input.source.control_index)?;
        if !sibling.common.treat_as_char
            || crate::renderer::layout::control_line_seg_index(para, *control_index)
                != Some(owner_line)
            || self.render_normalization.table_text_reflowed(sibling)
        {
            return None;
        }
        let anchor = input.prepared.budget_para_start_height;
        if !anchor.is_finite() || anchor < 0.0 || anchor > st.current_height {
            return None;
        }
        let top = anchor + host_before + vertical_offset;
        // 첫 조각의 선언 높이는 저장 호스트 줄과 같은 물리 축에서만 인증한다.
        // 앞 TAC의 흐름 끝을 호스트 원점으로 쓰면 선언된 첫 컷을 잃는다.
        let spacing_before = input
            .source
            .styles
            .para_styles
            .get(para.para_shape_id as usize)?
            .spacing_before;
        let source_host_origin = st
            .vpos_page_base
            .map(|base| (base, st.vpos_col_anchor))
            .or(st.vpos_stored_column_origin)
            .and_then(|(base, column_origin)| {
                line.vertical_pos.checked_sub(base).map(|position| {
                    column_origin + crate::renderer::hwpunit_to_px(position, self.dpi)
                })
            })
            .filter(|origin| (origin - anchor - spacing_before).abs() <= self.dpi / 7200.0)
            // 단 중간의 같은 원본 줄은 앞 TAC가 소비한 문단 앞 간격을 소유한다.
            // 전역 축이 닫혀 있어도 이 실제 호스트 위치를 선언 프레임과 대조한다.
            // 단 맨 위의 앞 간격 수용은 별도 계약이므로 여기서 추정하지 않는다.
            .or_else(|| (anchor > 0.0).then_some(anchor + spacing_before));
        if std::env::var("RHWP_TABLE_DRIFT").is_ok() {
            eprintln!(
                "COANCHORED_FIRST_FRAME: pi={} ci={} anchor={:.3} flow={:.3} top={:.3}",
                input.source.para_index, input.source.control_index, anchor, st.current_height, top
            );
        }
        let placement = ParagraphFloatPlacement {
            flow: ParagraphFloatFlow::NextLine,
            anchor_y: anchor,
            stored_host_origin: source_host_origin,
            stored_successor_line_origin: None,
            table_left: None,
            table_top: top,
            occupied_bottom: top,
        };
        Some(
            placement.clear_occupied_bands(st.visible_float_exclusions.iter().map(|zone| {
                zone.top
                    ..zone.bottom
                        + crate::renderer::hwpunit_to_px(
                            i32::from(table.outer_margin_top),
                            self.dpi,
                        )
            })),
        )
    }
}
