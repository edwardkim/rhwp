//! 현재 줄 범위의 앞부분·표·뒷부분을 통째 수용과 행 분할에 함께 전달한다.

use super::BlockTableInput;
use crate::renderer::{
    float_placement::ParagraphFloatPlacement,
    typeset::{paragraph::metrics::FormattedHostTail, PageItem, TypesetEngine, TypesetState},
};

pub(super) enum SplitHostHandoff {
    Placed,
    Fragment {
        placement: ParagraphFloatPlacement,
        tail: Box<FormattedHostTail>,
        available: f64,
    },
}

impl TypesetEngine {
    pub(super) fn place_computed_split_host(
        &self,
        st: &mut TypesetState,
        input: BlockTableInput<'_>,
        mut placement: ParagraphFloatPlacement,
        available: f64,
        footnote_margin: f64,
    ) -> Option<SplitHostHandoff> {
        let prefix = placement.split_host_prefix()?;
        let fmt = input.fmt;
        fmt.host_composition.as_ref()?;
        input.mt.filter(|table| !table.row_heights.is_empty())?;
        // 여러 개체의 공동 호스트는 전체 개체 순서의 기존 소유자가 처리한다.
        // 단일 개체 계획이 중간 표 뒤에서 호스트 꼬리를 먼저 소비하지 않는다.
        if !input.is_first_placed || !input.is_last_placed || prefix >= fmt.line_count() {
            return None;
        }
        let table_distance = placement.table_top - placement.anchor_y;
        let table_extent = placement.occupied_bottom - placement.anchor_y;
        // 앞줄 0개는 아무 글줄도 방출하지 않았다. 이 문단을 pre-emitted로
        // 표시하면 표 뒤의 같은 호스트까지 이미 그렸다고 오인해 누락한다.
        if prefix > 0 && !st.pre_emitted_host_paras.contains(&input.para_idx) {
            let mut column_prefix_height = 0.0;
            let mut column_prefix_content: f64 = 0.0;
            st.finish_deferred_host_line();
            for line in 0..prefix {
                let before = if line == 0 { fmt.spacing_before } else { 0.0 };
                let required = before + fmt.line_heights[line];
                if st.current_height + required > st.available_height()
                    && !st.current_items.is_empty()
                {
                    st.advance_column_or_new_page();
                    column_prefix_height = 0.0;
                    column_prefix_content = 0.0;
                }
                st.record_host_text_fragment(
                    input.para_idx,
                    line..line + 1,
                    input.para,
                    fmt,
                    self.dpi,
                );
                st.append_item(PageItem::PartialParagraph {
                    para_index: input.para_idx,
                    start_line: line,
                    end_line: line + 1,
                });
                column_prefix_content = column_prefix_content.max(column_prefix_height + required);
                let advance = before + fmt.line_advance(line);
                column_prefix_height += advance;
                st.advance_flow_by(advance);
            }
            st.record_pre_emitted_host_height(
                input.para_idx,
                column_prefix_height,
                column_prefix_content,
            );
            st.mark_pre_emitted_host(input.para_idx);
        }
        placement.anchor_y = st.current_height;
        placement.table_top = st.current_height + table_distance;
        placement.occupied_bottom = st.current_height + table_extent;
        // prefix가 다른 단에 남을 수 있다. 이전 문단 원점은 suffix 원점이 아니다.
        placement.stored_host_origin = None;
        placement.stored_successor_line_origin = None;
        let suffix_fit = fmt.line_content_height(prefix..fmt.line_count(), 0.0);
        let group_height = table_extent + suffix_fit;
        // 쪽 제한을 끈 NONE 표 등은 본체의 통째 배치 소유자가 우선한다.
        // TopAndBottom의 뒤 글줄 흐름과 쪽 영역 제한을 같은 조건으로 취급하지 않는다.
        let owns_whole_object = super::unrestricted_float::owns_whole_object(
            input,
            available - placement.anchor_y,
            st.base_available_height(),
        );
        let mut current_available = available.min(st.available_height());
        if !owns_whole_object
            && placement.occupied_bottom + suffix_fit > current_available
            && group_height <= st.base_available_height()
            && !st.current_items.is_empty()
        {
            st.advance_column_or_new_page();
            let footnotes = st.projected_footnote_height(
                input.ft.table_footnote_height,
                input.ft.table_footnote_count,
            );
            current_available = (st.base_available_height()
                - footnotes
                - if footnotes > 0.0 {
                    footnote_margin
                } else {
                    0.0
                }
                - st.current_zone_y_offset)
                .max(0.0)
                .min(st.available_height());
            placement.anchor_y = st.current_height;
            placement.table_top = st.current_height + table_distance;
            placement.occupied_bottom = st.current_height + table_extent;
        }
        st.record_paragraph_float_placement((input.para_idx, input.ctrl_idx), placement);
        if owns_whole_object || placement.occupied_bottom + suffix_fit <= current_available {
            self.place_table_with_text(
                st,
                input.para_idx,
                input.ctrl_idx,
                input.para,
                input.table,
                fmt,
                input.para_start_height,
                table_extent,
                input.is_first_placed,
                input.is_last_placed,
                input.ft.strict_following_plain_text_fit,
                input.styles,
            );
            Some(SplitHostHandoff::Placed)
        } else {
            Some(SplitHostHandoff::Fragment {
                placement,
                tail: Box::new(FormattedHostTail {
                    first_line: prefix,
                    paragraph: fmt.clone(),
                }),
                available: current_available,
            })
        }
    }
}
