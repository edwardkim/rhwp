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
            || table.caption.is_some()
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
        if std::env::var("RHWP_TABLE_DRIFT").is_ok() {
            eprintln!(
                "COANCHORED_FIRST_FRAME: pi={} ci={} anchor={:.3} flow={:.3} top={:.3}",
                input.source.para_index, input.source.control_index, anchor, st.current_height, top
            );
        }
        let placement = ParagraphFloatPlacement {
            flow: ParagraphFloatFlow::NextLine,
            anchor_y: anchor,
            stored_host_origin: None,
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
