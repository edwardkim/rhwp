//! 앞 단의 나누지 않는 표 뒤에서 수용한 문단 간격을 출력에도 전달한다.

use super::TypesetState;
use crate::model::{control::Control, paragraph::Paragraph, table::TablePageBreak};
use crate::renderer::pagination::PageItem;
use crate::renderer::{float_placement::is_para_topbottom_float, hwpunit_to_px};

impl TypesetState {
    pub(in crate::renderer::typeset) fn starts_after_overheight_whole_float(
        &self,
        paragraphs: &[Paragraph],
        para_index: usize,
        dpi: f64,
    ) -> bool {
        if !self.data.current_items.is_empty()
            || self.data.current_height != self.data.current_start_height
            || self.data.current_start_height != 0.0
            || self.data.current_endnote_flow
        {
            return false;
        }
        let Some((page, column)) = self
            .data
            .pages
            .iter()
            .rev()
            .find_map(|page| page.column_contents.last().map(|column| (page, column)))
        else {
            return false;
        };
        if page.section_index != self.data.section_index {
            return false;
        }
        // 실제로 앞 단이 소유한 통째 표를 확인한다. 좌표가 크다는 이유만으로
        // 일반 저장 줄이나 이어받기 조각의 앞 간격 계약을 바꾸지 않는다.
        let Some(PageItem::Table {
            para_index: owner,
            control_index,
        }) = column.items.last()
        else {
            return false;
        };
        if owner.checked_add(1) != Some(para_index) {
            return false;
        }
        let Some(Control::Table(table)) = paragraphs
            .get(*owner)
            .and_then(|host| host.controls.get(*control_index))
        else {
            return false;
        };
        table.page_break == TablePageBreak::None
            && !table.common.flow_with_text
            && is_para_topbottom_float(&table.common)
            && column
                .paragraph_float_placements
                .get(&(*owner, *control_index))
                .is_some_and(|placement| placement.occupied_bottom > page.layout.body_area.height)
            && paragraphs[para_index]
                .line_segs
                .first()
                .is_some_and(|line| {
                    !crate::renderer::typeset::is_synthetic_line_seg(line)
                        && hwpunit_to_px(line.vertical_pos, dpi) > page.layout.body_area.height
                })
    }
}
